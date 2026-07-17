extern "C" {
    pub type lua_State;
    pub type TSLanguage;
    pub type TSParser;
    pub type TSTree;
    pub type TSQuery;
    pub type TSQueryCursor;
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
    fn lua_gettop(L: *mut lua_State) -> ::core::ffi::c_int;
    fn lua_settop(L: *mut lua_State, idx: ::core::ffi::c_int);
    fn lua_pushvalue(L: *mut lua_State, idx: ::core::ffi::c_int);
    fn lua_isstring(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn lua_type(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn lua_tointeger(L: *mut lua_State, idx: ::core::ffi::c_int) -> lua_Integer;
    fn lua_toboolean(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn lua_tolstring(
        L: *mut lua_State,
        idx: ::core::ffi::c_int,
        len: *mut size_t,
    ) -> *const ::core::ffi::c_char;
    fn lua_objlen(L: *mut lua_State, idx: ::core::ffi::c_int) -> size_t;
    fn lua_touserdata(L: *mut lua_State, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_void;
    fn lua_pushnil(L: *mut lua_State);
    fn lua_pushnumber(L: *mut lua_State, n: lua_Number);
    fn lua_pushinteger(L: *mut lua_State, n: lua_Integer);
    fn lua_pushlstring(L: *mut lua_State, s: *const ::core::ffi::c_char, l: size_t);
    fn lua_pushstring(L: *mut lua_State, s: *const ::core::ffi::c_char);
    fn lua_pushcclosure(L: *mut lua_State, fn_0: lua_CFunction, n: ::core::ffi::c_int);
    fn lua_pushboolean(L: *mut lua_State, b: ::core::ffi::c_int);
    fn lua_getfield(L: *mut lua_State, idx: ::core::ffi::c_int, k: *const ::core::ffi::c_char);
    fn lua_rawgeti(L: *mut lua_State, idx: ::core::ffi::c_int, n: ::core::ffi::c_int);
    fn lua_createtable(L: *mut lua_State, narr: ::core::ffi::c_int, nrec: ::core::ffi::c_int);
    fn lua_newuserdata(L: *mut lua_State, sz: size_t) -> *mut ::core::ffi::c_void;
    fn lua_getfenv(L: *mut lua_State, idx: ::core::ffi::c_int);
    fn lua_setfield(L: *mut lua_State, idx: ::core::ffi::c_int, k: *const ::core::ffi::c_char);
    fn lua_rawseti(L: *mut lua_State, idx: ::core::ffi::c_int, n: ::core::ffi::c_int);
    fn lua_setmetatable(L: *mut lua_State, objindex: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn lua_setfenv(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn lua_pcall(
        L: *mut lua_State,
        nargs: ::core::ffi::c_int,
        nresults: ::core::ffi::c_int,
        errfunc: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn lua_error(L: *mut lua_State) -> ::core::ffi::c_int;
    fn lua_concat(L: *mut lua_State, n: ::core::ffi::c_int);
    fn luaL_register(L: *mut lua_State, libname: *const ::core::ffi::c_char, l: *const luaL_Reg);
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
    fn luaL_checknumber(L: *mut lua_State, numArg: ::core::ffi::c_int) -> lua_Number;
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
    fn luaL_error(L: *mut lua_State, fmt: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn luaL_ref(L: *mut lua_State, t: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn luaL_unref(L: *mut lua_State, t: ::core::ffi::c_int, ref_0: ::core::ffi::c_int);
    fn abort() -> !;
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
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn ts_parser_new() -> *mut TSParser;
    fn ts_parser_delete(self_0: *mut TSParser);
    fn ts_parser_language(self_0: *const TSParser) -> *const TSLanguage;
    fn ts_parser_set_language(self_0: *mut TSParser, language: *const TSLanguage) -> bool;
    fn ts_parser_set_included_ranges(
        self_0: *mut TSParser,
        ranges: *const TSRange,
        count: uint32_t,
    ) -> bool;
    fn ts_parser_included_ranges(self_0: *const TSParser, count: *mut uint32_t) -> *const TSRange;
    fn ts_parser_parse(
        self_0: *mut TSParser,
        old_tree: *const TSTree,
        input: TSInput,
    ) -> *mut TSTree;
    fn ts_parser_parse_with_options(
        self_0: *mut TSParser,
        old_tree: *const TSTree,
        input: TSInput,
        parse_options: TSParseOptions,
    ) -> *mut TSTree;
    fn ts_parser_parse_string(
        self_0: *mut TSParser,
        old_tree: *const TSTree,
        string: *const ::core::ffi::c_char,
        length: uint32_t,
    ) -> *mut TSTree;
    fn ts_parser_reset(self_0: *mut TSParser);
    fn ts_parser_set_logger(self_0: *mut TSParser, logger: TSLogger);
    fn ts_parser_logger(self_0: *const TSParser) -> TSLogger;
    fn ts_tree_copy(self_0: *const TSTree) -> *mut TSTree;
    fn ts_tree_delete(self_0: *mut TSTree);
    fn ts_tree_root_node(self_0: *const TSTree) -> TSNode;
    fn ts_tree_included_ranges(self_0: *const TSTree, length: *mut uint32_t) -> *mut TSRange;
    fn ts_tree_edit(self_0: *mut TSTree, edit: *const TSInputEdit);
    fn ts_tree_get_changed_ranges(
        old_tree: *const TSTree,
        new_tree: *const TSTree,
        length: *mut uint32_t,
    ) -> *mut TSRange;
    fn ts_node_type(self_0: TSNode) -> *const ::core::ffi::c_char;
    fn ts_node_symbol(self_0: TSNode) -> TSSymbol;
    fn ts_node_start_byte(self_0: TSNode) -> uint32_t;
    fn ts_node_start_point(self_0: TSNode) -> TSPoint;
    fn ts_node_end_byte(self_0: TSNode) -> uint32_t;
    fn ts_node_end_point(self_0: TSNode) -> TSPoint;
    fn ts_node_string(self_0: TSNode) -> *mut ::core::ffi::c_char;
    fn ts_node_is_null(self_0: TSNode) -> bool;
    fn ts_node_is_named(self_0: TSNode) -> bool;
    fn ts_node_is_missing(self_0: TSNode) -> bool;
    fn ts_node_is_extra(self_0: TSNode) -> bool;
    fn ts_node_has_changes(self_0: TSNode) -> bool;
    fn ts_node_has_error(self_0: TSNode) -> bool;
    fn ts_node_parent(self_0: TSNode) -> TSNode;
    fn ts_node_child_with_descendant(self_0: TSNode, descendant: TSNode) -> TSNode;
    fn ts_node_child(self_0: TSNode, child_index: uint32_t) -> TSNode;
    fn ts_node_field_name_for_child(
        self_0: TSNode,
        child_index: uint32_t,
    ) -> *const ::core::ffi::c_char;
    fn ts_node_child_count(self_0: TSNode) -> uint32_t;
    fn ts_node_named_child(self_0: TSNode, child_index: uint32_t) -> TSNode;
    fn ts_node_named_child_count(self_0: TSNode) -> uint32_t;
    fn ts_node_next_sibling(self_0: TSNode) -> TSNode;
    fn ts_node_prev_sibling(self_0: TSNode) -> TSNode;
    fn ts_node_next_named_sibling(self_0: TSNode) -> TSNode;
    fn ts_node_prev_named_sibling(self_0: TSNode) -> TSNode;
    fn ts_node_descendant_for_point_range(self_0: TSNode, start: TSPoint, end: TSPoint) -> TSNode;
    fn ts_node_named_descendant_for_point_range(
        self_0: TSNode,
        start: TSPoint,
        end: TSPoint,
    ) -> TSNode;
    fn ts_node_eq(self_0: TSNode, other: TSNode) -> bool;
    fn ts_query_new(
        language: *const TSLanguage,
        source: *const ::core::ffi::c_char,
        source_len: uint32_t,
        error_offset: *mut uint32_t,
        error_type: *mut TSQueryError,
    ) -> *mut TSQuery;
    fn ts_query_delete(self_0: *mut TSQuery);
    fn ts_query_pattern_count(self_0: *const TSQuery) -> uint32_t;
    fn ts_query_capture_count(self_0: *const TSQuery) -> uint32_t;
    fn ts_query_predicates_for_pattern(
        self_0: *const TSQuery,
        pattern_index: uint32_t,
        step_count: *mut uint32_t,
    ) -> *const TSQueryPredicateStep;
    fn ts_query_capture_name_for_id(
        self_0: *const TSQuery,
        index: uint32_t,
        length: *mut uint32_t,
    ) -> *const ::core::ffi::c_char;
    fn ts_query_string_value_for_id(
        self_0: *const TSQuery,
        index: uint32_t,
        length: *mut uint32_t,
    ) -> *const ::core::ffi::c_char;
    fn ts_query_disable_capture(
        self_0: *mut TSQuery,
        name: *const ::core::ffi::c_char,
        length: uint32_t,
    );
    fn ts_query_disable_pattern(self_0: *mut TSQuery, pattern_index: uint32_t);
    fn ts_query_cursor_new() -> *mut TSQueryCursor;
    fn ts_query_cursor_delete(self_0: *mut TSQueryCursor);
    fn ts_query_cursor_exec(self_0: *mut TSQueryCursor, query: *const TSQuery, node: TSNode);
    fn ts_query_cursor_set_match_limit(self_0: *mut TSQueryCursor, limit: uint32_t);
    fn ts_query_cursor_set_point_range(
        self_0: *mut TSQueryCursor,
        start_point: TSPoint,
        end_point: TSPoint,
    ) -> bool;
    fn ts_query_cursor_next_match(self_0: *mut TSQueryCursor, match_0: *mut TSQueryMatch) -> bool;
    fn ts_query_cursor_remove_match(self_0: *mut TSQueryCursor, match_id: uint32_t);
    fn ts_query_cursor_next_capture(
        self_0: *mut TSQueryCursor,
        match_0: *mut TSQueryMatch,
        capture_index: *mut uint32_t,
    ) -> bool;
    fn ts_query_cursor_set_max_start_depth(self_0: *mut TSQueryCursor, max_start_depth: uint32_t);
    fn ts_language_symbol_count(self_0: *const TSLanguage) -> uint32_t;
    fn ts_language_state_count(self_0: *const TSLanguage) -> uint32_t;
    fn ts_language_field_count(self_0: *const TSLanguage) -> uint32_t;
    fn ts_language_field_name_for_id(
        self_0: *const TSLanguage,
        id: TSFieldId,
    ) -> *const ::core::ffi::c_char;
    fn ts_language_supertypes(self_0: *const TSLanguage, length: *mut uint32_t) -> *const TSSymbol;
    fn ts_language_subtypes(
        self_0: *const TSLanguage,
        supertype: TSSymbol,
        length: *mut uint32_t,
    ) -> *const TSSymbol;
    fn ts_language_symbol_name(
        self_0: *const TSLanguage,
        symbol: TSSymbol,
    ) -> *const ::core::ffi::c_char;
    fn ts_language_symbol_type(self_0: *const TSLanguage, symbol: TSSymbol) -> TSSymbolType;
    fn ts_language_abi_version(self_0: *const TSLanguage) -> uint32_t;
    fn ts_language_metadata(self_0: *const TSLanguage) -> *const TSLanguageMetadata;
    fn ts_language_is_wasm(_: *const TSLanguage) -> bool;
    fn ts_set_allocator(
        new_malloc: Option<unsafe extern "C" fn(size_t) -> *mut ::core::ffi::c_void>,
        new_calloc: Option<unsafe extern "C" fn(size_t, size_t) -> *mut ::core::ffi::c_void>,
        new_realloc: Option<
            unsafe extern "C" fn(*mut ::core::ffi::c_void, size_t) -> *mut ::core::ffi::c_void,
        >,
        new_free: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    );
    fn uv_dlopen(filename: *const ::core::ffi::c_char, lib: *mut uv_lib_t) -> ::core::ffi::c_int;
    fn uv_dlclose(lib: *mut uv_lib_t);
    fn uv_dlsym(
        lib: *mut uv_lib_t,
        name: *const ::core::ffi::c_char,
        ptr: *mut *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int;
    fn uv_dlerror(lib: *const uv_lib_t) -> *const ::core::ffi::c_char;
    fn os_hrtime() -> uint64_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn memchrsub(
        data: *mut ::core::ffi::c_void,
        c: ::core::ffi::c_char,
        x: ::core::ffi::c_char,
        len: size_t,
    );
    fn xstrlcpy(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn mh_get_int(set: *mut Set_int, key: ::core::ffi::c_int) -> uint32_t;
    fn mh_get_cstr_t(set: *mut Set_cstr_t, key: cstr_t) -> uint32_t;
    fn map_del_cstr_t_ptr_t(
        map: *mut Map_cstr_t_ptr_t,
        key: cstr_t,
        key_alloc: *mut cstr_t,
    ) -> ptr_t;
    fn map_put_ref_cstr_t_ptr_t(
        map: *mut Map_cstr_t_ptr_t,
        key: cstr_t,
        key_alloc: *mut *mut cstr_t,
        new_item: *mut bool,
    ) -> *mut ptr_t;
    static mut buffer_handles: Map_int_ptr_t;
    static mut IObuff: [::core::ffi::c_char; 1025];
    static mut tslua_query_parse_count: uint64_t;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T;
    fn vim_snprintf(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
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
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type lua_CFunction = Option<unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int>;
pub type lua_Number = ::core::ffi::c_double;
pub type lua_Integer = ptrdiff_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct luaL_Reg {
    pub name: *const ::core::ffi::c_char,
    pub func: lua_CFunction,
}
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type time_t = __time_t;
pub type TSSymbol = uint16_t;
pub type TSFieldId = uint16_t;
pub type TSDecodeFunction =
    Option<unsafe extern "C" fn(*const uint8_t, uint32_t, *mut int32_t) -> uint32_t>;
pub type TSInputEncoding = ::core::ffi::c_uint;
pub const TSInputEncodingCustom: TSInputEncoding = 3;
pub const TSInputEncodingUTF16BE: TSInputEncoding = 2;
pub const TSInputEncodingUTF16LE: TSInputEncoding = 1;
pub const TSInputEncodingUTF8: TSInputEncoding = 0;
pub type TSSymbolType = ::core::ffi::c_uint;
pub const TSSymbolTypeAuxiliary: TSSymbolType = 3;
pub const TSSymbolTypeSupertype: TSSymbolType = 2;
pub const TSSymbolTypeAnonymous: TSSymbolType = 1;
pub const TSSymbolTypeRegular: TSSymbolType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSPoint {
    pub row: uint32_t,
    pub column: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSRange {
    pub start_point: TSPoint,
    pub end_point: TSPoint,
    pub start_byte: uint32_t,
    pub end_byte: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSInput {
    pub payload: *mut ::core::ffi::c_void,
    pub read: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
            uint32_t,
            TSPoint,
            *mut uint32_t,
        ) -> *const ::core::ffi::c_char,
    >,
    pub encoding: TSInputEncoding,
    pub decode: TSDecodeFunction,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSParseState {
    pub payload: *mut ::core::ffi::c_void,
    pub current_byte_offset: uint32_t,
    pub has_error: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSParseOptions {
    pub payload: *mut ::core::ffi::c_void,
    pub progress_callback: Option<unsafe extern "C" fn(*mut TSParseState) -> bool>,
}
pub type TSLogType = ::core::ffi::c_uint;
pub const TSLogTypeLex: TSLogType = 1;
pub const TSLogTypeParse: TSLogType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSLogger {
    pub payload: *mut ::core::ffi::c_void,
    pub log: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, TSLogType, *const ::core::ffi::c_char) -> (),
    >,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSInputEdit {
    pub start_byte: uint32_t,
    pub old_end_byte: uint32_t,
    pub new_end_byte: uint32_t,
    pub start_point: TSPoint,
    pub old_end_point: TSPoint,
    pub new_end_point: TSPoint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSNode {
    pub context: [uint32_t; 4],
    pub id: *const ::core::ffi::c_void,
    pub tree: *const TSTree,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQueryCapture {
    pub node: TSNode,
    pub index: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQueryMatch {
    pub id: uint32_t,
    pub pattern_index: uint16_t,
    pub capture_count: uint16_t,
    pub captures: *const TSQueryCapture,
}
pub type TSQueryPredicateStepType = ::core::ffi::c_uint;
pub const TSQueryPredicateStepTypeString: TSQueryPredicateStepType = 2;
pub const TSQueryPredicateStepTypeCapture: TSQueryPredicateStepType = 1;
pub const TSQueryPredicateStepTypeDone: TSQueryPredicateStepType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQueryPredicateStep {
    pub type_0: TSQueryPredicateStepType,
    pub value_id: uint32_t,
}
pub type TSQueryError = ::core::ffi::c_uint;
pub const TSQueryErrorLanguage: TSQueryError = 6;
pub const TSQueryErrorStructure: TSQueryError = 5;
pub const TSQueryErrorCapture: TSQueryError = 4;
pub const TSQueryErrorField: TSQueryError = 3;
pub const TSQueryErrorNodeType: TSQueryError = 2;
pub const TSQueryErrorSyntax: TSQueryError = 1;
pub const TSQueryErrorNone: TSQueryError = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSLanguageMetadata {
    pub major_version: uint8_t,
    pub minor_version: uint8_t,
    pub patch_version: uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_lib_t {
    pub handle: *mut ::core::ffi::c_void,
    pub errmsg: *mut ::core::ffi::c_char,
}
pub type Timestamp = uint64_t;
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
pub type cstr_t = *const ::core::ffi::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_int {
    pub h: MapHash,
    pub keys: *mut ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_cstr_t {
    pub h: MapHash,
    pub keys: *mut cstr_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_int_ptr_t {
    pub set: Set_int,
    pub values: *mut ptr_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_cstr_t_ptr_t {
    pub set: Set_cstr_t,
    pub values: *mut ptr_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSLuaTree {
    pub tree: *const TSTree,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSLuaLoggerOpts {
    pub cb: LuaRef,
    pub lstate: *mut lua_State,
    pub lex: bool,
    pub parse: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSLuaParserCallbackPayload {
    pub parse_start_time: uint64_t,
    pub timeout_threshold_ns: uint64_t,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_REGISTRYINDEX: ::core::ffi::c_int = -10000 as ::core::ffi::c_int;
pub const LUA_GLOBALSINDEX: ::core::ffi::c_int = -10002 as ::core::ffi::c_int;
pub const LUA_TNIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LUA_TNUMBER: ::core::ffi::c_int = 3;
pub const LUA_TSTRING: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const LUA_TTABLE: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const TREE_SITTER_LANGUAGE_VERSION: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION: ::core::ffi::c_int =
    13 as ::core::ffi::c_int;
static mut value_init_ptr_t: ptr_t = NULL;
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_cstr_t = Set_cstr_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<cstr_t>(),
};
pub const MAP_INIT: Map_cstr_t_ptr_t = Map_cstr_t_ptr_t {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_has_cstr_t(mut set: *mut Set_cstr_t, mut key: cstr_t) -> bool {
    return mh_get_cstr_t(set, key) != MH_TOMBSTONE as uint32_t;
}
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
#[inline]
unsafe extern "C" fn map_put_cstr_t_ptr_t(
    mut map: *mut Map_cstr_t_ptr_t,
    mut key: cstr_t,
    mut value: ptr_t,
) {
    let mut val: *mut ptr_t = map_put_ref_cstr_t_ptr_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut cstr_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
#[inline]
unsafe extern "C" fn map_get_cstr_t_ptr_t(
    mut map: *mut Map_cstr_t_ptr_t,
    mut key: cstr_t,
) -> ptr_t {
    let mut k: uint32_t = mh_get_cstr_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t
    } else {
        *(*map).values.offset(k as isize)
    };
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TS_META_PARSER: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"treesitter_parser\0")
};
pub const TS_META_TREE: [::core::ffi::c_char; 16] =
    unsafe { ::core::mem::transmute::<[u8; 16], [::core::ffi::c_char; 16]>(*b"treesitter_tree\0") };
pub const TS_META_NODE: [::core::ffi::c_char; 16] =
    unsafe { ::core::mem::transmute::<[u8; 16], [::core::ffi::c_char; 16]>(*b"treesitter_node\0") };
pub const TS_META_QUERY: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"treesitter_query\0")
};
pub const TS_META_QUERYCURSOR: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"treesitter_querycursor\0")
};
pub const TS_META_QUERYMATCH: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"treesitter_querymatch\0")
};
static mut langs: Map_cstr_t_ptr_t = MAP_INIT;
unsafe extern "C" fn tslua_has_language(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut lang_name: *const ::core::ffi::c_char = luaL_checklstring(
        L,
        1 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    );
    lua_pushboolean(
        L,
        set_has_cstr_t(&raw mut langs.set, lang_name as cstr_t) as ::core::ffi::c_int,
    );
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn tslua_add_language_from_object(mut L: *mut lua_State) -> ::core::ffi::c_int {
    return add_language(L, false_0 != 0);
}
unsafe extern "C" fn load_language_from_object(
    mut L: *mut lua_State,
    mut path: *const ::core::ffi::c_char,
    mut lang_name: *const ::core::ffi::c_char,
    mut symbol: *const ::core::ffi::c_char,
) -> *const TSLanguage {
    let mut lib: uv_lib_t = uv_lib_t {
        handle: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    if uv_dlopen(path, &raw mut lib) != 0 {
        xstrlcpy(
            &raw mut IObuff as *mut ::core::ffi::c_char,
            uv_dlerror(&raw mut lib),
            ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
        );
        uv_dlclose(&raw mut lib);
        luaL_error(
            L,
            b"Failed to load parser for language '%s': uv_dlopen: %s\0".as_ptr()
                as *const ::core::ffi::c_char,
            lang_name,
            &raw mut IObuff as *mut ::core::ffi::c_char,
        );
    }
    let mut symbol_buf: [::core::ffi::c_char; 128] = [0; 128];
    snprintf(
        &raw mut symbol_buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 128]>(),
        b"tree_sitter_%s\0".as_ptr() as *const ::core::ffi::c_char,
        symbol,
    );
    let mut lang_parser: Option<unsafe extern "C" fn() -> *mut TSLanguage> = None;
    if uv_dlsym(
        &raw mut lib,
        &raw mut symbol_buf as *mut ::core::ffi::c_char,
        &raw mut lang_parser as *mut *mut ::core::ffi::c_void,
    ) != 0
    {
        xstrlcpy(
            &raw mut IObuff as *mut ::core::ffi::c_char,
            uv_dlerror(&raw mut lib),
            ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
        );
        uv_dlclose(&raw mut lib);
        luaL_error(
            L,
            b"Failed to load parser: uv_dlsym: %s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw mut IObuff as *mut ::core::ffi::c_char,
        );
    }
    let mut lang: *mut TSLanguage = lang_parser.expect("non-null function pointer")();
    if lang.is_null() {
        uv_dlclose(&raw mut lib);
        luaL_error(
            L,
            b"Failed to load parser %s: internal error\0".as_ptr() as *const ::core::ffi::c_char,
            path,
        );
    }
    return lang;
}
unsafe extern "C" fn load_language_from_wasm(
    mut L: *mut lua_State,
    mut _path: *const ::core::ffi::c_char,
    mut _lang_name: *const ::core::ffi::c_char,
) -> *const TSLanguage {
    luaL_error(L, b"Not supported\0".as_ptr() as *const ::core::ffi::c_char);
    return ::core::ptr::null::<TSLanguage>();
}
unsafe extern "C" fn add_language(mut L: *mut lua_State, mut is_wasm: bool) -> ::core::ffi::c_int {
    let mut path: *const ::core::ffi::c_char = luaL_checklstring(
        L,
        1 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    );
    let mut lang_name: *const ::core::ffi::c_char = luaL_checklstring(
        L,
        2 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    );
    let mut symbol_name: *const ::core::ffi::c_char = lang_name;
    if !is_wasm
        && lua_gettop(L) >= 3 as ::core::ffi::c_int
        && !(lua_type(L, 3 as ::core::ffi::c_int) == LUA_TNIL)
    {
        symbol_name = luaL_checklstring(
            L,
            3 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<size_t>(),
        );
    }
    if set_has_cstr_t(&raw mut langs.set, lang_name as cstr_t) {
        lua_pushboolean(L, true_0);
        return 1 as ::core::ffi::c_int;
    }
    let mut lang: *const TSLanguage = if is_wasm as ::core::ffi::c_int != 0 {
        load_language_from_wasm(L, path, lang_name)
    } else {
        load_language_from_object(L, path, lang_name, symbol_name)
    };
    let mut lang_version: uint32_t = ts_language_abi_version(lang);
    if lang_version < TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION as uint32_t
        || lang_version > TREE_SITTER_LANGUAGE_VERSION as uint32_t
    {
        return luaL_error(
            L,
            b"ABI version mismatch for %s: supported between %d and %d, found %d\0".as_ptr()
                as *const ::core::ffi::c_char,
            path,
            TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION,
            TREE_SITTER_LANGUAGE_VERSION,
            lang_version,
        );
    }
    map_put_cstr_t_ptr_t(
        &raw mut langs,
        xstrdup(lang_name) as cstr_t,
        lang as *mut TSLanguage as ptr_t,
    );
    lua_pushboolean(L, true_0);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn tslua_remove_lang(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut lang_name: *const ::core::ffi::c_char = luaL_checklstring(
        L,
        1 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    );
    let mut present: bool = set_has_cstr_t(&raw mut langs.set, lang_name as cstr_t);
    if present {
        let mut key: cstr_t = ::core::ptr::null::<::core::ffi::c_char>();
        map_del_cstr_t_ptr_t(&raw mut langs, lang_name as cstr_t, &raw mut key);
        xfree(key as *mut ::core::ffi::c_void);
    }
    lua_pushboolean(L, present as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn lang_check(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> *mut TSLanguage {
    let mut lang_name: *const ::core::ffi::c_char =
        luaL_checklstring(L, index, ::core::ptr::null_mut::<size_t>());
    let mut lang: *mut TSLanguage =
        map_get_cstr_t_ptr_t(&raw mut langs, lang_name as cstr_t) as *mut TSLanguage;
    if lang.is_null() {
        luaL_error(
            L,
            b"no such language: %s\0".as_ptr() as *const ::core::ffi::c_char,
            lang_name,
        );
    }
    return lang;
}
unsafe extern "C" fn tslua_inspect_lang(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut lang: *mut TSLanguage = lang_check(L, 1 as ::core::ffi::c_int);
    lua_createtable(L, 0 as ::core::ffi::c_int, 2 as ::core::ffi::c_int);
    let mut nsymbols: uint32_t = ts_language_symbol_count(lang);
    '_c2rust_label: {
        if nsymbols < 2147483647 as uint32_t {
        } else {
            __assert_fail(
                b"nsymbols < INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/lua/treesitter.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                276 as ::core::ffi::c_uint,
                b"int tslua_inspect_lang(lua_State *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    lua_createtable(
        L,
        nsymbols.wrapping_sub(1 as uint32_t) as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    let mut i: uint32_t = 0 as uint32_t;
    while i < nsymbols {
        let mut t: TSSymbolType = ts_language_symbol_type(lang, i as TSSymbol);
        if t as ::core::ffi::c_uint
            != TSSymbolTypeAuxiliary as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut name: *const ::core::ffi::c_char = ts_language_symbol_name(lang, i as TSSymbol);
            let mut named: bool = t as ::core::ffi::c_uint
                != TSSymbolTypeAnonymous as ::core::ffi::c_int as ::core::ffi::c_uint;
            lua_pushboolean(L, named as ::core::ffi::c_int);
            if !named {
                let mut buf: [::core::ffi::c_char; 256] = [0; 256];
                snprintf(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
                    b"\"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
                    name,
                );
                lua_setfield(
                    L,
                    -2 as ::core::ffi::c_int,
                    &raw mut buf as *mut ::core::ffi::c_char,
                );
            } else {
                lua_setfield(L, -2 as ::core::ffi::c_int, name);
            }
        }
        i = i.wrapping_add(1);
    }
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"symbols\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut nfields: uint32_t = ts_language_field_count(lang);
    lua_createtable(L, nfields as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    let mut i_0: uint32_t = 1 as uint32_t;
    while i_0 <= nfields {
        lua_pushstring(L, ts_language_field_name_for_id(lang, i_0 as TSFieldId));
        lua_rawseti(L, -2 as ::core::ffi::c_int, i_0 as ::core::ffi::c_int);
        i_0 = i_0.wrapping_add(1);
    }
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"fields\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushboolean(L, ts_language_is_wasm(lang) as ::core::ffi::c_int);
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"_wasm\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushinteger(L, ts_language_abi_version(lang) as lua_Integer);
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"abi_version\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut meta: *const TSLanguageMetadata = ts_language_metadata(lang);
    if !meta.is_null() {
        lua_createtable(L, 0 as ::core::ffi::c_int, 3 as ::core::ffi::c_int);
        lua_pushinteger(L, (*meta).major_version as lua_Integer);
        lua_setfield(
            L,
            -2 as ::core::ffi::c_int,
            b"major_version\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushinteger(L, (*meta).minor_version as lua_Integer);
        lua_setfield(
            L,
            -2 as ::core::ffi::c_int,
            b"minor_version\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushinteger(L, (*meta).patch_version as lua_Integer);
        lua_setfield(
            L,
            -2 as ::core::ffi::c_int,
            b"patch_version\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_setfield(
            L,
            -2 as ::core::ffi::c_int,
            b"metadata\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    lua_pushinteger(L, ts_language_state_count(lang) as lua_Integer);
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"state_count\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut nsupertypes: uint32_t = 0;
    let mut supertypes: *const TSSymbol = ts_language_supertypes(lang, &raw mut nsupertypes);
    lua_createtable(
        L,
        0 as ::core::ffi::c_int,
        nsupertypes as ::core::ffi::c_int,
    );
    let mut i_1: uint32_t = 0 as uint32_t;
    while i_1 < nsupertypes {
        let supertype: TSSymbol = *supertypes.offset(i_1 as isize);
        let mut nsubtypes: uint32_t = 0;
        let mut subtypes: *const TSSymbol =
            ts_language_subtypes(lang, supertype, &raw mut nsubtypes);
        lua_createtable(L, nsubtypes as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        let mut j: uint32_t = 1 as uint32_t;
        while j <= nsubtypes {
            lua_pushstring(
                L,
                ts_language_symbol_name(lang, *subtypes.offset(j as isize)),
            );
            lua_rawseti(L, -2 as ::core::ffi::c_int, j as ::core::ffi::c_int);
            j = j.wrapping_add(1);
        }
        lua_setfield(
            L,
            -2 as ::core::ffi::c_int,
            ts_language_symbol_name(lang, supertype),
        );
        i_1 = i_1.wrapping_add(1);
    }
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"supertypes\0".as_ptr() as *const ::core::ffi::c_char,
    );
    return 1 as ::core::ffi::c_int;
}
static mut parser_meta: [luaL_Reg; 9] = [
    luaL_Reg {
        name: b"__gc\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(parser_gc as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(parser_tostring as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"parse\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(parser_parse as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"reset\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(parser_reset as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"set_included_ranges\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(parser_set_ranges as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"included_ranges\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(parser_get_ranges as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"_set_logger\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(parser_set_logger as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"_logger\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(parser_get_logger as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
];
unsafe extern "C" fn tslua_push_parser(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut lang: *mut TSLanguage = lang_check(L, 1 as ::core::ffi::c_int);
    let mut parser: *mut *mut TSParser =
        lua_newuserdata(L, ::core::mem::size_of::<*mut TSParser>()) as *mut *mut TSParser;
    *parser = ts_parser_new();
    if !ts_parser_set_language(*parser, lang) {
        ts_parser_delete(*parser);
        let mut lang_name: *const ::core::ffi::c_char = luaL_checklstring(
            L,
            1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<size_t>(),
        );
        return luaL_error(
            L,
            b"Failed to load language : %s\0".as_ptr() as *const ::core::ffi::c_char,
            lang_name,
        );
    }
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_PARSER.as_ptr());
    lua_setmetatable(L, -2 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn parser_check(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> *mut TSParser {
    let mut ud: *mut *mut TSParser =
        luaL_checkudata(L, index, TS_META_PARSER.as_ptr()) as *mut *mut TSParser;
    (!(*ud).is_null()
        || luaL_argerror(
            L,
            index,
            b"Parser has been deleted\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    return *ud;
}
unsafe extern "C" fn logger_gc(mut logger: TSLogger) {
    if logger.log.is_none() {
        return;
    }
    let mut opts: *mut TSLuaLoggerOpts = logger.payload as *mut TSLuaLoggerOpts;
    luaL_unref(
        (*opts).lstate,
        LUA_REGISTRYINDEX,
        (*opts).cb as ::core::ffi::c_int,
    );
    xfree(opts as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn parser_gc(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut ud: *mut *mut TSParser =
        luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_PARSER.as_ptr()) as *mut *mut TSParser;
    if !(*ud).is_null() {
        logger_gc(ts_parser_logger(*ud));
        ts_parser_delete(*ud);
        *ud = ::core::ptr::null_mut::<TSParser>();
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn parser_tostring(mut L: *mut lua_State) -> ::core::ffi::c_int {
    lua_pushstring(L, b"<parser>\0".as_ptr() as *const ::core::ffi::c_char);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_cb(
    mut payload: *mut ::core::ffi::c_void,
    mut _byte_index: uint32_t,
    mut position: TSPoint,
    mut bytes_read: *mut uint32_t,
) -> *const ::core::ffi::c_char {
    let mut bp: *mut buf_T = payload as *mut buf_T;
    static mut buf: [::core::ffi::c_char; 256] = [0; 256];
    if position.row as linenr_T >= (*bp).b_ml.ml_line_count {
        *bytes_read = 0 as uint32_t;
        return b"\0".as_ptr() as *const ::core::ffi::c_char;
    }
    let mut lnum: linenr_T = position.row as linenr_T + 1 as linenr_T;
    let mut line: *mut ::core::ffi::c_char = ml_get_buf(bp, lnum);
    let mut len: size_t = ml_get_buf_len(bp, lnum) as size_t;
    if position.column as size_t > len {
        *bytes_read = 0 as uint32_t;
        return b"\0".as_ptr() as *const ::core::ffi::c_char;
    }
    let mut tocopy: size_t = if len.wrapping_sub(position.column as size_t) < 256 as size_t {
        len.wrapping_sub(position.column as size_t)
    } else {
        256 as size_t
    };
    memcpy(
        &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        line.offset(position.column as isize) as *const ::core::ffi::c_void,
        tocopy,
    );
    memchrsub(
        &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        '\n' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        tocopy,
    );
    *bytes_read = tocopy as uint32_t;
    if tocopy < BUFSIZE as size_t {
        if lnum != (*bp).b_ml.ml_line_count
            || (*bp).b_p_bin == 0 && (*bp).b_p_fixeol != 0
            || lnum != (*bp).b_no_eol_lnum && (*bp).b_p_eol != 0
        {
            buf[tocopy as usize] = '\n' as ::core::ffi::c_char;
            *bytes_read = (*bytes_read).wrapping_add(1);
        }
    }
    return &raw mut buf as *mut ::core::ffi::c_char;
}
pub const BUFSIZE: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
unsafe extern "C" fn push_ranges(
    mut L: *mut lua_State,
    mut ranges: *const TSRange,
    length: size_t,
    mut include_bytes: bool,
) {
    lua_createtable(L, length as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    let mut i: size_t = 0 as size_t;
    while i < length {
        lua_createtable(
            L,
            if include_bytes as ::core::ffi::c_int != 0 {
                6 as ::core::ffi::c_int
            } else {
                4 as ::core::ffi::c_int
            },
            0 as ::core::ffi::c_int,
        );
        let mut j: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        lua_pushnumber(
            L,
            (*ranges.offset(i as isize)).start_point.row as lua_Number,
        );
        let c2rust_fresh2 = j;
        j = j + 1;
        lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh2);
        lua_pushnumber(
            L,
            (*ranges.offset(i as isize)).start_point.column as lua_Number,
        );
        let c2rust_fresh3 = j;
        j = j + 1;
        lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh3);
        if include_bytes {
            lua_pushnumber(L, (*ranges.offset(i as isize)).start_byte as lua_Number);
            let c2rust_fresh4 = j;
            j = j + 1;
            lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh4);
        }
        lua_pushnumber(L, (*ranges.offset(i as isize)).end_point.row as lua_Number);
        let c2rust_fresh5 = j;
        j = j + 1;
        lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh5);
        lua_pushnumber(
            L,
            (*ranges.offset(i as isize)).end_point.column as lua_Number,
        );
        let c2rust_fresh6 = j;
        j = j + 1;
        lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh6);
        if include_bytes {
            lua_pushnumber(L, (*ranges.offset(i as isize)).end_byte as lua_Number);
            let c2rust_fresh7 = j;
            j = j + 1;
            lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh7);
        }
        lua_rawseti(
            L,
            -2 as ::core::ffi::c_int,
            i.wrapping_add(1 as size_t) as ::core::ffi::c_int,
        );
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn on_parser_progress(mut state: *mut TSParseState) -> bool {
    let mut payload: *mut TSLuaParserCallbackPayload =
        (*state).payload as *mut TSLuaParserCallbackPayload;
    let mut parse_time: uint64_t = os_hrtime().wrapping_sub((*payload).parse_start_time);
    return parse_time >= (*payload).timeout_threshold_ns;
}
unsafe extern "C" fn parser_parse(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
    let mut old_tree: *const TSTree = ::core::ptr::null::<TSTree>();
    if !(lua_type(L, 2 as ::core::ffi::c_int) == LUA_TNIL) {
        let mut ud: *mut TSLuaTree =
            luaL_checkudata(L, 2 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
        old_tree = if !ud.is_null() {
            (*ud).tree
        } else {
            ::core::ptr::null::<TSTree>()
        };
    }
    let mut new_tree: *mut TSTree = ::core::ptr::null_mut::<TSTree>();
    let mut len: size_t = 0;
    let mut str: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut bufnr: handle_T = 0;
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut input: TSInput = TSInput {
        payload: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        read: None,
        encoding: TSInputEncodingUTF8,
        decode: None,
    };
    match lua_type(L, 3 as ::core::ffi::c_int) {
        LUA_TSTRING => {
            str = lua_tolstring(L, 3 as ::core::ffi::c_int, &raw mut len);
            new_tree = ts_parser_parse_string(p, old_tree, str, len as uint32_t);
        }
        LUA_TNUMBER => {
            bufnr = lua_tointeger(L, 3 as ::core::ffi::c_int) as handle_T;
            buf = map_get_int_ptr_t(&raw mut buffer_handles, bufnr as ::core::ffi::c_int)
                as *mut buf_T;
            if buf.is_null() {
                let mut ebuf: [::core::ffi::c_char; 256] = [
                    0 as ::core::ffi::c_char,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ];
                vim_snprintf(
                    &raw mut ebuf as *mut ::core::ffi::c_char,
                    BUFSIZE_0 as size_t,
                    b"invalid buffer handle: %d\0".as_ptr() as *const ::core::ffi::c_char,
                    bufnr,
                );
                return luaL_argerror(
                    L,
                    3 as ::core::ffi::c_int,
                    &raw mut ebuf as *mut ::core::ffi::c_char,
                );
            }
            input = TSInput {
                payload: buf as *mut ::core::ffi::c_void,
                read: Some(
                    input_cb
                        as unsafe extern "C" fn(
                            *mut ::core::ffi::c_void,
                            uint32_t,
                            TSPoint,
                            *mut uint32_t,
                        )
                            -> *const ::core::ffi::c_char,
                ),
                encoding: TSInputEncodingUTF8,
                decode: None,
            };
            if !(lua_type(L, 5 as ::core::ffi::c_int) == LUA_TNIL) {
                let mut timeout_ns: uint64_t =
                    lua_tointeger(L, 5 as ::core::ffi::c_int) as uint64_t;
                let mut payload: TSLuaParserCallbackPayload = TSLuaParserCallbackPayload {
                    parse_start_time: os_hrtime(),
                    timeout_threshold_ns: timeout_ns,
                };
                let mut parse_options: TSParseOptions = TSParseOptions {
                    payload: &raw mut payload as *mut ::core::ffi::c_void,
                    progress_callback: Some(
                        on_parser_progress as unsafe extern "C" fn(*mut TSParseState) -> bool,
                    ),
                };
                new_tree = ts_parser_parse_with_options(p, old_tree, input, parse_options);
            } else {
                new_tree = ts_parser_parse(p, old_tree, input);
            }
        }
        _ => {
            return luaL_argerror(
                L,
                3 as ::core::ffi::c_int,
                b"expected either string or buffer handle\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    }
    let mut include_bytes: bool =
        lua_gettop(L) >= 4 as ::core::ffi::c_int && lua_toboolean(L, 4 as ::core::ffi::c_int) != 0;
    if new_tree.is_null() {
        if ts_parser_language(p).is_null() {
            return luaL_error(
                L,
                b"Language was unset, or has an incompatible ABI.\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
        return 0 as ::core::ffi::c_int;
    }
    let mut n_ranges: uint32_t = 0 as uint32_t;
    let mut changed: *mut TSRange = if !old_tree.is_null() {
        ts_tree_get_changed_ranges(old_tree, new_tree, &raw mut n_ranges)
    } else {
        ts_tree_included_ranges(new_tree, &raw mut n_ranges)
    };
    push_tree(L, new_tree);
    push_ranges(L, changed, n_ranges as size_t, include_bytes);
    xfree(changed as *mut ::core::ffi::c_void);
    return 2 as ::core::ffi::c_int;
}
pub const BUFSIZE_0: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
unsafe extern "C" fn parser_reset(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
    ts_parser_reset(p);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn range_err(mut L: *mut lua_State) {
    luaL_error(
        L,
        b"Ranges can only be made from 6 element long tables or nodes.\0".as_ptr()
            as *const ::core::ffi::c_char,
    );
}
unsafe extern "C" fn lua_checkuint32(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> uint32_t {
    let mut value: lua_Number = luaL_checknumber(L, index);
    let mut converted: uint32_t = value as uint32_t;
    if value < 0 as ::core::ffi::c_int as lua_Number
        || value > UINT32_MAX as lua_Number
        || converted as lua_Number != value
    {
        luaL_error(
            L,
            b"Range value out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return converted;
}
unsafe extern "C" fn range_from_lua(mut L: *mut lua_State, mut range: *mut TSRange) {
    let mut node: TSNode = TSNode {
        context: [0; 4],
        id: ::core::ptr::null::<::core::ffi::c_void>(),
        tree: ::core::ptr::null::<TSTree>(),
    };
    if lua_type(L, -1 as ::core::ffi::c_int) == LUA_TTABLE {
        if lua_objlen(L, -1 as ::core::ffi::c_int) != 6 as size_t {
            range_err(L);
        }
        lua_rawgeti(L, -1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        let mut start_row: uint32_t = lua_checkuint32(L, -1 as ::core::ffi::c_int);
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_rawgeti(L, -1 as ::core::ffi::c_int, 2 as ::core::ffi::c_int);
        let mut start_col: uint32_t = lua_checkuint32(L, -1 as ::core::ffi::c_int);
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_rawgeti(L, -1 as ::core::ffi::c_int, 3 as ::core::ffi::c_int);
        let mut start_byte: uint32_t = lua_checkuint32(L, -1 as ::core::ffi::c_int);
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_rawgeti(L, -1 as ::core::ffi::c_int, 4 as ::core::ffi::c_int);
        let mut end_row: uint32_t = lua_checkuint32(L, -1 as ::core::ffi::c_int);
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_rawgeti(L, -1 as ::core::ffi::c_int, 5 as ::core::ffi::c_int);
        let mut end_col: uint32_t = lua_checkuint32(L, -1 as ::core::ffi::c_int);
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_rawgeti(L, -1 as ::core::ffi::c_int, 6 as ::core::ffi::c_int);
        let mut end_byte: uint32_t = lua_checkuint32(L, -1 as ::core::ffi::c_int);
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        *range = TSRange {
            start_point: TSPoint {
                row: start_row,
                column: start_col,
            },
            end_point: TSPoint {
                row: end_row,
                column: end_col,
            },
            start_byte: start_byte,
            end_byte: end_byte,
        };
    } else if node_check_opt(L, -1 as ::core::ffi::c_int, &raw mut node) {
        *range = TSRange {
            start_point: ts_node_start_point(node),
            end_point: ts_node_end_point(node),
            start_byte: ts_node_start_byte(node),
            end_byte: ts_node_end_byte(node),
        };
    } else {
        range_err(L);
    };
}
unsafe extern "C" fn parser_set_ranges(mut L: *mut lua_State) -> ::core::ffi::c_int {
    if lua_gettop(L) < 2 as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"not enough args to parser:set_included_ranges()\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
    }
    let mut p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
    (lua_type(L, 2 as ::core::ffi::c_int) == 5 as ::core::ffi::c_int
        || luaL_argerror(
            L,
            2 as ::core::ffi::c_int,
            b"table expected.\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    let mut tbl_len: size_t = lua_objlen(L, 2 as ::core::ffi::c_int);
    let mut ranges: *mut TSRange =
        xmalloc(::core::mem::size_of::<TSRange>().wrapping_mul(tbl_len)) as *mut TSRange;
    let mut index: size_t = 0 as size_t;
    while index < tbl_len {
        lua_rawgeti(
            L,
            2 as ::core::ffi::c_int,
            index as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
        );
        range_from_lua(L, ranges.offset(index as isize));
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        index = index.wrapping_add(1);
    }
    ts_parser_set_included_ranges(p, ranges, tbl_len as uint32_t);
    xfree(ranges as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn parser_get_ranges(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
    let mut include_bytes: bool =
        lua_gettop(L) >= 2 as ::core::ffi::c_int && lua_toboolean(L, 2 as ::core::ffi::c_int) != 0;
    let mut len: uint32_t = 0;
    let mut ranges: *const TSRange = ts_parser_included_ranges(p, &raw mut len);
    push_ranges(L, ranges, len as size_t, include_bytes);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn logger_cb(
    mut payload: *mut ::core::ffi::c_void,
    mut logtype: TSLogType,
    mut s: *const ::core::ffi::c_char,
) {
    let mut opts: *mut TSLuaLoggerOpts = payload as *mut TSLuaLoggerOpts;
    if !(*opts).lex
        && logtype as ::core::ffi::c_uint
            == TSLogTypeLex as ::core::ffi::c_int as ::core::ffi::c_uint
        || !(*opts).parse
            && logtype as ::core::ffi::c_uint
                == TSLogTypeParse as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    let mut lstate: *mut lua_State = (*opts).lstate;
    lua_rawgeti(lstate, LUA_REGISTRYINDEX, (*opts).cb as ::core::ffi::c_int);
    lua_pushstring(
        lstate,
        if logtype as ::core::ffi::c_uint
            == TSLogTypeParse as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            b"parse\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"lex\0".as_ptr() as *const ::core::ffi::c_char
        },
    );
    lua_pushstring(lstate, s);
    if lua_pcall(
        lstate,
        2 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    ) != 0
    {
        luaL_error(
            lstate,
            b"treesitter logger callback failed\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
}
unsafe extern "C" fn parser_set_logger(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
    (lua_type(L, 2 as ::core::ffi::c_int) == 1 as ::core::ffi::c_int
        || luaL_argerror(
            L,
            2 as ::core::ffi::c_int,
            b"boolean expected\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    (lua_type(L, 3 as ::core::ffi::c_int) == 1 as ::core::ffi::c_int
        || luaL_argerror(
            L,
            3 as ::core::ffi::c_int,
            b"boolean expected\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    (lua_type(L, 4 as ::core::ffi::c_int) == 6 as ::core::ffi::c_int
        || luaL_argerror(
            L,
            4 as ::core::ffi::c_int,
            b"function expected\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    let mut opts: *mut TSLuaLoggerOpts =
        xmalloc(::core::mem::size_of::<TSLuaLoggerOpts>()) as *mut TSLuaLoggerOpts;
    lua_pushvalue(L, 4 as ::core::ffi::c_int);
    let mut ref_0: LuaRef = luaL_ref(L, LUA_REGISTRYINDEX);
    *opts = TSLuaLoggerOpts {
        cb: ref_0,
        lstate: L,
        lex: lua_toboolean(L, 2 as ::core::ffi::c_int) != 0,
        parse: lua_toboolean(L, 3 as ::core::ffi::c_int) != 0,
    };
    let mut logger: TSLogger = TSLogger {
        payload: opts as *mut ::core::ffi::c_void,
        log: Some(
            logger_cb
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    TSLogType,
                    *const ::core::ffi::c_char,
                ) -> (),
        ),
    };
    ts_parser_set_logger(p, logger);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn parser_get_logger(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
    let mut logger: TSLogger = ts_parser_logger(p);
    if logger.log.is_some() {
        let mut opts: *mut TSLuaLoggerOpts = logger.payload as *mut TSLuaLoggerOpts;
        lua_rawgeti(L, LUA_REGISTRYINDEX, (*opts).cb as ::core::ffi::c_int);
    } else {
        lua_pushnil(L);
    }
    return 1 as ::core::ffi::c_int;
}
static mut tree_meta: [luaL_Reg; 7] = [
    luaL_Reg {
        name: b"__gc\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(tree_gc as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(tree_tostring as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"root\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(tree_root as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"edit\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(tree_edit as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"included_ranges\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(tree_get_ranges as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"copy\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(tree_copy as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
];
unsafe extern "C" fn push_tree(mut L: *mut lua_State, mut tree: *const TSTree) {
    if tree.is_null() {
        lua_pushnil(L);
        return;
    }
    let mut ud: *mut TSLuaTree =
        lua_newuserdata(L, ::core::mem::size_of::<TSLuaTree>()) as *mut TSLuaTree;
    (*ud).tree = tree;
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_TREE.as_ptr());
    lua_setmetatable(L, -2 as ::core::ffi::c_int);
}
unsafe extern "C" fn tree_copy(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut ud: *mut TSLuaTree =
        luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
    let mut copy: *mut TSTree = ts_tree_copy((*ud).tree);
    push_tree(L, copy);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn tree_edit(mut L: *mut lua_State) -> ::core::ffi::c_int {
    if lua_gettop(L) < 10 as ::core::ffi::c_int {
        lua_pushstring(
            L,
            b"not enough args to tree:edit()\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return lua_error(L);
    }
    let mut ud: *mut TSLuaTree =
        luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
    let mut start_byte: uint32_t =
        luaL_checkinteger(L, 2 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t;
    let mut old_end_byte: uint32_t =
        luaL_checkinteger(L, 3 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t;
    let mut new_end_byte: uint32_t =
        luaL_checkinteger(L, 4 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t;
    let mut start_point: TSPoint = TSPoint {
        row: luaL_checkinteger(L, 5 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t,
        column: luaL_checkinteger(L, 6 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t,
    };
    let mut old_end_point: TSPoint = TSPoint {
        row: luaL_checkinteger(L, 7 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t,
        column: luaL_checkinteger(L, 8 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t,
    };
    let mut new_end_point: TSPoint = TSPoint {
        row: luaL_checkinteger(L, 9 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t,
        column: luaL_checkinteger(L, 10 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t,
    };
    let mut edit: TSInputEdit = TSInputEdit {
        start_byte: start_byte,
        old_end_byte: old_end_byte,
        new_end_byte: new_end_byte,
        start_point: start_point,
        old_end_point: old_end_point,
        new_end_point: new_end_point,
    };
    let mut new_tree: *mut TSTree = ts_tree_copy((*ud).tree);
    ts_tree_edit(new_tree, &raw mut edit);
    push_tree(L, new_tree);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn tree_get_ranges(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut ud: *mut TSLuaTree =
        luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
    let mut include_bytes: bool =
        lua_gettop(L) >= 2 as ::core::ffi::c_int && lua_toboolean(L, 2 as ::core::ffi::c_int) != 0;
    let mut len: uint32_t = 0;
    let mut ranges: *mut TSRange = ts_tree_included_ranges((*ud).tree, &raw mut len);
    push_ranges(L, ranges, len as size_t, include_bytes);
    xfree(ranges as *mut ::core::ffi::c_void);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn tree_gc(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut ud: *mut TSLuaTree =
        luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
    let mut tree: *mut TSTree = (*ud).tree as *mut TSTree;
    ts_tree_delete(tree);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tree_tostring(mut L: *mut lua_State) -> ::core::ffi::c_int {
    lua_pushstring(L, b"<tree>\0".as_ptr() as *const ::core::ffi::c_char);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn tree_root(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut ud: *mut TSLuaTree =
        luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
    let mut root: TSNode = ts_tree_root_node((*ud).tree);
    let mut node_ud: *mut TSNode =
        lua_newuserdata(L, ::core::mem::size_of::<TSNode>()) as *mut TSNode;
    *node_ud = root;
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_NODE.as_ptr());
    lua_setmetatable(L, -2 as ::core::ffi::c_int);
    lua_createtable(L, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    lua_pushvalue(L, 1 as ::core::ffi::c_int);
    lua_rawseti(L, -2 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    lua_setfenv(L, -2 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
static mut node_meta: [luaL_Reg; 36] = [
    luaL_Reg {
        name: b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_tostring as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"__eq\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_eq as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"__len\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_child_count as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"id\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_id as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"range\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_range as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"start\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_start as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"end_\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_end as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"type\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_type as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"symbol\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_symbol as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"field\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_field as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"named\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_named as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"missing\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_missing as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"extra\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_extra as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"has_changes\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_has_changes as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"has_error\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_has_error as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"sexpr\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_sexpr as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"child_count\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_child_count as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"named_child_count\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_named_child_count as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"child\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_child as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"named_child\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_named_child as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"descendant_for_range\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_descendant_for_range as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"named_descendant_for_range\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_named_descendant_for_range
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"parent\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_parent as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"__has_ancestor\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(__has_ancestor as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"child_with_descendant\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_child_with_descendant
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"iter_children\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_iter_children as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"next_sibling\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_next_sibling as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"prev_sibling\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_prev_sibling as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"next_named_sibling\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_next_named_sibling as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"prev_named_sibling\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_prev_named_sibling as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"named_children\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_named_children as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"root\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_root as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"tree\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_tree as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"byte_length\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_byte_length as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"equal\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_equal as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
];
unsafe extern "C" fn push_node(
    mut L: *mut lua_State,
    mut node: TSNode,
    mut uindex: ::core::ffi::c_int,
) {
    '_c2rust_label: {
        if uindex > 0 as ::core::ffi::c_int || uindex < -20 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"uindex > 0 || uindex < -LUA_MINSTACK\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/lua/treesitter.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                941 as ::core::ffi::c_uint,
                b"void push_node(lua_State *, TSNode, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if ts_node_is_null(node) {
        lua_pushnil(L);
        return;
    }
    let mut ud: *mut TSNode = lua_newuserdata(L, ::core::mem::size_of::<TSNode>()) as *mut TSNode;
    *ud = node;
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_NODE.as_ptr());
    lua_setmetatable(L, -2 as ::core::ffi::c_int);
    lua_getfenv(L, uindex);
    lua_setfenv(L, -2 as ::core::ffi::c_int);
}
unsafe extern "C" fn node_check_opt(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
    mut res: *mut TSNode,
) -> bool {
    let mut ud: *mut TSNode = luaL_checkudata(L, index, TS_META_NODE.as_ptr()) as *mut TSNode;
    if !ud.is_null() {
        *res = *ud;
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn node_check(mut L: *mut lua_State, mut index: ::core::ffi::c_int) -> TSNode {
    let mut ud: *mut TSNode = luaL_checkudata(L, index, TS_META_NODE.as_ptr()) as *mut TSNode;
    return *ud;
}
unsafe extern "C" fn node_tostring(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_pushstring(L, b"<node \0".as_ptr() as *const ::core::ffi::c_char);
    lua_pushstring(L, ts_node_type(node));
    lua_pushstring(L, b">\0".as_ptr() as *const ::core::ffi::c_char);
    lua_concat(L, 3 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_eq(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut node2: TSNode = node_check(L, 2 as ::core::ffi::c_int);
    lua_pushboolean(L, ts_node_eq(node, node2) as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_id(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_pushlstring(
        L,
        &raw mut node.id as *const ::core::ffi::c_char,
        ::core::mem::size_of::<*const ::core::ffi::c_void>(),
    );
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_range(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut include_bytes: bool =
        lua_gettop(L) >= 2 as ::core::ffi::c_int && lua_toboolean(L, 2 as ::core::ffi::c_int) != 0;
    let mut start: TSPoint = ts_node_start_point(node);
    let mut end: TSPoint = ts_node_end_point(node);
    if include_bytes {
        lua_pushinteger(L, start.row as lua_Integer);
        lua_pushinteger(L, start.column as lua_Integer);
        lua_pushinteger(L, ts_node_start_byte(node) as lua_Integer);
        lua_pushinteger(L, end.row as lua_Integer);
        lua_pushinteger(L, end.column as lua_Integer);
        lua_pushinteger(L, ts_node_end_byte(node) as lua_Integer);
        return 6 as ::core::ffi::c_int;
    }
    lua_pushinteger(L, start.row as lua_Integer);
    lua_pushinteger(L, start.column as lua_Integer);
    lua_pushinteger(L, end.row as lua_Integer);
    lua_pushinteger(L, end.column as lua_Integer);
    return 4 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_start(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut start: TSPoint = ts_node_start_point(node);
    let mut start_byte: uint32_t = ts_node_start_byte(node);
    lua_pushinteger(L, start.row as lua_Integer);
    lua_pushinteger(L, start.column as lua_Integer);
    lua_pushinteger(L, start_byte as lua_Integer);
    return 3 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_end(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut end: TSPoint = ts_node_end_point(node);
    let mut end_byte: uint32_t = ts_node_end_byte(node);
    lua_pushinteger(L, end.row as lua_Integer);
    lua_pushinteger(L, end.column as lua_Integer);
    lua_pushinteger(L, end_byte as lua_Integer);
    return 3 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_child_count(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut count: uint32_t = ts_node_child_count(node);
    lua_pushinteger(L, count as lua_Integer);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_named_child_count(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut count: uint32_t = ts_node_named_child_count(node);
    lua_pushinteger(L, count as lua_Integer);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_type(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_pushstring(L, ts_node_type(node));
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_symbol(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut symbol: TSSymbol = ts_node_symbol(node);
    lua_pushinteger(L, symbol as lua_Integer);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_field(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut count: uint32_t = ts_node_child_count(node);
    let mut curr_index: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut name_len: size_t = 0;
    let mut field_name: *const ::core::ffi::c_char =
        luaL_checklstring(L, 2 as ::core::ffi::c_int, &raw mut name_len);
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    let mut i: uint32_t = 0 as uint32_t;
    while i < count {
        let mut child_field_name: *const ::core::ffi::c_char =
            ts_node_field_name_for_child(node, i);
        if strequal(field_name, child_field_name) {
            let mut child: TSNode = ts_node_child(node, i);
            push_node(L, child, 1 as ::core::ffi::c_int);
            curr_index += 1;
            lua_rawseti(L, -2 as ::core::ffi::c_int, curr_index);
        }
        i = i.wrapping_add(1);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_named(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_pushboolean(L, ts_node_is_named(node) as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_sexpr(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut allocated: *mut ::core::ffi::c_char = ts_node_string(node);
    lua_pushstring(L, allocated);
    xfree(allocated as *mut ::core::ffi::c_void);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_missing(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_pushboolean(L, ts_node_is_missing(node) as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_extra(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_pushboolean(L, ts_node_is_extra(node) as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_has_changes(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_pushboolean(L, ts_node_has_changes(node) as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_has_error(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_pushboolean(L, ts_node_has_error(node) as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_child(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut num: uint32_t = lua_tointeger(L, 2 as ::core::ffi::c_int) as uint32_t;
    let mut child: TSNode = ts_node_child(node, num);
    push_node(L, child, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_named_child(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut num: uint32_t = lua_tointeger(L, 2 as ::core::ffi::c_int) as uint32_t;
    let mut child: TSNode = ts_node_named_child(node, num);
    push_node(L, child, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_descendant_for_range(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut start: TSPoint = TSPoint {
        row: lua_tointeger(L, 2 as ::core::ffi::c_int) as uint32_t,
        column: lua_tointeger(L, 3 as ::core::ffi::c_int) as uint32_t,
    };
    let mut end: TSPoint = TSPoint {
        row: lua_tointeger(L, 4 as ::core::ffi::c_int) as uint32_t,
        column: lua_tointeger(L, 5 as ::core::ffi::c_int) as uint32_t,
    };
    let mut child: TSNode = ts_node_descendant_for_point_range(node, start, end);
    push_node(L, child, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_named_descendant_for_range(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut start: TSPoint = TSPoint {
        row: lua_tointeger(L, 2 as ::core::ffi::c_int) as uint32_t,
        column: lua_tointeger(L, 3 as ::core::ffi::c_int) as uint32_t,
    };
    let mut end: TSPoint = TSPoint {
        row: lua_tointeger(L, 4 as ::core::ffi::c_int) as uint32_t,
        column: lua_tointeger(L, 5 as ::core::ffi::c_int) as uint32_t,
    };
    let mut child: TSNode = ts_node_named_descendant_for_point_range(node, start, end);
    push_node(L, child, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_next_child(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut child_index: *mut uint32_t =
        lua_touserdata(L, LUA_GLOBALSINDEX - 1 as ::core::ffi::c_int) as *mut uint32_t;
    let mut source: TSNode = node_check(L, LUA_GLOBALSINDEX - 2 as ::core::ffi::c_int);
    if *child_index >= ts_node_child_count(source) {
        return 0 as ::core::ffi::c_int;
    }
    let mut child: TSNode = ts_node_child(source, *child_index);
    push_node(L, child, LUA_GLOBALSINDEX - 2 as ::core::ffi::c_int);
    let mut field: *const ::core::ffi::c_char = ts_node_field_name_for_child(source, *child_index);
    if !field.is_null() {
        lua_pushstring(L, field);
    } else {
        lua_pushnil(L);
    }
    *child_index = (*child_index).wrapping_add(1);
    return 2 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_iter_children(mut L: *mut lua_State) -> ::core::ffi::c_int {
    node_check(L, 1 as ::core::ffi::c_int);
    let mut child_index: *mut uint32_t =
        lua_newuserdata(L, ::core::mem::size_of::<uint32_t>()) as *mut uint32_t;
    *child_index = 0 as uint32_t;
    lua_pushvalue(L, 1 as ::core::ffi::c_int);
    lua_pushcclosure(
        L,
        Some(node_next_child as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        2 as ::core::ffi::c_int,
    );
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_parent(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut parent: TSNode = ts_node_parent(node);
    push_node(L, parent, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn __has_ancestor(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut descendant: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    if lua_type(L, 2 as ::core::ffi::c_int) != LUA_TTABLE {
        lua_pushboolean(L, false_0);
        return 1 as ::core::ffi::c_int;
    }
    let pred_len: ::core::ffi::c_int = lua_objlen(L, 2 as ::core::ffi::c_int) as ::core::ffi::c_int;
    let mut node: TSNode = ts_tree_root_node(descendant.tree);
    while node.id != descendant.id && !ts_node_is_null(node) {
        let mut node_type_0: *const ::core::ffi::c_char = ts_node_type(node);
        let mut node_type_len: size_t = strlen(node_type_0);
        let mut i: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
        while i <= pred_len {
            lua_rawgeti(L, 2 as ::core::ffi::c_int, i);
            if lua_type(L, -1 as ::core::ffi::c_int) == LUA_TSTRING {
                let mut check_len: size_t = 0;
                let mut check_str: *const ::core::ffi::c_char =
                    lua_tolstring(L, -1 as ::core::ffi::c_int, &raw mut check_len);
                if node_type_len == check_len
                    && memcmp(
                        node_type_0 as *const ::core::ffi::c_void,
                        check_str as *const ::core::ffi::c_void,
                        check_len,
                    ) == 0 as ::core::ffi::c_int
                {
                    lua_pushboolean(L, true_0);
                    return 1 as ::core::ffi::c_int;
                }
            }
            lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            i += 1;
        }
        node = ts_node_child_with_descendant(node, descendant);
    }
    lua_pushboolean(L, false_0);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_child_with_descendant(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut descendant: TSNode = node_check(L, 2 as ::core::ffi::c_int);
    let mut child: TSNode = ts_node_child_with_descendant(node, descendant);
    push_node(L, child, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_next_sibling(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut sibling: TSNode = ts_node_next_sibling(node);
    push_node(L, sibling, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_prev_sibling(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut sibling: TSNode = ts_node_prev_sibling(node);
    push_node(L, sibling, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_next_named_sibling(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut sibling: TSNode = ts_node_next_named_sibling(node);
    push_node(L, sibling, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_prev_named_sibling(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut sibling: TSNode = ts_node_prev_named_sibling(node);
    push_node(L, sibling, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_named_children(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut source: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    let mut curr_index: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut n: uint32_t = ts_node_child_count(source);
    let mut i: uint32_t = 0 as uint32_t;
    while i < n {
        let mut child: TSNode = ts_node_child(source, i);
        if ts_node_is_named(child) {
            push_node(L, child, 1 as ::core::ffi::c_int);
            curr_index += 1;
            lua_rawseti(L, -2 as ::core::ffi::c_int, curr_index);
        }
        i = i.wrapping_add(1);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_root(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut root: TSNode = ts_tree_root_node(node.tree);
    push_node(L, root, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_tree(mut L: *mut lua_State) -> ::core::ffi::c_int {
    node_check(L, 1 as ::core::ffi::c_int);
    lua_getfenv(L, 1 as ::core::ffi::c_int);
    lua_rawgeti(L, 2 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_byte_length(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut start_byte: uint32_t = ts_node_start_byte(node);
    let mut end_byte: uint32_t = ts_node_end_byte(node);
    lua_pushinteger(L, end_byte.wrapping_sub(start_byte) as lua_Integer);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn node_equal(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node1: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut node2: TSNode = node_check(L, 2 as ::core::ffi::c_int);
    lua_pushboolean(L, ts_node_eq(node1, node2) as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
static mut querycursor_meta: [luaL_Reg; 5] = [
    luaL_Reg {
        name: b"remove_match\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            querycursor_remove_match as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"next_capture\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            querycursor_next_capture as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"next_match\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            querycursor_next_match as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"__gc\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(querycursor_gc as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
];
unsafe extern "C" fn tslua_push_querycursor(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut query: *mut TSQuery = query_check(L, 2 as ::core::ffi::c_int);
    let mut cursor: *mut TSQueryCursor = ts_query_cursor_new();
    if lua_gettop(L) >= 3 as ::core::ffi::c_int
        && !(lua_type(L, 3 as ::core::ffi::c_int) == LUA_TNIL)
    {
        (lua_type(L, 3 as ::core::ffi::c_int) == 5 as ::core::ffi::c_int
            || luaL_argerror(
                L,
                3 as ::core::ffi::c_int,
                b"table expected\0".as_ptr() as *const ::core::ffi::c_char,
            ) != 0) as ::core::ffi::c_int;
    }
    lua_getfield(
        L,
        3 as ::core::ffi::c_int,
        b"start_row\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut start_row: uint32_t = luaL_checkinteger(L, -1 as ::core::ffi::c_int) as uint32_t;
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    lua_getfield(
        L,
        3 as ::core::ffi::c_int,
        b"start_col\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut start_col: uint32_t = luaL_checkinteger(L, -1 as ::core::ffi::c_int) as uint32_t;
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    lua_getfield(
        L,
        3 as ::core::ffi::c_int,
        b"end_row\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut end_row: uint32_t = luaL_checkinteger(L, -1 as ::core::ffi::c_int) as uint32_t;
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    lua_getfield(
        L,
        3 as ::core::ffi::c_int,
        b"end_col\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut end_col: uint32_t = luaL_checkinteger(L, -1 as ::core::ffi::c_int) as uint32_t;
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    ts_query_cursor_set_point_range(
        cursor,
        TSPoint {
            row: start_row,
            column: start_col,
        },
        TSPoint {
            row: end_row,
            column: end_col,
        },
    );
    lua_getfield(
        L,
        3 as ::core::ffi::c_int,
        b"max_start_depth\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if !(lua_type(L, -1 as ::core::ffi::c_int) == LUA_TNIL) {
        let mut max_start_depth: uint32_t =
            luaL_checkinteger(L, -1 as ::core::ffi::c_int) as uint32_t;
        ts_query_cursor_set_max_start_depth(cursor, max_start_depth);
    }
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    lua_getfield(
        L,
        3 as ::core::ffi::c_int,
        b"match_limit\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if !(lua_type(L, -1 as ::core::ffi::c_int) == LUA_TNIL) {
        let mut match_limit: uint32_t = luaL_checkinteger(L, -1 as ::core::ffi::c_int) as uint32_t;
        ts_query_cursor_set_match_limit(cursor, match_limit);
    }
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    ts_query_cursor_exec(cursor, query, node);
    let mut ud: *mut *mut TSQueryCursor =
        lua_newuserdata(L, ::core::mem::size_of::<*mut TSQueryCursor>()) as *mut *mut TSQueryCursor;
    *ud = cursor;
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_QUERYCURSOR.as_ptr());
    lua_setmetatable(L, -2 as ::core::ffi::c_int);
    lua_getfenv(L, 1 as ::core::ffi::c_int);
    lua_setfenv(L, -2 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn querycursor_remove_match(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut cursor: *mut TSQueryCursor = querycursor_check(L, 1 as ::core::ffi::c_int);
    let mut match_id: uint32_t = luaL_checkinteger(L, 2 as ::core::ffi::c_int) as uint32_t;
    ts_query_cursor_remove_match(cursor, match_id);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn querycursor_next_capture(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut cursor: *mut TSQueryCursor = querycursor_check(L, 1 as ::core::ffi::c_int);
    let mut match_0: TSQueryMatch = TSQueryMatch {
        id: 0,
        pattern_index: 0,
        capture_count: 0,
        captures: ::core::ptr::null::<TSQueryCapture>(),
    };
    let mut capture_index: uint32_t = 0;
    if !ts_query_cursor_next_capture(cursor, &raw mut match_0, &raw mut capture_index) {
        return 0 as ::core::ffi::c_int;
    }
    let mut capture: TSQueryCapture = *match_0.captures.offset(capture_index as isize);
    lua_pushinteger(L, capture.index.wrapping_add(1 as uint32_t) as lua_Integer);
    push_node(L, capture.node, 1 as ::core::ffi::c_int);
    push_querymatch(L, &raw mut match_0, 1 as ::core::ffi::c_int);
    return 3 as ::core::ffi::c_int;
}
unsafe extern "C" fn querycursor_next_match(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut cursor: *mut TSQueryCursor = querycursor_check(L, 1 as ::core::ffi::c_int);
    let mut match_0: TSQueryMatch = TSQueryMatch {
        id: 0,
        pattern_index: 0,
        capture_count: 0,
        captures: ::core::ptr::null::<TSQueryCapture>(),
    };
    if !ts_query_cursor_next_match(cursor, &raw mut match_0) {
        return 0 as ::core::ffi::c_int;
    }
    push_querymatch(L, &raw mut match_0, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn querycursor_check(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> *mut TSQueryCursor {
    let mut ud: *mut *mut TSQueryCursor =
        luaL_checkudata(L, index, TS_META_QUERYCURSOR.as_ptr()) as *mut *mut TSQueryCursor;
    (!(*ud).is_null()
        || luaL_argerror(
            L,
            index,
            b"TSQueryCursor expected\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    return *ud;
}
unsafe extern "C" fn querycursor_gc(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut cursor: *mut TSQueryCursor = querycursor_check(L, 1 as ::core::ffi::c_int);
    ts_query_cursor_delete(cursor);
    return 0 as ::core::ffi::c_int;
}
static mut querymatch_meta: [luaL_Reg; 3] = [
    luaL_Reg {
        name: b"info\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(querymatch_info as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"captures\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            querymatch_captures as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
];
unsafe extern "C" fn push_querymatch(
    mut L: *mut lua_State,
    mut match_0: *mut TSQueryMatch,
    mut uindex: ::core::ffi::c_int,
) {
    let mut ud: *mut TSQueryMatch =
        lua_newuserdata(L, ::core::mem::size_of::<TSQueryMatch>()) as *mut TSQueryMatch;
    *ud = *match_0;
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_QUERYMATCH.as_ptr());
    lua_setmetatable(L, -2 as ::core::ffi::c_int);
    lua_getfenv(L, uindex);
    lua_setfenv(L, -2 as ::core::ffi::c_int);
}
unsafe extern "C" fn querymatch_info(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut match_0: *mut TSQueryMatch =
        luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_QUERYMATCH.as_ptr())
            as *mut TSQueryMatch;
    lua_pushinteger(L, (*match_0).id as lua_Integer);
    lua_pushinteger(
        L,
        ((*match_0).pattern_index as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as lua_Integer,
    );
    return 2 as ::core::ffi::c_int;
}
unsafe extern "C" fn querymatch_captures(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut match_0: *mut TSQueryMatch =
        luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_QUERYMATCH.as_ptr())
            as *mut TSQueryMatch;
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    let mut i: size_t = 0 as size_t;
    while i < (*match_0).capture_count as size_t {
        let mut capture: TSQueryCapture = *(*match_0).captures.offset(i as isize);
        let mut index: ::core::ffi::c_int =
            capture.index as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
        lua_rawgeti(L, -1 as ::core::ffi::c_int, index);
        if lua_type(L, -1 as ::core::ffi::c_int) == LUA_TNIL {
            lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        }
        push_node(L, capture.node, 1 as ::core::ffi::c_int);
        lua_rawseti(
            L,
            -2 as ::core::ffi::c_int,
            lua_objlen(L, -2 as ::core::ffi::c_int) as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
        );
        lua_rawseti(L, -2 as ::core::ffi::c_int, index);
        i = i.wrapping_add(1);
    }
    return 1 as ::core::ffi::c_int;
}
static mut query_meta: [luaL_Reg; 6] = [
    luaL_Reg {
        name: b"__gc\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(query_gc as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(query_tostring as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"inspect\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(query_inspect as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"disable_capture\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            query_disable_capture as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"disable_pattern\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            query_disable_pattern as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
];
unsafe extern "C" fn tslua_parse_query(mut L: *mut lua_State) -> ::core::ffi::c_int {
    if lua_gettop(L) < 2 as ::core::ffi::c_int
        || lua_isstring(L, 1 as ::core::ffi::c_int) == 0
        || lua_isstring(L, 2 as ::core::ffi::c_int) == 0
    {
        return luaL_error(
            L,
            b"string expected\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut lang: *mut TSLanguage = lang_check(L, 1 as ::core::ffi::c_int);
    let mut len: size_t = 0;
    let mut src: *const ::core::ffi::c_char =
        lua_tolstring(L, 2 as ::core::ffi::c_int, &raw mut len);
    tslua_query_parse_count = tslua_query_parse_count.wrapping_add(1);
    let mut error_offset: uint32_t = 0;
    let mut error_type: TSQueryError = TSQueryErrorNone;
    let mut query: *mut TSQuery = ts_query_new(
        lang,
        src,
        len as uint32_t,
        &raw mut error_offset,
        &raw mut error_type,
    );
    if query.is_null() {
        let mut err_msg: [::core::ffi::c_char; 1025] = [0; 1025];
        query_err_string(
            src,
            error_offset as ::core::ffi::c_int,
            error_type,
            &raw mut err_msg as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
        );
        return luaL_error(
            L,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw mut err_msg as *mut ::core::ffi::c_char,
        );
    }
    let mut ud: *mut *mut TSQuery =
        lua_newuserdata(L, ::core::mem::size_of::<*mut TSQuery>()) as *mut *mut TSQuery;
    *ud = query;
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_QUERY.as_ptr());
    lua_setmetatable(L, -2 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn query_err_to_string(
    mut error_type: TSQueryError,
) -> *const ::core::ffi::c_char {
    match error_type as ::core::ffi::c_uint {
        1 => return b"Invalid syntax:\n\0".as_ptr() as *const ::core::ffi::c_char,
        2 => return b"Invalid node type \0".as_ptr() as *const ::core::ffi::c_char,
        3 => return b"Invalid field name \0".as_ptr() as *const ::core::ffi::c_char,
        4 => return b"Invalid capture name \0".as_ptr() as *const ::core::ffi::c_char,
        5 => return b"Impossible pattern:\n\0".as_ptr() as *const ::core::ffi::c_char,
        _ => return b"error\0".as_ptr() as *const ::core::ffi::c_char,
    };
}
unsafe extern "C" fn query_err_string(
    mut src: *const ::core::ffi::c_char,
    mut error_offset: ::core::ffi::c_int,
    mut error_type: TSQueryError,
    mut err: *mut ::core::ffi::c_char,
    mut errlen: size_t,
) {
    let mut line_start: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut error_line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut error_line_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut end_str: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    loop {
        let mut src_tmp: *const ::core::ffi::c_char = src.offset(line_start as isize);
        end_str = strchr(src_tmp, '\n' as ::core::ffi::c_int);
        let mut line_length: ::core::ffi::c_int = if !end_str.is_null() {
            end_str.offset_from(src_tmp) as ::core::ffi::c_int
        } else {
            strlen(src_tmp) as ::core::ffi::c_int
        };
        let mut line_end: ::core::ffi::c_int = line_start + line_length;
        if line_end > error_offset {
            error_line = src_tmp;
            error_line_len = line_length;
            break;
        } else {
            line_start = line_end + 1 as ::core::ffi::c_int;
            row += 1;
            if end_str.is_null() {
                break;
            }
        }
    }
    let mut column: ::core::ffi::c_int = error_offset - line_start;
    let mut type_msg: *const ::core::ffi::c_char = query_err_to_string(error_type);
    snprintf(
        err,
        errlen,
        b"Query error at %d:%d. %s\0".as_ptr() as *const ::core::ffi::c_char,
        row + 1 as ::core::ffi::c_int,
        column + 1 as ::core::ffi::c_int,
        type_msg,
    );
    let mut offset: size_t = strlen(err);
    errlen = errlen.wrapping_sub(offset);
    err = err.offset(offset as isize);
    if error_type as ::core::ffi::c_uint
        == TSQueryErrorNodeType as ::core::ffi::c_int as ::core::ffi::c_uint
        || error_type as ::core::ffi::c_uint
            == TSQueryErrorField as ::core::ffi::c_int as ::core::ffi::c_uint
        || error_type as ::core::ffi::c_uint
            == TSQueryErrorCapture as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut suffix: *const ::core::ffi::c_char = src.offset(error_offset as isize);
        let mut is_anonymous: bool = error_type as ::core::ffi::c_uint
            == TSQueryErrorNodeType as ::core::ffi::c_int as ::core::ffi::c_uint
            && *suffix.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '"' as ::core::ffi::c_int;
        let mut suffix_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut c: ::core::ffi::c_char = *suffix.offset(suffix_len as isize);
        if is_anonymous {
            let mut backslashes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while c as ::core::ffi::c_int != '"' as ::core::ffi::c_int
                || backslashes % 2 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
            {
                if c as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                    backslashes += 1 as ::core::ffi::c_int;
                } else {
                    backslashes = 0 as ::core::ffi::c_int;
                }
                suffix_len += 1;
                c = *suffix.offset(suffix_len as isize);
            }
        } else {
            while *(*__ctype_b_loc()).offset(c as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
                || c as ::core::ffi::c_int == '_' as ::core::ffi::c_int
                || c as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                || c as ::core::ffi::c_int == '.' as ::core::ffi::c_int
            {
                suffix_len += 1;
                c = *suffix.offset(suffix_len as isize);
            }
        }
        snprintf(
            err,
            errlen,
            b"\"%.*s\":\n\0".as_ptr() as *const ::core::ffi::c_char,
            suffix_len,
            suffix,
        );
        offset = strlen(err);
        errlen = errlen.wrapping_sub(offset);
        err = err.offset(offset as isize);
    }
    if error_line.is_null() {
        snprintf(
            err,
            errlen,
            b"Unexpected EOF\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    snprintf(
        err,
        errlen,
        b"%.*s\n%*s^\n\0".as_ptr() as *const ::core::ffi::c_char,
        error_line_len,
        error_line,
        column,
        b"\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
unsafe extern "C" fn query_check(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> *mut TSQuery {
    let mut ud: *mut *mut TSQuery =
        luaL_checkudata(L, index, TS_META_QUERY.as_ptr()) as *mut *mut TSQuery;
    (!(*ud).is_null()
        || luaL_argerror(
            L,
            index,
            b"TSQuery expected\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    return *ud;
}
unsafe extern "C" fn query_gc(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut query: *mut TSQuery = query_check(L, 1 as ::core::ffi::c_int);
    ts_query_delete(query);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn query_tostring(mut L: *mut lua_State) -> ::core::ffi::c_int {
    lua_pushstring(L, b"<query>\0".as_ptr() as *const ::core::ffi::c_char);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn query_inspect(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut query: *mut TSQuery = query_check(L, 1 as ::core::ffi::c_int);
    lua_createtable(L, 0 as ::core::ffi::c_int, 2 as ::core::ffi::c_int);
    let mut n_pat: uint32_t = ts_query_pattern_count(query);
    lua_createtable(L, n_pat as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    let mut i: size_t = 0 as size_t;
    while i < n_pat as size_t {
        let mut len: uint32_t = 0;
        let mut step: *const TSQueryPredicateStep =
            ts_query_predicates_for_pattern(query, i as uint32_t, &raw mut len);
        if len != 0 as uint32_t {
            lua_createtable(
                L,
                len as ::core::ffi::c_int / 4 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            lua_createtable(L, 3 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
            let mut nextpred: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            let mut nextitem: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            let mut k: size_t = 0 as size_t;
            while k < len as size_t {
                if (*step.offset(k as isize)).type_0 as ::core::ffi::c_uint
                    == TSQueryPredicateStepTypeDone as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let c2rust_fresh0 = nextpred;
                    nextpred = nextpred + 1;
                    lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh0);
                    lua_createtable(L, 3 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
                    nextitem = 1 as ::core::ffi::c_int;
                } else {
                    if (*step.offset(k as isize)).type_0 as ::core::ffi::c_uint
                        == TSQueryPredicateStepTypeString as ::core::ffi::c_int
                            as ::core::ffi::c_uint
                    {
                        let mut strlen_0: uint32_t = 0;
                        let mut str: *const ::core::ffi::c_char = ts_query_string_value_for_id(
                            query,
                            (*step.offset(k as isize)).value_id,
                            &raw mut strlen_0,
                        );
                        lua_pushlstring(L, str, strlen_0 as size_t);
                    } else if (*step.offset(k as isize)).type_0 as ::core::ffi::c_uint
                        == TSQueryPredicateStepTypeCapture as ::core::ffi::c_int
                            as ::core::ffi::c_uint
                    {
                        lua_pushinteger(
                            L,
                            (*step.offset(k as isize))
                                .value_id
                                .wrapping_add(1 as uint32_t)
                                as lua_Integer,
                        );
                    } else {
                        abort();
                    }
                    let c2rust_fresh1 = nextitem;
                    nextitem = nextitem + 1;
                    lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh1);
                }
                k = k.wrapping_add(1);
            }
            lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            lua_rawseti(
                L,
                -2 as ::core::ffi::c_int,
                i as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
            );
        }
        i = i.wrapping_add(1);
    }
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"patterns\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut n_captures: uint32_t = ts_query_capture_count(query);
    lua_createtable(L, n_captures as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    let mut i_0: size_t = 0 as size_t;
    while i_0 < n_captures as size_t {
        let mut strlen_1: uint32_t = 0;
        let mut str_0: *const ::core::ffi::c_char =
            ts_query_capture_name_for_id(query, i_0 as uint32_t, &raw mut strlen_1);
        lua_pushlstring(L, str_0, strlen_1 as size_t);
        lua_rawseti(
            L,
            -2 as ::core::ffi::c_int,
            i_0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
        );
        i_0 = i_0.wrapping_add(1);
    }
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"captures\0".as_ptr() as *const ::core::ffi::c_char,
    );
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn query_disable_capture(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut query: *mut TSQuery = query_check(L, 1 as ::core::ffi::c_int);
    let mut name_len: size_t = 0;
    let mut name: *const ::core::ffi::c_char =
        luaL_checklstring(L, 2 as ::core::ffi::c_int, &raw mut name_len);
    ts_query_disable_capture(query, name, name_len as uint32_t);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn query_disable_pattern(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut query: *mut TSQuery = query_check(L, 1 as ::core::ffi::c_int);
    let pattern_index: uint32_t = luaL_checkinteger(L, 2 as ::core::ffi::c_int) as uint32_t;
    ts_query_disable_pattern(query, pattern_index.wrapping_sub(1 as uint32_t));
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn build_meta(
    mut L: *mut lua_State,
    mut tname: *const ::core::ffi::c_char,
    mut meta: *const luaL_Reg,
) {
    if luaL_newmetatable(L, tname) != 0 {
        luaL_register(L, ::core::ptr::null::<::core::ffi::c_char>(), meta);
        lua_pushvalue(L, -1 as ::core::ffi::c_int);
        lua_setfield(
            L,
            -2 as ::core::ffi::c_int,
            b"__index\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
}
unsafe extern "C" fn tslua_init(mut L: *mut lua_State) {
    build_meta(
        L,
        TS_META_PARSER.as_ptr(),
        &raw mut parser_meta as *mut luaL_Reg,
    );
    build_meta(
        L,
        TS_META_TREE.as_ptr(),
        &raw mut tree_meta as *mut luaL_Reg,
    );
    build_meta(
        L,
        TS_META_NODE.as_ptr(),
        &raw mut node_meta as *mut luaL_Reg,
    );
    build_meta(
        L,
        TS_META_QUERY.as_ptr(),
        &raw mut query_meta as *mut luaL_Reg,
    );
    build_meta(
        L,
        TS_META_QUERYCURSOR.as_ptr(),
        &raw mut querycursor_meta as *mut luaL_Reg,
    );
    build_meta(
        L,
        TS_META_QUERYMATCH.as_ptr(),
        &raw mut querymatch_meta as *mut luaL_Reg,
    );
    ts_set_allocator(
        Some(xmalloc as unsafe extern "C" fn(size_t) -> *mut ::core::ffi::c_void),
        Some(xcalloc as unsafe extern "C" fn(size_t, size_t) -> *mut ::core::ffi::c_void),
        Some(
            xrealloc
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    size_t,
                ) -> *mut ::core::ffi::c_void,
        ),
        Some(xfree as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
    );
}
unsafe extern "C" fn tslua_get_language_version(mut L: *mut lua_State) -> ::core::ffi::c_int {
    lua_pushnumber(L, TREE_SITTER_LANGUAGE_VERSION as lua_Number);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn tslua_get_minimum_language_version(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    lua_pushnumber(L, TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION as lua_Number);
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn nlua_treesitter_free() {}
#[no_mangle]
pub unsafe extern "C" fn nlua_treesitter_init(lstate: *mut lua_State) {
    tslua_init(lstate);
    lua_pushcclosure(
        lstate,
        Some(tslua_push_parser as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_create_ts_parser\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(tslua_push_querycursor as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_create_ts_querycursor\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            tslua_add_language_from_object
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_ts_add_language_from_object\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(tslua_has_language as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_ts_has_language\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(tslua_remove_lang as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_ts_remove_language\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(tslua_inspect_lang as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_ts_inspect_language\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(tslua_parse_query as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_ts_parse_query\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            tslua_get_language_version
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_ts_get_language_version\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            tslua_get_minimum_language_version
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_ts_get_minimum_language_version\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
