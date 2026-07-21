use crate::src::nvim::global_cell::GlobalCell;
extern "C" {
    pub type lua_State;
    fn lua_gettop(L: *mut lua_State) -> ::core::ffi::c_int;
    fn lua_pushstring(L: *mut lua_State, s: *const ::core::ffi::c_char);
    fn lua_pushcclosure(L: *mut lua_State, fn_0: lua_CFunction, n: ::core::ffi::c_int);
    fn lua_createtable(L: *mut lua_State, narr: ::core::ffi::c_int, nrec: ::core::ffi::c_int);
    fn lua_setfield(L: *mut lua_State, idx: ::core::ffi::c_int, k: *const ::core::ffi::c_char);
    fn lua_error(L: *mut lua_State) -> ::core::ffi::c_int;
    fn lua_concat(L: *mut lua_State, n: ::core::ffi::c_int);
    fn luaL_where(L: *mut lua_State, lvl: ::core::ffi::c_int);
    fn luaL_error(L: *mut lua_State, fmt: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn arena_finish(arena: *mut Arena) -> ArenaMem;
    fn arena_mem_free(mem: ArenaMem);
    fn KeyDict_empty_get_field(str: *const ::core::ffi::c_char, len: size_t) -> *mut KeySetLink;
    fn KeyDict_context_get_field(str: *const ::core::ffi::c_char, len: size_t) -> *mut KeySetLink;
    fn KeyDict_set_decoration_provider_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_set_extmark_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_get_extmark_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_get_extmarks_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_keymap_get_field(str: *const ::core::ffi::c_char, len: size_t) -> *mut KeySetLink;
    fn KeyDict_get_commands_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_user_command_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_win_config_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_tabpage_config_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_runtime_get_field(str: *const ::core::ffi::c_char, len: size_t) -> *mut KeySetLink;
    fn KeyDict_eval_statusline_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_option_get_field(str: *const ::core::ffi::c_char, len: size_t) -> *mut KeySetLink;
    fn KeyDict_highlight_get_field(str: *const ::core::ffi::c_char, len: size_t)
        -> *mut KeySetLink;
    fn KeyDict_get_highlight_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_get_ns_get_field(str: *const ::core::ffi::c_char, len: size_t) -> *mut KeySetLink;
    fn KeyDict_win_text_height_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_clear_autocmds_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_create_autocmd_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_exec_autocmds_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_get_autocmds_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_create_augroup_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_cmd_get_field(str: *const ::core::ffi::c_char, len: size_t) -> *mut KeySetLink;
    fn KeyDict_cmd_opts_get_field(str: *const ::core::ffi::c_char, len: size_t) -> *mut KeySetLink;
    fn KeyDict_echo_opts_get_field(str: *const ::core::ffi::c_char, len: size_t)
        -> *mut KeySetLink;
    fn KeyDict_exec_opts_get_field(str: *const ::core::ffi::c_char, len: size_t)
        -> *mut KeySetLink;
    fn KeyDict_buf_attach_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_buf_delete_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_open_term_get_field(str: *const ::core::ffi::c_char, len: size_t)
        -> *mut KeySetLink;
    fn KeyDict_complete_set_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_redraw_get_field(str: *const ::core::ffi::c_char, len: size_t) -> *mut KeySetLink;
    fn KeyDict_ns_opts_get_field(str: *const ::core::ffi::c_char, len: size_t) -> *mut KeySetLink;
    static empty_table: GlobalCell<[KeySetLink; 1]>;
    static context_table: GlobalCell<[KeySetLink; 2]>;
    static set_decoration_provider_table: GlobalCell<[KeySetLink; 10]>;
    static set_extmark_table: GlobalCell<[KeySetLink; 36]>;
    static get_extmark_table: GlobalCell<[KeySetLink; 3]>;
    static get_extmarks_table: GlobalCell<[KeySetLink; 6]>;
    static keymap_table: GlobalCell<[KeySetLink; 10]>;
    static get_commands_table: GlobalCell<[KeySetLink; 2]>;
    static user_command_table: GlobalCell<[KeySetLink; 13]>;
    static win_config_table: GlobalCell<[KeySetLink; 25]>;
    static tabpage_config_table: GlobalCell<[KeySetLink; 2]>;
    static runtime_table: GlobalCell<[KeySetLink; 3]>;
    static eval_statusline_table: GlobalCell<[KeySetLink; 8]>;
    static option_table: GlobalCell<[KeySetLink; 5]>;
    static highlight_table: GlobalCell<[KeySetLink; 36]>;
    static get_highlight_table: GlobalCell<[KeySetLink; 5]>;
    static get_ns_table: GlobalCell<[KeySetLink; 2]>;
    static win_text_height_table: GlobalCell<[KeySetLink; 6]>;
    static clear_autocmds_table: GlobalCell<[KeySetLink; 6]>;
    static create_autocmd_table: GlobalCell<[KeySetLink; 10]>;
    static exec_autocmds_table: GlobalCell<[KeySetLink; 7]>;
    static get_autocmds_table: GlobalCell<[KeySetLink; 7]>;
    static create_augroup_table: GlobalCell<[KeySetLink; 2]>;
    static cmd_table: GlobalCell<[KeySetLink; 12]>;
    static cmd_opts_table: GlobalCell<[KeySetLink; 2]>;
    static echo_opts_table: GlobalCell<[KeySetLink; 11]>;
    static exec_opts_table: GlobalCell<[KeySetLink; 2]>;
    static buf_attach_table: GlobalCell<[KeySetLink; 8]>;
    static buf_delete_table: GlobalCell<[KeySetLink; 3]>;
    static open_term_table: GlobalCell<[KeySetLink; 3]>;
    static complete_set_table: GlobalCell<[KeySetLink; 2]>;
    static redraw_table: GlobalCell<[KeySetLink; 11]>;
    static ns_opts_table: GlobalCell<[KeySetLink; 2]>;
    fn api_free_string(value: String_0);
    fn api_free_object(value: Object);
    fn api_free_dict(value: Dict);
    fn api_clear_error(value: *mut Error);
    fn api_set_error(err: *mut Error, errType: ErrorType, format: *const ::core::ffi::c_char, ...);
    fn api_luarefs_free_object(value: Object);
    fn api_luarefs_free_keydict(dict: *mut ::core::ffi::c_void, table: *mut KeySetLink);
    static e_textlock: [::core::ffi::c_char; 0];
    static e_fast_api_disabled: [::core::ffi::c_char; 0];
    fn expr_map_locked() -> bool;
    fn text_locked() -> bool;
    fn get_text_locked_msg() -> *const ::core::ffi::c_char;
    static mut textlock: ::core::ffi::c_int;
    fn nlua_push_String(lstate: *mut lua_State, s: String_0, flags: ::core::ffi::c_int);
    fn nlua_push_Integer(lstate: *mut lua_State, n: Integer, flags: ::core::ffi::c_int);
    fn nlua_push_Float(lstate: *mut lua_State, f: Float, flags: ::core::ffi::c_int);
    fn nlua_push_Boolean(lstate: *mut lua_State, b: Boolean, flags: ::core::ffi::c_int);
    fn nlua_push_Dict(lstate: *mut lua_State, dict: Dict, flags: ::core::ffi::c_int);
    fn nlua_push_Array(lstate: *mut lua_State, array: Array, flags: ::core::ffi::c_int);
    fn nlua_push_handle(lstate: *mut lua_State, item: handle_T, flags: ::core::ffi::c_int);
    fn nlua_push_Object(lstate: *mut lua_State, obj: *mut Object, flags: ::core::ffi::c_int);
    fn nlua_pop_String(lstate: *mut lua_State, arena: *mut Arena, err: *mut Error) -> String_0;
    fn nlua_pop_Integer(lstate: *mut lua_State, arena: *mut Arena, err: *mut Error) -> Integer;
    fn nlua_pop_Boolean(lstate: *mut lua_State, arena: *mut Arena, err: *mut Error) -> Boolean;
    fn nlua_pop_Float(lstate: *mut lua_State, arena: *mut Arena, err: *mut Error) -> Float;
    fn nlua_pop_Array(lstate: *mut lua_State, arena: *mut Arena, err: *mut Error) -> Array;
    fn nlua_pop_Dict(
        lstate: *mut lua_State,
        ref_0: bool,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Dict;
    fn nlua_pop_Object(
        lstate: *mut lua_State,
        ref_0: bool,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn nlua_pop_LuaRef(lstate: *mut lua_State, arena: *mut Arena, err: *mut Error) -> LuaRef;
    fn nlua_pop_handle(lstate: *mut lua_State, arena: *mut Arena, err: *mut Error) -> handle_T;
    fn nlua_pop_keydict(
        L: *mut lua_State,
        retval: *mut ::core::ffi::c_void,
        hashy: FieldHashfn,
        err_opt: *mut *mut ::core::ffi::c_char,
        arena: *mut Arena,
        err: *mut Error,
    );
    fn nlua_push_keydict(
        L: *mut lua_State,
        value: *mut ::core::ffi::c_void,
        table: *mut KeySetLink,
    );
    fn api_free_luaref(ref_0: LuaRef);
    fn nlua_is_deferred_safe() -> bool;
    static active_lstate: GlobalCell<*mut lua_State>;
    fn nvim_get_autocmds(
        opts: *mut KeyDict_get_autocmds,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Array;
    fn nvim_create_autocmd(
        channel_id: uint64_t,
        event: Object,
        opts: *mut KeyDict_create_autocmd,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Integer;
    fn nvim_del_autocmd(id: Integer, err: *mut Error);
    fn nvim_clear_autocmds(opts: *mut KeyDict_clear_autocmds, arena: *mut Arena, err: *mut Error);
    fn nvim_create_augroup(
        channel_id: uint64_t,
        name: String_0,
        opts: *mut KeyDict_create_augroup,
        err: *mut Error,
    ) -> Integer;
    fn nvim_del_augroup_by_id(id: Integer, err: *mut Error);
    fn nvim_del_augroup_by_name(name: String_0, err: *mut Error);
    fn nvim_exec_autocmds(
        event: Object,
        opts: *mut KeyDict_exec_autocmds,
        arena: *mut Arena,
        err: *mut Error,
    );
    fn nvim_buf_line_count(buf: Buffer, err: *mut Error) -> Integer;
    fn nvim_buf_attach(
        channel_id: uint64_t,
        buf: Buffer,
        send_buffer: Boolean,
        opts: *mut KeyDict_buf_attach,
        err: *mut Error,
    ) -> Boolean;
    fn nvim_buf_get_lines(
        channel_id: uint64_t,
        buf: Buffer,
        start: Integer,
        end: Integer,
        strict_indexing: Boolean,
        arena: *mut Arena,
        lstate: *mut lua_State,
        err: *mut Error,
    ) -> Array;
    fn nvim_buf_set_lines(
        channel_id: uint64_t,
        buf: Buffer,
        start: Integer,
        end: Integer,
        strict_indexing: Boolean,
        replacement: Array,
        arena: *mut Arena,
        err: *mut Error,
    );
    fn nvim_buf_set_text(
        channel_id: uint64_t,
        buf: Buffer,
        start_row: Integer,
        start_col: Integer,
        end_row: Integer,
        end_col: Integer,
        replacement: Array,
        arena: *mut Arena,
        err: *mut Error,
    );
    fn nvim_buf_get_text(
        channel_id: uint64_t,
        buf: Buffer,
        start_row: Integer,
        start_col: Integer,
        end_row: Integer,
        end_col: Integer,
        opts: *mut KeyDict_empty,
        arena: *mut Arena,
        lstate: *mut lua_State,
        err: *mut Error,
    ) -> Array;
    fn nvim_buf_get_offset(buf: Buffer, index: Integer, err: *mut Error) -> Integer;
    fn nvim_buf_get_var(buf: Buffer, name: String_0, arena: *mut Arena, err: *mut Error) -> Object;
    fn nvim_buf_get_changedtick(buf: Buffer, err: *mut Error) -> Integer;
    fn nvim_buf_get_keymap(
        buf: Buffer,
        mode: String_0,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Array;
    fn nvim_buf_set_keymap(
        channel_id: uint64_t,
        buf: Buffer,
        mode: String_0,
        lhs: String_0,
        rhs: String_0,
        opts: *mut KeyDict_keymap,
        err: *mut Error,
    );
    fn nvim_buf_del_keymap(
        channel_id: uint64_t,
        buf: Buffer,
        mode: String_0,
        lhs: String_0,
        err: *mut Error,
    );
    fn nvim_buf_set_var(buf: Buffer, name: String_0, value: Object, err: *mut Error);
    fn nvim_buf_del_var(buf: Buffer, name: String_0, err: *mut Error);
    fn nvim_buf_get_name(buf: Buffer, err: *mut Error) -> String_0;
    fn nvim_buf_set_name(buf: Buffer, name: String_0, err: *mut Error);
    fn nvim_buf_is_loaded(buf: Buffer) -> Boolean;
    fn nvim_buf_delete(buf: Buffer, opts: *mut KeyDict_buf_delete, err: *mut Error);
    fn nvim_buf_is_valid(buf: Buffer) -> Boolean;
    fn nvim_buf_del_mark(buf: Buffer, name: String_0, err: *mut Error) -> Boolean;
    fn nvim_buf_set_mark(
        buf: Buffer,
        name: String_0,
        line: Integer,
        col: Integer,
        opts: *mut KeyDict_empty,
        err: *mut Error,
    ) -> Boolean;
    fn nvim_buf_get_mark(buf: Buffer, name: String_0, arena: *mut Arena, err: *mut Error) -> Array;
    fn nvim_buf_call(buf: Buffer, fun: LuaRef, err: *mut Error) -> Object;
    fn nvim__buf_stats(buf: Buffer, arena: *mut Arena, err: *mut Error) -> Dict;
    fn nvim_parse_cmd(
        str: String_0,
        opts: *mut KeyDict_empty,
        arena: *mut Arena,
        err: *mut Error,
    ) -> KeyDict_cmd;
    fn nvim_cmd(
        channel_id: uint64_t,
        cmd: *mut KeyDict_cmd,
        opts: *mut KeyDict_cmd_opts,
        arena: *mut Arena,
        err: *mut Error,
    ) -> String_0;
    fn nvim_create_user_command(
        channel_id: uint64_t,
        name: String_0,
        cmd: Object,
        opts: *mut KeyDict_user_command,
        err: *mut Error,
    );
    fn nvim_del_user_command(name: String_0, err: *mut Error);
    fn nvim_buf_create_user_command(
        channel_id: uint64_t,
        buf: Buffer,
        name: String_0,
        cmd: Object,
        opts: *mut KeyDict_user_command,
        err: *mut Error,
    );
    fn nvim_buf_del_user_command(buf: Buffer, name: String_0, err: *mut Error);
    fn nvim_get_commands(
        opts: *mut KeyDict_get_commands,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Dict;
    fn nvim_buf_get_commands(
        buf: Buffer,
        opts: *mut KeyDict_get_commands,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Dict;
    fn nvim_exec(channel_id: uint64_t, src: String_0, output: Boolean, err: *mut Error)
        -> String_0;
    fn nvim_command_output(channel_id: uint64_t, command: String_0, err: *mut Error) -> String_0;
    fn nvim_buf_get_number(buffer: Buffer, err: *mut Error) -> Integer;
    fn nvim_buf_clear_highlight(
        buffer: Buffer,
        ns_id: Integer,
        line_start: Integer,
        line_end: Integer,
        err: *mut Error,
    );
    fn nvim_buf_add_highlight(
        buffer: Buffer,
        ns_id: Integer,
        hl_group: String_0,
        line: Integer,
        col_start: Integer,
        col_end: Integer,
        err: *mut Error,
    ) -> Integer;
    fn nvim_buf_set_virtual_text(
        buffer: Buffer,
        src_id: Integer,
        line: Integer,
        chunks: Array,
        opts: *mut KeyDict_empty,
        err: *mut Error,
    ) -> Integer;
    fn nvim_get_hl_by_id(hl_id: Integer, rgb: Boolean, arena: *mut Arena, err: *mut Error) -> Dict;
    fn nvim_get_hl_by_name(
        name: String_0,
        rgb: Boolean,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Dict;
    fn nvim_get_option_info(name: String_0, arena: *mut Arena, err: *mut Error) -> Dict;
    fn nvim_set_option(channel_id: uint64_t, name: String_0, value: Object, err: *mut Error);
    fn nvim_get_option(name: String_0, err: *mut Error) -> Object;
    fn nvim_buf_get_option(buffer: Buffer, name: String_0, err: *mut Error) -> Object;
    fn nvim_buf_set_option(
        channel_id: uint64_t,
        buffer: Buffer,
        name: String_0,
        value: Object,
        err: *mut Error,
    );
    fn nvim_win_get_option(window: Window, name: String_0, err: *mut Error) -> Object;
    fn nvim_win_set_option(
        channel_id: uint64_t,
        window: Window,
        name: String_0,
        value: Object,
        err: *mut Error,
    );
    fn nvim_out_write(str: String_0);
    fn nvim_err_write(str: String_0);
    fn nvim_err_writeln(str: String_0);
    fn nvim_notify(
        msg: String_0,
        log_level: Integer,
        opts: Dict,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn nvim_create_namespace(name: String_0) -> Integer;
    fn nvim_get_namespaces(arena: *mut Arena) -> Dict;
    fn nvim_buf_get_extmark_by_id(
        buf: Buffer,
        ns_id: Integer,
        id: Integer,
        opts: *mut KeyDict_get_extmark,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Array;
    fn nvim_buf_get_extmarks(
        buf: Buffer,
        ns_id: Integer,
        start: Object,
        end: Object,
        opts: *mut KeyDict_get_extmarks,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Array;
    fn nvim_buf_set_extmark(
        buf: Buffer,
        ns_id: Integer,
        line: Integer,
        col: Integer,
        opts: *mut KeyDict_set_extmark,
        err: *mut Error,
    ) -> Integer;
    fn nvim_buf_del_extmark(buf: Buffer, ns_id: Integer, id: Integer, err: *mut Error) -> Boolean;
    fn nvim_buf_clear_namespace(
        buf: Buffer,
        ns_id: Integer,
        line_start: Integer,
        line_end: Integer,
        err: *mut Error,
    );
    fn nvim_set_decoration_provider(
        ns_id: Integer,
        opts: *mut KeyDict_set_decoration_provider,
        err: *mut Error,
    );
    fn nvim__buf_debug_extmarks(
        buf: Buffer,
        keys: Boolean,
        dot: Boolean,
        err: *mut Error,
    ) -> String_0;
    fn nvim__ns_set(ns_id: Integer, opts: *mut KeyDict_ns_opts, err: *mut Error);
    fn nvim__ns_get(ns_id: Integer, arena: *mut Arena, err: *mut Error) -> KeyDict_ns_opts;
    fn nvim_get_option_value(name: String_0, opts: *mut KeyDict_option, err: *mut Error) -> Object;
    fn nvim_set_option_value(
        channel_id: uint64_t,
        name: String_0,
        value: Object,
        opts: *mut KeyDict_option,
        err: *mut Error,
    );
    fn nvim_get_all_options_info(arena: *mut Arena, err: *mut Error) -> Dict;
    fn nvim_get_option_info2(
        name: String_0,
        opts: *mut KeyDict_option,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Dict;
    fn nvim_tabpage_list_wins(tabpage: Tabpage, arena: *mut Arena, err: *mut Error) -> Array;
    fn nvim_tabpage_get_var(
        tabpage: Tabpage,
        name: String_0,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn nvim_tabpage_set_var(tabpage: Tabpage, name: String_0, value: Object, err: *mut Error);
    fn nvim_tabpage_del_var(tabpage: Tabpage, name: String_0, err: *mut Error);
    fn nvim_tabpage_get_win(tabpage: Tabpage, err: *mut Error) -> Window;
    fn nvim_tabpage_set_win(tabpage: Tabpage, win: Window, err: *mut Error);
    fn nvim_tabpage_get_number(tabpage: Tabpage, err: *mut Error) -> Integer;
    fn nvim_tabpage_is_valid(tabpage: Tabpage) -> Boolean;
    fn nvim_open_tabpage(
        buf: Buffer,
        enter: Boolean,
        config: *mut KeyDict_tabpage_config,
        err: *mut Error,
    ) -> Tabpage;
    fn nvim_get_hl_id_by_name(name: String_0) -> Integer;
    fn nvim_get_hl(
        ns_id: Integer,
        opts: *mut KeyDict_get_highlight,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Dict;
    fn nvim_set_hl(
        channel_id: uint64_t,
        ns_id: Integer,
        name: String_0,
        val: *mut KeyDict_highlight,
        err: *mut Error,
    );
    fn nvim_get_hl_ns(opts: *mut KeyDict_get_ns, err: *mut Error) -> Integer;
    fn nvim_set_hl_ns(ns_id: Integer, err: *mut Error);
    fn nvim_set_hl_ns_fast(ns_id: Integer, err: *mut Error);
    fn nvim_feedkeys(keys: String_0, mode: String_0, escape_ks: Boolean);
    fn nvim_input(channel_id: uint64_t, keys: String_0) -> Integer;
    fn nvim_input_mouse(
        button: String_0,
        action: String_0,
        modifier: String_0,
        grid: Integer,
        row: Integer,
        col: Integer,
        err: *mut Error,
    );
    fn nvim_replace_termcodes(
        str: String_0,
        from_part: Boolean,
        do_lt: Boolean,
        special: Boolean,
    ) -> String_0;
    fn nvim_strwidth(text: String_0, err: *mut Error) -> Integer;
    fn nvim_list_runtime_paths(arena: *mut Arena, err: *mut Error) -> Array;
    fn nvim__runtime_inspect(arena: *mut Arena) -> Array;
    fn nvim_get_runtime_file(
        name: String_0,
        all: Boolean,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Array;
    fn nvim__get_lib_dir() -> String_0;
    fn nvim__get_runtime(
        pat: Array,
        all: Boolean,
        opts: *mut KeyDict_runtime,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Array;
    fn nvim_set_current_dir(dir: String_0, err: *mut Error);
    fn nvim_get_current_line(arena: *mut Arena, err: *mut Error) -> String_0;
    fn nvim_set_current_line(line: String_0, arena: *mut Arena, err: *mut Error);
    fn nvim_del_current_line(arena: *mut Arena, err: *mut Error);
    fn nvim_get_var(name: String_0, arena: *mut Arena, err: *mut Error) -> Object;
    fn nvim_set_var(name: String_0, value: Object, err: *mut Error);
    fn nvim_del_var(name: String_0, err: *mut Error);
    fn nvim_get_vvar(name: String_0, arena: *mut Arena, err: *mut Error) -> Object;
    fn nvim_set_vvar(name: String_0, value: Object, err: *mut Error);
    fn nvim_echo(
        chunks: Array,
        history: Boolean,
        opts: *mut KeyDict_echo_opts,
        err: *mut Error,
    ) -> Object;
    fn nvim_list_bufs(arena: *mut Arena) -> Array;
    fn nvim_get_current_buf() -> Buffer;
    fn nvim_set_current_buf(buf: Buffer, err: *mut Error);
    fn nvim_list_wins(arena: *mut Arena) -> Array;
    fn nvim_get_current_win() -> Window;
    fn nvim_set_current_win(win: Window, err: *mut Error);
    fn nvim_create_buf(listed: Boolean, scratch: Boolean, err: *mut Error) -> Buffer;
    fn nvim_open_term(buf: Buffer, opts: *mut KeyDict_open_term, err: *mut Error) -> Integer;
    fn nvim_chan_send(chan: Integer, data: String_0, err: *mut Error);
    fn nvim_list_tabpages(arena: *mut Arena) -> Array;
    fn nvim_get_current_tabpage() -> Tabpage;
    fn nvim_set_current_tabpage(tabpage: Tabpage, err: *mut Error);
    fn nvim_paste(
        channel_id: uint64_t,
        data: String_0,
        crlf: Boolean,
        phase: Integer,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Boolean;
    fn nvim_put(
        lines: Array,
        type_0: String_0,
        after: Boolean,
        follow: Boolean,
        arena: *mut Arena,
        err: *mut Error,
    );
    fn nvim_get_color_by_name(name: String_0) -> Integer;
    fn nvim_get_color_map(arena: *mut Arena) -> Dict;
    fn nvim_get_context(opts: *mut KeyDict_context, arena: *mut Arena, err: *mut Error) -> Dict;
    fn nvim_load_context(dict: Dict, err: *mut Error) -> Object;
    fn nvim_get_mode(arena: *mut Arena) -> Dict;
    fn nvim_get_keymap(mode: String_0, arena: *mut Arena) -> Array;
    fn nvim_set_keymap(
        channel_id: uint64_t,
        mode: String_0,
        lhs: String_0,
        rhs: String_0,
        opts: *mut KeyDict_keymap,
        err: *mut Error,
    );
    fn nvim_del_keymap(channel_id: uint64_t, mode: String_0, lhs: String_0, err: *mut Error);
    fn nvim_get_chan_info(
        channel_id: uint64_t,
        chan: Integer,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Dict;
    fn nvim_list_chans(arena: *mut Arena) -> Array;
    fn nvim__id(obj: Object, arena: *mut Arena) -> Object;
    fn nvim__id_array(arr: Array, arena: *mut Arena) -> Array;
    fn nvim__id_dict(dct: Dict, arena: *mut Arena) -> Dict;
    fn nvim__id_float(flt: Float) -> Float;
    fn nvim__stats(arena: *mut Arena) -> Dict;
    fn nvim_list_uis(arena: *mut Arena) -> Array;
    fn nvim_get_proc_children(pid: Integer, arena: *mut Arena, err: *mut Error) -> Array;
    fn nvim_get_proc(pid: Integer, arena: *mut Arena, err: *mut Error) -> Object;
    fn nvim_select_popupmenu_item(
        item: Integer,
        insert: Boolean,
        finish: Boolean,
        opts: *mut KeyDict_empty,
        err: *mut Error,
    );
    fn nvim__inspect_cell(
        grid: Integer,
        row: Integer,
        col: Integer,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Array;
    fn nvim__screenshot(path: String_0);
    fn nvim__invalidate_glyph_cache();
    fn nvim__unpack(str: String_0, arena: *mut Arena, err: *mut Error) -> Object;
    fn nvim_del_mark(name: String_0, err: *mut Error) -> Boolean;
    fn nvim_get_mark(
        name: String_0,
        opts: *mut KeyDict_empty,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Array;
    fn nvim_eval_statusline(
        str: String_0,
        opts: *mut KeyDict_eval_statusline,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Dict;
    fn nvim__complete_set(
        index: Integer,
        opts: *mut KeyDict_complete_set,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Dict;
    fn nvim__redraw(opts: *mut KeyDict_redraw, err: *mut Error);
    fn nvim_exec2(
        channel_id: uint64_t,
        src: String_0,
        opts: *mut KeyDict_exec_opts,
        err: *mut Error,
    ) -> Dict;
    fn nvim_command(cmd: String_0, err: *mut Error);
    fn nvim_eval(expr: String_0, arena: *mut Arena, err: *mut Error) -> Object;
    fn nvim_call_function(
        fn_0: String_0,
        args: Array,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn nvim_call_dict_function(
        dict: Object,
        fn_0: String_0,
        args: Array,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn nvim_parse_expression(
        expr: String_0,
        flags: String_0,
        hl: Boolean,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Dict;
    fn nvim_open_win(
        buf: Buffer,
        enter: Boolean,
        config: *mut KeyDict_win_config,
        err: *mut Error,
    ) -> Window;
    fn nvim_win_set_config(win: Window, config: *mut KeyDict_win_config, err: *mut Error);
    fn nvim_win_get_config(win: Window, arena: *mut Arena, err: *mut Error) -> KeyDict_win_config;
    fn nvim_win_get_buf(win: Window, err: *mut Error) -> Buffer;
    fn nvim_win_set_buf(win: Window, buf: Buffer, err: *mut Error);
    fn nvim_win_get_cursor(win: Window, arena: *mut Arena, err: *mut Error) -> Array;
    fn nvim_win_set_cursor(win: Window, pos: Array, err: *mut Error);
    fn nvim_win_get_height(win: Window, err: *mut Error) -> Integer;
    fn nvim_win_set_height(win: Window, height: Integer, err: *mut Error);
    fn nvim_win_get_width(win: Window, err: *mut Error) -> Integer;
    fn nvim_win_set_width(win: Window, width: Integer, err: *mut Error);
    fn nvim_win_get_var(win: Window, name: String_0, arena: *mut Arena, err: *mut Error) -> Object;
    fn nvim_win_set_var(win: Window, name: String_0, value: Object, err: *mut Error);
    fn nvim_win_del_var(win: Window, name: String_0, err: *mut Error);
    fn nvim_win_get_position(win: Window, arena: *mut Arena, err: *mut Error) -> Array;
    fn nvim_win_get_tabpage(win: Window, err: *mut Error) -> Tabpage;
    fn nvim_win_get_number(win: Window, err: *mut Error) -> Integer;
    fn nvim_win_is_valid(win: Window) -> Boolean;
    fn nvim_win_hide(win: Window, err: *mut Error);
    fn nvim_win_close(win: Window, force: Boolean, err: *mut Error);
    fn nvim_win_call(win: Window, fun: LuaRef, err: *mut Error) -> Object;
    fn nvim_win_set_hl_ns(win: Window, ns_id: Integer, err: *mut Error);
    fn nvim_win_text_height(
        win: Window,
        opts: *mut KeyDict_win_text_height,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Dict;
    fn nvim_ui_send(channel_id: uint64_t, content: String_0, err: *mut Error);
}
pub type size_t = usize;
pub type lua_CFunction = Option<unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int>;
pub type int64_t = i64;
pub type uint64_t = u64;
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
pub type handle_T = ::core::ffi::c_int;
pub type LuaRef = ::core::ffi::c_int;
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
pub type Window = handle_T;
pub type Buffer = handle_T;
pub type Tabpage = handle_T;
pub type OptionalKeys = uint64_t;
pub type HLGroupID = Integer;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeySetLink {
    pub str: *mut ::core::ffi::c_char,
    pub ptr_off: size_t,
    pub type_0: ::core::ffi::c_int,
    pub opt_index: ::core::ffi::c_int,
    pub is_hlgroup: bool,
}
pub type FieldHashfn =
    Option<unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink>;
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const kNluaPushFreeRefs: C2Rust_Unnamed_0 = 2;
pub const kNluaPushSpecial: C2Rust_Unnamed_0 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_win_text_height {
    pub is_set__win_text_height_: OptionalKeys,
    pub start_row: Integer,
    pub end_row: Integer,
    pub start_vcol: Integer,
    pub end_vcol: Integer,
    pub max_height: Integer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_win_config {
    pub is_set__win_config_: OptionalKeys,
    pub external: Boolean,
    pub fixed: Boolean,
    pub focusable: Boolean,
    pub footer: Object,
    pub footer_pos: String_0,
    pub hide: Boolean,
    pub height: Integer,
    pub mouse: Boolean,
    pub relative: String_0,
    pub row: Float,
    pub style: String_0,
    pub noautocmd: Boolean,
    pub vertical: Boolean,
    pub win: Window,
    pub width: Integer,
    pub zindex: Integer,
    pub anchor: String_0,
    pub border: Object,
    pub bufpos: Array,
    pub col: Float,
    pub split: String_0,
    pub title: Object,
    pub title_pos: String_0,
    pub _cmdline_offset: Integer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_exec_opts {
    pub output: Boolean,
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
pub struct KeyDict_complete_set {
    pub is_set__complete_set_: OptionalKeys,
    pub info: String_0,
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
pub struct KeyDict_empty {
    pub is_set__empty_: OptionalKeys,
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
pub struct KeyDict_context {
    pub is_set__context_: OptionalKeys,
    pub types: Array,
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
pub struct KeyDict_runtime {
    pub is_lua: Boolean,
    pub do_source: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_get_ns {
    pub is_set__get_ns_: OptionalKeys,
    pub winid: Window,
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
pub struct KeyDict_tabpage_config {
    pub is_set__tabpage_config_: OptionalKeys,
    pub after: Integer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_option {
    pub is_set__option_: OptionalKeys,
    pub scope: String_0,
    pub win: Window,
    pub buf: Buffer,
    pub filetype: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_ns_opts {
    pub is_set__ns_opts_: OptionalKeys,
    pub wins: Array,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_set_decoration_provider {
    pub is_set__set_decoration_provider_: OptionalKeys,
    pub on_start: LuaRef,
    pub on_buf: LuaRef,
    pub on_win: LuaRef,
    pub on_line: LuaRef,
    pub on_range: LuaRef,
    pub on_end: LuaRef,
    pub _on_hl_def: LuaRef,
    pub _on_spell_nav: LuaRef,
    pub _on_conceal_line: LuaRef,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_set_extmark {
    pub is_set__set_extmark_: OptionalKeys,
    pub id: Integer,
    pub end_line: Integer,
    pub end_row: Integer,
    pub end_col: Integer,
    pub hl_group: Object,
    pub virt_text: Array,
    pub virt_text_pos: String_0,
    pub virt_text_win_col: Integer,
    pub virt_text_hide: Boolean,
    pub virt_text_repeat_linebreak: Boolean,
    pub hl_eol: Boolean,
    pub hl_mode: String_0,
    pub invalidate: Boolean,
    pub ephemeral: Boolean,
    pub priority: Integer,
    pub right_gravity: Boolean,
    pub end_right_gravity: Boolean,
    pub virt_lines: Array,
    pub virt_lines_above: Boolean,
    pub virt_lines_leftcol: Boolean,
    pub virt_lines_overflow: String_0,
    pub strict: Boolean,
    pub sign_text: String_0,
    pub sign_hl_group: HLGroupID,
    pub number_hl_group: HLGroupID,
    pub line_hl_group: HLGroupID,
    pub cursorline_hl_group: HLGroupID,
    pub conceal: String_0,
    pub conceal_lines: String_0,
    pub spell: Boolean,
    pub ui_watched: Boolean,
    pub undo_restore: Boolean,
    pub url: String_0,
    pub scoped: Boolean,
    pub _subpriority: Integer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_get_extmarks {
    pub is_set__get_extmarks_: OptionalKeys,
    pub limit: Integer,
    pub details: Boolean,
    pub hl_name: Boolean,
    pub overlap: Boolean,
    pub type_0: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_get_extmark {
    pub is_set__get_extmark_: OptionalKeys,
    pub details: Boolean,
    pub hl_name: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_get_commands {
    pub builtin: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_user_command {
    pub is_set__user_command_: OptionalKeys,
    pub addr: Object,
    pub bang: Boolean,
    pub bar: Boolean,
    pub complete: Object,
    pub count: Object,
    pub desc: Object,
    pub force: Boolean,
    pub keepscript: Boolean,
    pub nargs: Object,
    pub preview: Object,
    pub range: Object,
    pub register_: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_cmd_opts {
    pub output: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_cmd {
    pub is_set__cmd_: OptionalKeys,
    pub cmd: String_0,
    pub range: Array,
    pub count: Integer,
    pub reg: String_0,
    pub bang: Boolean,
    pub args: Array,
    pub magic: Dict,
    pub mods: Dict,
    pub nargs: Object,
    pub addr: String_0,
    pub nextcmd: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_buf_delete {
    pub is_set__buf_delete_: OptionalKeys,
    pub force: Boolean,
    pub unload: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_buf_attach {
    pub is_set__buf_attach_: OptionalKeys,
    pub on_lines: LuaRef,
    pub on_bytes: LuaRef,
    pub on_changedtick: LuaRef,
    pub on_detach: LuaRef,
    pub on_reload: LuaRef,
    pub utf_sizes: Boolean,
    pub preview: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_exec_autocmds {
    pub is_set__exec_autocmds_: OptionalKeys,
    pub buffer: Buffer,
    pub buf: Buffer,
    pub group: Object,
    pub modeline: Boolean,
    pub pattern: Object,
    pub data: Object,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_create_augroup {
    pub is_set__create_augroup_: OptionalKeys,
    pub clear: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_clear_autocmds {
    pub is_set__clear_autocmds_: OptionalKeys,
    pub buffer: Buffer,
    pub buf: Buffer,
    pub event: Object,
    pub group: Object,
    pub pattern: Object,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_create_autocmd {
    pub is_set__create_autocmd_: OptionalKeys,
    pub buffer: Buffer,
    pub buf: Buffer,
    pub callback: Object,
    pub command: String_0,
    pub desc: String_0,
    pub group: Object,
    pub nested: Boolean,
    pub once: Boolean,
    pub pattern: Object,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_get_autocmds {
    pub is_set__get_autocmds_: OptionalKeys,
    pub event: Object,
    pub group: Object,
    pub pattern: Object,
    pub buffer: Object,
    pub buf: Object,
    pub id: Integer,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
pub const INTERNAL_CALL_MASK: uint64_t = (1 as ::core::ffi::c_int as uint64_t)
    << ::core::mem::size_of::<uint64_t>()
        .wrapping_mul(8 as usize)
        .wrapping_sub(1 as usize);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL.wrapping_add(1 as uint64_t);
unsafe extern "C" fn nlua_api_nvim_get_autocmds(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg1: KeyDict_get_autocmds = KeyDict_get_autocmds {
        is_set__get_autocmds_: 0,
        event: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        group: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        pattern: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        buffer: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        buf: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        id: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_autocmds\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = KeyDict_get_autocmds {
            is_set__get_autocmds_: 0 as OptionalKeys,
            event: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            group: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            pattern: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            buffer: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            buf: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            id: 0,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg1 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_get_autocmds_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_autocmds(&raw mut arg1, &raw mut arena, &raw mut err);
            nlua_push_Array(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
        api_luarefs_free_keydict(
            &raw mut arg1 as *mut ::core::ffi::c_void,
            get_autocmds_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_create_autocmd(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut arg2: KeyDict_create_autocmd = KeyDict_create_autocmd {
        is_set__create_autocmd_: 0,
        buffer: 0,
        buf: 0,
        callback: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        command: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        desc: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        group: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        nested: false,
        once: false,
        pattern: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_create_autocmd\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_create_autocmd {
            is_set__create_autocmd_: 0 as OptionalKeys,
            buffer: 0,
            buf: 0,
            callback: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            command: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            desc: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            group: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            nested: false,
            once: false,
            pattern: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_create_autocmd_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"event\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_create_autocmd(
                    LUA_INTERNAL_CALL,
                    arg1,
                    &raw mut arg2,
                    &raw mut arena,
                    &raw mut err,
                );
                nlua_push_Integer(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
                api_luarefs_free_object(arg1);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            create_autocmd_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_del_autocmd(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_del_autocmd\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_del_autocmd(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_clear_autocmds(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg1: KeyDict_clear_autocmds = KeyDict_clear_autocmds {
        is_set__clear_autocmds_: 0,
        buffer: 0,
        buf: 0,
        event: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        group: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        pattern: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_clear_autocmds\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = KeyDict_clear_autocmds {
            is_set__clear_autocmds_: 0 as OptionalKeys,
            buffer: 0,
            buf: 0,
            event: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            group: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            pattern: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg1 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_clear_autocmds_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_clear_autocmds(&raw mut arg1, &raw mut arena, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
        api_luarefs_free_keydict(
            &raw mut arg1 as *mut ::core::ffi::c_void,
            clear_autocmds_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_create_augroup(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut arg2: KeyDict_create_augroup = KeyDict_create_augroup {
        is_set__create_augroup_: 0,
        clear: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_create_augroup\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_create_augroup {
            is_set__create_augroup_: 0 as OptionalKeys,
            clear: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_create_augroup_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_create_augroup(LUA_INTERNAL_CALL, arg1, &raw mut arg2, &raw mut err);
                nlua_push_Integer(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            create_augroup_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_del_augroup_by_id(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_del_augroup_by_id\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_del_augroup_by_id(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_del_augroup_by_name(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_del_augroup_by_name\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_del_augroup_by_name(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_exec_autocmds(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: KeyDict_exec_autocmds = KeyDict_exec_autocmds {
        is_set__exec_autocmds_: 0,
        buffer: 0,
        buf: 0,
        group: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        modeline: false,
        pattern: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        data: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_exec_autocmds\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_exec_autocmds {
            is_set__exec_autocmds_: 0 as OptionalKeys,
            buffer: 0,
            buf: 0,
            group: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            modeline: false,
            pattern: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            data: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_exec_autocmds_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"event\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_exec_autocmds(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                active_lstate.set(save_active_lstate);
                api_luarefs_free_object(arg1);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            exec_autocmds_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_line_count(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_line_count\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_buf_line_count(arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_attach(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut arg3: KeyDict_buf_attach = KeyDict_buf_attach {
        is_set__buf_attach_: 0,
        on_lines: 0,
        on_bytes: 0,
        on_changedtick: 0,
        on_detach: 0,
        on_reload: 0,
        utf_sizes: false,
        preview: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_attach\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = KeyDict_buf_attach {
            is_set__buf_attach_: 0 as OptionalKeys,
            on_lines: 0,
            on_bytes: 0,
            on_changedtick: 0,
            on_detach: 0,
            on_reload: 0,
            utf_sizes: false,
            preview: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg3 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_buf_attach_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param = b"send_buffer\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret =
                        nvim_buf_attach(LUA_INTERNAL_CALL, arg1, arg2, &raw mut arg3, &raw mut err);
                    nlua_push_Boolean(
                        lstate,
                        ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg3 as *mut ::core::ffi::c_void,
            buf_attach_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_lines(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg4: Boolean = false;
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_lines\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg4 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"strict_indexing\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        } else {
            arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"end\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"start\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        ret = nvim_buf_get_lines(
                            LUA_INTERNAL_CALL,
                            arg1,
                            arg2,
                            arg3,
                            arg4,
                            &raw mut arena,
                            lstate,
                            &raw mut err,
                        );
                        if lua_gettop(lstate) == 0 as ::core::ffi::c_int {
                            nlua_push_Array(
                                lstate,
                                ret,
                                kNluaPushSpecial as ::core::ffi::c_int
                                    | kNluaPushFreeRefs as ::core::ffi::c_int,
                            );
                        }
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_lines(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg5: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg4: Boolean = false;
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 5 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 5 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_lines\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0 {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg5 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param = b"replacement\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else {
                arg4 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"strict_indexing\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"end\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"start\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                            if err.type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            } else {
                                save_active_lstate = active_lstate.get();
                                active_lstate.set(lstate);
                                nvim_buf_set_lines(
                                    LUA_INTERNAL_CALL,
                                    arg1,
                                    arg2,
                                    arg3,
                                    arg4,
                                    arg5,
                                    &raw mut arena,
                                    &raw mut err,
                                );
                                active_lstate.set(save_active_lstate);
                            }
                        }
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_text(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg6: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg5: Integer = 0;
    let mut arg4: Integer = 0;
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 6 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 6 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_text\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0 {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg6 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param = b"replacement\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else {
                arg5 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"end_col\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg4 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"end_row\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"start_col\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                            if err.type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                err_param = b"start_row\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            } else {
                                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                                if err.type_0 as ::core::ffi::c_int
                                    != kErrorTypeNone as ::core::ffi::c_int
                                {
                                    err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char;
                                } else {
                                    save_active_lstate = active_lstate.get();
                                    active_lstate.set(lstate);
                                    nvim_buf_set_text(
                                        LUA_INTERNAL_CALL,
                                        arg1,
                                        arg2,
                                        arg3,
                                        arg4,
                                        arg5,
                                        arg6,
                                        &raw mut arena,
                                        &raw mut err,
                                    );
                                    active_lstate.set(save_active_lstate);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_text(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg5: Integer = 0;
    let mut arg4: Integer = 0;
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg6: KeyDict_empty = KeyDict_empty { is_set__empty_: 0 };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 6 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 6 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_text\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg6 = KeyDict_empty {
            is_set__empty_: 0 as OptionalKeys,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg6 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_empty_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg5 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"end_col\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg4 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"end_row\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"start_col\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"start_row\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                            if err.type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            } else {
                                save_active_lstate = active_lstate.get();
                                active_lstate.set(lstate);
                                ret = nvim_buf_get_text(
                                    LUA_INTERNAL_CALL,
                                    arg1,
                                    arg2,
                                    arg3,
                                    arg4,
                                    arg5,
                                    &raw mut arg6,
                                    &raw mut arena,
                                    lstate,
                                    &raw mut err,
                                );
                                if lua_gettop(lstate) == 0 as ::core::ffi::c_int {
                                    nlua_push_Array(
                                        lstate,
                                        ret,
                                        kNluaPushSpecial as ::core::ffi::c_int
                                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                                    );
                                }
                                active_lstate.set(save_active_lstate);
                            }
                        }
                    }
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg6 as *mut ::core::ffi::c_void,
            empty_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_offset(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_offset\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"index\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_buf_get_offset(arg1, arg2, &raw mut err);
                nlua_push_Integer(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_buf_get_var(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_changedtick(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_changedtick\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_buf_get_changedtick(arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_keymap(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_keymap\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"mode\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_buf_get_keymap(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Array(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_keymap(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg4: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg3: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg5: KeyDict_keymap = KeyDict_keymap {
        is_set__keymap_: 0,
        noremap: false,
        nowait: false,
        silent: false,
        script: false,
        expr: false,
        unique: false,
        callback: 0,
        desc: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        replace_keycodes: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 5 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 5 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_keymap\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg5 = KeyDict_keymap {
            is_set__keymap_: 0 as OptionalKeys,
            noremap: false,
            nowait: false,
            silent: false,
            script: false,
            expr: false,
            unique: false,
            callback: 0,
            desc: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            replace_keycodes: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg5 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_keymap_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg4 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"rhs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg3 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"lhs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"mode\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            save_active_lstate = active_lstate.get();
                            active_lstate.set(lstate);
                            nvim_buf_set_keymap(
                                LUA_INTERNAL_CALL,
                                arg1,
                                arg2,
                                arg3,
                                arg4,
                                &raw mut arg5,
                                &raw mut err,
                            );
                            active_lstate.set(save_active_lstate);
                        }
                    }
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg5 as *mut ::core::ffi::c_void,
            keymap_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_del_keymap(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg3: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_del_keymap\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"lhs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"mode\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_buf_del_keymap(LUA_INTERNAL_CALL, arg1, arg2, arg3, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg3: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_buf_set_var(arg1, arg2, arg3, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
            api_luarefs_free_object(arg3);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_del_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_del_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_buf_del_var(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_name(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_name\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_buf_get_name(arg1, &raw mut err);
            nlua_push_String(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_name(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_name\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_buf_set_name(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_is_loaded(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_is_loaded\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_buf_is_loaded(arg1);
            nlua_push_Boolean(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_delete(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: KeyDict_buf_delete = KeyDict_buf_delete {
        is_set__buf_delete_: 0,
        force: false,
        unload: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_delete\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if text_locked() {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                get_text_locked_msg(),
            );
        } else {
            arg2 = KeyDict_buf_delete {
                is_set__buf_delete_: 0 as OptionalKeys,
                force: false,
                unload: false,
            };
            nlua_pop_keydict(
                lstate,
                &raw mut arg2 as *mut ::core::ffi::c_void,
                Some(
                    KeyDict_buf_delete_get_field
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            size_t,
                        ) -> *mut KeySetLink,
                ),
                &raw mut err_param,
                &raw mut arena,
                &raw mut err,
            );
            if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_buf_delete(arg1, &raw mut arg2, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
            api_luarefs_free_keydict(
                &raw mut arg2 as *mut ::core::ffi::c_void,
                buf_delete_table.ptr() as *mut KeySetLink,
            );
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_is_valid(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_buf_is_valid(arg1);
            nlua_push_Boolean(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_del_mark(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_del_mark\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_buf_del_mark(arg1, arg2, &raw mut err);
                nlua_push_Boolean(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_mark(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg4: Integer = 0;
    let mut arg3: Integer = 0;
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut arg5: KeyDict_empty = KeyDict_empty { is_set__empty_: 0 };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 5 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 5 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_mark\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg5 = KeyDict_empty {
            is_set__empty_: 0 as OptionalKeys,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg5 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_empty_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg4 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"col\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"line\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"name\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            save_active_lstate = active_lstate.get();
                            active_lstate.set(lstate);
                            ret = nvim_buf_set_mark(
                                arg1,
                                arg2,
                                arg3,
                                arg4,
                                &raw mut arg5,
                                &raw mut err,
                            );
                            nlua_push_Boolean(
                                lstate,
                                ret,
                                kNluaPushSpecial as ::core::ffi::c_int
                                    | kNluaPushFreeRefs as ::core::ffi::c_int,
                            );
                            active_lstate.set(save_active_lstate);
                        }
                    }
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg5 as *mut ::core::ffi::c_void,
            empty_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_mark(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_mark\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_buf_get_mark(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Array(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_call(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: LuaRef = 0;
    let mut arg1: Buffer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_call\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_LuaRef(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"fun\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_buf_call(arg1, arg2, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
            api_free_luaref(arg2);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__buf_stats(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__buf_stats\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim__buf_stats(arg1, &raw mut arena, &raw mut err);
            nlua_push_Dict(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_parse_cmd(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: KeyDict_cmd = KeyDict_cmd {
        is_set__cmd_: 0,
        cmd: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        range: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        count: 0,
        reg: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        bang: false,
        args: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        magic: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
        mods: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
        nargs: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        addr: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        nextcmd: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut arg2: KeyDict_empty = KeyDict_empty { is_set__empty_: 0 };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg2 = KeyDict_empty {
            is_set__empty_: 0 as OptionalKeys,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_empty_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"str\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_parse_cmd(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                nlua_push_keydict(
                    lstate,
                    &raw mut ret as *mut ::core::ffi::c_void,
                    cmd_table.ptr() as *mut KeySetLink,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            empty_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_cmd(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: KeyDict_cmd = KeyDict_cmd {
        is_set__cmd_: 0,
        cmd: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        range: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        count: 0,
        reg: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        bang: false,
        args: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        magic: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
        mods: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
        nargs: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        addr: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        nextcmd: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut arg2: KeyDict_cmd_opts = KeyDict_cmd_opts { output: false };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_cmd\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_cmd_opts { output: false };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_cmd_opts_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = KeyDict_cmd {
                is_set__cmd_: 0 as OptionalKeys,
                cmd: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                range: Array {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<Object>(),
                },
                count: 0,
                reg: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                bang: false,
                args: Array {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<Object>(),
                },
                magic: Dict {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<KeyValuePair>(),
                },
                mods: Dict {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<KeyValuePair>(),
                },
                nargs: Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                },
                addr: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                nextcmd: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
            };
            nlua_pop_keydict(
                lstate,
                &raw mut arg1 as *mut ::core::ffi::c_void,
                Some(
                    KeyDict_cmd_get_field
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            size_t,
                        ) -> *mut KeySetLink,
                ),
                &raw mut err_param,
                &raw mut arena,
                &raw mut err,
            );
            if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_cmd(
                    LUA_INTERNAL_CALL,
                    &raw mut arg1,
                    &raw mut arg2,
                    &raw mut arena,
                    &raw mut err,
                );
                nlua_push_String(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
            api_luarefs_free_keydict(
                &raw mut arg1 as *mut ::core::ffi::c_void,
                cmd_table.ptr() as *mut KeySetLink,
            );
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            cmd_opts_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_create_user_command(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg3: KeyDict_user_command = KeyDict_user_command {
        is_set__user_command_: 0,
        addr: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        bang: false,
        bar: false,
        complete: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        count: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        desc: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        force: false,
        keepscript: false,
        nargs: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        preview: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        range: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        register_: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_create_user_command\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = KeyDict_user_command {
            is_set__user_command_: 0 as OptionalKeys,
            addr: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            bang: false,
            bar: false,
            complete: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            count: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            desc: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            force: false,
            keepscript: false,
            nargs: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            preview: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            range: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            register_: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg3 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_user_command_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg2 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"cmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"name\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_create_user_command(
                        LUA_INTERNAL_CALL,
                        arg1,
                        arg2,
                        &raw mut arg3,
                        &raw mut err,
                    );
                    active_lstate.set(save_active_lstate);
                }
                api_luarefs_free_object(arg2);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg3 as *mut ::core::ffi::c_void,
            user_command_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_del_user_command(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_del_user_command\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_del_user_command(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_create_user_command(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg3: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg4: KeyDict_user_command = KeyDict_user_command {
        is_set__user_command_: 0,
        addr: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        bang: false,
        bar: false,
        complete: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        count: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        desc: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        force: false,
        keepscript: false,
        nargs: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        preview: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        range: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        register_: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_create_user_command\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg4 = KeyDict_user_command {
            is_set__user_command_: 0 as OptionalKeys,
            addr: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            bang: false,
            bar: false,
            complete: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            count: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            desc: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            force: false,
            keepscript: false,
            nargs: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            preview: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            range: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            register_: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg4 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_user_command_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg3 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"cmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"name\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        nvim_buf_create_user_command(
                            LUA_INTERNAL_CALL,
                            arg1,
                            arg2,
                            arg3,
                            &raw mut arg4,
                            &raw mut err,
                        );
                        active_lstate.set(save_active_lstate);
                    }
                }
                api_luarefs_free_object(arg3);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg4 as *mut ::core::ffi::c_void,
            user_command_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_del_user_command(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_del_user_command\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_buf_del_user_command(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_commands(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg1: KeyDict_get_commands = KeyDict_get_commands { builtin: false };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_commands\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = KeyDict_get_commands { builtin: false };
        nlua_pop_keydict(
            lstate,
            &raw mut arg1 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_get_commands_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_commands(&raw mut arg1, &raw mut arena, &raw mut err);
            nlua_push_Dict(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
        api_luarefs_free_keydict(
            &raw mut arg1 as *mut ::core::ffi::c_void,
            get_commands_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_commands(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg2: KeyDict_get_commands = KeyDict_get_commands { builtin: false };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_commands\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_get_commands { builtin: false };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_get_commands_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_buf_get_commands(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            get_commands_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_exec(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_exec\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"output\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"src\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_exec(LUA_INTERNAL_CALL, arg1, arg2, &raw mut err);
                nlua_push_String(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
                api_free_string(ret);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_command_output(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_command_output\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"command\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_command_output(LUA_INTERNAL_CALL, arg1, &raw mut err);
            nlua_push_String(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
            api_free_string(ret);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_number(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_number\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"buffer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_buf_get_number(arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_clear_highlight(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg4: Integer = 0;
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_clear_highlight\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg4 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"line_end\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param = b"line_start\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"ns_id\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"buffer\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        nvim_buf_clear_highlight(arg1, arg2, arg3, arg4, &raw mut err);
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_add_highlight(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg6: Integer = 0;
    let mut arg5: Integer = 0;
    let mut arg4: Integer = 0;
    let mut arg3: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 6 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 6 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_add_highlight\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg6 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"col_end\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg5 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param = b"col_start\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else {
                arg4 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"line\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg3 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"hl_group\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"ns_id\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                            if err.type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                err_param = b"buffer\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            } else {
                                save_active_lstate = active_lstate.get();
                                active_lstate.set(lstate);
                                ret = nvim_buf_add_highlight(
                                    arg1,
                                    arg2,
                                    arg3,
                                    arg4,
                                    arg5,
                                    arg6,
                                    &raw mut err,
                                );
                                nlua_push_Integer(
                                    lstate,
                                    ret,
                                    kNluaPushSpecial as ::core::ffi::c_int
                                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                                );
                                active_lstate.set(save_active_lstate);
                            }
                        }
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_virtual_text(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg4: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut arg5: KeyDict_empty = KeyDict_empty { is_set__empty_: 0 };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 5 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 5 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_virtual_text\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg5 = KeyDict_empty {
            is_set__empty_: 0 as OptionalKeys,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg5 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_empty_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg4 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"chunks\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"line\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"src_id\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"buffer\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            save_active_lstate = active_lstate.get();
                            active_lstate.set(lstate);
                            ret = nvim_buf_set_virtual_text(
                                arg1,
                                arg2,
                                arg3,
                                arg4,
                                &raw mut arg5,
                                &raw mut err,
                            );
                            nlua_push_Integer(
                                lstate,
                                ret,
                                kNluaPushSpecial as ::core::ffi::c_int
                                    | kNluaPushFreeRefs as ::core::ffi::c_int,
                            );
                            active_lstate.set(save_active_lstate);
                        }
                    }
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg5 as *mut ::core::ffi::c_void,
            empty_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_hl_by_id(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_hl_by_id\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"rgb\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"hl_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_get_hl_by_id(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_hl_by_name(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_hl_by_name\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"rgb\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_get_hl_by_name(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_option_info(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_option_info\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_option_info(arg1, &raw mut arena, &raw mut err);
            nlua_push_Dict(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_option(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_option\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_set_option(LUA_INTERNAL_CALL, arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
            api_luarefs_free_object(arg2);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_option(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_option\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_option(arg1, &raw mut err);
            nlua_push_Object(
                lstate,
                &raw mut ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
            api_free_object(ret);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_option(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_option\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buffer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_buf_get_option(arg1, arg2, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
                api_free_object(ret);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_option(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg3: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_option\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"buffer\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_buf_set_option(LUA_INTERNAL_CALL, arg1, arg2, arg3, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
            api_luarefs_free_object(arg3);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_option(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_option\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"window\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_win_get_option(arg1, arg2, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
                api_free_object(ret);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_set_option(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg3: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_set_option\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"window\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_win_set_option(LUA_INTERNAL_CALL, arg1, arg2, arg3, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
            api_luarefs_free_object(arg3);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_out_write(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_out_write\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"str\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_out_write(arg1);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_err_write(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_err_write\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"str\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_err_write(arg1);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_err_writeln(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_err_writeln\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"str\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_err_writeln(arg1);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_notify(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg3: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg2: Integer = 0;
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_notify\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Dict(lstate, false_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"opts\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param = b"log_level\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"msg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret = nvim_notify(arg1, arg2, arg3, &raw mut arena, &raw mut err);
                    nlua_push_Object(
                        lstate,
                        &raw mut ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_create_namespace(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_create_namespace\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_create_namespace(arg1);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_namespaces(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_namespaces\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_get_namespaces(&raw mut arena);
        nlua_push_Dict(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_extmark_by_id(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg4: KeyDict_get_extmark = KeyDict_get_extmark {
        is_set__get_extmark_: 0,
        details: false,
        hl_name: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_extmark_by_id\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg4 = KeyDict_get_extmark {
            is_set__get_extmark_: 0 as OptionalKeys,
            details: false,
            hl_name: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg4 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_get_extmark_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"ns_id\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        ret = nvim_buf_get_extmark_by_id(
                            arg1,
                            arg2,
                            arg3,
                            &raw mut arg4,
                            &raw mut arena,
                            &raw mut err,
                        );
                        nlua_push_Array(
                            lstate,
                            ret,
                            kNluaPushSpecial as ::core::ffi::c_int
                                | kNluaPushFreeRefs as ::core::ffi::c_int,
                        );
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg4 as *mut ::core::ffi::c_void,
            get_extmark_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_get_extmarks(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg2: Integer = 0;
    let mut arg4: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg3: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg5: KeyDict_get_extmarks = KeyDict_get_extmarks {
        is_set__get_extmarks_: 0,
        limit: 0,
        details: false,
        hl_name: false,
        overlap: false,
        type_0: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 5 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 5 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_get_extmarks\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg5 = KeyDict_get_extmarks {
            is_set__get_extmarks_: 0 as OptionalKeys,
            limit: 0,
            details: false,
            hl_name: false,
            overlap: false,
            type_0: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg5 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_get_extmarks_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg4 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"end\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg3 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"start\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"ns_id\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            save_active_lstate = active_lstate.get();
                            active_lstate.set(lstate);
                            ret = nvim_buf_get_extmarks(
                                arg1,
                                arg2,
                                arg3,
                                arg4,
                                &raw mut arg5,
                                &raw mut arena,
                                &raw mut err,
                            );
                            nlua_push_Array(
                                lstate,
                                ret,
                                kNluaPushSpecial as ::core::ffi::c_int
                                    | kNluaPushFreeRefs as ::core::ffi::c_int,
                            );
                            active_lstate.set(save_active_lstate);
                        }
                    }
                    api_luarefs_free_object(arg3);
                }
                api_luarefs_free_object(arg4);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg5 as *mut ::core::ffi::c_void,
            get_extmarks_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_set_extmark(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg4: Integer = 0;
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut arg5: KeyDict_set_extmark = KeyDict_set_extmark {
        is_set__set_extmark_: 0,
        id: 0,
        end_line: 0,
        end_row: 0,
        end_col: 0,
        hl_group: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        virt_text: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        virt_text_pos: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        virt_text_win_col: 0,
        virt_text_hide: false,
        virt_text_repeat_linebreak: false,
        hl_eol: false,
        hl_mode: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        invalidate: false,
        ephemeral: false,
        priority: 0,
        right_gravity: false,
        end_right_gravity: false,
        virt_lines: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        virt_lines_above: false,
        virt_lines_leftcol: false,
        virt_lines_overflow: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        strict: false,
        sign_text: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        sign_hl_group: 0,
        number_hl_group: 0,
        line_hl_group: 0,
        cursorline_hl_group: 0,
        conceal: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        conceal_lines: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        spell: false,
        ui_watched: false,
        undo_restore: false,
        url: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        scoped: false,
        _subpriority: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 5 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 5 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_set_extmark\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg5 = KeyDict_set_extmark {
            is_set__set_extmark_: 0 as OptionalKeys,
            id: 0,
            end_line: 0,
            end_row: 0,
            end_col: 0,
            hl_group: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            virt_text: Array {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<Object>(),
            },
            virt_text_pos: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            virt_text_win_col: 0,
            virt_text_hide: false,
            virt_text_repeat_linebreak: false,
            hl_eol: false,
            hl_mode: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            invalidate: false,
            ephemeral: false,
            priority: 0,
            right_gravity: false,
            end_right_gravity: false,
            virt_lines: Array {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<Object>(),
            },
            virt_lines_above: false,
            virt_lines_leftcol: false,
            virt_lines_overflow: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            strict: false,
            sign_text: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            sign_hl_group: 0,
            number_hl_group: 0,
            line_hl_group: 0,
            cursorline_hl_group: 0,
            conceal: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            conceal_lines: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            spell: false,
            ui_watched: false,
            undo_restore: false,
            url: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            scoped: false,
            _subpriority: 0,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg5 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_set_extmark_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg4 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"col\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"line\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"ns_id\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            save_active_lstate = active_lstate.get();
                            active_lstate.set(lstate);
                            ret = nvim_buf_set_extmark(
                                arg1,
                                arg2,
                                arg3,
                                arg4,
                                &raw mut arg5,
                                &raw mut err,
                            );
                            nlua_push_Integer(
                                lstate,
                                ret,
                                kNluaPushSpecial as ::core::ffi::c_int
                                    | kNluaPushFreeRefs as ::core::ffi::c_int,
                            );
                            active_lstate.set(save_active_lstate);
                        }
                    }
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg5 as *mut ::core::ffi::c_void,
            set_extmark_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_del_extmark(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_del_extmark\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"ns_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret = nvim_buf_del_extmark(arg1, arg2, arg3, &raw mut err);
                    nlua_push_Boolean(
                        lstate,
                        ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_buf_clear_namespace(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg4: Integer = 0;
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_buf_clear_namespace\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg4 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"line_end\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param = b"line_start\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"ns_id\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        nvim_buf_clear_namespace(arg1, arg2, arg3, arg4, &raw mut err);
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_decoration_provider(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: KeyDict_set_decoration_provider = KeyDict_set_decoration_provider {
        is_set__set_decoration_provider_: 0,
        on_start: 0,
        on_buf: 0,
        on_win: 0,
        on_line: 0,
        on_range: 0,
        on_end: 0,
        _on_hl_def: 0,
        _on_spell_nav: 0,
        _on_conceal_line: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_decoration_provider\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_set_decoration_provider {
            is_set__set_decoration_provider_: 0 as OptionalKeys,
            on_start: 0,
            on_buf: 0,
            on_win: 0,
            on_line: 0,
            on_range: 0,
            on_end: 0,
            _on_hl_def: 0,
            _on_spell_nav: 0,
            _on_conceal_line: 0,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_set_decoration_provider_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"ns_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_set_decoration_provider(arg1, &raw mut arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            set_decoration_provider_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__buf_debug_extmarks(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg3: Boolean = false;
    let mut arg2: Boolean = false;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__buf_debug_extmarks\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"dot\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"keys\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret = nvim__buf_debug_extmarks(arg1, arg2, arg3, &raw mut err);
                    nlua_push_String(
                        lstate,
                        ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                    api_free_string(ret);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__ns_set(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: KeyDict_ns_opts = KeyDict_ns_opts {
        is_set__ns_opts_: 0,
        wins: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__ns_set\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_ns_opts {
            is_set__ns_opts_: 0 as OptionalKeys,
            wins: Array {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<Object>(),
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_ns_opts_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"ns_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim__ns_set(arg1, &raw mut arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            ns_opts_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__ns_get(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: KeyDict_ns_opts = KeyDict_ns_opts {
        is_set__ns_opts_: 0,
        wins: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__ns_get\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"ns_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim__ns_get(arg1, &raw mut arena, &raw mut err);
            nlua_push_keydict(
                lstate,
                &raw mut ret as *mut ::core::ffi::c_void,
                ns_opts_table.ptr() as *mut KeySetLink,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_option_value(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: KeyDict_option = KeyDict_option {
        is_set__option_: 0,
        scope: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        win: 0,
        buf: 0,
        filetype: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_option_value\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_option {
            is_set__option_: 0 as OptionalKeys,
            scope: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            win: 0,
            buf: 0,
            filetype: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_option_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_get_option_value(arg1, &raw mut arg2, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
                api_free_object(ret);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            option_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_option_value(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg3: KeyDict_option = KeyDict_option {
        is_set__option_: 0,
        scope: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        win: 0,
        buf: 0,
        filetype: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_option_value\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = KeyDict_option {
            is_set__option_: 0 as OptionalKeys,
            scope: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            win: 0,
            buf: 0,
            filetype: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg3 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_option_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg2 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"name\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_set_option_value(
                        LUA_INTERNAL_CALL,
                        arg1,
                        arg2,
                        &raw mut arg3,
                        &raw mut err,
                    );
                    active_lstate.set(save_active_lstate);
                }
                api_luarefs_free_object(arg2);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg3 as *mut ::core::ffi::c_void,
            option_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_all_options_info(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_all_options_info\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_get_all_options_info(&raw mut arena, &raw mut err);
        nlua_push_Dict(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_option_info2(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg2: KeyDict_option = KeyDict_option {
        is_set__option_: 0,
        scope: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        win: 0,
        buf: 0,
        filetype: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_option_info2\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_option {
            is_set__option_: 0 as OptionalKeys,
            scope: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            win: 0,
            buf: 0,
            filetype: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_option_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_get_option_info2(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    0 as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            option_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_tabpage_list_wins(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_tabpage_list_wins\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"tabpage\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_tabpage_list_wins(arg1, &raw mut arena, &raw mut err);
            nlua_push_Array(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_tabpage_get_var(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_tabpage_get_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"tabpage\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_tabpage_get_var(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_tabpage_set_var(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg3: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_tabpage_set_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"tabpage\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_tabpage_set_var(arg1, arg2, arg3, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
            api_luarefs_free_object(arg3);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_tabpage_del_var(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_tabpage_del_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"tabpage\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_tabpage_del_var(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_tabpage_get_win(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Window = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_tabpage_get_win\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"tabpage\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_tabpage_get_win(arg1, &raw mut err);
            nlua_push_handle(
                lstate,
                ret as handle_T,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_tabpage_set_win(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: Window = 0;
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_tabpage_set_win\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"tabpage\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_tabpage_set_win(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_tabpage_get_number(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_tabpage_get_number\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"tabpage\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_tabpage_get_number(arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_tabpage_is_valid(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_tabpage_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"tabpage\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_tabpage_is_valid(arg1);
            nlua_push_Boolean(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_open_tabpage(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Tabpage = 0;
    let mut arg3: KeyDict_tabpage_config = KeyDict_tabpage_config {
        is_set__tabpage_config_: 0,
        after: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_open_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0 {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg3 = KeyDict_tabpage_config {
                is_set__tabpage_config_: 0 as OptionalKeys,
                after: 0,
            };
            nlua_pop_keydict(
                lstate,
                &raw mut arg3 as *mut ::core::ffi::c_void,
                Some(
                    KeyDict_tabpage_config_get_field
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            size_t,
                        ) -> *mut KeySetLink,
                ),
                &raw mut err_param,
                &raw mut arena,
                &raw mut err,
            );
            if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"enter\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        ret = nvim_open_tabpage(arg1, arg2, &raw mut arg3, &raw mut err);
                        nlua_push_handle(
                            lstate,
                            ret as handle_T,
                            0 as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
                        );
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
            api_luarefs_free_keydict(
                &raw mut arg3 as *mut ::core::ffi::c_void,
                tabpage_config_table.ptr() as *mut KeySetLink,
            );
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_ui_send(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_ui_send\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"content\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_ui_send(LUA_INTERNAL_CALL, arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_hl_id_by_name(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_hl_id_by_name\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_hl_id_by_name(arg1);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_hl(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg2: KeyDict_get_highlight = KeyDict_get_highlight {
        is_set__get_highlight_: 0,
        id: 0,
        name: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        link: false,
        create: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_hl\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_get_highlight {
            is_set__get_highlight_: 0 as OptionalKeys,
            id: 0,
            name: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            link: false,
            create: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_get_highlight_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"ns_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_get_hl(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    0 as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            get_highlight_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_hl(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg3: KeyDict_highlight = KeyDict_highlight {
        is_set__highlight_: 0,
        altfont: false,
        blink: false,
        bold: false,
        conceal: false,
        dim: false,
        italic: false,
        nocombine: false,
        overline: false,
        reverse: false,
        standout: false,
        strikethrough: false,
        undercurl: false,
        underdashed: false,
        underdotted: false,
        underdouble: false,
        underline: false,
        default_: false,
        cterm: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
        foreground: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        fg: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        background: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        bg: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        ctermfg: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        ctermbg: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        special: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        sp: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        link: 0,
        link_global: 0,
        fallback: false,
        blend: 0,
        fg_indexed: false,
        bg_indexed: false,
        force: false,
        update: false,
        url: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_hl\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = KeyDict_highlight {
            is_set__highlight_: 0 as OptionalKeys,
            altfont: false,
            blink: false,
            bold: false,
            conceal: false,
            dim: false,
            italic: false,
            nocombine: false,
            overline: false,
            reverse: false,
            standout: false,
            strikethrough: false,
            undercurl: false,
            underdashed: false,
            underdotted: false,
            underdouble: false,
            underline: false,
            default_: false,
            cterm: Dict {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            },
            foreground: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            fg: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            background: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            bg: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            ctermfg: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            ctermbg: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            special: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            sp: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            link: 0,
            link_global: 0,
            fallback: false,
            blend: 0,
            fg_indexed: false,
            bg_indexed: false,
            force: false,
            update: false,
            url: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg3 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_highlight_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"ns_id\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_set_hl(LUA_INTERNAL_CALL, arg1, arg2, &raw mut arg3, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg3 as *mut ::core::ffi::c_void,
            highlight_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_hl_ns(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut arg1: KeyDict_get_ns = KeyDict_get_ns {
        is_set__get_ns_: 0,
        winid: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_hl_ns\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = KeyDict_get_ns {
            is_set__get_ns_: 0 as OptionalKeys,
            winid: 0,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg1 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_get_ns_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_hl_ns(&raw mut arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                0 as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
        api_luarefs_free_keydict(
            &raw mut arg1 as *mut ::core::ffi::c_void,
            get_ns_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_hl_ns(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_hl_ns\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"ns_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_set_hl_ns(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_hl_ns_fast(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"ns_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_set_hl_ns_fast(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_feedkeys(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg3: Boolean = false;
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_feedkeys\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"escape_ks\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"mode\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"keys\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_feedkeys(arg1, arg2, arg3);
                    active_lstate.set(save_active_lstate);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_input(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"keys\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_input(LUA_INTERNAL_CALL, arg1);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_input_mouse(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg6: Integer = 0;
    let mut arg5: Integer = 0;
    let mut arg4: Integer = 0;
    let mut arg3: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 6 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 6 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg6 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"col\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg5 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"row\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg4 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"grid\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg3 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"modifier\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"action\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                            if err.type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                err_param = b"button\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            } else {
                                save_active_lstate = active_lstate.get();
                                active_lstate.set(lstate);
                                nvim_input_mouse(arg1, arg2, arg3, arg4, arg5, arg6, &raw mut err);
                                active_lstate.set(save_active_lstate);
                            }
                        }
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_replace_termcodes(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg4: Boolean = false;
    let mut arg3: Boolean = false;
    let mut arg2: Boolean = false;
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_replace_termcodes\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg4 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"special\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg3 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"do_lt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"from_part\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"str\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        ret = nvim_replace_termcodes(arg1, arg2, arg3, arg4);
                        nlua_push_String(
                            lstate,
                            ret,
                            kNluaPushSpecial as ::core::ffi::c_int
                                | kNluaPushFreeRefs as ::core::ffi::c_int,
                        );
                        active_lstate.set(save_active_lstate);
                        api_free_string(ret);
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_strwidth(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_strwidth\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"text\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_strwidth(arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_list_runtime_paths(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_list_runtime_paths\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_list_runtime_paths(&raw mut arena, &raw mut err);
        nlua_push_Array(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__runtime_inspect(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__runtime_inspect\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim__runtime_inspect(&raw mut arena);
        nlua_push_Array(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_runtime_file(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"all\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_get_runtime_file(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Array(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__get_lib_dir(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__get_lib_dir\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim__get_lib_dir();
        nlua_push_String(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
        api_free_string(ret);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__get_runtime(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg3: KeyDict_runtime = KeyDict_runtime {
        is_lua: false,
        do_source: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg3 = KeyDict_runtime {
            is_lua: false,
            do_source: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg3 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_runtime_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"all\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"pat\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret =
                        nvim__get_runtime(arg1, arg2, &raw mut arg3, &raw mut arena, &raw mut err);
                    nlua_push_Array(
                        lstate,
                        ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg3 as *mut ::core::ffi::c_void,
            runtime_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_current_dir(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_current_dir\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"dir\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_set_current_dir(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_current_line(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_current_line\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_get_current_line(&raw mut arena, &raw mut err);
        nlua_push_String(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_current_line(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_current_line\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0 {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"line\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_set_current_line(arg1, &raw mut arena, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_del_current_line(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_del_current_line\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0 {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_del_current_line(&raw mut arena, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_var(arg1, &raw mut arena, &raw mut err);
            nlua_push_Object(
                lstate,
                &raw mut ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_set_var(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
            api_luarefs_free_object(arg2);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_del_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_del_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_del_var(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_vvar(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_vvar\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_vvar(arg1, &raw mut arena, &raw mut err);
            nlua_push_Object(
                lstate,
                &raw mut ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_vvar(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_vvar\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_set_vvar(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
            api_luarefs_free_object(arg2);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_echo(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg3: KeyDict_echo_opts = KeyDict_echo_opts {
        is_set__echo_opts_: 0,
        err: false,
        verbose: false,
        _truncate: false,
        kind: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        id: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        title: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        status: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        percent: 0,
        source: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        data: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_echo\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = KeyDict_echo_opts {
            is_set__echo_opts_: 0 as OptionalKeys,
            err: false,
            verbose: false,
            _truncate: false,
            kind: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            id: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            title: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            status: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            percent: 0,
            source: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            data: Dict {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg3 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_echo_opts_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"history\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"chunks\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret = nvim_echo(arg1, arg2, &raw mut arg3, &raw mut err);
                    nlua_push_Object(
                        lstate,
                        &raw mut ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg3 as *mut ::core::ffi::c_void,
            echo_opts_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_list_bufs(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_list_bufs\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_list_bufs(&raw mut arena);
        nlua_push_Array(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_current_buf(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Buffer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_current_buf\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_get_current_buf();
        nlua_push_handle(
            lstate,
            ret as handle_T,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_current_buf(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_current_buf\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if text_locked() {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                get_text_locked_msg(),
            );
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_set_current_buf(arg1, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_list_wins(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_list_wins\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_list_wins(&raw mut arena);
        nlua_push_Array(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_current_win(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Window = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_current_win\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_get_current_win();
        nlua_push_handle(
            lstate,
            ret as handle_T,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_current_win(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_current_win\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if text_locked() {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                get_text_locked_msg(),
            );
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_set_current_win(arg1, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_create_buf(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: Boolean = false;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Buffer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_create_buf\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"scratch\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"listed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_create_buf(arg1, arg2, &raw mut err);
                nlua_push_handle(
                    lstate,
                    ret as handle_T,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_open_term(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut arg2: KeyDict_open_term = KeyDict_open_term {
        is_set__open_term_: 0,
        on_input: 0,
        force_crlf: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_open_term\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0 {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg2 = KeyDict_open_term {
                is_set__open_term_: 0 as OptionalKeys,
                on_input: 0,
                force_crlf: false,
            };
            nlua_pop_keydict(
                lstate,
                &raw mut arg2 as *mut ::core::ffi::c_void,
                Some(
                    KeyDict_open_term_get_field
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            size_t,
                        ) -> *mut KeySetLink,
                ),
                &raw mut err_param,
                &raw mut arena,
                &raw mut err,
            );
            if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret = nvim_open_term(arg1, &raw mut arg2, &raw mut err);
                    nlua_push_Integer(
                        lstate,
                        ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                }
            }
            api_luarefs_free_keydict(
                &raw mut arg2 as *mut ::core::ffi::c_void,
                open_term_table.ptr() as *mut KeySetLink,
            );
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_chan_send(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_chan_send\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"data\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"chan\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_chan_send(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_list_tabpages(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_list_tabpages\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_list_tabpages(&raw mut arena);
        nlua_push_Array(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_current_tabpage(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Tabpage = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_current_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_get_current_tabpage();
        nlua_push_handle(
            lstate,
            ret as handle_T,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_current_tabpage(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Tabpage = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_current_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if text_locked() {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                get_text_locked_msg(),
            );
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"tabpage\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_set_current_tabpage(arg1, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_paste(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg3: Integer = 0;
    let mut arg2: Boolean = false;
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_paste\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0 {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"phase\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"crlf\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"data\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        ret = nvim_paste(
                            LUA_INTERNAL_CALL,
                            arg1,
                            arg2,
                            arg3,
                            &raw mut arena,
                            &raw mut err,
                        );
                        nlua_push_Boolean(
                            lstate,
                            ret,
                            kNluaPushSpecial as ::core::ffi::c_int
                                | kNluaPushFreeRefs as ::core::ffi::c_int,
                        );
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_put(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg4: Boolean = false;
    let mut arg3: Boolean = false;
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_put\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0 {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg4 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"follow\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg3 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"after\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"type\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        arg1 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            err_param = b"lines\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            save_active_lstate = active_lstate.get();
                            active_lstate.set(lstate);
                            nvim_put(arg1, arg2, arg3, arg4, &raw mut arena, &raw mut err);
                            active_lstate.set(save_active_lstate);
                        }
                    }
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_color_by_name(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_color_by_name\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_color_by_name(arg1);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_color_map(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_color_map\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_get_color_map(&raw mut arena);
        nlua_push_Dict(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_context(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg1: KeyDict_context = KeyDict_context {
        is_set__context_: 0,
        types: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_context\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = KeyDict_context {
            is_set__context_: 0 as OptionalKeys,
            types: Array {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<Object>(),
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg1 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_context_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_context(&raw mut arg1, &raw mut arena, &raw mut err);
            nlua_push_Dict(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
        api_luarefs_free_keydict(
            &raw mut arg1 as *mut ::core::ffi::c_void,
            context_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_load_context(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_load_context\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Dict(lstate, false_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"dict\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_load_context(arg1, &raw mut err);
            nlua_push_Object(
                lstate,
                &raw mut ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_mode(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_get_mode(&raw mut arena);
        nlua_push_Dict(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_keymap(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_keymap\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"mode\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_keymap(arg1, &raw mut arena);
            nlua_push_Array(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_set_keymap(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg3: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg4: KeyDict_keymap = KeyDict_keymap {
        is_set__keymap_: 0,
        noremap: false,
        nowait: false,
        silent: false,
        script: false,
        expr: false,
        unique: false,
        callback: 0,
        desc: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        replace_keycodes: false,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_set_keymap\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg4 = KeyDict_keymap {
            is_set__keymap_: 0 as OptionalKeys,
            noremap: false,
            nowait: false,
            silent: false,
            script: false,
            expr: false,
            unique: false,
            callback: 0,
            desc: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            replace_keycodes: false,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg4 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_keymap_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg3 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"rhs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"lhs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"mode\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        nvim_set_keymap(
                            LUA_INTERNAL_CALL,
                            arg1,
                            arg2,
                            arg3,
                            &raw mut arg4,
                            &raw mut err,
                        );
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg4 as *mut ::core::ffi::c_void,
            keymap_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_del_keymap(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_del_keymap\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"lhs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"mode\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_del_keymap(LUA_INTERNAL_CALL, arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_chan_info(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_chan_info\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"chan\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_chan_info(LUA_INTERNAL_CALL, arg1, &raw mut arena, &raw mut err);
            nlua_push_Dict(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_list_chans(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_list_chans\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_list_chans(&raw mut arena);
        nlua_push_Array(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__id(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__id\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"obj\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim__id(arg1, &raw mut arena);
            nlua_push_Object(
                lstate,
                &raw mut ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
            api_luarefs_free_object(arg1);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__id_array(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__id_array\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"arr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim__id_array(arg1, &raw mut arena);
            nlua_push_Array(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__id_dict(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__id_dict\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Dict(lstate, false_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"dct\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim__id_dict(arg1, &raw mut arena);
            nlua_push_Dict(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__id_float(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Float = 0.;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Float = 0.;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__id_float\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Float(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"flt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim__id_float(arg1);
            nlua_push_Float(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__stats(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__stats\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim__stats(&raw mut arena);
        nlua_push_Dict(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_list_uis(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_list_uis\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        ret = nvim_list_uis(&raw mut arena);
        nlua_push_Array(
            lstate,
            ret,
            kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
        );
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_proc_children(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_proc_children\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"pid\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_proc_children(arg1, &raw mut arena, &raw mut err);
            nlua_push_Array(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_proc(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_proc\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"pid\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_get_proc(arg1, &raw mut arena, &raw mut err);
            nlua_push_Object(
                lstate,
                &raw mut ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_select_popupmenu_item(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg3: Boolean = false;
    let mut arg2: Boolean = false;
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg4: KeyDict_empty = KeyDict_empty { is_set__empty_: 0 };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 4 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 4 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_select_popupmenu_item\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg4 = KeyDict_empty {
            is_set__empty_: 0 as OptionalKeys,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg4 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_empty_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg3 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"finish\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"insert\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"item\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        nvim_select_popupmenu_item(arg1, arg2, arg3, &raw mut arg4, &raw mut err);
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg4 as *mut ::core::ffi::c_void,
            empty_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__inspect_cell(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg3: Integer = 0;
    let mut arg2: Integer = 0;
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__inspect_cell\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"col\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"row\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"grid\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret = nvim__inspect_cell(arg1, arg2, arg3, &raw mut arena, &raw mut err);
                    nlua_push_Array(
                        lstate,
                        ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__screenshot(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"path\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim__screenshot(arg1);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__invalidate_glyph_cache(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 0 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 0 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__invalidate_glyph_cache\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save_active_lstate = active_lstate.get();
        active_lstate.set(lstate);
        nvim__invalidate_glyph_cache();
        active_lstate.set(save_active_lstate);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__unpack(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"str\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim__unpack(arg1, &raw mut arena, &raw mut err);
            nlua_push_Object(
                lstate,
                &raw mut ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_del_mark(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_del_mark\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_del_mark(arg1, &raw mut err);
            nlua_push_Boolean(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_get_mark(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg2: KeyDict_empty = KeyDict_empty { is_set__empty_: 0 };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_get_mark\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_empty {
            is_set__empty_: 0 as OptionalKeys,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_empty_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_get_mark(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                nlua_push_Array(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            empty_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_eval_statusline(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg2: KeyDict_eval_statusline = KeyDict_eval_statusline {
        is_set__eval_statusline_: 0,
        winid: 0,
        maxwidth: 0,
        fillchar: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        highlights: false,
        use_winbar: false,
        use_tabline: false,
        use_statuscol_lnum: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg2 = KeyDict_eval_statusline {
            is_set__eval_statusline_: 0 as OptionalKeys,
            winid: 0,
            maxwidth: 0,
            fillchar: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            highlights: false,
            use_winbar: false,
            use_tabline: false,
            use_statuscol_lnum: 0,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_eval_statusline_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"str\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_eval_statusline(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            eval_statusline_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__complete_set(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Integer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg2: KeyDict_complete_set = KeyDict_complete_set {
        is_set__complete_set_: 0,
        info: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__complete_set\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_complete_set {
            is_set__complete_set_: 0 as OptionalKeys,
            info: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_complete_set_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"index\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim__complete_set(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            complete_set_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim__redraw(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg1: KeyDict_redraw = KeyDict_redraw {
        is_set__redraw_: 0,
        flush: false,
        cursor: false,
        valid: false,
        statuscolumn: false,
        statusline: false,
        tabline: false,
        winbar: false,
        range: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        win: 0,
        buf: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim__redraw\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = KeyDict_redraw {
            is_set__redraw_: 0 as OptionalKeys,
            flush: false,
            cursor: false,
            valid: false,
            statuscolumn: false,
            statusline: false,
            tabline: false,
            winbar: false,
            range: Array {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<Object>(),
            },
            win: 0,
            buf: 0,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg1 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_redraw_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim__redraw(&raw mut arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
        api_luarefs_free_keydict(
            &raw mut arg1 as *mut ::core::ffi::c_void,
            redraw_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_exec2(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg2: KeyDict_exec_opts = KeyDict_exec_opts { output: false };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_exec2\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_exec_opts { output: false };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_exec_opts_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"src\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_exec2(LUA_INTERNAL_CALL, arg1, &raw mut arg2, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    0 as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
                api_free_dict(ret);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            exec_opts_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_command(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_command\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"cmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            nvim_command(arg1, &raw mut err);
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_eval(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_eval\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"expr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_eval(arg1, &raw mut arena, &raw mut err);
            nlua_push_Object(
                lstate,
                &raw mut ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_call_function(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_call_function\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"args\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"fn\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_call_function(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_call_dict_function(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg3: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_call_dict_function\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"args\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"fn\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"dict\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret = nvim_call_dict_function(arg1, arg2, arg3, &raw mut arena, &raw mut err);
                    nlua_push_Object(
                        lstate,
                        &raw mut ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                    api_luarefs_free_object(arg1);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_parse_expression(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg3: Boolean = false;
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        arg3 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"hl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"flags\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"expr\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    ret = nvim_parse_expression(arg1, arg2, arg3, &raw mut arena, &raw mut err);
                    nlua_push_Dict(
                        lstate,
                        ret,
                        kNluaPushSpecial as ::core::ffi::c_int
                            | kNluaPushFreeRefs as ::core::ffi::c_int,
                    );
                    active_lstate.set(save_active_lstate);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_open_win(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: Buffer = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Window = 0;
    let mut arg3: KeyDict_win_config = KeyDict_win_config {
        is_set__win_config_: 0,
        external: false,
        fixed: false,
        focusable: false,
        footer: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        footer_pos: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        hide: false,
        height: 0,
        mouse: false,
        relative: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        row: 0.,
        style: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        noautocmd: false,
        vertical: false,
        win: 0,
        width: 0,
        zindex: 0,
        anchor: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        border: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        bufpos: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        col: 0.,
        split: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        title: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        title_pos: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        _cmdline_offset: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_open_win\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0 {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg3 = KeyDict_win_config {
                is_set__win_config_: 0 as OptionalKeys,
                external: false,
                fixed: false,
                focusable: false,
                footer: Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                },
                footer_pos: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                hide: false,
                height: 0,
                mouse: false,
                relative: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                row: 0.,
                style: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                noautocmd: false,
                vertical: false,
                win: 0,
                width: 0,
                zindex: 0,
                anchor: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                border: Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                },
                bufpos: Array {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<Object>(),
                },
                col: 0.,
                split: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                title: Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                },
                title_pos: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
                _cmdline_offset: 0,
            };
            nlua_pop_keydict(
                lstate,
                &raw mut arg3 as *mut ::core::ffi::c_void,
                Some(
                    KeyDict_win_config_get_field
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            size_t,
                        ) -> *mut KeySetLink,
                ),
                &raw mut err_param,
                &raw mut arena,
                &raw mut err,
            );
            if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param = b"enter\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        err_param = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    } else {
                        save_active_lstate = active_lstate.get();
                        active_lstate.set(lstate);
                        ret = nvim_open_win(arg1, arg2, &raw mut arg3, &raw mut err);
                        nlua_push_handle(
                            lstate,
                            ret as handle_T,
                            kNluaPushSpecial as ::core::ffi::c_int
                                | kNluaPushFreeRefs as ::core::ffi::c_int,
                        );
                        active_lstate.set(save_active_lstate);
                    }
                }
            }
            api_luarefs_free_keydict(
                &raw mut arg3 as *mut ::core::ffi::c_void,
                win_config_table.ptr() as *mut KeySetLink,
            );
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_set_config(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg2: KeyDict_win_config = KeyDict_win_config {
        is_set__win_config_: 0,
        external: false,
        fixed: false,
        focusable: false,
        footer: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        footer_pos: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        hide: false,
        height: 0,
        mouse: false,
        relative: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        row: 0.,
        style: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        noautocmd: false,
        vertical: false,
        win: 0,
        width: 0,
        zindex: 0,
        anchor: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        border: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        bufpos: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        col: 0.,
        split: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        title: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        title_pos: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        _cmdline_offset: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_set_config\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_win_config {
            is_set__win_config_: 0 as OptionalKeys,
            external: false,
            fixed: false,
            focusable: false,
            footer: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            footer_pos: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            hide: false,
            height: 0,
            mouse: false,
            relative: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            row: 0.,
            style: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            noautocmd: false,
            vertical: false,
            win: 0,
            width: 0,
            zindex: 0,
            anchor: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            border: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            bufpos: Array {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<Object>(),
            },
            col: 0.,
            split: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            title: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            title_pos: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            _cmdline_offset: 0,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_win_config_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_win_set_config(arg1, &raw mut arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            win_config_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_config(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: KeyDict_win_config = KeyDict_win_config {
        is_set__win_config_: 0,
        external: false,
        fixed: false,
        focusable: false,
        footer: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        footer_pos: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        hide: false,
        height: 0,
        mouse: false,
        relative: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        row: 0.,
        style: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        noautocmd: false,
        vertical: false,
        win: 0,
        width: 0,
        zindex: 0,
        anchor: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        border: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        bufpos: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        col: 0.,
        split: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        title: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        title_pos: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        _cmdline_offset: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_config\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_get_config(arg1, &raw mut arena, &raw mut err);
            nlua_push_keydict(
                lstate,
                &raw mut ret as *mut ::core::ffi::c_void,
                win_config_table.ptr() as *mut KeySetLink,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_buf(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Buffer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_buf\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_get_buf(arg1, &raw mut err);
            nlua_push_handle(
                lstate,
                ret as handle_T,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_set_buf(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Buffer = 0;
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_set_buf\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0 {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg2 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_win_set_buf(arg1, arg2, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_cursor(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_cursor\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_get_cursor(arg1, &raw mut arena, &raw mut err);
            nlua_push_Array(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_set_cursor(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_set_cursor\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"pos\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_win_set_cursor(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_height(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_height\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_get_height(arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_set_height(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg2: Integer = 0;
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_set_height\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"height\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_win_set_height(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_width(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_width\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_get_width(arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_set_width(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Integer = 0;
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_set_width\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"width\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_win_set_width(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_win_get_var(arg1, arg2, &raw mut arena, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_set_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut arg3: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_set_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg3 = nlua_pop_Object(lstate, true_0 != 0, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"value\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_win_set_var(arg1, arg2, arg3, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
            api_luarefs_free_object(arg3);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_del_var(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_del_var\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_String(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"name\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_win_del_var(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_position(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_position\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_get_position(arg1, &raw mut arena, &raw mut err);
            nlua_push_Array(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_tabpage(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Tabpage = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_get_tabpage(arg1, &raw mut err);
            nlua_push_handle(
                lstate,
                ret as handle_T,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_get_number(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Integer = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_get_number\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_get_number(arg1, &raw mut err);
            nlua_push_Integer(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_is_valid(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Boolean = false;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            save_active_lstate = active_lstate.get();
            active_lstate.set(lstate);
            ret = nvim_win_is_valid(arg1);
            nlua_push_Boolean(
                lstate,
                ret,
                kNluaPushSpecial as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
            );
            active_lstate.set(save_active_lstate);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_hide(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 1 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_hide\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0 {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_win_hide(arg1, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_close(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Boolean = false;
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_close\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if textlock != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0 {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_textlock as *const ::core::ffi::c_char,
            );
        } else {
            arg2 = nlua_pop_Boolean(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"force\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    err_param =
                        b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    save_active_lstate = active_lstate.get();
                    active_lstate.set(lstate);
                    nvim_win_close(arg1, arg2, &raw mut err);
                    active_lstate.set(save_active_lstate);
                }
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_call(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut arg2: LuaRef = 0;
    let mut arg1: Window = 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_call\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_LuaRef(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param = b"fun\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_win_call(arg1, arg2, &raw mut err);
                nlua_push_Object(
                    lstate,
                    &raw mut ret,
                    kNluaPushSpecial as ::core::ffi::c_int
                        | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
            api_free_luaref(arg2);
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_set_hl_ns(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut arg2: Integer = 0;
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_set_hl_ns\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = nlua_pop_Integer(lstate, &raw mut arena, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            err_param =
                b"ns_id\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                nvim_win_set_hl_ns(arg1, arg2, &raw mut err);
                active_lstate.set(save_active_lstate);
            }
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_api_nvim_win_text_height(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut arg1: Window = 0;
    let mut save_active_lstate: *mut lua_State = ::core::ptr::null_mut::<lua_State>();
    let mut ret: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut arg2: KeyDict_win_text_height = KeyDict_win_text_height {
        is_set__win_text_height_: 0,
        start_row: 0,
        end_row: 0,
        start_vcol: 0,
        end_vcol: 0,
        max_height: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lua_gettop(lstate) != 2 as ::core::ffi::c_int {
        api_set_error(
            &raw mut err,
            kErrorTypeValidation,
            b"Expected 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"nvim_win_text_height\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arg2 = KeyDict_win_text_height {
            is_set__win_text_height_: 0 as OptionalKeys,
            start_row: 0,
            end_row: 0,
            start_vcol: 0,
            end_vcol: 0,
            max_height: 0,
        };
        nlua_pop_keydict(
            lstate,
            &raw mut arg2 as *mut ::core::ffi::c_void,
            Some(
                KeyDict_win_text_height_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            &raw mut err_param,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            arg1 = nlua_pop_handle(lstate, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                err_param =
                    b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                save_active_lstate = active_lstate.get();
                active_lstate.set(lstate);
                ret = nvim_win_text_height(arg1, &raw mut arg2, &raw mut arena, &raw mut err);
                nlua_push_Dict(
                    lstate,
                    ret,
                    0 as ::core::ffi::c_int | kNluaPushFreeRefs as ::core::ffi::c_int,
                );
                active_lstate.set(save_active_lstate);
            }
        }
        api_luarefs_free_keydict(
            &raw mut arg2 as *mut ::core::ffi::c_void,
            win_text_height_table.ptr() as *mut KeySetLink,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        if !err_param.is_null() {
            lua_pushstring(
                lstate,
                b"Invalid '\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, b"': \0".as_ptr() as *const ::core::ffi::c_char);
        }
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(
            lstate,
            if !err_param.is_null() {
                5 as ::core::ffi::c_int
            } else {
                2 as ::core::ffi::c_int
            },
        );
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn nlua_add_api_functions(mut lstate: *mut lua_State) {
    lua_createtable(lstate, 0 as ::core::ffi::c_int, 181 as ::core::ffi::c_int);
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_autocmds
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_autocmds\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_create_autocmd
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_create_autocmd\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_del_autocmd as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_del_autocmd\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_clear_autocmds
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_clear_autocmds\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_create_augroup
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_create_augroup\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_del_augroup_by_id
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_del_augroup_by_id\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_del_augroup_by_name
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_del_augroup_by_name\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_exec_autocmds
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_exec_autocmds\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_line_count
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_line_count\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_attach as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_attach\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_lines
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_lines\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_lines
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_lines\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_text
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_text\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_text
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_text\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_offset
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_offset\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_changedtick
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_changedtick\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_keymap
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_keymap\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_keymap
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_keymap\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_del_keymap
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_del_keymap\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_del_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_del_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_name
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_name\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_name
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_name\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_is_loaded
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_is_loaded\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_delete as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_delete\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_is_valid
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_del_mark
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_del_mark\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_mark
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_mark\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_mark
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_mark\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_buf_call as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_call\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__buf_stats as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__buf_stats\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_parse_cmd as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_parse_cmd\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_cmd as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_cmd\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_create_user_command
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_create_user_command\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_del_user_command
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_del_user_command\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_create_user_command
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_create_user_command\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_del_user_command
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_del_user_command\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_commands
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_commands\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_commands
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_commands\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_exec as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_exec\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_command_output
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_command_output\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_number
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_number\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_clear_highlight
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_clear_highlight\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_add_highlight
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_add_highlight\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_virtual_text
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_virtual_text\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_hl_by_id
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_hl_by_id\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_hl_by_name
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_hl_by_name\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_option_info
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_option_info\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_option as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_option\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_option as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_option\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_option
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_option\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_option
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_option\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_option
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_option\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_set_option
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_set_option\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_out_write as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_out_write\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_err_write as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_err_write\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_err_writeln as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_err_writeln\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_notify as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_notify\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_create_namespace
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_create_namespace\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_namespaces
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_namespaces\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_extmark_by_id
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_extmark_by_id\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_get_extmarks
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_get_extmarks\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_set_extmark
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_set_extmark\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_del_extmark
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_del_extmark\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_buf_clear_namespace
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_buf_clear_namespace\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_decoration_provider
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_decoration_provider\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__buf_debug_extmarks
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__buf_debug_extmarks\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__ns_set as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__ns_set\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__ns_get as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__ns_get\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_option_value
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_option_value\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_option_value
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_option_value\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_all_options_info
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_all_options_info\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_option_info2
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_option_info2\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_tabpage_list_wins
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_tabpage_list_wins\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_tabpage_get_var
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_tabpage_get_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_tabpage_set_var
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_tabpage_set_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_tabpage_del_var
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_tabpage_del_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_tabpage_get_win
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_tabpage_get_win\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_tabpage_set_win
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_tabpage_set_win\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_tabpage_get_number
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_tabpage_get_number\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_tabpage_is_valid
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_tabpage_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_open_tabpage
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_open_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_ui_send as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_ui_send\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_hl_id_by_name
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_hl_id_by_name\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_get_hl as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_hl\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_set_hl as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_hl\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_get_hl_ns as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_hl_ns\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_set_hl_ns as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_hl_ns\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_hl_ns_fast
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_hl_ns_fast\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_feedkeys as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_feedkeys\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_input as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_input\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_input_mouse as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_input_mouse\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_replace_termcodes
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_replace_termcodes\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_strwidth as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_strwidth\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_list_runtime_paths
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_list_runtime_paths\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__runtime_inspect
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__runtime_inspect\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_runtime_file
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_runtime_file\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__get_lib_dir
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__get_lib_dir\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__get_runtime
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__get_runtime\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_current_dir
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_current_dir\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_current_line
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_current_line\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_current_line
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_current_line\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_del_current_line
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_del_current_line\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_get_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_set_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_del_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_del_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_get_vvar as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_vvar\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_set_vvar as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_vvar\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_echo as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_echo\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_list_bufs as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_list_bufs\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_current_buf
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_current_buf\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_current_buf
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_current_buf\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_list_wins as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_list_wins\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_current_win
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_current_win\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_current_win
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_current_win\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_create_buf as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_create_buf\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_open_term as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_open_term\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_chan_send as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_chan_send\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_list_tabpages
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_list_tabpages\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_current_tabpage
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_current_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_current_tabpage
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_current_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_paste as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_paste\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_put as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_put\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_color_by_name
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_color_by_name\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_color_map
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_color_map\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_context as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_context\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_load_context
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_load_context\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_get_mode as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_mode\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_keymap as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_keymap\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_set_keymap as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_set_keymap\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_del_keymap as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_del_keymap\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_chan_info
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_chan_info\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_list_chans as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_list_chans\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__id as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__id\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__id_array as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__id_array\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__id_dict as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__id_dict\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__id_float as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__id_float\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__stats as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__stats\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_list_uis as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_list_uis\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_get_proc_children
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_proc_children\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_get_proc as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_proc\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_select_popupmenu_item
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_select_popupmenu_item\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__inspect_cell
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__inspect_cell\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__screenshot as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__screenshot\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__invalidate_glyph_cache
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__invalidate_glyph_cache\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__unpack as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__unpack\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_del_mark as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_del_mark\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_get_mark as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_get_mark\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_eval_statusline
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_eval_statusline\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim__complete_set
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__complete_set\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim__redraw as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim__redraw\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_exec2 as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_exec2\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_command as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_command\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_eval as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_eval\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_call_function
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_call_function\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_call_dict_function
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_call_dict_function\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_parse_expression
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_parse_expression\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_open_win as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_open_win\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_set_config
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_set_config\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_config
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_config\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_buf as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_buf\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_set_buf as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_set_buf\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_cursor
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_cursor\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_set_cursor
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_set_cursor\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_height
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_height\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_set_height
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_set_height\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_width
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_width\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_set_width
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_set_width\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_set_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_set_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_del_var as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_del_var\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_position
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_position\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_tabpage
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_get_number
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_get_number\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_is_valid
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_is_valid\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_win_hide as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_hide\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_win_close as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_close\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_api_nvim_win_call as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_call\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_set_hl_ns
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_set_hl_ns\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_api_nvim_win_text_height
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nvim_win_text_height\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"api\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
