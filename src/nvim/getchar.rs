extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
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
    static mut stderr: *mut FILE;
    fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn putc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
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
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xmemcpyz(
        dst: *mut ::core::ffi::c_void,
        src: *const ::core::ffi::c_void,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn arena_finish(arena: *mut Arena) -> ArenaMem;
    fn arena_mem_free(mem: ArenaMem);
    fn api_clear_error(value: *mut Error);
    fn nvim_paste(
        channel_id: uint64_t,
        data: String_0,
        crlf: Boolean,
        phase: Integer,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Boolean;
    static mut p_fs: ::core::ffi::c_int;
    static mut p_langmap: *mut ::core::ffi::c_char;
    static mut p_lrm: ::core::ffi::c_int;
    static mut p_lz: ::core::ffi::c_int;
    static mut p_mmd: OptInt;
    static mut p_paste: ::core::ffi::c_int;
    static mut p_sc: ::core::ffi::c_int;
    static mut p_smd: ::core::ffi::c_int;
    static mut p_timeout: ::core::ffi::c_int;
    static mut p_tm: OptInt;
    static mut p_ttimeout: ::core::ffi::c_int;
    static mut p_ttm: OptInt;
    static mut p_uc: OptInt;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn uv_strerror(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn get_cursor_line_ptr() -> *mut ::core::ffi::c_char;
    fn update_screen() -> ::core::ffi::c_int;
    fn setcursor();
    fn showmode() -> ::core::ffi::c_int;
    fn unshowmode(force: bool);
    fn edit_putchar(c: ::core::ffi::c_int, highlight: bool);
    fn edit_unputchar();
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static e_invarg2: [::core::ffi::c_char; 0];
    static e_invargNval: [::core::ffi::c_char; 0];
    static e_nesting: [::core::ffi::c_char; 0];
    static e_notopen_2: [::core::ffi::c_char; 0];
    static e_toocompl: [::core::ffi::c_char; 0];
    fn garbage_collect(testing: bool) -> bool;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn semsg_multiline(
        kind: *const ::core::ffi::c_char,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn iemsg(s: *const ::core::ffi::c_char);
    fn internal_error(where_0: *const ::core::ffi::c_char);
    fn tv_dict_has_key(d: *const dict_T, key: *const ::core::ffi::c_char) -> bool;
    fn tv_dict_get_bool(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        def: ::core::ffi::c_int,
    ) -> varnumber_T;
    fn tv_dict_get_string(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        save: bool,
    ) -> *mut ::core::ffi::c_char;
    fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T;
    fn tv_check_for_opt_dict_arg(
        args: *const typval_T,
        idx: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn set_vim_var_nr(idx: VimVarIndex, val: varnumber_T);
    fn multiqueue_empty(self_0: *mut MultiQueue) -> bool;
    fn check_secure() -> bool;
    fn update_topline_cursor();
    fn putcmdline(c: ::core::ffi::c_char, shift: bool);
    fn unputcmdline();
    fn redrawcmdline();
    fn redrawcmd();
    fn get_cmdline_info() -> *mut CmdlineInfo;
    static mut test_disable_char_avail: bool;
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn ga_concat_len(gap: *mut garray_T, s: *const ::core::ffi::c_char, len: size_t);
    fn ga_append(gap: *mut garray_T, c: uint8_t);
    static mut mod_mask: ::core::ffi::c_int;
    static mut vgetc_mod_mask: ::core::ffi::c_int;
    static mut vgetc_char: ::core::ffi::c_int;
    static mut cmdline_row: ::core::ffi::c_int;
    static mut redraw_cmdline: bool;
    static mut mode_displayed: bool;
    static mut cmdline_star: ::core::ffi::c_int;
    static mut msg_col: ::core::ffi::c_int;
    static mut msg_row: ::core::ffi::c_int;
    static mut msg_scroll: ::core::ffi::c_int;
    static mut msg_didout: bool;
    static mut did_emsg: ::core::ffi::c_int;
    static mut called_emsg: ::core::ffi::c_int;
    static mut need_wait_return: bool;
    static mut vgetc_busy: ::core::ffi::c_int;
    static mut debug_did_msg: bool;
    static mut may_garbage_collect: bool;
    static mut want_garbage_collect: bool;
    static mut mouse_grid: ::core::ffi::c_int;
    static mut mouse_row: ::core::ffi::c_int;
    static mut mouse_col: ::core::ffi::c_int;
    static mut firstwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut curbuf: *mut buf_T;
    static mut VIsual: pos_T;
    static mut VIsual_active: bool;
    static mut VIsual_select: bool;
    static mut VIsual_reselect: ::core::ffi::c_int;
    static mut redo_VIsual_busy: bool;
    static mut did_ai: bool;
    static mut State: ::core::ffi::c_int;
    static mut finish_op: bool;
    static mut exmode_active: bool;
    static mut pending_exmode_active: bool;
    static mut reg_recording: ::core::ffi::c_int;
    static mut reg_executing: ::core::ffi::c_int;
    static mut pending_end_reg_executing: bool;
    static mut no_mapping: ::core::ffi::c_int;
    static mut no_zero_mapping: ::core::ffi::c_int;
    static mut allow_keys: ::core::ffi::c_int;
    static mut restart_edit: ::core::ffi::c_int;
    static mut arrow_used: bool;
    static mut mapped_ctrl_c: ::core::ffi::c_int;
    static mut ctrl_c_interrupts: bool;
    static mut msg_silent: ::core::ffi::c_int;
    static mut emsg_silent: ::core::ffi::c_int;
    static mut cmd_silent: bool;
    static mut NameBuff: [::core::ffi::c_char; 4096];
    static mut typebuf: typebuf_T;
    static mut typebuf_was_empty: bool;
    static mut ex_normal_busy: ::core::ffi::c_int;
    static mut ignore_script: bool;
    static mut KeyTyped: bool;
    static mut KeyStuffed: ::core::ffi::c_int;
    static mut maptick: ::core::ffi::c_int;
    static mut must_redraw: ::core::ffi::c_int;
    static mut scriptout: *mut FILE;
    static mut got_int: bool;
    static mut did_outofmem_msg: bool;
    static mut did_swapwrite_msg: bool;
    static mut langmap_mapchar: [uint8_t; 256];
    static mut cmdwin_type: ::core::ffi::c_int;
    static mut typebuf_was_filled: bool;
    fn get_keystroke(events: *mut MultiQueue) -> ::core::ffi::c_int;
    fn ctrl_x_mode_not_default() -> bool;
    fn compl_status_local() -> bool;
    fn vim_is_ctrl_x_key(c: ::core::ffi::c_int) -> bool;
    fn special_to_buf(
        key: ::core::ffi::c_int,
        modifiers: ::core::ffi::c_int,
        escape_ks: bool,
        dst: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_uint;
    fn nlua_call_ref(
        ref_0: LuaRef,
        name: *const ::core::ffi::c_char,
        args: Array,
        mode: LuaRetMode,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn nlua_execute_on_key(c: ::core::ffi::c_int, typed_buf: *mut ::core::ffi::c_char) -> bool;
    fn get_maphash_list(state: ::core::ffi::c_int, c: ::core::ffi::c_int) -> *mut mapblock_T;
    fn get_buf_maphash_list(state: ::core::ffi::c_int, c: ::core::ffi::c_int) -> *mut mapblock_T;
    fn eval_map_expr(mp: *mut mapblock_T, c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn langmap_adjust_mb(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    static mut main_loop: Loop;
    fn utf_ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2CharInfo_impl(p: *const uint8_t, len: uintptr_t) -> int32_t;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_cptr2char_adv(pp: *mut *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn utfc_next_impl(cur: StrCharInfo) -> StrCharInfo;
    fn mb_unescape(pp: *mut *const ::core::ffi::c_char) -> *const ::core::ffi::c_char;
    static utf8len_tab: [uint8_t; 256];
    fn ml_sync_all(check_file: ::core::ffi::c_int, check_char: ::core::ffi::c_int, do_fsync: bool);
    fn is_mouse_key(c: ::core::ffi::c_int) -> bool;
    fn mouse_comp_pos(
        win: *mut win_T,
        rowp: *mut ::core::ffi::c_int,
        colp: *mut ::core::ffi::c_int,
        lnump: *mut linenr_T,
    ) -> bool;
    fn mouse_find_win_inner(
        gridp: *mut ::core::ffi::c_int,
        rowp: *mut ::core::ffi::c_int,
        colp: *mut ::core::ffi::c_int,
    ) -> *mut win_T;
    fn validate_cursor(wp: *mut win_T);
    fn win_col_off(wp: *mut win_T) -> ::core::ffi::c_int;
    fn add_to_showcmd(c: ::core::ffi::c_int) -> bool;
    fn push_showcmd();
    fn pop_showcmd();
    fn normal_cmd(oap: *mut oparg_T, toplevel: bool);
    fn clear_oparg(oap: *mut oparg_T);
    static mut repeat_luaref: LuaRef;
    fn file_open(
        ret_fp: *mut FileDescriptor,
        fname: *const ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        mode: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn file_open_stdin(fp: *mut FileDescriptor) -> ::core::ffi::c_int;
    fn file_close(fp: *mut FileDescriptor, do_fsync: bool) -> ::core::ffi::c_int;
    fn file_read(
        fp: *mut FileDescriptor,
        ret_buf: *mut ::core::ffi::c_char,
        size: size_t,
    ) -> ptrdiff_t;
    fn input_get(
        buf: *mut uint8_t,
        maxlen: ::core::ffi::c_int,
        ms: ::core::ffi::c_int,
        tb_change_cnt: ::core::ffi::c_int,
        events: *mut MultiQueue,
    ) -> ::core::ffi::c_int;
    fn os_breakcheck();
    fn line_breakcheck();
    fn input_available() -> size_t;
    fn expand_env(
        src: *mut ::core::ffi::c_char,
        dst: *mut ::core::ffi::c_char,
        dstlen: ::core::ffi::c_int,
    ) -> size_t;
    fn init_charsize_arg(
        csarg: *mut CharsizeArg,
        wp: *mut win_T,
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
    ) -> CSType;
    fn charsize_regular(
        csarg: *mut CharsizeArg,
        cur: *mut ::core::ffi::c_char,
        vcol: colnr_T,
        cur_char: int32_t,
    ) -> CharSize;
    fn charsize_fast(
        csarg: *mut CharsizeArg,
        cur: *const ::core::ffi::c_char,
        vcol: colnr_T,
        cur_char: int32_t,
    ) -> CharSize;
    fn state_handle_k_event();
    fn get_real_state() -> ::core::ffi::c_int;
    fn state_no_longer_safe(reason: *const ::core::ffi::c_char);
    fn ui_busy_start();
    fn ui_busy_stop();
    fn vim_beep(val: ::core::ffi::c_uint);
    fn ui_cursor_goto(new_row: ::core::ffi::c_int, new_col: ::core::ffi::c_int);
    fn ui_flush();
    fn u_sync(force: bool);
}
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type __time_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type uintptr_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: ::core::ffi::c_int,
    pub _IO_read_ptr: *mut ::core::ffi::c_char,
    pub _IO_read_end: *mut ::core::ffi::c_char,
    pub _IO_read_base: *mut ::core::ffi::c_char,
    pub _IO_write_base: *mut ::core::ffi::c_char,
    pub _IO_write_ptr: *mut ::core::ffi::c_char,
    pub _IO_write_end: *mut ::core::ffi::c_char,
    pub _IO_buf_base: *mut ::core::ffi::c_char,
    pub _IO_buf_end: *mut ::core::ffi::c_char,
    pub _IO_save_base: *mut ::core::ffi::c_char,
    pub _IO_backup_base: *mut ::core::ffi::c_char,
    pub _IO_save_end: *mut ::core::ffi::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: ::core::ffi::c_int,
    pub _flags2: ::core::ffi::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: ::core::ffi::c_ushort,
    pub _vtable_offset: ::core::ffi::c_schar,
    pub _shortbuf: [::core::ffi::c_char; 1],
    pub _lock: *mut ::core::ffi::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut ::core::ffi::c_void,
    pub _prevchain: *mut *mut _IO_FILE,
    pub _mode: ::core::ffi::c_int,
    pub _unused2: [::core::ffi::c_char; 20],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;
pub type ssize_t = isize;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub union EvalFuncData {
    pub float_func: Option<unsafe extern "C" fn(float_T) -> float_T>,
    pub api_handler: *const MsgpackRpcRequestHandler,
    pub null: *mut ::core::ffi::c_void,
}
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
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const MAXMAPLEN: C2Rust_Unnamed_27 = 50;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MarkTreeIter {
    pub pos: MTPos,
    pub lvl: ::core::ffi::c_int,
    pub x: *mut MTNode,
    pub i: ::core::ffi::c_int,
    pub s: [C2Rust_Unnamed_28; 20],
    pub intersect_idx: size_t,
    pub intersect_pos: MTPos,
    pub intersect_pos_x: MTPos,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_28 {
    pub oldcol: ::core::ffi::c_int,
    pub i: ::core::ffi::c_int,
}
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
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const kOptBoFlagWildmode: C2Rust_Unnamed_29 = 524288;
pub const kOptBoFlagTerm: C2Rust_Unnamed_29 = 262144;
pub const kOptBoFlagSpell: C2Rust_Unnamed_29 = 131072;
pub const kOptBoFlagShell: C2Rust_Unnamed_29 = 65536;
pub const kOptBoFlagRegister: C2Rust_Unnamed_29 = 32768;
pub const kOptBoFlagOperator: C2Rust_Unnamed_29 = 16384;
pub const kOptBoFlagShowmatch: C2Rust_Unnamed_29 = 8192;
pub const kOptBoFlagMess: C2Rust_Unnamed_29 = 4096;
pub const kOptBoFlagLang: C2Rust_Unnamed_29 = 2048;
pub const kOptBoFlagInsertmode: C2Rust_Unnamed_29 = 1024;
pub const kOptBoFlagHangul: C2Rust_Unnamed_29 = 512;
pub const kOptBoFlagEx: C2Rust_Unnamed_29 = 256;
pub const kOptBoFlagEsc: C2Rust_Unnamed_29 = 128;
pub const kOptBoFlagError: C2Rust_Unnamed_29 = 64;
pub const kOptBoFlagCtrlg: C2Rust_Unnamed_29 = 32;
pub const kOptBoFlagCopy: C2Rust_Unnamed_29 = 16;
pub const kOptBoFlagComplete: C2Rust_Unnamed_29 = 8;
pub const kOptBoFlagCursor: C2Rust_Unnamed_29 = 4;
pub const kOptBoFlagBackspace: C2Rust_Unnamed_29 = 2;
pub const kOptBoFlagAll: C2Rust_Unnamed_29 = 1;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharInfo {
    pub value: int32_t,
    pub len: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StrCharInfo {
    pub ptr: *mut ::core::ffi::c_char,
    pub chr: CharInfo,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FileDescriptor {
    pub fd: ::core::ffi::c_int,
    pub buffer: *mut ::core::ffi::c_char,
    pub read_pos: *mut ::core::ffi::c_char,
    pub write_pos: *mut ::core::ffi::c_char,
    pub wr: bool,
    pub eof: bool,
    pub non_blocking: bool,
    pub bytes_read: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct buffblock {
    pub b_next: *mut buffblock,
    pub b_strlen: size_t,
    pub b_str: [::core::ffi::c_char; 1],
}
pub type buffblock_T = buffblock;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct buffheader_T {
    pub bh_first: buffblock_T,
    pub bh_curr: *mut buffblock_T,
    pub bh_index: size_t,
    pub bh_space: size_t,
    pub bh_create_newblock: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct save_redo_T {
    pub sr_redobuff: buffheader_T,
    pub sr_old_redobuff: buffheader_T,
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tasave_T {
    pub save_typebuf: typebuf_T,
    pub typebuf_valid: bool,
    pub old_char: ::core::ffi::c_int,
    pub old_mod_mask: ::core::ffi::c_int,
    pub save_readbuf1: buffheader_T,
    pub save_readbuf2: buffheader_T,
    pub save_inputbuf: String_0,
}
pub type RemapValues = ::core::ffi::c_int;
pub const REMAP_SKIP: RemapValues = -3;
pub const REMAP_SCRIPT: RemapValues = -2;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CmdlineColorChunk {
    pub start: ::core::ffi::c_int,
    pub end: ::core::ffi::c_int,
    pub hl_id: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CmdlineColors {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut CmdlineColorChunk,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ColoredCmdline {
    pub prompt_id: ::core::ffi::c_uint,
    pub cmdbuff: *mut ::core::ffi::c_char,
    pub colors: CmdlineColors,
}
pub type CmdRedraw = ::core::ffi::c_uint;
pub const kCmdRedrawAll: CmdRedraw = 2;
pub const kCmdRedrawPos: CmdRedraw = 1;
pub const kCmdRedrawNone: CmdRedraw = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdline_info {
    pub cmdbuff: *mut ::core::ffi::c_char,
    pub cmdbufflen: ::core::ffi::c_int,
    pub cmdlen: ::core::ffi::c_int,
    pub cmdpos: ::core::ffi::c_int,
    pub cmdspos: ::core::ffi::c_int,
    pub cmdfirstc: ::core::ffi::c_int,
    pub cmdindent: ::core::ffi::c_int,
    pub cmdprompt: *mut ::core::ffi::c_char,
    pub hl_id: ::core::ffi::c_int,
    pub overstrike: ::core::ffi::c_int,
    pub xpc: *mut expand_T,
    pub xp_context: ::core::ffi::c_int,
    pub xp_arg: *mut ::core::ffi::c_char,
    pub input_fn: ::core::ffi::c_int,
    pub cmdbuff_replaced: bool,
    pub prompt_id: ::core::ffi::c_uint,
    pub highlight_callback: Callback,
    pub last_colors: ColoredCmdline,
    pub level: ::core::ffi::c_int,
    pub prev_ccline: *mut CmdlineInfo,
    pub special_char: ::core::ffi::c_char,
    pub special_shift: bool,
    pub redraw_state: CmdRedraw,
    pub one_key: bool,
    pub mouse_used: *mut bool,
}
pub type CmdlineInfo = cmdline_info;
pub type flush_buffers_T = ::core::ffi::c_uint;
pub const FLUSH_INPUT: flush_buffers_T = 2;
pub const FLUSH_TYPEAHEAD: flush_buffers_T = 1;
pub const FLUSH_MINIMAL: flush_buffers_T = 0;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const NSCRIPT: C2Rust_Unnamed_30 = 15;
pub const MODE_HITRETURN: C2Rust_Unnamed_32 = 8193;
pub const RM_SCRIPT: C2Rust_Unnamed_36 = 2;
pub const RM_NONE: C2Rust_Unnamed_36 = 1;
pub const RM_YES: C2Rust_Unnamed_36 = 0;
pub const RM_ABBR: C2Rust_Unnamed_36 = 4;
pub const KE_IGNORE: key_extra = 53;
pub const MODE_CMDLINE: C2Rust_Unnamed_32 = 8;
pub const MODE_INSERT: C2Rust_Unnamed_32 = 16;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct gotchars_state_T {
    pub buf: [uint8_t; 67],
    pub prev_c: ::core::ffi::c_int,
    pub buflen: size_t,
    pub pending_special: ::core::ffi::c_uint,
    pub pending_mbyte: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_31 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_char,
    pub init_array: [::core::ffi::c_char; 51],
}
pub const KEYLEN_PART_KEY: C2Rust_Unnamed_37 = -1;
pub const SHOWCMD_COLS: C2Rust_Unnamed_33 = 10;
pub const MODE_LANGMAP: C2Rust_Unnamed_32 = 32;
pub const MODE_NORMAL: C2Rust_Unnamed_32 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharSize {
    pub width: ::core::ffi::c_int,
    pub head: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharsizeArg {
    pub win: *mut win_T,
    pub line: *mut ::core::ffi::c_char,
    pub use_tabstop: bool,
    pub indent_width: ::core::ffi::c_int,
    pub virt_row: ::core::ffi::c_int,
    pub cur_text_width_left: ::core::ffi::c_int,
    pub cur_text_width_right: ::core::ffi::c_int,
    pub max_head_vcol: ::core::ffi::c_int,
    pub iter: [MarkTreeIter; 1],
}
pub type CSType = bool;
pub const kCharsizeFast: C2Rust_Unnamed_35 = 1;
pub const map_result_get: map_result_T = 1;
pub type map_result_T = ::core::ffi::c_uint;
pub const map_result_nomatch: map_result_T = 3;
pub const map_result_retry: map_result_T = 2;
pub const map_result_fail: map_result_T = 0;
pub const MODE_VISUAL: C2Rust_Unnamed_32 = 2;
pub const MODE_TERMINAL: C2Rust_Unnamed_32 = 128;
pub const KEYLEN_PART_MAP: C2Rust_Unnamed_37 = -2;
pub const KE_SNR: key_extra = 82;
pub const MODE_SELECT: C2Rust_Unnamed_32 = 64;
pub const MODE_ASKMORE: C2Rust_Unnamed_32 = 12288;
pub const KE_PLUG: key_extra = 83;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct oparg_T {
    pub op_type: ::core::ffi::c_int,
    pub regname: ::core::ffi::c_int,
    pub motion_type: MotionType,
    pub motion_force: ::core::ffi::c_int,
    pub use_reg_one: bool,
    pub inclusive: bool,
    pub end_adjusted: bool,
    pub start: pos_T,
    pub end: pos_T,
    pub cursor_start: pos_T,
    pub line_count: linenr_T,
    pub empty: bool,
    pub is_VIsual: bool,
    pub start_vcol: colnr_T,
    pub end_vcol: colnr_T,
    pub prev_opcount: ::core::ffi::c_int,
    pub prev_count0: ::core::ffi::c_int,
    pub excl_tr_ws: bool,
}
pub type MotionType = ::core::ffi::c_int;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub const kFileReadOnly: C2Rust_Unnamed_34 = 1;
pub const kFileNonBlocking: C2Rust_Unnamed_34 = 128;
pub type LuaRetMode = ::core::ffi::c_uint;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub const KE_LUA: key_extra = 103;
pub const KE_COMMAND: key_extra = 104;
pub const KE_XRIGHT: key_extra = 68;
pub const KE_XLEFT: key_extra = 67;
pub const KE_XDOWN: key_extra = 66;
pub const KE_XUP: key_extra = 65;
pub const KE_C_END: key_extra = 88;
pub const KE_ZEND: key_extra = 62;
pub const KE_XEND: key_extra = 61;
pub const KE_C_HOME: key_extra = 87;
pub const KE_ZHOME: key_extra = 64;
pub const KE_XHOME: key_extra = 63;
pub const KE_MOUSEMOVE: key_extra = 100;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_32 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_32 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_32 = 16384;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_32 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_32 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_32 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_32 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_32 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_32 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_32 = 255;
pub const MODE_OP_PENDING: C2Rust_Unnamed_32 = 4;
pub type key_extra = ::core::ffi::c_uint;
pub const KE_WILD: key_extra = 108;
pub const KE_EVENT: key_extra = 102;
pub const KE_NOP: key_extra = 97;
pub const KE_DROP: key_extra = 95;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_CMDWIN: key_extra = 84;
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
pub const KE_XF4: key_extra = 60;
pub const KE_XF3: key_extra = 59;
pub const KE_XF2: key_extra = 58;
pub const KE_XF1: key_extra = 57;
pub const KE_S_TAB_OLD: key_extra = 55;
pub const KE_TAB: key_extra = 54;
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
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const kFileMkDir: C2Rust_Unnamed_34 = 256;
pub const kFileAppend: C2Rust_Unnamed_34 = 64;
pub const kFileTruncate: C2Rust_Unnamed_34 = 32;
pub const kFileCreateOnly: C2Rust_Unnamed_34 = 16;
pub const kFileNoSymlink: C2Rust_Unnamed_34 = 8;
pub const kFileWriteOnly: C2Rust_Unnamed_34 = 4;
pub const kFileCreate: C2Rust_Unnamed_34 = 2;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const kCharsizeRegular: C2Rust_Unnamed_35 = 0;
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_37 = ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
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
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const INTERNAL_CALL_MASK: uint64_t = (1 as ::core::ffi::c_int as uint64_t)
    << ::core::mem::size_of::<uint64_t>()
        .wrapping_mul(8 as usize)
        .wrapping_sub(1 as usize);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL.wrapping_add(1 as uint64_t);
#[inline(always)]
unsafe extern "C" fn is_internal_call(channel_id: uint64_t) -> bool {
    return channel_id & INTERNAL_CALL_MASK != 0;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const NL_STR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"\n\0") };
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const DEL: ::core::ffi::c_int = 0x7f as ::core::ffi::c_int;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const Ctrl_N: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const Ctrl_O: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const Ctrl_P: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
static mut curscript: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
static mut scriptin: [FileDescriptor; 15] = [
    FileDescriptor {
        fd: 0 as ::core::ffi::c_int,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
];
static mut redobuff: buffheader_T = buffheader_T {
    bh_first: buffblock {
        b_next: ::core::ptr::null_mut::<buffblock>(),
        b_strlen: 0 as size_t,
        b_str: [NUL as ::core::ffi::c_char],
    },
    bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
    bh_index: 0 as size_t,
    bh_space: 0 as size_t,
    bh_create_newblock: false_0 != 0,
};
static mut old_redobuff: buffheader_T = buffheader_T {
    bh_first: buffblock {
        b_next: ::core::ptr::null_mut::<buffblock>(),
        b_strlen: 0 as size_t,
        b_str: [NUL as ::core::ffi::c_char],
    },
    bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
    bh_index: 0 as size_t,
    bh_space: 0 as size_t,
    bh_create_newblock: false_0 != 0,
};
static mut recordbuff: buffheader_T = buffheader_T {
    bh_first: buffblock {
        b_next: ::core::ptr::null_mut::<buffblock>(),
        b_strlen: 0 as size_t,
        b_str: [NUL as ::core::ffi::c_char],
    },
    bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
    bh_index: 0 as size_t,
    bh_space: 0 as size_t,
    bh_create_newblock: false_0 != 0,
};
static mut readbuf1: buffheader_T = buffheader_T {
    bh_first: buffblock {
        b_next: ::core::ptr::null_mut::<buffblock>(),
        b_strlen: 0 as size_t,
        b_str: [NUL as ::core::ffi::c_char],
    },
    bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
    bh_index: 0 as size_t,
    bh_space: 0 as size_t,
    bh_create_newblock: false_0 != 0,
};
static mut readbuf2: buffheader_T = buffheader_T {
    bh_first: buffblock {
        b_next: ::core::ptr::null_mut::<buffblock>(),
        b_strlen: 0 as size_t,
        b_str: [NUL as ::core::ffi::c_char],
    },
    bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
    bh_index: 0 as size_t,
    bh_space: 0 as size_t,
    bh_create_newblock: false_0 != 0,
};
static mut on_key_buf: C2Rust_Unnamed_31 = C2Rust_Unnamed_31 {
    size: 0,
    capacity: 0,
    items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    init_array: [0; 51],
};
static mut on_key_ignore_len: size_t = 0 as size_t;
static mut typeahead_char: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut block_redo: bool = false_0 != 0;
static mut KeyNoremap: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut typebuf_init: [uint8_t; 265] = [0; 265];
static mut noremapbuf_init: [uint8_t; 265] = [0; 265];
static mut last_recorded_len: size_t = 0 as size_t;
static mut e_recursive_mapping: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E223: Recursive mapping\0")
};
static mut e_cmd_mapping_must_end_with_cr: [::core::ffi::c_char; 40] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"E1255: <Cmd> mapping must end with <CR>\0",
    )
};
static mut e_cmd_mapping_must_end_with_cr_before_second_cmd: [::core::ffi::c_char; 60] = unsafe {
    ::core::mem::transmute::<[u8; 60], [::core::ffi::c_char; 60]>(
        *b"E1136: <Cmd> mapping must end with <CR> before second <Cmd>\0",
    )
};
unsafe extern "C" fn free_buff(mut buf: *mut buffheader_T) {
    let mut np: *mut buffblock_T = ::core::ptr::null_mut::<buffblock_T>();
    let mut p: *mut buffblock_T = (*buf).bh_first.b_next as *mut buffblock_T;
    while !p.is_null() {
        np = (*p).b_next as *mut buffblock_T;
        xfree(p as *mut ::core::ffi::c_void);
        p = np;
    }
    (*buf).bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
    (*buf).bh_curr = ::core::ptr::null_mut::<buffblock_T>();
}
unsafe extern "C" fn get_buffcont(
    mut buffer: *mut buffheader_T,
    mut dozero: ::core::ffi::c_int,
    mut len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut count: size_t = 0 as size_t;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: size_t = 0 as size_t;
    let mut bp: *const buffblock_T = (*buffer).bh_first.b_next;
    while !bp.is_null() {
        count = count.wrapping_add((*bp).b_strlen);
        bp = (*bp).b_next;
    }
    if count > 0 as size_t || dozero != 0 {
        p = xmalloc(count.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        let mut p2: *mut ::core::ffi::c_char = p;
        let mut bp_0: *const buffblock_T = (*buffer).bh_first.b_next;
        while !bp_0.is_null() {
            let mut str: *const ::core::ffi::c_char =
                &raw const (*bp_0).b_str as *const ::core::ffi::c_char;
            while *str != 0 {
                let c2rust_fresh0 = str;
                str = str.offset(1);
                let c2rust_fresh1 = p2;
                p2 = p2.offset(1);
                *c2rust_fresh1 = *c2rust_fresh0;
            }
            bp_0 = (*bp_0).b_next;
        }
        *p2 = NUL as ::core::ffi::c_char;
        i = p2.offset_from(p) as size_t;
    }
    if !len.is_null() {
        *len = i;
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn get_recorded() -> *mut ::core::ffi::c_char {
    let mut len: size_t = 0;
    let mut p: *mut ::core::ffi::c_char = get_buffcont(&raw mut recordbuff, true_0, &raw mut len);
    if p.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    free_buff(&raw mut recordbuff);
    if len >= last_recorded_len {
        len = len.wrapping_sub(last_recorded_len);
        *p.offset(len as isize) = NUL as ::core::ffi::c_char;
    }
    if len > 0 as size_t
        && restart_edit != 0 as ::core::ffi::c_int
        && *p.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int == Ctrl_O
    {
        *p.offset(len.wrapping_sub(1 as size_t) as isize) = NUL as ::core::ffi::c_char;
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn get_inserted() -> String_0 {
    let mut len: size_t = 0 as size_t;
    let mut str: *mut ::core::ffi::c_char = get_buffcont(&raw mut redobuff, false_0, &raw mut len);
    return String_0 {
        data: str,
        size: len,
    };
}
unsafe extern "C" fn add_buff(
    buf: *mut buffheader_T,
    s: *const ::core::ffi::c_char,
    mut slen: ptrdiff_t,
) {
    if slen < 0 as ptrdiff_t {
        slen = strlen(s) as ptrdiff_t;
    }
    if slen == 0 as ptrdiff_t {
        return;
    }
    if (*buf).bh_first.b_next.is_null() {
        (*buf).bh_curr = &raw mut (*buf).bh_first;
        (*buf).bh_create_newblock = true_0 != 0;
    } else if (*buf).bh_curr.is_null() {
        iemsg(gettext(
            b"E222: Add to read buffer\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return;
    } else if (*buf).bh_index != 0 as size_t {
        memmove(
            &raw mut (*(*buf).bh_first.b_next).b_str as *mut ::core::ffi::c_char
                as *mut ::core::ffi::c_void,
            (&raw mut (*(*buf).bh_first.b_next).b_str as *mut ::core::ffi::c_char)
                .offset((*buf).bh_index as isize) as *const ::core::ffi::c_void,
            (*(*buf).bh_first.b_next)
                .b_strlen
                .wrapping_sub((*buf).bh_index)
                .wrapping_add(1 as size_t),
        );
        (*(*buf).bh_first.b_next).b_strlen = (*(*buf).bh_first.b_next)
            .b_strlen
            .wrapping_sub((*buf).bh_index);
        (*buf).bh_space = (*buf).bh_space.wrapping_add((*buf).bh_index);
    }
    (*buf).bh_index = 0 as size_t;
    if !(*buf).bh_create_newblock && (*buf).bh_space >= slen as size_t {
        xmemcpyz(
            (&raw mut (*(*buf).bh_curr).b_str as *mut ::core::ffi::c_char)
                .offset((*(*buf).bh_curr).b_strlen as isize)
                as *mut ::core::ffi::c_void,
            s as *const ::core::ffi::c_void,
            slen as size_t,
        );
        (*(*buf).bh_curr).b_strlen = (*(*buf).bh_curr).b_strlen.wrapping_add(slen as size_t);
        (*buf).bh_space = (*buf).bh_space.wrapping_sub(slen as size_t);
    } else {
        let mut len: size_t = if 20 as size_t > slen as size_t {
            20 as size_t
        } else {
            slen as size_t
        };
        let mut p: *mut buffblock_T =
            xmalloc((16 as size_t).wrapping_add(len).wrapping_add(1 as size_t)) as *mut buffblock_T;
        xmemcpyz(
            &raw mut (*p).b_str as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            s as *const ::core::ffi::c_void,
            slen as size_t,
        );
        (*p).b_strlen = slen as size_t;
        (*buf).bh_space = len.wrapping_sub(slen as size_t);
        (*buf).bh_create_newblock = false_0 != 0;
        (*p).b_next = (*(*buf).bh_curr).b_next;
        (*(*buf).bh_curr).b_next = p as *mut buffblock;
        (*buf).bh_curr = p;
    };
}
unsafe extern "C" fn delete_buff_tail(mut buf: *mut buffheader_T, mut slen: ::core::ffi::c_int) {
    if (*buf).bh_curr.is_null() {
        return;
    }
    if (*(*buf).bh_curr).b_strlen < slen as size_t {
        return;
    }
    *(&raw mut (*(*buf).bh_curr).b_str as *mut ::core::ffi::c_char)
        .offset((*(*buf).bh_curr).b_strlen.wrapping_sub(slen as size_t) as isize) =
        NUL as ::core::ffi::c_char;
    (*(*buf).bh_curr).b_strlen = (*(*buf).bh_curr).b_strlen.wrapping_sub(slen as size_t);
    (*buf).bh_space = (*buf).bh_space.wrapping_add(slen as size_t);
}
unsafe extern "C" fn add_num_buff(mut buf: *mut buffheader_T, mut n: ::core::ffi::c_int) {
    let mut number: [::core::ffi::c_char; 32] = [0; 32];
    let mut numberlen: ::core::ffi::c_int = snprintf(
        &raw mut number as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 32]>(),
        b"%d\0".as_ptr() as *const ::core::ffi::c_char,
        n,
    );
    add_buff(
        buf,
        &raw mut number as *mut ::core::ffi::c_char,
        numberlen as ptrdiff_t,
    );
}
unsafe extern "C" fn add_byte_buff(mut buf: *mut buffheader_T, mut c: ::core::ffi::c_int) {
    let mut temp: [::core::ffi::c_char; 4] = [0; 4];
    let mut templen: ptrdiff_t = 0;
    if c < 0 as ::core::ffi::c_int || c == K_SPECIAL || c == NUL {
        temp[0 as ::core::ffi::c_int as usize] = K_SPECIAL as ::core::ffi::c_char;
        temp[1 as ::core::ffi::c_int as usize] = (if c == K_SPECIAL {
            KS_SPECIAL
        } else if c == NUL {
            KS_ZERO
        } else {
            -c & 0xff as ::core::ffi::c_int
        }) as ::core::ffi::c_char;
        temp[2 as ::core::ffi::c_int as usize] = (if c == K_SPECIAL || c == NUL {
            KE_FILLER as ::core::ffi::c_uint
        } else {
            -c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint
        }) as ::core::ffi::c_char;
        temp[3 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        templen = 3 as ptrdiff_t;
    } else {
        temp[0 as ::core::ffi::c_int as usize] = c as ::core::ffi::c_char;
        temp[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        templen = 1 as ptrdiff_t;
    }
    add_buff(buf, &raw mut temp as *mut ::core::ffi::c_char, templen);
}
unsafe extern "C" fn add_char_buff(mut buf: *mut buffheader_T, mut c: ::core::ffi::c_int) {
    let mut bytes: [uint8_t; 22] = [0; 22];
    let mut len: ::core::ffi::c_int = 0;
    if c < 0 as ::core::ffi::c_int {
        len = 1 as ::core::ffi::c_int;
    } else {
        len = utf_char2bytes(
            c,
            &raw mut bytes as *mut uint8_t as *mut ::core::ffi::c_char,
        );
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < len {
        if !(c < 0 as ::core::ffi::c_int) {
            c = bytes[i as usize] as ::core::ffi::c_int;
        }
        add_byte_buff(buf, c);
        i += 1;
    }
}
unsafe extern "C" fn read_readbuffers(mut advance: bool) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = read_readbuf(&raw mut readbuf1, advance);
    if c == NUL {
        c = read_readbuf(&raw mut readbuf2, advance);
    }
    return c;
}
unsafe extern "C" fn read_readbuf(
    mut buf: *mut buffheader_T,
    mut advance: bool,
) -> ::core::ffi::c_int {
    if (*buf).bh_first.b_next.is_null() {
        return NUL;
    }
    let curr: *mut buffblock_T = (*buf).bh_first.b_next as *mut buffblock_T;
    let mut c: uint8_t = *(&raw mut (*curr).b_str as *mut ::core::ffi::c_char)
        .offset((*buf).bh_index as isize) as uint8_t;
    if advance {
        (*buf).bh_index = (*buf).bh_index.wrapping_add(1);
        if *(&raw mut (*curr).b_str as *mut ::core::ffi::c_char).offset((*buf).bh_index as isize)
            as ::core::ffi::c_int
            == NUL
        {
            (*buf).bh_first.b_next = (*curr).b_next;
            xfree(curr as *mut ::core::ffi::c_void);
            (*buf).bh_index = 0 as size_t;
        }
    }
    return c as ::core::ffi::c_int;
}
unsafe extern "C" fn start_stuff() {
    if !readbuf1.bh_first.b_next.is_null() {
        readbuf1.bh_curr = &raw mut readbuf1.bh_first;
        readbuf1.bh_create_newblock = true_0 != 0;
    }
    if !readbuf2.bh_first.b_next.is_null() {
        readbuf2.bh_curr = &raw mut readbuf2.bh_first;
        readbuf2.bh_create_newblock = true_0 != 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn stuff_empty() -> bool {
    return readbuf1.bh_first.b_next.is_null() && readbuf2.bh_first.b_next.is_null();
}
#[no_mangle]
pub unsafe extern "C" fn readbuf1_empty() -> bool {
    return readbuf1.bh_first.b_next.is_null();
}
#[no_mangle]
pub unsafe extern "C" fn typeahead_noflush(mut c: ::core::ffi::c_int) {
    typeahead_char = c;
}
#[no_mangle]
pub unsafe extern "C" fn flush_buffers(mut flush_typeahead: flush_buffers_T) {
    init_typebuf();
    start_stuff();
    while read_readbuffers(true_0 != 0) != NUL {}
    if flush_typeahead as ::core::ffi::c_uint
        == FLUSH_MINIMAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if typebuf.tb_off + typebuf.tb_maplen >= typebuf.tb_buflen {
            typebuf.tb_off = MAXMAPLEN as ::core::ffi::c_int;
            typebuf.tb_len = 0 as ::core::ffi::c_int;
        } else {
            typebuf.tb_off += typebuf.tb_maplen;
            typebuf.tb_len -= typebuf.tb_maplen;
        }
        if typebuf.tb_len == 0 as ::core::ffi::c_int {
            typebuf_was_filled = false_0 != 0;
        }
    } else {
        if flush_typeahead as ::core::ffi::c_uint
            == FLUSH_INPUT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            while inchar(
                typebuf.tb_buf,
                typebuf.tb_buflen - 1 as ::core::ffi::c_int,
                10 as ::core::ffi::c_long,
            ) != 0 as ::core::ffi::c_int
            {}
        }
        typebuf.tb_off = MAXMAPLEN as ::core::ffi::c_int;
        typebuf.tb_len = 0 as ::core::ffi::c_int;
        typebuf_was_filled = false_0 != 0;
    }
    typebuf.tb_maplen = 0 as ::core::ffi::c_int;
    typebuf.tb_silent = 0 as ::core::ffi::c_int;
    cmd_silent = false_0 != 0;
    typebuf.tb_no_abbr_cnt = 0 as ::core::ffi::c_int;
    typebuf.tb_change_cnt += 1;
    if typebuf.tb_change_cnt == 0 as ::core::ffi::c_int {
        typebuf.tb_change_cnt = 1 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn beep_flush() {
    if emsg_silent == 0 as ::core::ffi::c_int {
        flush_buffers(FLUSH_MINIMAL);
        vim_beep(kOptBoFlagError as ::core::ffi::c_int as ::core::ffi::c_uint);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ResetRedobuff() {
    if block_redo {
        return;
    }
    free_buff(&raw mut old_redobuff);
    old_redobuff = redobuff;
    redobuff.bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
}
#[no_mangle]
pub unsafe extern "C" fn CancelRedo() {
    if block_redo {
        return;
    }
    free_buff(&raw mut redobuff);
    redobuff = old_redobuff;
    old_redobuff.bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
    start_stuff();
    while read_readbuffers(true_0 != 0) != NUL {}
}
#[no_mangle]
pub unsafe extern "C" fn saveRedobuff(mut save_redo: *mut save_redo_T) {
    (*save_redo).sr_redobuff = redobuff;
    redobuff.bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
    (*save_redo).sr_old_redobuff = old_redobuff;
    old_redobuff.bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
    let mut slen: size_t = 0;
    let s: *mut ::core::ffi::c_char =
        get_buffcont(&raw mut (*save_redo).sr_redobuff, false_0, &raw mut slen);
    if s.is_null() {
        return;
    }
    add_buff(&raw mut redobuff, s, slen as ptrdiff_t);
    xfree(s as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn restoreRedobuff(mut save_redo: *mut save_redo_T) {
    free_buff(&raw mut redobuff);
    redobuff = (*save_redo).sr_redobuff;
    free_buff(&raw mut old_redobuff);
    old_redobuff = (*save_redo).sr_old_redobuff;
}
#[no_mangle]
pub unsafe extern "C" fn AppendToRedobuff(mut s: *const ::core::ffi::c_char) {
    if !block_redo {
        add_buff(&raw mut redobuff, s, -1 as ptrdiff_t);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AppendToRedobuffLit(
    mut str: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) {
    if block_redo {
        return;
    }
    let mut s: *const ::core::ffi::c_char = str;
    while if len < 0 as ::core::ffi::c_int {
        (*s as ::core::ffi::c_int != NUL) as ::core::ffi::c_int
    } else {
        (s.offset_from(str) < len as isize) as ::core::ffi::c_int
    } != 0
    {
        let mut start: *const ::core::ffi::c_char = s;
        while *s as ::core::ffi::c_int >= ' ' as ::core::ffi::c_int
            && (*s as ::core::ffi::c_int) < DEL
            && (len < 0 as ::core::ffi::c_int || s.offset_from(str) < len as isize)
        {
            s = s.offset(1);
        }
        if *s as ::core::ffi::c_int == NUL
            && (*s.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '0' as ::core::ffi::c_int
                || *s.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '^' as ::core::ffi::c_int)
        {
            s = s.offset(-1);
        }
        if s > start {
            add_buff(&raw mut redobuff, start, s.offset_from(start));
        }
        if *s as ::core::ffi::c_int == NUL
            || len >= 0 as ::core::ffi::c_int && s.offset_from(str) >= len as isize
        {
            break;
        }
        let c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut s);
        if c < ' ' as ::core::ffi::c_int
            || c == DEL
            || *s as ::core::ffi::c_int == NUL
                && (c == '0' as ::core::ffi::c_int || c == '^' as ::core::ffi::c_int)
        {
            add_char_buff(&raw mut redobuff, Ctrl_V);
        }
        if *s as ::core::ffi::c_int == NUL && c == '0' as ::core::ffi::c_int {
            add_buff(
                &raw mut redobuff,
                b"048\0".as_ptr() as *const ::core::ffi::c_char,
                3 as ptrdiff_t,
            );
        } else {
            add_char_buff(&raw mut redobuff, c);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn AppendToRedobuffSpec(mut s: *const ::core::ffi::c_char) {
    if block_redo {
        return;
    }
    while *s as ::core::ffi::c_int != NUL {
        if *s as uint8_t as ::core::ffi::c_int == K_SPECIAL
            && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            add_buff(&raw mut redobuff, s, 3 as ptrdiff_t);
            s = s.offset(3 as ::core::ffi::c_int as isize);
        } else {
            add_char_buff(&raw mut redobuff, mb_cptr2char_adv(&raw mut s));
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn AppendCharToRedobuff(mut c: ::core::ffi::c_int) {
    if !block_redo {
        add_char_buff(&raw mut redobuff, c);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AppendNumberToRedobuff(mut n: ::core::ffi::c_int) {
    if !block_redo {
        add_num_buff(&raw mut redobuff, n);
    }
}
#[no_mangle]
pub unsafe extern "C" fn stuffReadbuff(mut s: *const ::core::ffi::c_char) {
    add_buff(&raw mut readbuf1, s, -1 as ptrdiff_t);
}
#[no_mangle]
pub unsafe extern "C" fn stuffRedoReadbuff(mut s: *const ::core::ffi::c_char) {
    add_buff(&raw mut readbuf2, s, -1 as ptrdiff_t);
}
#[no_mangle]
pub unsafe extern "C" fn stuffReadbuffLen(mut s: *const ::core::ffi::c_char, mut len: ptrdiff_t) {
    add_buff(&raw mut readbuf1, s, len);
}
#[no_mangle]
pub unsafe extern "C" fn stuffReadbuffSpec(mut s: *const ::core::ffi::c_char) {
    while *s as ::core::ffi::c_int != NUL {
        if *s as uint8_t as ::core::ffi::c_int == K_SPECIAL
            && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            stuffReadbuffLen(s, 3 as ptrdiff_t);
            s = s.offset(3 as ::core::ffi::c_int as isize);
        } else {
            let mut c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut s);
            if c == CAR || c == NL || c == ESC {
                c = ' ' as ::core::ffi::c_int;
            }
            stuffcharReadbuff(c);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn stuffcharReadbuff(mut c: ::core::ffi::c_int) {
    add_char_buff(&raw mut readbuf1, c);
}
#[no_mangle]
pub unsafe extern "C" fn stuffnumReadbuff(mut n: ::core::ffi::c_int) {
    add_num_buff(&raw mut readbuf1, n);
}
#[no_mangle]
pub unsafe extern "C" fn stuffescaped(mut arg: *const ::core::ffi::c_char, mut literally: bool) {
    while *arg as ::core::ffi::c_int != NUL {
        let start: *const ::core::ffi::c_char = arg;
        while *arg as ::core::ffi::c_int >= ' ' as ::core::ffi::c_int
            && (*arg as ::core::ffi::c_int) < DEL
            || *arg as uint8_t as ::core::ffi::c_int == K_SPECIAL && !literally
        {
            arg = arg.offset(1);
        }
        if arg > start {
            stuffReadbuffLen(start, arg.offset_from(start));
        }
        if *arg as ::core::ffi::c_int != NUL {
            let c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut arg);
            if literally as ::core::ffi::c_int != 0
                && (c < ' ' as ::core::ffi::c_int && c != TAB || c == DEL)
            {
                stuffcharReadbuff(Ctrl_V);
            }
            stuffcharReadbuff(c);
        }
    }
}
unsafe extern "C" fn read_redo(mut init: bool, mut old_redo: bool) -> ::core::ffi::c_int {
    static mut bp: *mut buffblock_T = ::core::ptr::null_mut::<buffblock_T>();
    static mut p: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut c: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut buf: [uint8_t; 22] = [0; 22];
    if init {
        bp = (if old_redo as ::core::ffi::c_int != 0 {
            old_redobuff.bh_first.b_next
        } else {
            redobuff.bh_first.b_next
        }) as *mut buffblock_T;
        if bp.is_null() {
            return FAIL;
        }
        p = &raw mut (*bp).b_str as *mut ::core::ffi::c_char as *mut uint8_t;
        return OK;
    }
    c = *p as ::core::ffi::c_int;
    if c == NUL {
        return c;
    }
    if c != K_SPECIAL
        || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_SPECIAL
    {
        n = if c < 0 as ::core::ffi::c_int || c > 255 as ::core::ffi::c_int {
            1 as ::core::ffi::c_int
        } else {
            utf8len_tab[c as usize] as ::core::ffi::c_int
        };
    } else {
        n = 1 as ::core::ffi::c_int;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        if c == K_SPECIAL {
            c = if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_SPECIAL {
                K_SPECIAL
            } else if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_ZERO {
                K_ZERO
            } else {
                -(*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ((*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                        << 8 as ::core::ffi::c_int))
            };
            p = p.offset(2 as ::core::ffi::c_int as isize);
        }
        p = p.offset(1);
        if *p as ::core::ffi::c_int == NUL && !(*bp).b_next.is_null() {
            bp = (*bp).b_next as *mut buffblock_T;
            p = &raw mut (*bp).b_str as *mut ::core::ffi::c_char as *mut uint8_t;
        }
        buf[i as usize] = c as uint8_t;
        if i == n - 1 as ::core::ffi::c_int {
            if n != 1 as ::core::ffi::c_int {
                c = utf_ptr2char(&raw mut buf as *mut uint8_t as *mut ::core::ffi::c_char);
            }
            break;
        } else {
            c = *p as ::core::ffi::c_int;
            if c == NUL {
                break;
            }
            i += 1;
        }
    }
    return c;
}
unsafe extern "C" fn copy_redo(mut old_redo: bool) {
    let mut c: ::core::ffi::c_int = 0;
    loop {
        c = read_redo(false_0 != 0, old_redo);
        if c == NUL {
            break;
        }
        add_char_buff(&raw mut readbuf2, c);
    }
}
#[no_mangle]
pub unsafe extern "C" fn start_redo(
    mut count: ::core::ffi::c_int,
    mut old_redo: bool,
) -> ::core::ffi::c_int {
    if read_redo(true_0 != 0, old_redo) == FAIL {
        return FAIL;
    }
    let mut c: ::core::ffi::c_int = read_redo(false_0 != 0, old_redo);
    if c == '"' as ::core::ffi::c_int {
        add_buff(
            &raw mut readbuf2,
            b"\"\0".as_ptr() as *const ::core::ffi::c_char,
            1 as ptrdiff_t,
        );
        c = read_redo(false_0 != 0, old_redo);
        if c >= '1' as ::core::ffi::c_int && c < '9' as ::core::ffi::c_int {
            c += 1;
        }
        add_char_buff(&raw mut readbuf2, c);
        if c == '=' as ::core::ffi::c_int {
            add_char_buff(&raw mut readbuf2, CAR);
            cmd_silent = true_0 != 0;
        }
        c = read_redo(false_0 != 0, old_redo);
    }
    if c == 'v' as ::core::ffi::c_int {
        VIsual = (*curwin).w_cursor;
        VIsual_active = true_0 != 0;
        VIsual_select = false_0 != 0;
        VIsual_reselect = true_0;
        redo_VIsual_busy = true_0 != 0;
        c = read_redo(false_0 != 0, old_redo);
    }
    if count != 0 {
        while ascii_isdigit(c) {
            c = read_redo(false_0 != 0, old_redo);
        }
        add_num_buff(&raw mut readbuf2, count);
    }
    add_char_buff(&raw mut readbuf2, c);
    copy_redo(old_redo);
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn start_redo_ins() -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    if read_redo(true_0 != 0, false_0 != 0) == FAIL {
        return FAIL;
    }
    start_stuff();
    loop {
        c = read_redo(false_0 != 0, false_0 != 0);
        if c == NUL {
            break;
        }
        if vim_strchr(b"AaIiRrOo\0".as_ptr() as *const ::core::ffi::c_char, c).is_null() {
            continue;
        }
        if c == 'O' as ::core::ffi::c_int || c == 'o' as ::core::ffi::c_int {
            add_buff(&raw mut readbuf2, NL_STR.as_ptr(), -1 as ptrdiff_t);
        }
        break;
    }
    copy_redo(false_0 != 0);
    block_redo = true_0 != 0;
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn stop_redo_ins() {
    block_redo = false_0 != 0;
}
unsafe extern "C" fn init_typebuf() {
    if !typebuf.tb_buf.is_null() {
        return;
    }
    typebuf.tb_buf = &raw mut typebuf_init as *mut uint8_t;
    typebuf.tb_noremap = &raw mut noremapbuf_init as *mut uint8_t;
    typebuf.tb_buflen =
        5 as ::core::ffi::c_int * (MAXMAPLEN as ::core::ffi::c_int + 3 as ::core::ffi::c_int);
    typebuf.tb_len = 0 as ::core::ffi::c_int;
    typebuf.tb_off = MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int;
    typebuf.tb_change_cnt = 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn noremap_keys() -> bool {
    return KeyNoremap & (RM_NONE as ::core::ffi::c_int | RM_SCRIPT as ::core::ffi::c_int) != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ins_typebuf(
    mut str: *mut ::core::ffi::c_char,
    mut noremap: ::core::ffi::c_int,
    mut offset: ::core::ffi::c_int,
    mut nottyped: bool,
    mut silent: bool,
) -> ::core::ffi::c_int {
    let mut val: ::core::ffi::c_int = 0;
    let mut nrm: ::core::ffi::c_int = 0;
    init_typebuf();
    typebuf.tb_change_cnt += 1;
    if typebuf.tb_change_cnt == 0 as ::core::ffi::c_int {
        typebuf.tb_change_cnt = 1 as ::core::ffi::c_int;
    }
    state_no_longer_safe(b"ins_typebuf()\0".as_ptr() as *const ::core::ffi::c_char);
    let mut addlen: ::core::ffi::c_int = strlen(str) as ::core::ffi::c_int;
    if offset == 0 as ::core::ffi::c_int && addlen <= typebuf.tb_off {
        typebuf.tb_off -= addlen;
        memmove(
            typebuf.tb_buf.offset(typebuf.tb_off as isize) as *mut ::core::ffi::c_void,
            str as *const ::core::ffi::c_void,
            addlen as size_t,
        );
    } else if typebuf.tb_len == 0 as ::core::ffi::c_int
        && typebuf.tb_buflen
            >= addlen
                + 3 as ::core::ffi::c_int
                    * (MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int)
    {
        typebuf.tb_off = (typebuf.tb_buflen
            - addlen
            - 3 as ::core::ffi::c_int
                * (MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int))
            / 2 as ::core::ffi::c_int;
        memmove(
            typebuf.tb_buf.offset(typebuf.tb_off as isize) as *mut ::core::ffi::c_void,
            str as *const ::core::ffi::c_void,
            addlen as size_t,
        );
    } else {
        let mut newoff: ::core::ffi::c_int =
            MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int;
        let mut extra: ::core::ffi::c_int = addlen
            + newoff
            + 4 as ::core::ffi::c_int * (MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int);
        if typebuf.tb_len > INT_MAX - extra {
            emsg(gettext(&raw const e_toocompl as *const ::core::ffi::c_char));
            setcursor();
            return FAIL;
        }
        let mut newlen: ::core::ffi::c_int = typebuf.tb_len + extra;
        let mut s1: *mut uint8_t = xmalloc(newlen as size_t) as *mut uint8_t;
        let mut s2: *mut uint8_t = xmalloc(newlen as size_t) as *mut uint8_t;
        typebuf.tb_buflen = newlen;
        memmove(
            s1.offset(newoff as isize) as *mut ::core::ffi::c_void,
            typebuf.tb_buf.offset(typebuf.tb_off as isize) as *const ::core::ffi::c_void,
            offset as size_t,
        );
        memmove(
            s1.offset(newoff as isize).offset(offset as isize) as *mut ::core::ffi::c_void,
            str as *const ::core::ffi::c_void,
            addlen as size_t,
        );
        let mut bytes: ::core::ffi::c_int = typebuf.tb_len - offset + 1 as ::core::ffi::c_int;
        '_c2rust_label: {
            if bytes > 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"bytes > 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/getchar.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    978 as ::core::ffi::c_uint,
                    b"int ins_typebuf(char *, int, int, _Bool, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        memmove(
            s1.offset(newoff as isize)
                .offset(offset as isize)
                .offset(addlen as isize) as *mut ::core::ffi::c_void,
            typebuf
                .tb_buf
                .offset(typebuf.tb_off as isize)
                .offset(offset as isize) as *const ::core::ffi::c_void,
            bytes as size_t,
        );
        if typebuf.tb_buf != &raw mut typebuf_init as *mut uint8_t {
            xfree(typebuf.tb_buf as *mut ::core::ffi::c_void);
        }
        typebuf.tb_buf = s1;
        memmove(
            s2.offset(newoff as isize) as *mut ::core::ffi::c_void,
            typebuf.tb_noremap.offset(typebuf.tb_off as isize) as *const ::core::ffi::c_void,
            offset as size_t,
        );
        memmove(
            s2.offset(newoff as isize)
                .offset(offset as isize)
                .offset(addlen as isize) as *mut ::core::ffi::c_void,
            typebuf
                .tb_noremap
                .offset(typebuf.tb_off as isize)
                .offset(offset as isize) as *const ::core::ffi::c_void,
            (typebuf.tb_len - offset) as size_t,
        );
        if typebuf.tb_noremap != &raw mut noremapbuf_init as *mut uint8_t {
            xfree(typebuf.tb_noremap as *mut ::core::ffi::c_void);
        }
        typebuf.tb_noremap = s2;
        typebuf.tb_off = newoff;
    }
    typebuf.tb_len += addlen;
    if noremap == REMAP_SCRIPT as ::core::ffi::c_int {
        val = RM_SCRIPT as ::core::ffi::c_int;
    } else if noremap == REMAP_SKIP as ::core::ffi::c_int {
        val = RM_ABBR as ::core::ffi::c_int;
    } else {
        val = RM_NONE as ::core::ffi::c_int;
    }
    if noremap == REMAP_SKIP as ::core::ffi::c_int {
        nrm = 1 as ::core::ffi::c_int;
    } else if noremap < 0 as ::core::ffi::c_int {
        nrm = addlen;
    } else {
        nrm = noremap;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < addlen {
        nrm -= 1;
        *typebuf
            .tb_noremap
            .offset((typebuf.tb_off + i + offset) as isize) = (if nrm >= 0 as ::core::ffi::c_int {
            val
        } else {
            RM_YES as ::core::ffi::c_int
        }) as uint8_t;
        i += 1;
    }
    if nottyped as ::core::ffi::c_int != 0 || typebuf.tb_maplen > offset {
        typebuf.tb_maplen += addlen;
    }
    if silent as ::core::ffi::c_int != 0 || typebuf.tb_silent > offset {
        typebuf.tb_silent += addlen;
        cmd_silent = true_0 != 0;
    }
    if typebuf.tb_no_abbr_cnt != 0 && offset == 0 as ::core::ffi::c_int {
        typebuf.tb_no_abbr_cnt += addlen;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn ins_char_typebuf(
    mut c: ::core::ffi::c_int,
    mut modifiers: ::core::ffi::c_int,
    mut on_key_ignore: bool,
) -> ::core::ffi::c_int {
    let mut buf: [::core::ffi::c_char; 67] = [0; 67];
    let mut len: ::core::ffi::c_uint = special_to_buf(
        c,
        modifiers,
        true_0 != 0,
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    '_c2rust_label: {
        if (len as usize) < ::core::mem::size_of::<[::core::ffi::c_char; 67]>() {
        } else {
            __assert_fail(
                b"len < sizeof(buf)\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/getchar.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                1056 as ::core::ffi::c_uint,
                b"int ins_char_typebuf(int, int, _Bool)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    buf[len as usize] = NUL as ::core::ffi::c_char;
    ins_typebuf(
        &raw mut buf as *mut ::core::ffi::c_char,
        KeyNoremap,
        0 as ::core::ffi::c_int,
        !KeyTyped,
        cmd_silent,
    );
    if KeyTyped as ::core::ffi::c_int != 0 && on_key_ignore as ::core::ffi::c_int != 0 {
        on_key_ignore_len = on_key_ignore_len.wrapping_add(len as size_t);
    }
    return len as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn typebuf_changed(mut tb_change_cnt: ::core::ffi::c_int) -> bool {
    return tb_change_cnt != 0 as ::core::ffi::c_int
        && (typebuf.tb_change_cnt != tb_change_cnt
            || typebuf_was_filled as ::core::ffi::c_int != 0);
}
#[no_mangle]
pub unsafe extern "C" fn typebuf_typed() -> ::core::ffi::c_int {
    return (typebuf.tb_maplen == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn typebuf_maplen() -> ::core::ffi::c_int {
    return typebuf.tb_maplen;
}
#[no_mangle]
pub unsafe extern "C" fn del_typebuf(mut len: ::core::ffi::c_int, mut offset: ::core::ffi::c_int) {
    if len == 0 as ::core::ffi::c_int {
        return;
    }
    typebuf.tb_len -= len;
    if offset == 0 as ::core::ffi::c_int
        && typebuf.tb_buflen - (typebuf.tb_off + len)
            >= 3 as ::core::ffi::c_int * MAXMAPLEN as ::core::ffi::c_int + 3 as ::core::ffi::c_int
    {
        typebuf.tb_off += len;
    } else {
        let mut i: ::core::ffi::c_int = typebuf.tb_off + offset;
        if typebuf.tb_off > MAXMAPLEN as ::core::ffi::c_int {
            memmove(
                typebuf
                    .tb_buf
                    .offset(MAXMAPLEN as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                typebuf.tb_buf.offset(typebuf.tb_off as isize) as *const ::core::ffi::c_void,
                offset as size_t,
            );
            memmove(
                typebuf
                    .tb_noremap
                    .offset(MAXMAPLEN as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                typebuf.tb_noremap.offset(typebuf.tb_off as isize) as *const ::core::ffi::c_void,
                offset as size_t,
            );
            typebuf.tb_off = MAXMAPLEN as ::core::ffi::c_int;
        }
        let mut bytes: ::core::ffi::c_int = typebuf.tb_len - offset + 1 as ::core::ffi::c_int;
        '_c2rust_label: {
            if bytes > 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"bytes > 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/getchar.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    1122 as ::core::ffi::c_uint,
                    b"void del_typebuf(int, int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        memmove(
            typebuf
                .tb_buf
                .offset(typebuf.tb_off as isize)
                .offset(offset as isize) as *mut ::core::ffi::c_void,
            typebuf.tb_buf.offset(i as isize).offset(len as isize) as *const ::core::ffi::c_void,
            bytes as size_t,
        );
        memmove(
            typebuf
                .tb_noremap
                .offset(typebuf.tb_off as isize)
                .offset(offset as isize) as *mut ::core::ffi::c_void,
            typebuf.tb_noremap.offset(i as isize).offset(len as isize)
                as *const ::core::ffi::c_void,
            (typebuf.tb_len - offset) as size_t,
        );
    }
    if typebuf.tb_maplen > offset {
        if typebuf.tb_maplen < offset + len {
            typebuf.tb_maplen = offset;
        } else {
            typebuf.tb_maplen -= len;
        }
    }
    if typebuf.tb_silent > offset {
        if typebuf.tb_silent < offset + len {
            typebuf.tb_silent = offset;
        } else {
            typebuf.tb_silent -= len;
        }
    }
    if typebuf.tb_no_abbr_cnt > offset {
        if typebuf.tb_no_abbr_cnt < offset + len {
            typebuf.tb_no_abbr_cnt = offset;
        } else {
            typebuf.tb_no_abbr_cnt -= len;
        }
    }
    typebuf_was_filled = false_0 != 0;
    typebuf.tb_change_cnt += 1;
    if typebuf.tb_change_cnt == 0 as ::core::ffi::c_int {
        typebuf.tb_change_cnt = 1 as ::core::ffi::c_int;
    }
}
unsafe extern "C" fn gotchars_add_byte(
    mut state: *mut gotchars_state_T,
    mut byte: uint8_t,
) -> bool {
    let c2rust_fresh4 = (*state).buflen;
    (*state).buflen = (*state).buflen.wrapping_add(1);
    let c2rust_lvalue_ptr = &raw mut (*state).buf[c2rust_fresh4 as usize];
    *c2rust_lvalue_ptr = byte;
    let mut c: ::core::ffi::c_int = *c2rust_lvalue_ptr as ::core::ffi::c_int;
    let mut retval: bool = false_0 != 0;
    let in_special: bool = (*state).pending_special > 0 as ::core::ffi::c_uint;
    let in_mbyte: bool = (*state).pending_mbyte > 0 as ::core::ffi::c_uint;
    if in_special {
        (*state).pending_special = (*state).pending_special.wrapping_sub(1);
    } else if c == K_SPECIAL {
        (*state).pending_special = 2 as ::core::ffi::c_uint;
    }
    '_ret_false: {
        if (*state).pending_special <= 0 as ::core::ffi::c_uint {
            if in_mbyte {
                (*state).pending_mbyte = (*state).pending_mbyte.wrapping_sub(1);
            } else {
                if in_special {
                    if (*state).prev_c == KS_MODIFIER {
                        break '_ret_false;
                    } else {
                        c = if (*state).prev_c == KS_SPECIAL {
                            K_SPECIAL
                        } else if (*state).prev_c == KS_ZERO {
                            K_ZERO
                        } else {
                            -((*state).prev_c + (c << 8 as ::core::ffi::c_int))
                        };
                    }
                }
                (*state).pending_mbyte =
                    ((if c < 0 as ::core::ffi::c_int || c > 255 as ::core::ffi::c_int {
                        1 as ::core::ffi::c_int
                    } else {
                        utf8len_tab[c as usize] as ::core::ffi::c_int
                    }) - 1 as ::core::ffi::c_int) as ::core::ffi::c_uint;
            }
            if (*state).pending_mbyte <= 0 as ::core::ffi::c_uint {
                retval = true_0 != 0;
            }
        }
    }
    (*state).prev_c = c;
    return retval;
}
unsafe extern "C" fn gotchars(mut chars: *const uint8_t, mut len: size_t) {
    let mut s: *const uint8_t = chars;
    let mut todo: size_t = len;
    static mut state: gotchars_state_T = gotchars_state_T {
        buf: [0; 67],
        prev_c: 0,
        buflen: 0,
        pending_special: 0,
        pending_mbyte: 0,
    };
    loop {
        let c2rust_fresh2 = todo;
        todo = todo.wrapping_sub(1);
        if c2rust_fresh2 <= 0 as size_t {
            break;
        }
        let c2rust_fresh3 = s;
        s = s.offset(1);
        if !gotchars_add_byte(&raw mut state, *c2rust_fresh3) {
            continue;
        }
        let mut i: size_t = 0 as size_t;
        while i < state.buflen {
            updatescript(state.buf[i as usize] as ::core::ffi::c_int);
            i = i.wrapping_add(1);
        }
        if state.buflen > on_key_ignore_len {
            if state.buflen.wrapping_sub(on_key_ignore_len) > 0 as size_t {
                if on_key_buf.capacity
                    < on_key_buf
                        .size
                        .wrapping_add(state.buflen)
                        .wrapping_sub(on_key_ignore_len)
                {
                    on_key_buf.capacity = on_key_buf
                        .size
                        .wrapping_add(state.buflen)
                        .wrapping_sub(on_key_ignore_len);
                    on_key_buf.capacity = on_key_buf.capacity.wrapping_sub(1);
                    on_key_buf.capacity |= on_key_buf.capacity >> 1 as ::core::ffi::c_int;
                    on_key_buf.capacity |= on_key_buf.capacity >> 2 as ::core::ffi::c_int;
                    on_key_buf.capacity |= on_key_buf.capacity >> 4 as ::core::ffi::c_int;
                    on_key_buf.capacity |= on_key_buf.capacity >> 8 as ::core::ffi::c_int;
                    on_key_buf.capacity |= on_key_buf.capacity >> 16 as ::core::ffi::c_int;
                    on_key_buf.capacity = on_key_buf.capacity.wrapping_add(1);
                    on_key_buf.capacity = if on_key_buf.capacity
                        > ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                            .wrapping_div(
                                (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        on_key_buf.capacity
                    } else {
                        ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                            .wrapping_div(
                                (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    on_key_buf.items = (if on_key_buf.capacity
                        == ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                            .wrapping_div(
                                (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if on_key_buf.items
                            == &raw mut on_key_buf.init_array as *mut ::core::ffi::c_char
                        {
                            on_key_buf.items as *mut ::core::ffi::c_void
                        } else {
                            _memcpy_free(
                                &raw mut on_key_buf.init_array as *mut ::core::ffi::c_char
                                    as *mut ::core::ffi::c_void,
                                on_key_buf.items as *mut ::core::ffi::c_void,
                                on_key_buf
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                            )
                        }
                    } else {
                        if on_key_buf.items
                            == &raw mut on_key_buf.init_array as *mut ::core::ffi::c_char
                        {
                            memcpy(
                                xmalloc(
                                    on_key_buf
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                                ),
                                on_key_buf.items as *const ::core::ffi::c_void,
                                on_key_buf
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                            )
                        } else {
                            xrealloc(
                                on_key_buf.items as *mut ::core::ffi::c_void,
                                on_key_buf
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                            )
                        }
                    }) as *mut ::core::ffi::c_char;
                }
                '_c2rust_label: {
                    if !on_key_buf.items.is_null() {
                    } else {
                        __assert_fail(
                            b"(on_key_buf).items\0".as_ptr() as *const ::core::ffi::c_char,
                            b"/home/overlord/projects/neovim/neovim/src/nvim/getchar.c\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            1230 as ::core::ffi::c_uint,
                            b"void gotchars(const uint8_t *, size_t)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                memcpy(
                    on_key_buf.items.offset(on_key_buf.size as isize) as *mut ::core::ffi::c_void,
                    (&raw mut state.buf as *mut uint8_t as *mut ::core::ffi::c_char)
                        .offset(on_key_ignore_len as isize)
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(state.buflen)
                        .wrapping_sub(on_key_ignore_len),
                );
                on_key_buf.size = on_key_buf
                    .size
                    .wrapping_add(state.buflen)
                    .wrapping_sub(on_key_ignore_len);
            }
            on_key_ignore_len = 0 as size_t;
        } else {
            on_key_ignore_len = on_key_ignore_len.wrapping_sub(state.buflen);
        }
        if reg_recording != 0 as ::core::ffi::c_int {
            state.buf[state.buflen as usize] = NUL as uint8_t;
            add_buff(
                &raw mut recordbuff,
                &raw mut state.buf as *mut uint8_t as *mut ::core::ffi::c_char,
                state.buflen as ptrdiff_t,
            );
            last_recorded_len = last_recorded_len.wrapping_add(state.buflen);
        }
        state.buflen = 0 as size_t;
    }
    may_sync_undo();
    debug_did_msg = false_0 != 0;
    maptick += 1;
}
#[no_mangle]
pub unsafe extern "C" fn gotchars_ignore() {
    let mut nop_buf: [uint8_t; 3] = [
        K_SPECIAL as uint8_t,
        KS_EXTRA as uint8_t,
        KE_IGNORE as ::core::ffi::c_int as uint8_t,
    ];
    on_key_ignore_len = on_key_ignore_len.wrapping_add(3 as size_t);
    gotchars(&raw mut nop_buf as *mut uint8_t, 3 as size_t);
}
#[no_mangle]
pub unsafe extern "C" fn ungetchars(mut len: ::core::ffi::c_int) {
    if reg_recording == 0 as ::core::ffi::c_int {
        return;
    }
    delete_buff_tail(&raw mut recordbuff, len);
    last_recorded_len = last_recorded_len.wrapping_sub(len as size_t);
}
#[no_mangle]
pub unsafe extern "C" fn may_sync_undo() {
    if (State & (MODE_INSERT as ::core::ffi::c_int | MODE_CMDLINE as ::core::ffi::c_int) == 0
        || arrow_used as ::core::ffi::c_int != 0)
        && curscript < 0 as ::core::ffi::c_int
    {
        u_sync(false_0 != 0);
    }
}
unsafe extern "C" fn alloc_typebuf() {
    typebuf.tb_buf = xmalloc(
        (5 as ::core::ffi::c_int * (MAXMAPLEN as ::core::ffi::c_int + 3 as ::core::ffi::c_int))
            as size_t,
    ) as *mut uint8_t;
    typebuf.tb_noremap = xmalloc(
        (5 as ::core::ffi::c_int * (MAXMAPLEN as ::core::ffi::c_int + 3 as ::core::ffi::c_int))
            as size_t,
    ) as *mut uint8_t;
    typebuf.tb_buflen =
        5 as ::core::ffi::c_int * (MAXMAPLEN as ::core::ffi::c_int + 3 as ::core::ffi::c_int);
    typebuf.tb_off = MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int;
    typebuf.tb_len = 0 as ::core::ffi::c_int;
    typebuf.tb_maplen = 0 as ::core::ffi::c_int;
    typebuf.tb_silent = 0 as ::core::ffi::c_int;
    typebuf.tb_no_abbr_cnt = 0 as ::core::ffi::c_int;
    typebuf.tb_change_cnt += 1;
    if typebuf.tb_change_cnt == 0 as ::core::ffi::c_int {
        typebuf.tb_change_cnt = 1 as ::core::ffi::c_int;
    }
    typebuf_was_filled = false_0 != 0;
}
unsafe extern "C" fn free_typebuf() {
    if typebuf.tb_buf == &raw mut typebuf_init as *mut uint8_t {
        internal_error(b"Free typebuf 1\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut typebuf.tb_buf as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        *ptr_;
    }
    if typebuf.tb_noremap == &raw mut noremapbuf_init as *mut uint8_t {
        internal_error(b"Free typebuf 2\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut typebuf.tb_noremap as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        *ptr__0;
    };
}
static mut saved_typebuf: [typebuf_T; 15] = [typebuf_T {
    tb_buf: ::core::ptr::null_mut::<uint8_t>(),
    tb_noremap: ::core::ptr::null_mut::<uint8_t>(),
    tb_buflen: 0,
    tb_off: 0,
    tb_len: 0,
    tb_maplen: 0,
    tb_silent: 0,
    tb_no_abbr_cnt: 0,
    tb_change_cnt: 0,
}; 15];
unsafe extern "C" fn save_typebuf() {
    '_c2rust_label: {
        if curscript >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"curscript >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/getchar.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                1330 as ::core::ffi::c_uint,
                b"void save_typebuf(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    init_typebuf();
    saved_typebuf[curscript as usize] = typebuf;
    alloc_typebuf();
}
static mut old_char: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
static mut old_mod_mask: ::core::ffi::c_int = 0;
static mut old_mouse_grid: ::core::ffi::c_int = 0;
static mut old_mouse_row: ::core::ffi::c_int = 0;
static mut old_mouse_col: ::core::ffi::c_int = 0;
static mut old_KeyStuffed: ::core::ffi::c_int = 0;
unsafe extern "C" fn can_get_old_char() -> bool {
    return old_char != -1 as ::core::ffi::c_int
        && (old_KeyStuffed != 0 || stuff_empty() as ::core::ffi::c_int != 0);
}
#[no_mangle]
pub unsafe extern "C" fn save_typeahead(mut tp: *mut tasave_T) {
    (*tp).save_typebuf = typebuf;
    alloc_typebuf();
    (*tp).typebuf_valid = true_0 != 0;
    (*tp).old_char = old_char;
    (*tp).old_mod_mask = old_mod_mask;
    old_char = -1 as ::core::ffi::c_int;
    (*tp).save_readbuf1 = readbuf1;
    readbuf1.bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
    (*tp).save_readbuf2 = readbuf2;
    readbuf2.bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
}
#[no_mangle]
pub unsafe extern "C" fn restore_typeahead(mut tp: *mut tasave_T) {
    if (*tp).typebuf_valid {
        free_typebuf();
        typebuf = (*tp).save_typebuf;
    }
    old_char = (*tp).old_char;
    old_mod_mask = (*tp).old_mod_mask;
    free_buff(&raw mut readbuf1);
    readbuf1 = (*tp).save_readbuf1;
    free_buff(&raw mut readbuf2);
    readbuf2 = (*tp).save_readbuf2;
}
#[no_mangle]
pub unsafe extern "C" fn openscript(mut name: *mut ::core::ffi::c_char, mut directly: bool) {
    if curscript + 1 as ::core::ffi::c_int == NSCRIPT as ::core::ffi::c_int {
        emsg(gettext(&raw const e_nesting as *const ::core::ffi::c_char));
        return;
    }
    if check_secure() {
        return;
    }
    if ignore_script {
        return;
    }
    curscript += 1;
    expand_env(
        name,
        &raw mut NameBuff as *mut ::core::ffi::c_char,
        MAXPATHL,
    );
    let mut error: ::core::ffi::c_int = file_open(
        (&raw mut scriptin as *mut FileDescriptor).offset(curscript as isize),
        &raw mut NameBuff as *mut ::core::ffi::c_char,
        kFileReadOnly as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if error != 0 {
        semsg(
            gettext(&raw const e_notopen_2 as *const ::core::ffi::c_char),
            name,
            uv_strerror(error),
        );
        curscript -= 1;
        return;
    }
    save_typebuf();
    if directly {
        let mut oa: oparg_T = oparg_T {
            op_type: 0,
            regname: 0,
            motion_type: kMTCharWise,
            motion_force: 0,
            use_reg_one: false,
            inclusive: false,
            end_adjusted: false,
            start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            end: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            cursor_start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            line_count: 0,
            empty: false,
            is_VIsual: false,
            start_vcol: 0,
            end_vcol: 0,
            prev_opcount: 0,
            prev_count0: 0,
            excl_tr_ws: false,
        };
        let mut save_State: ::core::ffi::c_int = State;
        let mut save_restart_edit: ::core::ffi::c_int = restart_edit;
        let mut save_finish_op: ::core::ffi::c_int = finish_op as ::core::ffi::c_int;
        let mut save_msg_scroll: ::core::ffi::c_int = msg_scroll;
        State = MODE_NORMAL as ::core::ffi::c_int;
        msg_scroll = false_0;
        restart_edit = 0 as ::core::ffi::c_int;
        clear_oparg(&raw mut oa);
        finish_op = false_0 != 0;
        let mut oldcurscript: ::core::ffi::c_int = curscript;
        loop {
            update_topline_cursor();
            normal_cmd(&raw mut oa, false_0 != 0);
            vpeekc();
            if curscript < oldcurscript {
                break;
            }
        }
        State = save_State;
        msg_scroll = save_msg_scroll;
        restart_edit = save_restart_edit;
        finish_op = save_finish_op != 0;
    }
}
unsafe extern "C" fn closescript() {
    '_c2rust_label: {
        if curscript >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"curscript >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/getchar.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                1450 as ::core::ffi::c_uint,
                b"void closescript(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    free_typebuf();
    typebuf = saved_typebuf[curscript as usize];
    file_close(
        (&raw mut scriptin as *mut FileDescriptor).offset(curscript as isize),
        false_0 != 0,
    );
    curscript -= 1;
}
#[no_mangle]
pub unsafe extern "C" fn open_scriptin(mut scriptin_name: *mut ::core::ffi::c_char) -> bool {
    '_c2rust_label: {
        if curscript == -1 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"curscript == -1\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/getchar.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                1471 as ::core::ffi::c_uint,
                b"_Bool open_scriptin(char *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    curscript += 1;
    let mut error: ::core::ffi::c_int = 0;
    if strequal(scriptin_name, b"-\0".as_ptr() as *const ::core::ffi::c_char) {
        error = file_open_stdin(
            (&raw mut scriptin as *mut FileDescriptor).offset(0 as ::core::ffi::c_int as isize),
        );
    } else {
        error = file_open(
            (&raw mut scriptin as *mut FileDescriptor).offset(0 as ::core::ffi::c_int as isize),
            scriptin_name,
            kFileReadOnly as ::core::ffi::c_int | kFileNonBlocking as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
    if error != 0 {
        fprintf(
            stderr,
            gettext(
                b"Cannot open for reading: \"%s\": %s\n\0".as_ptr() as *const ::core::ffi::c_char
            ),
            scriptin_name,
            uv_strerror(error),
        );
        curscript -= 1;
        return false_0 != 0;
    }
    save_typebuf();
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn using_script() -> ::core::ffi::c_int {
    return (curscript >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn before_blocking() {
    updatescript(0 as ::core::ffi::c_int);
    if may_garbage_collect {
        garbage_collect(false_0 != 0);
    }
}
unsafe extern "C" fn updatescript(mut c: ::core::ffi::c_int) {
    static mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if c != 0 && !scriptout.is_null() {
        putc(c, scriptout);
    }
    let mut idle: bool = c == 0 as ::core::ffi::c_int;
    if idle as ::core::ffi::c_int != 0
        || p_uc > 0 as OptInt && {
            count += 1;
            count as OptInt >= p_uc
        }
    {
        ml_sync_all(
            idle as ::core::ffi::c_int,
            true_0,
            p_fs != 0 || idle as ::core::ffi::c_int != 0,
        );
        count = 0 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn merge_modifiers(
    mut c_arg: ::core::ffi::c_int,
    mut modifiers: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = c_arg;
    if *modifiers & MOD_MASK_CTRL != 0 {
        if c >= '@' as ::core::ffi::c_int && c <= 0x7f as ::core::ffi::c_int {
            c &= 0x1f as ::core::ffi::c_int;
            if c == NUL {
                c = K_ZERO;
            }
        } else if c == '6' as ::core::ffi::c_int {
            c = 0x1e as ::core::ffi::c_int;
        }
        if c != c_arg {
            *modifiers &= !MOD_MASK_CTRL;
        }
    }
    return c;
}
unsafe extern "C" fn add_byte_to_showcmd(mut byte: uint8_t) {
    static mut state: gotchars_state_T = gotchars_state_T {
        buf: [0; 67],
        prev_c: 0,
        buflen: 0,
        pending_special: 0,
        pending_mbyte: 0,
    };
    if p_sc == 0 || msg_silent != 0 as ::core::ffi::c_int {
        return;
    }
    if !gotchars_add_byte(&raw mut state, byte) {
        return;
    }
    state.buf[state.buflen as usize] = NUL as uint8_t;
    state.buflen = 0 as size_t;
    let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut c: ::core::ffi::c_int = NUL;
    let mut ptr: *const uint8_t = &raw mut state.buf as *mut uint8_t;
    if *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == K_SPECIAL
        && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_MODIFIER
        && *ptr.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
    {
        modifiers = *ptr.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
        ptr = ptr.offset(3 as ::core::ffi::c_int as isize);
    }
    if *ptr as ::core::ffi::c_int != NUL {
        let mut mb_ptr: *const ::core::ffi::c_char =
            mb_unescape(&raw mut ptr as *mut *const ::core::ffi::c_char);
        c = if !mb_ptr.is_null() {
            utf_ptr2char(mb_ptr)
        } else {
            let c2rust_fresh7 = ptr;
            ptr = ptr.offset(1);
            *c2rust_fresh7 as ::core::ffi::c_int
        };
        if c <= 0x7f as ::core::ffi::c_int {
            let mut modifiers_after: ::core::ffi::c_int = modifiers;
            let mut mod_c: ::core::ffi::c_int = merge_modifiers(c, &raw mut modifiers_after);
            if modifiers_after == 0 as ::core::ffi::c_int {
                modifiers = 0 as ::core::ffi::c_int;
                c = mod_c;
            }
        }
    }
    if modifiers != 0 as ::core::ffi::c_int {
        add_to_showcmd(K_SPECIAL);
        add_to_showcmd(KS_MODIFIER);
        add_to_showcmd(modifiers);
    }
    if c != NUL {
        add_to_showcmd(c);
    }
    while *ptr as ::core::ffi::c_int != NUL {
        let c2rust_fresh8 = ptr;
        ptr = ptr.offset(1);
        add_to_showcmd(*c2rust_fresh8 as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn vgetc() -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    let mut buf: [uint8_t; 22] = [0; 22];
    if may_garbage_collect as ::core::ffi::c_int != 0
        && want_garbage_collect as ::core::ffi::c_int != 0
    {
        garbage_collect(false_0 != 0);
    }
    if can_get_old_char() {
        c = old_char;
        old_char = -1 as ::core::ffi::c_int;
        mod_mask = old_mod_mask;
        mouse_grid = old_mouse_grid;
        mouse_row = old_mouse_row;
        mouse_col = old_mouse_col;
    } else {
        static mut last_vgetc_recorded_len: size_t = 0 as size_t;
        mod_mask = 0 as ::core::ffi::c_int;
        vgetc_mod_mask = 0 as ::core::ffi::c_int;
        vgetc_char = 0 as ::core::ffi::c_int;
        last_recorded_len = last_recorded_len.wrapping_sub(last_vgetc_recorded_len);
        loop {
            let mut did_inc: bool = false_0 != 0;
            if mod_mask != 0 {
                no_mapping += 1;
                allow_keys += 1;
                did_inc = true_0 != 0;
            }
            c = vgetorpeek(true_0 != 0);
            if did_inc {
                no_mapping -= 1;
                allow_keys -= 1;
            }
            if c == K_SPECIAL {
                let mut save_allow_keys: ::core::ffi::c_int = allow_keys;
                no_mapping += 1;
                allow_keys = 0 as ::core::ffi::c_int;
                let mut c2: ::core::ffi::c_int = vgetorpeek(true_0 != 0);
                c = vgetorpeek(true_0 != 0);
                no_mapping -= 1;
                allow_keys = save_allow_keys;
                if c2 == KS_MODIFIER {
                    mod_mask = c;
                    continue;
                } else {
                    c = if c2 == KS_SPECIAL {
                        K_SPECIAL
                    } else if c2 == KS_ZERO {
                        K_ZERO
                    } else {
                        -(c2 + (c << 8 as ::core::ffi::c_int))
                    };
                }
            }
            let mut n: ::core::ffi::c_int = 0;
            n = if c < 0 as ::core::ffi::c_int || c > 255 as ::core::ffi::c_int {
                1 as ::core::ffi::c_int
            } else {
                utf8len_tab[c as usize] as ::core::ffi::c_int
            };
            if n > 1 as ::core::ffi::c_int {
                no_mapping += 1;
                buf[0 as ::core::ffi::c_int as usize] = c as uint8_t;
                let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                while i < n {
                    buf[i as usize] = vgetorpeek(true_0 != 0) as uint8_t;
                    if buf[i as usize] as ::core::ffi::c_int == K_SPECIAL {
                        vgetorpeek(true_0 != 0);
                        vgetorpeek(true_0 != 0);
                    }
                    i += 1;
                }
                no_mapping -= 1;
                c = utf_ptr2char(&raw mut buf as *mut uint8_t as *mut ::core::ffi::c_char);
            }
            if no_mapping == 0
                && KeyTyped as ::core::ffi::c_int != 0
                && mod_mask == MOD_MASK_ALT
                && State & MODE_TERMINAL as ::core::ffi::c_int == 0
                && !is_mouse_key(c)
            {
                mod_mask = 0 as ::core::ffi::c_int;
                let mut len: ::core::ffi::c_int =
                    ins_char_typebuf(c, 0 as ::core::ffi::c_int, false_0 != 0);
                ins_char_typebuf(ESC, 0 as ::core::ffi::c_int, false_0 != 0);
                let mut old_len: ::core::ffi::c_int = len + 3 as ::core::ffi::c_int;
                ungetchars(old_len);
                if on_key_buf.size >= old_len as size_t {
                    on_key_buf.size = on_key_buf.size.wrapping_sub(old_len as size_t);
                }
            } else {
                if vgetc_char == 0 as ::core::ffi::c_int {
                    vgetc_mod_mask = mod_mask;
                    vgetc_char = c;
                }
                match c {
                    K_KPLUS => {
                        c = '+' as ::core::ffi::c_int;
                    }
                    K_KMINUS => {
                        c = '-' as ::core::ffi::c_int;
                    }
                    K_KDIVIDE => {
                        c = '/' as ::core::ffi::c_int;
                    }
                    K_KMULTIPLY => {
                        c = '*' as ::core::ffi::c_int;
                    }
                    K_KENTER => {
                        c = CAR;
                    }
                    K_KPOINT => {
                        c = '.' as ::core::ffi::c_int;
                    }
                    K_KCOMMA => {
                        c = ',' as ::core::ffi::c_int;
                    }
                    K_KEQUAL => {
                        c = '=' as ::core::ffi::c_int;
                    }
                    K_K0 => {
                        c = '0' as ::core::ffi::c_int;
                    }
                    K_K1 => {
                        c = '1' as ::core::ffi::c_int;
                    }
                    K_K2 => {
                        c = '2' as ::core::ffi::c_int;
                    }
                    K_K3 => {
                        c = '3' as ::core::ffi::c_int;
                    }
                    K_K4 => {
                        c = '4' as ::core::ffi::c_int;
                    }
                    K_K5 => {
                        c = '5' as ::core::ffi::c_int;
                    }
                    K_K6 => {
                        c = '6' as ::core::ffi::c_int;
                    }
                    K_K7 => {
                        c = '7' as ::core::ffi::c_int;
                    }
                    K_K8 => {
                        c = '8' as ::core::ffi::c_int;
                    }
                    K_K9 => {
                        c = '9' as ::core::ffi::c_int;
                    }
                    K_XHOME | K_ZHOME => {
                        if mod_mask == MOD_MASK_SHIFT {
                            c = K_S_HOME;
                            mod_mask = 0 as ::core::ffi::c_int;
                        } else if mod_mask == MOD_MASK_CTRL {
                            c = -(253 as ::core::ffi::c_int
                                + ((KE_C_HOME as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
                            mod_mask = 0 as ::core::ffi::c_int;
                        } else {
                            c = K_HOME;
                        }
                    }
                    K_XEND | K_ZEND => {
                        if mod_mask == MOD_MASK_SHIFT {
                            c = K_S_END;
                            mod_mask = 0 as ::core::ffi::c_int;
                        } else if mod_mask == MOD_MASK_CTRL {
                            c = -(253 as ::core::ffi::c_int
                                + ((KE_C_END as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
                            mod_mask = 0 as ::core::ffi::c_int;
                        } else {
                            c = K_END;
                        }
                    }
                    K_KUP | K_XUP => {
                        c = K_UP;
                    }
                    K_KDOWN | K_XDOWN => {
                        c = K_DOWN;
                    }
                    K_KLEFT | K_XLEFT => {
                        c = K_LEFT;
                    }
                    K_KRIGHT | K_XRIGHT => {
                        c = K_RIGHT;
                    }
                    _ => {}
                }
                break;
            }
        }
        last_vgetc_recorded_len = last_recorded_len;
    }
    may_garbage_collect = false_0 != 0;
    if on_key_buf.size == on_key_buf.capacity {
        on_key_buf.capacity = if on_key_buf.capacity << 1 as ::core::ffi::c_int
            > ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            on_key_buf.capacity << 1 as ::core::ffi::c_int
        } else {
            ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as size_t,
                )
        };
        on_key_buf.items = (if on_key_buf.capacity
            == ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            if on_key_buf.items == &raw mut on_key_buf.init_array as *mut ::core::ffi::c_char {
                on_key_buf.items as *mut ::core::ffi::c_void
            } else {
                _memcpy_free(
                    &raw mut on_key_buf.init_array as *mut ::core::ffi::c_char
                        as *mut ::core::ffi::c_void,
                    on_key_buf.items as *mut ::core::ffi::c_void,
                    on_key_buf
                        .size
                        .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                )
            }
        } else {
            if on_key_buf.items == &raw mut on_key_buf.init_array as *mut ::core::ffi::c_char {
                memcpy(
                    xmalloc(
                        on_key_buf
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                    ),
                    on_key_buf.items as *const ::core::ffi::c_void,
                    on_key_buf
                        .size
                        .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                )
            } else {
                xrealloc(
                    on_key_buf.items as *mut ::core::ffi::c_void,
                    on_key_buf
                        .capacity
                        .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                )
            }
        }) as *mut ::core::ffi::c_char;
    } else {
    };
    let c2rust_fresh10 = on_key_buf.size;
    on_key_buf.size = on_key_buf.size.wrapping_add(1);
    *on_key_buf.items.offset(c2rust_fresh10 as isize) = '\0' as ::core::ffi::c_char;
    if nlua_execute_on_key(c, on_key_buf.items) {
        if c == -(253 as ::core::ffi::c_int
            + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            xfree(
                getcmdkeycmd(NUL, NULL_0, 0 as ::core::ffi::c_int, false_0 != 0)
                    as *mut ::core::ffi::c_void,
            );
        } else if c
            == -(253 as ::core::ffi::c_int
                + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            map_execute_lua(false_0 != 0, true_0 != 0);
        } else if c == K_PASTE_START {
            paste_repeat(0 as ::core::ffi::c_int);
        }
        c = -(253 as ::core::ffi::c_int
            + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
    }
    if on_key_buf.items != &raw mut on_key_buf.init_array as *mut ::core::ffi::c_char {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut on_key_buf.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        *ptr_;
    }
    on_key_buf.capacity = ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
        .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
        .wrapping_div(
            (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    on_key_buf.size = 0 as size_t;
    on_key_buf.items = &raw mut on_key_buf.init_array as *mut ::core::ffi::c_char;
    if c != -(253 as ::core::ffi::c_int
        + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        state_no_longer_safe(b"key typed\0".as_ptr() as *const ::core::ffi::c_char);
    }
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn safe_vgetc() -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = vgetc();
    if c == NUL {
        c = get_keystroke(::core::ptr::null_mut::<MultiQueue>());
    }
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn plain_vgetc() -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    loop {
        c = safe_vgetc();
        if !(c
            == -(253 as ::core::ffi::c_int
                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == K_VER_SCROLLBAR
            || c == K_HOR_SCROLLBAR
            || c == -(253 as ::core::ffi::c_int
                + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)))
        {
            break;
        }
    }
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn vpeekc() -> ::core::ffi::c_int {
    if can_get_old_char() {
        return old_char;
    }
    return vgetorpeek(false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn vpeekc_any() -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = vpeekc();
    if c == NUL && typebuf.tb_len > 0 as ::core::ffi::c_int {
        c = ESC;
    }
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn char_avail() -> bool {
    if test_disable_char_avail {
        return false_0 != 0;
    }
    no_mapping += 1;
    let mut retval: ::core::ffi::c_int = vpeekc();
    no_mapping -= 1;
    return retval != NUL;
}
static mut no_reduce_keys: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
unsafe extern "C" fn getchar_common(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut allow_number: bool,
) {
    let mut n: varnumber_T = 0 as varnumber_T;
    let called_emsg_start: ::core::ffi::c_int = called_emsg;
    let mut error: bool = false_0 != 0;
    let mut simplify: bool = true_0 != 0;
    let mut cursor_flag: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_check_for_opt_dict_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut d: *mut dict_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        if allow_number {
            allow_number = tv_dict_get_bool(
                d,
                b"number\0".as_ptr() as *const ::core::ffi::c_char,
                true_0,
            ) != 0;
        } else if tv_dict_has_key(d, b"number\0".as_ptr() as *const ::core::ffi::c_char) {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                b"number\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        simplify = tv_dict_get_bool(
            d,
            b"simplify\0".as_ptr() as *const ::core::ffi::c_char,
            true_0,
        ) != 0;
        let mut cursor_str: *const ::core::ffi::c_char = tv_dict_get_string(
            d,
            b"cursor\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        if !cursor_str.is_null() {
            if strcmp(cursor_str, b"hide\0".as_ptr() as *const ::core::ffi::c_char)
                != 0 as ::core::ffi::c_int
                && strcmp(cursor_str, b"keep\0".as_ptr() as *const ::core::ffi::c_char)
                    != 0 as ::core::ffi::c_int
                && strcmp(cursor_str, b"msg\0".as_ptr() as *const ::core::ffi::c_char)
                    != 0 as ::core::ffi::c_int
            {
                semsg(
                    gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
                    b"cursor\0".as_ptr() as *const ::core::ffi::c_char,
                    cursor_str,
                );
            } else {
                cursor_flag = *cursor_str.offset(0 as ::core::ffi::c_int as isize);
            }
        }
    }
    if called_emsg != called_emsg_start {
        return;
    }
    if cursor_flag as ::core::ffi::c_int == 'h' as ::core::ffi::c_int {
        ui_busy_start();
    }
    no_mapping += 1;
    allow_keys += 1;
    if !simplify {
        no_reduce_keys += 1;
    }
    loop {
        if cursor_flag as ::core::ffi::c_int == 'm' as ::core::ffi::c_int
            || cursor_flag as ::core::ffi::c_int == NUL && msg_col > 0 as ::core::ffi::c_int
        {
            ui_cursor_goto(msg_row, msg_col);
        }
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_number
                    == -1 as varnumber_T
        {
            if !char_avail() {
                ui_flush();
                input_get(
                    ::core::ptr::null_mut::<uint8_t>(),
                    0 as ::core::ffi::c_int,
                    -1 as ::core::ffi::c_int,
                    typebuf.tb_change_cnt,
                    main_loop.events,
                );
                if input_available() == 0 && !multiqueue_empty(main_loop.events) {
                    state_handle_k_event();
                    continue;
                }
            }
            n = safe_vgetc() as varnumber_T;
        } else if tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) == 1 as varnumber_T
        {
            n = vpeekc_any() as varnumber_T;
        } else if error as ::core::ffi::c_int != 0 || vpeekc_any() == NUL {
            n = 0 as varnumber_T;
        } else {
            n = safe_vgetc() as varnumber_T;
        }
        if !(n
            == -(253 as ::core::ffi::c_int
                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                as varnumber_T
            || n == -(253 as ::core::ffi::c_int
                + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                as varnumber_T
            || n == K_VER_SCROLLBAR as varnumber_T
            || n == K_HOR_SCROLLBAR as varnumber_T)
        {
            break;
        }
    }
    no_mapping -= 1;
    allow_keys -= 1;
    if !simplify {
        no_reduce_keys -= 1;
    }
    if cursor_flag as ::core::ffi::c_int == 'h' as ::core::ffi::c_int {
        ui_busy_stop();
    }
    set_vim_var_nr(VV_MOUSE_WIN, 0 as varnumber_T);
    set_vim_var_nr(VV_MOUSE_WINID, 0 as varnumber_T);
    set_vim_var_nr(VV_MOUSE_LNUM, 0 as varnumber_T);
    set_vim_var_nr(VV_MOUSE_COL, 0 as varnumber_T);
    if n != 0 as varnumber_T
        && (!allow_number || n < 0 as varnumber_T || mod_mask != 0 as ::core::ffi::c_int)
    {
        let mut temp: [::core::ffi::c_char; 10] = [0; 10];
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if mod_mask != 0 as ::core::ffi::c_int {
            let c2rust_fresh11 = i;
            i = i + 1;
            temp[c2rust_fresh11 as usize] = K_SPECIAL as ::core::ffi::c_char;
            let c2rust_fresh12 = i;
            i = i + 1;
            temp[c2rust_fresh12 as usize] = KS_MODIFIER as ::core::ffi::c_char;
            let c2rust_fresh13 = i;
            i = i + 1;
            temp[c2rust_fresh13 as usize] = mod_mask as ::core::ffi::c_char;
        }
        if n < 0 as varnumber_T {
            let c2rust_fresh14 = i;
            i = i + 1;
            temp[c2rust_fresh14 as usize] = K_SPECIAL as ::core::ffi::c_char;
            let c2rust_fresh15 = i;
            i = i + 1;
            temp[c2rust_fresh15 as usize] = (if n == K_SPECIAL as varnumber_T {
                KS_SPECIAL as varnumber_T
            } else if n == NUL as varnumber_T {
                KS_ZERO as varnumber_T
            } else {
                -n & 0xff as varnumber_T
            }) as ::core::ffi::c_char;
            let c2rust_fresh16 = i;
            i = i + 1;
            temp[c2rust_fresh16 as usize] = (if n == K_SPECIAL as varnumber_T
                || n == NUL as varnumber_T
            {
                KE_FILLER as ::core::ffi::c_uint
            } else {
                -n as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint
            }) as ::core::ffi::c_char;
        } else {
            i += utf_char2bytes(
                n as ::core::ffi::c_int,
                (&raw mut temp as *mut ::core::ffi::c_char).offset(i as isize),
            );
        }
        '_c2rust_label: {
            if i < 10 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"i < 10\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/getchar.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    2021 as ::core::ffi::c_uint,
                    b"void getchar_common(typval_T *, typval_T *, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        temp[i as usize] = NUL as ::core::ffi::c_char;
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = xmemdupz(
            &raw mut temp as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            i as size_t,
        ) as *mut ::core::ffi::c_char;
        if is_mouse_key(n as ::core::ffi::c_int) {
            let mut row: ::core::ffi::c_int = mouse_row;
            let mut col: ::core::ffi::c_int = mouse_col;
            let mut grid: ::core::ffi::c_int = mouse_grid;
            let mut lnum: linenr_T = 0;
            let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
            if row >= 0 as ::core::ffi::c_int && col >= 0 as ::core::ffi::c_int {
                let mut winnr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                let win: *mut win_T =
                    mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col);
                if win.is_null() {
                    return;
                }
                mouse_comp_pos(win, &raw mut row, &raw mut col, &raw mut lnum);
                wp = firstwin;
                while wp != win {
                    winnr += 1;
                    wp = (*wp).w_next;
                }
                set_vim_var_nr(VV_MOUSE_WIN, winnr as varnumber_T);
                set_vim_var_nr(VV_MOUSE_WINID, (*wp).handle as varnumber_T);
                set_vim_var_nr(VV_MOUSE_LNUM, lnum as varnumber_T);
                set_vim_var_nr(VV_MOUSE_COL, (col + 1 as ::core::ffi::c_int) as varnumber_T);
            }
        }
    } else if !allow_number {
        (*rettv).v_type = VAR_STRING;
    } else {
        (*rettv).vval.v_number = n;
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_getchar(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    getchar_common(argvars, rettv, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_getcharstr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    getchar_common(argvars, rettv, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_getcharmod(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = mod_mask as varnumber_T;
}
unsafe extern "C" fn put_string_in_typebuf(
    mut offset: ::core::ffi::c_int,
    mut slen: ::core::ffi::c_int,
    mut string: *mut uint8_t,
    mut new_slen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut extra: ::core::ffi::c_int = new_slen - slen;
    *string.offset(new_slen as isize) = NUL as uint8_t;
    if extra < 0 as ::core::ffi::c_int {
        del_typebuf(-extra, offset);
    } else if extra > 0 as ::core::ffi::c_int {
        if ins_typebuf(
            (string as *mut ::core::ffi::c_char).offset(slen as isize),
            REMAP_YES as ::core::ffi::c_int,
            offset,
            false_0 != 0,
            false_0 != 0,
        ) == FAIL
        {
            return FAIL;
        }
    }
    memmove(
        typebuf
            .tb_buf
            .offset(typebuf.tb_off as isize)
            .offset(offset as isize) as *mut ::core::ffi::c_void,
        string as *const ::core::ffi::c_void,
        new_slen as size_t,
    );
    return OK;
}
unsafe extern "C" fn at_ins_compl_key() -> bool {
    let mut p: *mut uint8_t = typebuf.tb_buf.offset(typebuf.tb_off as isize);
    let mut c: ::core::ffi::c_int = *p as ::core::ffi::c_int;
    if typebuf.tb_len > 3 as ::core::ffi::c_int
        && c == K_SPECIAL
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_MODIFIER
        && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int & MOD_MASK_CTRL != 0
    {
        c = *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int;
    }
    return ctrl_x_mode_not_default() as ::core::ffi::c_int != 0
        && vim_is_ctrl_x_key(c) as ::core::ffi::c_int != 0
        || compl_status_local() as ::core::ffi::c_int != 0 && (c == Ctrl_N || c == Ctrl_P);
}
unsafe extern "C" fn check_simplify_modifier(
    mut max_offset: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if State & MODE_TERMINAL as ::core::ffi::c_int != 0 || no_reduce_keys > 0 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    let mut offset: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while offset < max_offset {
        if offset + 3 as ::core::ffi::c_int >= typebuf.tb_len {
            break;
        }
        let mut tp: *mut uint8_t = typebuf
            .tb_buf
            .offset(typebuf.tb_off as isize)
            .offset(offset as isize);
        if *tp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == K_SPECIAL
            && *tp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_MODIFIER
        {
            let mut modifier: ::core::ffi::c_int =
                *tp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
            let mut c: ::core::ffi::c_int =
                *tp.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
            let mut new_c: ::core::ffi::c_int = merge_modifiers(c, &raw mut modifier);
            if new_c != c {
                if offset == 0 as ::core::ffi::c_int {
                    vgetc_char = c;
                    vgetc_mod_mask =
                        *tp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
                }
                let mut new_string: [uint8_t; 21] = [0; 21];
                let mut len: ::core::ffi::c_int = 0;
                if new_c < 0 as ::core::ffi::c_int {
                    new_string[0 as ::core::ffi::c_int as usize] = K_SPECIAL as uint8_t;
                    new_string[1 as ::core::ffi::c_int as usize] = (if new_c == K_SPECIAL {
                        KS_SPECIAL
                    } else if new_c == NUL {
                        KS_ZERO
                    } else {
                        -new_c & 0xff as ::core::ffi::c_int
                    })
                        as uint8_t;
                    new_string[2 as ::core::ffi::c_int as usize] =
                        (if new_c == K_SPECIAL || new_c == NUL {
                            KE_FILLER as ::core::ffi::c_uint
                        } else {
                            -new_c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int
                                & 0xff as ::core::ffi::c_uint
                        }) as uint8_t;
                    len = 3 as ::core::ffi::c_int;
                } else {
                    len = utf_char2bytes(
                        new_c,
                        &raw mut new_string as *mut uint8_t as *mut ::core::ffi::c_char,
                    );
                }
                if modifier == 0 as ::core::ffi::c_int {
                    if put_string_in_typebuf(
                        offset,
                        4 as ::core::ffi::c_int,
                        &raw mut new_string as *mut uint8_t,
                        len,
                    ) == FAIL
                    {
                        return -1 as ::core::ffi::c_int;
                    }
                } else {
                    *tp.offset(2 as ::core::ffi::c_int as isize) = modifier as uint8_t;
                    if put_string_in_typebuf(
                        offset + 3 as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                        &raw mut new_string as *mut uint8_t,
                        len,
                    ) == FAIL
                    {
                        return -1 as ::core::ffi::c_int;
                    }
                }
                return len;
            }
        }
        offset += 1;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn handle_mapping(
    mut keylenp: *mut ::core::ffi::c_int,
    mut timedout: *const bool,
    mut mapdepth: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
    let mut mp2: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
    let mut mp_match: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
    let mut mp_match_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut max_mlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut keylen: ::core::ffi::c_int = *keylenp;
    let mut local_State: ::core::ffi::c_int = get_real_state();
    let mut is_plug_map: bool = false_0 != 0;
    if typebuf.tb_len >= 3 as ::core::ffi::c_int
        && *typebuf.tb_buf.offset(typebuf.tb_off as isize) as ::core::ffi::c_int == K_SPECIAL
        && *typebuf
            .tb_buf
            .offset((typebuf.tb_off + 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            == KS_EXTRA
        && *typebuf
            .tb_buf
            .offset((typebuf.tb_off + 2 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            == KE_PLUG as ::core::ffi::c_int
    {
        is_plug_map = true_0 != 0;
    }
    let mut tb_c1: ::core::ffi::c_int =
        *typebuf.tb_buf.offset(typebuf.tb_off as isize) as ::core::ffi::c_int;
    if no_mapping == 0 as ::core::ffi::c_int
        && (no_zero_mapping == 0 as ::core::ffi::c_int || tb_c1 != '0' as ::core::ffi::c_int)
        && (typebuf.tb_maplen == 0 as ::core::ffi::c_int
            || is_plug_map as ::core::ffi::c_int != 0
            || *typebuf.tb_noremap.offset(typebuf.tb_off as isize) as ::core::ffi::c_int
                & (RM_NONE as ::core::ffi::c_int | RM_ABBR as ::core::ffi::c_int)
                == 0)
        && !(p_paste != 0
            && State & (MODE_INSERT as ::core::ffi::c_int | MODE_CMDLINE as ::core::ffi::c_int)
                != 0)
        && !(State == MODE_HITRETURN as ::core::ffi::c_int
            && (tb_c1 == CAR || tb_c1 == ' ' as ::core::ffi::c_int))
        && State != MODE_ASKMORE as ::core::ffi::c_int
        && !at_ins_compl_key()
    {
        let mut mlen: ::core::ffi::c_int = 0;
        let mut nolmaplen: ::core::ffi::c_int = 0;
        if tb_c1 == K_SPECIAL {
            nolmaplen = 2 as ::core::ffi::c_int;
        } else {
            if *p_langmap as ::core::ffi::c_int != 0
                && (State
                    & (MODE_CMDLINE as ::core::ffi::c_int | MODE_INSERT as ::core::ffi::c_int)
                    == 0 as ::core::ffi::c_int
                    && get_real_state() != MODE_SELECT as ::core::ffi::c_int)
                && (p_lrm != 0
                    || (if vgetc_busy != 0 {
                        (typebuf_maplen() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                    } else {
                        KeyTyped as ::core::ffi::c_int
                    }) != 0)
                && KeyStuffed == 0
                && tb_c1 >= 0 as ::core::ffi::c_int
            {
                if tb_c1 < 256 as ::core::ffi::c_int {
                    tb_c1 = langmap_mapchar[tb_c1 as usize] as ::core::ffi::c_int;
                } else {
                    tb_c1 = langmap_adjust_mb(tb_c1);
                }
            }
            nolmaplen = 0 as ::core::ffi::c_int;
        }
        mp = get_buf_maphash_list(local_State, tb_c1);
        mp2 = get_maphash_list(local_State, tb_c1);
        if mp.is_null() {
            mp = mp2;
            mp2 = ::core::ptr::null_mut::<mapblock_T>();
        }
        mp_match = ::core::ptr::null_mut::<mapblock_T>();
        mp_match_len = 0 as ::core::ffi::c_int;
        while !mp.is_null() {
            if *(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                as ::core::ffi::c_int
                == tb_c1
                && (*mp).m_mode & local_State != 0
                && ((*mp).m_mode & MODE_LANGMAP as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                    || typebuf.tb_maplen == 0 as ::core::ffi::c_int)
            {
                let mut nomap: ::core::ffi::c_int = nolmaplen;
                let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                mlen = 1 as ::core::ffi::c_int;
                while mlen < typebuf.tb_len {
                    let mut c2: ::core::ffi::c_int =
                        *typebuf.tb_buf.offset((typebuf.tb_off + mlen) as isize)
                            as ::core::ffi::c_int;
                    if nomap > 0 as ::core::ffi::c_int {
                        if nomap == 2 as ::core::ffi::c_int && c2 == KS_MODIFIER {
                            modifiers = 1 as ::core::ffi::c_int;
                        } else if nomap == 1 as ::core::ffi::c_int
                            && modifiers == 1 as ::core::ffi::c_int
                        {
                            modifiers = c2;
                        }
                        nomap -= 1;
                    } else {
                        if c2 == K_SPECIAL {
                            nomap = 2 as ::core::ffi::c_int;
                        } else if merge_modifiers(c2, &raw mut modifiers) == c2 {
                            if *p_langmap as ::core::ffi::c_int != 0
                                && true
                                && (p_lrm != 0
                                    || (if vgetc_busy != 0 {
                                        (typebuf_maplen() == 0 as ::core::ffi::c_int)
                                            as ::core::ffi::c_int
                                    } else {
                                        KeyTyped as ::core::ffi::c_int
                                    }) != 0)
                                && KeyStuffed == 0
                                && c2 >= 0 as ::core::ffi::c_int
                            {
                                if c2 < 256 as ::core::ffi::c_int {
                                    c2 = langmap_mapchar[c2 as usize] as ::core::ffi::c_int;
                                } else {
                                    c2 = langmap_adjust_mb(c2);
                                }
                            }
                        }
                        modifiers = 0 as ::core::ffi::c_int;
                    }
                    if *(*mp).m_keys.offset(mlen as isize) as uint8_t as ::core::ffi::c_int != c2 {
                        break;
                    }
                    mlen += 1;
                }
                let mut p1: *const ::core::ffi::c_char = (*mp).m_keys;
                let mut p2: *const ::core::ffi::c_char = mb_unescape(&raw mut p1);
                if !p2.is_null()
                    && utf8len_tab[tb_c1 as usize] as ::core::ffi::c_int > utfc_ptr2len(p2)
                {
                    mlen = 0 as ::core::ffi::c_int;
                }
                keylen = (*mp).m_keylen;
                if mlen == keylen || mlen == typebuf.tb_len && typebuf.tb_len < keylen {
                    let mut n: ::core::ffi::c_int = 0;
                    let mut s: *mut uint8_t = typebuf.tb_noremap.offset(typebuf.tb_off as isize);
                    if !(*s as ::core::ffi::c_int == RM_SCRIPT as ::core::ffi::c_int
                        && (*(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int
                            != K_SPECIAL
                            || *(*mp).m_keys.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int
                                != KS_EXTRA
                            || *(*mp).m_keys.offset(2 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                != KE_SNR as ::core::ffi::c_int))
                    {
                        n = mlen;
                        loop {
                            n -= 1;
                            if n < 0 as ::core::ffi::c_int {
                                break;
                            }
                            let c2rust_fresh9 = s;
                            s = s.offset(1);
                            if *c2rust_fresh9 as ::core::ffi::c_int
                                & (RM_NONE as ::core::ffi::c_int | RM_ABBR as ::core::ffi::c_int)
                                != 0
                            {
                                break;
                            }
                        }
                        if !(!is_plug_map && n >= 0 as ::core::ffi::c_int) {
                            if keylen > typebuf.tb_len {
                                if !*timedout
                                    && !(!mp_match.is_null()
                                        && (*mp_match).m_nowait as ::core::ffi::c_int != 0)
                                {
                                    keylen = KEYLEN_PART_MAP as ::core::ffi::c_int;
                                    break;
                                }
                            } else if keylen > mp_match_len
                                || keylen == mp_match_len
                                    && !mp_match.is_null()
                                    && (*mp_match).m_mode & MODE_LANGMAP as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    && (*mp).m_mode & MODE_LANGMAP as ::core::ffi::c_int
                                        != 0 as ::core::ffi::c_int
                            {
                                mp_match = mp;
                                mp_match_len = keylen;
                            }
                        }
                    }
                } else {
                    max_mlen = if max_mlen > mlen { max_mlen } else { mlen };
                }
            }
            if (*mp).m_next.is_null() {
                mp = mp2;
                mp2 = ::core::ptr::null_mut::<mapblock_T>();
            } else {
                mp = (*mp).m_next;
            };
        }
        if keylen != KEYLEN_PART_MAP as ::core::ffi::c_int && !mp_match.is_null() {
            mp = mp_match;
            keylen = mp_match_len;
        }
    }
    if (mp.is_null() || max_mlen > mp_match_len) && keylen != KEYLEN_PART_MAP as ::core::ffi::c_int
    {
        if no_mapping == 0 as ::core::ffi::c_int || allow_keys != 0 as ::core::ffi::c_int {
            if tb_c1 == K_SPECIAL
                && (typebuf.tb_len < 2 as ::core::ffi::c_int
                    || *typebuf
                        .tb_buf
                        .offset((typebuf.tb_off + 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        == KS_MODIFIER
                        && typebuf.tb_len < 4 as ::core::ffi::c_int)
            {
                keylen = KEYLEN_PART_KEY as ::core::ffi::c_int;
            } else {
                keylen = check_simplify_modifier(max_mlen + 1 as ::core::ffi::c_int);
                if keylen < 0 as ::core::ffi::c_int {
                    return map_result_fail as ::core::ffi::c_int;
                }
            }
        } else {
            keylen = 0 as ::core::ffi::c_int;
        }
        if keylen == 0 as ::core::ffi::c_int {
            if mp.is_null() {
                *keylenp = keylen;
                return map_result_get as ::core::ffi::c_int;
            }
        }
        if keylen > 0 as ::core::ffi::c_int {
            *keylenp = keylen;
            return map_result_retry as ::core::ffi::c_int;
        }
        if keylen < 0 as ::core::ffi::c_int {
            '_c2rust_label: {
                if keylen == KEYLEN_PART_KEY as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"keylen == KEYLEN_PART_KEY\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/getchar.c\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        2385 as ::core::ffi::c_uint,
                        b"int handle_mapping(int *, const _Bool *, int *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
        } else {
            '_c2rust_label_0: {
                if !mp.is_null() {
                } else {
                    __assert_fail(
                        b"mp != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/getchar.c\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        2387 as ::core::ffi::c_uint,
                        b"int handle_mapping(int *, const _Bool *, int *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            keylen = mp_match_len;
        }
    }
    if keylen >= 0 as ::core::ffi::c_int && keylen <= typebuf.tb_len {
        let mut i: ::core::ffi::c_int = 0;
        let mut map_str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if keylen > typebuf.tb_maplen
            && (*mp).m_mode & MODE_LANGMAP as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        {
            gotchars(
                typebuf
                    .tb_buf
                    .offset(typebuf.tb_off as isize)
                    .offset(typebuf.tb_maplen as isize),
                (keylen - typebuf.tb_maplen) as size_t,
            );
        }
        cmd_silent = typebuf.tb_silent > 0 as ::core::ffi::c_int;
        del_typebuf(keylen, 0 as ::core::ffi::c_int);
        *mapdepth += 1;
        if *mapdepth as OptInt >= p_mmd {
            emsg(gettext(
                &raw const e_recursive_mapping as *const ::core::ffi::c_char,
            ));
            if State & MODE_CMDLINE as ::core::ffi::c_int != 0 {
                redrawcmdline();
            } else {
                setcursor();
            }
            flush_buffers(FLUSH_MINIMAL);
            *mapdepth = 0 as ::core::ffi::c_int;
            *keylenp = keylen;
            return map_result_fail as ::core::ffi::c_int;
        }
        if VIsual_active as ::core::ffi::c_int != 0
            && VIsual_select as ::core::ffi::c_int != 0
            && (*mp).m_mode & MODE_VISUAL as ::core::ffi::c_int != 0
        {
            VIsual_select = false_0 != 0;
            ins_typebuf(
                K_SELECT_STRING.as_ptr() as *mut ::core::ffi::c_char,
                REMAP_NONE as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                true_0 != 0,
                false_0 != 0,
            );
        }
        let save_m_expr: bool = (*mp).m_expr != 0;
        let save_m_noremap: ::core::ffi::c_int = (*mp).m_noremap;
        let save_m_silent: bool = (*mp).m_silent != 0;
        let mut save_m_keys: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut save_alt_m_keys: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let save_alt_m_keylen: ::core::ffi::c_int = if !(*mp).m_alt.is_null() {
            (*(*mp).m_alt).m_keylen
        } else {
            0 as ::core::ffi::c_int
        };
        if (*mp).m_expr != 0 {
            let save_vgetc_busy: ::core::ffi::c_int = vgetc_busy;
            let save_may_garbage_collect: bool = may_garbage_collect;
            let prev_did_emsg: ::core::ffi::c_int = did_emsg;
            vgetc_busy = 0 as ::core::ffi::c_int;
            may_garbage_collect = false_0 != 0;
            save_m_keys = xmemdupz(
                (*mp).m_keys as *const ::core::ffi::c_void,
                (*mp).m_keylen as size_t,
            ) as *mut ::core::ffi::c_char;
            save_alt_m_keys = (if !(*mp).m_alt.is_null() {
                xmemdupz(
                    (*(*mp).m_alt).m_keys as *const ::core::ffi::c_void,
                    save_alt_m_keylen as size_t,
                )
            } else {
                NULL_0
            }) as *mut ::core::ffi::c_char;
            map_str = eval_map_expr(mp, NUL);
            if map_str.is_null() || *map_str as ::core::ffi::c_int == NUL {
                if prev_did_emsg != did_emsg {
                    let mut buf: [::core::ffi::c_char; 4] = [0; 4];
                    xfree(map_str as *mut ::core::ffi::c_void);
                    buf[0 as ::core::ffi::c_int as usize] = K_SPECIAL as ::core::ffi::c_char;
                    buf[1 as ::core::ffi::c_int as usize] = KS_EXTRA as ::core::ffi::c_char;
                    buf[2 as ::core::ffi::c_int as usize] =
                        KE_IGNORE as ::core::ffi::c_int as ::core::ffi::c_char;
                    buf[3 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                    map_str = xmemdupz(
                        &raw mut buf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                        3 as size_t,
                    ) as *mut ::core::ffi::c_char;
                    if State & MODE_CMDLINE as ::core::ffi::c_int != 0 {
                        msg_didout = true_0 != 0;
                        msg_row = if msg_row > cmdline_row {
                            msg_row
                        } else {
                            cmdline_row
                        };
                        redrawcmd();
                    }
                } else if State
                    & (MODE_NORMAL as ::core::ffi::c_int | MODE_INSERT as ::core::ffi::c_int)
                    != 0
                {
                    setcursor();
                }
            }
            vgetc_busy = save_vgetc_busy;
            may_garbage_collect = save_may_garbage_collect;
        } else {
            map_str = (*mp).m_str;
        }
        if map_str.is_null() {
            i = FAIL;
        } else {
            let mut noremap: ::core::ffi::c_int = 0;
            if keylen > typebuf.tb_maplen
                && (*mp).m_mode & MODE_LANGMAP as ::core::ffi::c_int != 0 as ::core::ffi::c_int
            {
                gotchars(map_str as *mut uint8_t, strlen(map_str));
            }
            if save_m_noremap != REMAP_YES as ::core::ffi::c_int {
                noremap = save_m_noremap;
            } else if if save_m_expr as ::core::ffi::c_int != 0 {
                (strncmp(map_str, save_m_keys, keylen as size_t) == 0 as ::core::ffi::c_int
                    || !save_alt_m_keys.is_null()
                        && strncmp(map_str, save_alt_m_keys, save_alt_m_keylen as size_t)
                            == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            } else {
                (strncmp(map_str, (*mp).m_keys, keylen as size_t) == 0 as ::core::ffi::c_int
                    || !(*mp).m_alt.is_null()
                        && strncmp(
                            map_str,
                            (*(*mp).m_alt).m_keys,
                            (*(*mp).m_alt).m_keylen as size_t,
                        ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            } != 0
            {
                noremap = REMAP_SKIP as ::core::ffi::c_int;
            } else {
                noremap = REMAP_YES as ::core::ffi::c_int;
            }
            i = ins_typebuf(
                map_str,
                noremap,
                0 as ::core::ffi::c_int,
                true_0 != 0,
                cmd_silent as ::core::ffi::c_int != 0 || save_m_silent as ::core::ffi::c_int != 0,
            );
            if save_m_expr {
                xfree(map_str as *mut ::core::ffi::c_void);
            }
        }
        xfree(save_m_keys as *mut ::core::ffi::c_void);
        xfree(save_alt_m_keys as *mut ::core::ffi::c_void);
        *keylenp = keylen;
        if i == FAIL {
            return map_result_fail as ::core::ffi::c_int;
        }
        return map_result_retry as ::core::ffi::c_int;
    }
    *keylenp = keylen;
    return map_result_nomatch as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn vungetc(mut c: ::core::ffi::c_int) {
    old_char = c;
    old_mod_mask = mod_mask;
    old_mouse_grid = mouse_grid;
    old_mouse_row = mouse_row;
    old_mouse_col = mouse_col;
    old_KeyStuffed = KeyStuffed;
}
#[no_mangle]
pub unsafe extern "C" fn check_end_reg_executing(mut advance: bool) {
    if reg_executing != 0 as ::core::ffi::c_int
        && (typebuf.tb_maplen == 0 as ::core::ffi::c_int
            || pending_end_reg_executing as ::core::ffi::c_int != 0)
    {
        if advance {
            reg_executing = 0 as ::core::ffi::c_int;
            pending_end_reg_executing = false_0 != 0;
        } else {
            pending_end_reg_executing = true_0 != 0;
        }
    }
}
unsafe extern "C" fn vgetorpeek(mut advance: bool) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    let mut timedout: bool = false_0 != 0;
    let mut mapdepth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut mode_deleted: bool = false_0 != 0;
    if vgetc_busy > 0 as ::core::ffi::c_int && ex_normal_busy == 0 as ::core::ffi::c_int {
        return NUL;
    }
    vgetc_busy += 1;
    if advance {
        KeyStuffed = false_0;
        typebuf_was_empty = false_0 != 0;
    }
    init_typebuf();
    start_stuff();
    check_end_reg_executing(advance);
    loop {
        if typeahead_char != 0 as ::core::ffi::c_int {
            c = typeahead_char;
            if advance {
                typeahead_char = 0 as ::core::ffi::c_int;
            }
        } else {
            c = read_readbuffers(advance);
        }
        if c != NUL && !got_int {
            if advance {
                KeyStuffed = true_0;
            }
            if typebuf.tb_no_abbr_cnt == 0 as ::core::ffi::c_int {
                typebuf.tb_no_abbr_cnt = 1 as ::core::ffi::c_int;
            }
        } else {
            loop {
                check_end_reg_executing(advance);
                if typebuf.tb_maplen != 0 {
                    line_breakcheck();
                } else {
                    if (mapped_ctrl_c | (*curbuf).b_mapped_ctrl_c) & get_real_state() != 0 {
                        ctrl_c_interrupts = false_0 != 0;
                    }
                    os_breakcheck();
                    ctrl_c_interrupts = true_0 != 0;
                }
                let mut keylen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                if got_int {
                    c = inchar(
                        typebuf.tb_buf,
                        typebuf.tb_buflen - 1 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_long,
                    );
                    if (c != 0 || typebuf.tb_maplen != 0)
                        && State
                            & (MODE_INSERT as ::core::ffi::c_int
                                | MODE_CMDLINE as ::core::ffi::c_int)
                            != 0
                    {
                        c = ESC;
                    } else {
                        c = Ctrl_C;
                    }
                    flush_buffers(FLUSH_INPUT);
                    if advance {
                        *typebuf.tb_buf = c as uint8_t;
                        gotchars(typebuf.tb_buf, 1 as size_t);
                    }
                    cmd_silent = false_0 != 0;
                    break;
                } else {
                    if typebuf.tb_len > 0 as ::core::ffi::c_int {
                        let mut result: map_result_T =
                            handle_mapping(&raw mut keylen, &raw mut timedout, &raw mut mapdepth)
                                as map_result_T;
                        if result as ::core::ffi::c_uint
                            == map_result_retry as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            continue;
                        }
                        if result as ::core::ffi::c_uint
                            == map_result_fail as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            c = -1 as ::core::ffi::c_int;
                            break;
                        } else if result as ::core::ffi::c_uint
                            == map_result_get as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            c = *typebuf.tb_buf.offset(typebuf.tb_off as isize)
                                as ::core::ffi::c_int;
                            if advance {
                                cmd_silent = typebuf.tb_silent > 0 as ::core::ffi::c_int;
                                if typebuf.tb_maplen > 0 as ::core::ffi::c_int {
                                    KeyTyped = false_0 != 0;
                                } else {
                                    KeyTyped = true_0 != 0;
                                    gotchars(
                                        typebuf.tb_buf.offset(typebuf.tb_off as isize),
                                        1 as size_t,
                                    );
                                }
                                KeyNoremap = *typebuf.tb_noremap.offset(typebuf.tb_off as isize)
                                    as ::core::ffi::c_uchar
                                    as ::core::ffi::c_int;
                                del_typebuf(1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
                            }
                            break;
                        }
                    }
                    c = 0 as ::core::ffi::c_int;
                    let mut new_wcol: ::core::ffi::c_int = (*curwin).w_wcol;
                    let mut new_wrow: ::core::ffi::c_int = (*curwin).w_wrow;
                    if advance as ::core::ffi::c_int != 0
                        && typebuf.tb_len == 1 as ::core::ffi::c_int
                        && *typebuf.tb_buf.offset(typebuf.tb_off as isize) as ::core::ffi::c_int
                            == ESC
                        && no_mapping == 0
                        && ex_normal_busy == 0 as ::core::ffi::c_int
                        && typebuf.tb_maplen == 0 as ::core::ffi::c_int
                        && State & MODE_INSERT as ::core::ffi::c_int != 0
                        && (p_timeout != 0
                            || keylen == KEYLEN_PART_KEY as ::core::ffi::c_int && p_ttimeout != 0)
                        && {
                            c = inchar(
                                typebuf
                                    .tb_buf
                                    .offset(typebuf.tb_off as isize)
                                    .offset(typebuf.tb_len as isize),
                                3 as ::core::ffi::c_int,
                                25 as ::core::ffi::c_long,
                            );
                            c == 0 as ::core::ffi::c_int
                        }
                    {
                        if mode_displayed {
                            unshowmode(true_0 != 0);
                            mode_deleted = true_0 != 0;
                        }
                        validate_cursor(curwin);
                        let mut old_wcol: ::core::ffi::c_int = (*curwin).w_wcol;
                        let mut old_wrow: ::core::ffi::c_int = (*curwin).w_wrow;
                        if (*curwin).w_cursor.col != 0 as ::core::ffi::c_int {
                            let mut col: colnr_T = 0 as colnr_T;
                            let mut ptr: *mut ::core::ffi::c_char =
                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                            if (*curwin).w_wcol > 0 as ::core::ffi::c_int {
                                if did_ai as ::core::ffi::c_int != 0
                                    && *skipwhite(
                                        get_cursor_line_ptr()
                                            .offset((*curwin).w_cursor.col as isize),
                                    ) as ::core::ffi::c_int
                                        == NUL
                                {
                                    (*curwin).w_wcol = 0 as ::core::ffi::c_int;
                                    ptr = get_cursor_line_ptr();
                                    let mut endptr: *mut ::core::ffi::c_char =
                                        ptr.offset((*curwin).w_cursor.col as isize);
                                    let mut csarg: CharsizeArg = CharsizeArg {
                                        win: ::core::ptr::null_mut::<win_T>(),
                                        line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                        use_tabstop: false,
                                        indent_width: 0,
                                        virt_row: 0,
                                        cur_text_width_left: 0,
                                        cur_text_width_right: 0,
                                        max_head_vcol: 0,
                                        iter: [MarkTreeIter {
                                            pos: MTPos { row: 0, col: 0 },
                                            lvl: 0,
                                            x: ::core::ptr::null_mut::<MTNode>(),
                                            i: 0,
                                            s: [C2Rust_Unnamed_28 { oldcol: 0, i: 0 }; 20],
                                            intersect_idx: 0,
                                            intersect_pos: MTPos { row: 0, col: 0 },
                                            intersect_pos_x: MTPos { row: 0, col: 0 },
                                        }; 1],
                                    };
                                    let mut cstype: CSType = init_charsize_arg(
                                        &raw mut csarg,
                                        curwin,
                                        (*curwin).w_cursor.lnum,
                                        ptr,
                                    );
                                    let mut ci: StrCharInfo = utf_ptr2StrCharInfo(ptr);
                                    let mut vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    while ci.ptr < endptr {
                                        if !ascii_iswhite(ci.chr.value as ::core::ffi::c_int) {
                                            (*curwin).w_wcol = vcol;
                                        }
                                        vcol += win_charsize(
                                            cstype,
                                            vcol,
                                            ci.ptr,
                                            ci.chr.value,
                                            &raw mut csarg,
                                        )
                                        .width;
                                        ci = utfc_next(ci);
                                    }
                                    (*curwin).w_wrow = (*curwin).w_cline_row
                                        + (*curwin).w_wcol / (*curwin).w_view_width;
                                    (*curwin).w_wcol %= (*curwin).w_view_width;
                                    (*curwin).w_wcol += win_col_off(curwin);
                                    col = 0 as ::core::ffi::c_int as colnr_T;
                                } else {
                                    (*curwin).w_wcol -= 1;
                                    col = ((*curwin).w_cursor.col as ::core::ffi::c_int
                                        - 1 as ::core::ffi::c_int)
                                        as colnr_T;
                                }
                            } else if (*curwin).w_onebuf_opt.wo_wrap != 0 && (*curwin).w_wrow != 0 {
                                (*curwin).w_wrow -= 1;
                                (*curwin).w_wcol = (*curwin).w_view_width - 1 as ::core::ffi::c_int;
                                col = ((*curwin).w_cursor.col as ::core::ffi::c_int
                                    - 1 as ::core::ffi::c_int)
                                    as colnr_T;
                            }
                            if col > 0 as ::core::ffi::c_int
                                && (*curwin).w_wcol > 0 as ::core::ffi::c_int
                            {
                                ptr = get_cursor_line_ptr();
                                col -= utf_head_off(ptr, ptr.offset(col as isize));
                                if utf_ptr2cells(ptr.offset(col as isize)) > 1 as ::core::ffi::c_int
                                {
                                    (*curwin).w_wcol -= 1;
                                }
                            }
                        }
                        setcursor();
                        ui_flush();
                        new_wcol = (*curwin).w_wcol;
                        new_wrow = (*curwin).w_wrow;
                        (*curwin).w_wcol = old_wcol;
                        (*curwin).w_wrow = old_wrow;
                    }
                    if c < 0 as ::core::ffi::c_int {
                        continue;
                    }
                    let mut n: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                    while n <= c {
                        *typebuf.tb_noremap.offset((typebuf.tb_off + n) as isize) =
                            RM_YES as ::core::ffi::c_int as uint8_t;
                        n += 1;
                    }
                    typebuf.tb_len += c;
                    if typebuf.tb_len >= typebuf.tb_maplen + MAXMAPLEN as ::core::ffi::c_int {
                        timedout = true_0 != 0;
                    } else if ex_normal_busy > 0 as ::core::ffi::c_int {
                        static mut tc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if typebuf.tb_len > 0 as ::core::ffi::c_int {
                            timedout = true_0 != 0;
                        } else {
                            c = if State & MODE_CMDLINE as ::core::ffi::c_int != 0
                                || cmdwin_type > 0 as ::core::ffi::c_int && tc == ESC
                            {
                                Ctrl_C
                            } else {
                                ESC
                            };
                            tc = c;
                            if advance {
                                typebuf_was_empty = true_0 != 0;
                            }
                            if pending_exmode_active {
                                exmode_active = true_0 != 0;
                            }
                            typebuf.tb_no_abbr_cnt = 0 as ::core::ffi::c_int;
                            break;
                        }
                    } else {
                        if (State & MODE_INSERT as ::core::ffi::c_int != 0 as ::core::ffi::c_int
                            || p_lz != 0)
                            && State & MODE_CMDLINE as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                            && advance as ::core::ffi::c_int != 0
                            && must_redraw != 0 as ::core::ffi::c_int
                            && !need_wait_return
                        {
                            update_screen();
                            setcursor();
                        }
                        let mut showcmd_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut showing_partial: bool = false_0 != 0;
                        if typebuf.tb_len > 0 as ::core::ffi::c_int
                            && advance as ::core::ffi::c_int != 0
                            && !exmode_active
                        {
                            if (State
                                & (MODE_NORMAL as ::core::ffi::c_int
                                    | MODE_INSERT as ::core::ffi::c_int)
                                != 0
                                || State == MODE_LANGMAP as ::core::ffi::c_int)
                                && State != MODE_HITRETURN as ::core::ffi::c_int
                            {
                                if State & MODE_INSERT as ::core::ffi::c_int != 0
                                    && ptr2cells(
                                        (typebuf.tb_buf as *mut ::core::ffi::c_char)
                                            .offset(typebuf.tb_off as isize)
                                            .offset(typebuf.tb_len as isize)
                                            .offset(-(1 as ::core::ffi::c_int as isize)),
                                    ) == 1 as ::core::ffi::c_int
                                {
                                    edit_putchar(
                                        *typebuf.tb_buf.offset(
                                            (typebuf.tb_off + typebuf.tb_len
                                                - 1 as ::core::ffi::c_int)
                                                as isize,
                                        )
                                            as ::core::ffi::c_int,
                                        false_0 != 0,
                                    );
                                    setcursor();
                                    showing_partial = true_0 != 0;
                                }
                                let mut old_wcol_0: ::core::ffi::c_int = (*curwin).w_wcol;
                                let mut old_wrow_0: ::core::ffi::c_int = (*curwin).w_wrow;
                                (*curwin).w_wcol = new_wcol;
                                (*curwin).w_wrow = new_wrow;
                                push_showcmd();
                                if typebuf.tb_len > SHOWCMD_COLS as ::core::ffi::c_int {
                                    showcmd_idx =
                                        typebuf.tb_len - SHOWCMD_COLS as ::core::ffi::c_int;
                                }
                                while showcmd_idx < typebuf.tb_len {
                                    let c2rust_fresh5 = showcmd_idx;
                                    showcmd_idx = showcmd_idx + 1;
                                    add_byte_to_showcmd(
                                        *typebuf
                                            .tb_buf
                                            .offset((typebuf.tb_off + c2rust_fresh5) as isize),
                                    );
                                }
                                (*curwin).w_wcol = old_wcol_0;
                                (*curwin).w_wrow = old_wrow_0;
                            }
                            if State & MODE_CMDLINE as ::core::ffi::c_int != 0
                                && !(*get_cmdline_info()).cmdbuff.is_null()
                                && cmdline_star == 0 as ::core::ffi::c_int
                            {
                                let mut p: *mut ::core::ffi::c_char = (typebuf.tb_buf
                                    as *mut ::core::ffi::c_char)
                                    .offset(typebuf.tb_off as isize)
                                    .offset(typebuf.tb_len as isize)
                                    .offset(-(1 as ::core::ffi::c_int as isize));
                                if ptr2cells(p) == 1 as ::core::ffi::c_int
                                    && (*p as uint8_t as ::core::ffi::c_int)
                                        < 128 as ::core::ffi::c_int
                                {
                                    putcmdline(*p, false_0 != 0);
                                    showing_partial = true_0 != 0;
                                }
                            }
                        }
                        if typebuf.tb_len == 0 as ::core::ffi::c_int {
                            timedout = false_0 != 0;
                        }
                        let mut wait_time: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if advance {
                            if typebuf.tb_len == 0 as ::core::ffi::c_int
                                || !(p_timeout != 0
                                    || p_ttimeout != 0
                                        && keylen == KEYLEN_PART_KEY as ::core::ffi::c_int)
                            {
                                wait_time = -1 as ::core::ffi::c_int;
                            } else if keylen == KEYLEN_PART_KEY as ::core::ffi::c_int
                                && p_ttm >= 0 as OptInt
                            {
                                wait_time = p_ttm as ::core::ffi::c_int;
                            } else {
                                wait_time = p_tm as ::core::ffi::c_int;
                            }
                        }
                        let mut wait_tb_len: ::core::ffi::c_int = typebuf.tb_len;
                        c = inchar(
                            typebuf
                                .tb_buf
                                .offset(typebuf.tb_off as isize)
                                .offset(typebuf.tb_len as isize),
                            typebuf.tb_buflen
                                - typebuf.tb_off
                                - typebuf.tb_len
                                - 1 as ::core::ffi::c_int,
                            wait_time as ::core::ffi::c_long,
                        );
                        if showcmd_idx != 0 as ::core::ffi::c_int {
                            pop_showcmd();
                        }
                        if showing_partial as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
                            if State & MODE_INSERT as ::core::ffi::c_int != 0 {
                                edit_unputchar();
                            }
                            if State & MODE_CMDLINE as ::core::ffi::c_int != 0
                                && !(*get_cmdline_info()).cmdbuff.is_null()
                            {
                                unputcmdline();
                            } else {
                                setcursor();
                            }
                        }
                        if c < 0 as ::core::ffi::c_int {
                            continue;
                        }
                        if c == NUL {
                            if !advance {
                                break;
                            }
                            if wait_tb_len <= 0 as ::core::ffi::c_int {
                                continue;
                            }
                            timedout = true_0 != 0;
                        } else {
                            while *typebuf
                                .tb_buf
                                .offset((typebuf.tb_off + typebuf.tb_len) as isize)
                                as ::core::ffi::c_int
                                != NUL
                            {
                                let c2rust_fresh6 = typebuf.tb_len;
                                typebuf.tb_len = typebuf.tb_len + 1;
                                *typebuf
                                    .tb_noremap
                                    .offset((typebuf.tb_off + c2rust_fresh6) as isize) =
                                    RM_YES as ::core::ffi::c_int as uint8_t;
                            }
                        }
                    }
                }
            }
        }
        if !(c < 0 as ::core::ffi::c_int || advance as ::core::ffi::c_int != 0 && c == NUL) {
            break;
        }
    }
    if advance as ::core::ffi::c_int != 0
        && p_smd != 0
        && msg_silent == 0 as ::core::ffi::c_int
        && State & MODE_INSERT as ::core::ffi::c_int != 0
    {
        if c == ESC && !mode_deleted && no_mapping == 0 && mode_displayed as ::core::ffi::c_int != 0
        {
            if typebuf.tb_len != 0 && !KeyTyped {
                redraw_cmdline = true_0 != 0;
            } else {
                unshowmode(false_0 != 0);
            }
        } else if c != ESC && mode_deleted as ::core::ffi::c_int != 0 {
            if typebuf.tb_len != 0 && !KeyTyped {
                redraw_cmdline = true_0 != 0;
            } else {
                showmode();
            }
        }
    }
    if timedout as ::core::ffi::c_int != 0 && c == ESC {
        gotchars_ignore();
    }
    vgetc_busy -= 1;
    return c;
}
unsafe extern "C" fn inchar(
    mut buf: *mut uint8_t,
    mut maxlen: ::core::ffi::c_int,
    mut wait_time: ::core::ffi::c_long,
) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut retesc: ::core::ffi::c_int = false_0;
    let tb_change_cnt: ::core::ffi::c_int = typebuf.tb_change_cnt;
    if wait_time == -1 as ::core::ffi::c_long || wait_time > 100 as ::core::ffi::c_long {
        ui_flush();
    }
    if State != MODE_HITRETURN as ::core::ffi::c_int {
        did_outofmem_msg = false_0 != 0;
        did_swapwrite_msg = false_0 != 0;
    }
    let mut read_size: ptrdiff_t = -1 as ptrdiff_t;
    while curscript >= 0 as ::core::ffi::c_int && read_size <= 0 as ptrdiff_t && !ignore_script {
        let mut script_char: ::core::ffi::c_char = 0;
        if got_int as ::core::ffi::c_int != 0 || {
            read_size = file_read(
                (&raw mut scriptin as *mut FileDescriptor).offset(curscript as isize),
                &raw mut script_char,
                1 as size_t,
            );
            read_size != 1 as ptrdiff_t
        } {
            closescript();
            if got_int {
                retesc = true_0;
            } else {
                return -1 as ::core::ffi::c_int;
            }
        } else {
            *buf.offset(0 as ::core::ffi::c_int as isize) = script_char as uint8_t;
            len = 1 as ::core::ffi::c_int;
        }
    }
    if read_size <= 0 as ptrdiff_t {
        if got_int {
            let mut dum: [uint8_t; 154] = [0; 154];
            loop {
                len = input_get(
                    &raw mut dum as *mut uint8_t,
                    MAXMAPLEN as ::core::ffi::c_int * 3 as ::core::ffi::c_int
                        + 3 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<MultiQueue>(),
                );
                if len == 0 as ::core::ffi::c_int
                    || len == 1 as ::core::ffi::c_int
                        && dum[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == Ctrl_C
                {
                    break;
                }
            }
            return retesc;
        }
        if wait_time == -1 as ::core::ffi::c_long || wait_time > 10 as ::core::ffi::c_long {
            ui_flush();
        }
        len = input_get(
            buf,
            maxlen / 3 as ::core::ffi::c_int,
            wait_time as ::core::ffi::c_int,
            tb_change_cnt,
            ::core::ptr::null_mut::<MultiQueue>(),
        );
    }
    if typebuf_changed(tb_change_cnt) {
        return 0 as ::core::ffi::c_int;
    }
    if len > 0 as ::core::ffi::c_int && {
        typebuf.tb_change_cnt += 1;
        typebuf.tb_change_cnt == 0 as ::core::ffi::c_int
    } {
        typebuf.tb_change_cnt = 1 as ::core::ffi::c_int;
    }
    return fix_input_buffer(buf, len);
}
#[no_mangle]
pub unsafe extern "C" fn fix_input_buffer(
    mut buf: *mut uint8_t,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if using_script() == 0 {
        *buf.offset(len as isize) = NUL as uint8_t;
        return len;
    }
    let mut p: *mut uint8_t = buf;
    let mut i: ::core::ffi::c_int = len;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == K_SPECIAL
                && (i < 2 as ::core::ffi::c_int
                    || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != KS_EXTRA)
        {
            memmove(
                p.offset(3 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                i as size_t,
            );
            *p.offset(2 as ::core::ffi::c_int as isize) =
                (if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == K_SPECIAL
                    || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                {
                    KE_FILLER as ::core::ffi::c_uint
                } else {
                    -(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                        as ::core::ffi::c_uint
                        >> 8 as ::core::ffi::c_int
                        & 0xff as ::core::ffi::c_uint
                }) as uint8_t;
            *p.offset(1 as ::core::ffi::c_int as isize) = (if *p
                .offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == K_SPECIAL
            {
                KS_SPECIAL
            } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                KS_ZERO
            } else {
                -(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    & 0xff as ::core::ffi::c_int
            }) as uint8_t;
            *p.offset(0 as ::core::ffi::c_int as isize) = K_SPECIAL as uint8_t;
            p = p.offset(2 as ::core::ffi::c_int as isize);
            len += 2 as ::core::ffi::c_int;
        }
        p = p.offset(1);
    }
    *p = NUL as uint8_t;
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn getcmdkeycmd(
    mut _promptc: ::core::ffi::c_int,
    mut _cookie: *mut ::core::ffi::c_void,
    mut _indent: ::core::ffi::c_int,
    mut _do_concat: bool,
) -> *mut ::core::ffi::c_char {
    let mut line_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut c1: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut cmod: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut aborted: bool = false_0 != 0;
    ga_init(
        &raw mut line_ga,
        1 as ::core::ffi::c_int,
        32 as ::core::ffi::c_int,
    );
    no_mapping += 1;
    got_int = false_0 != 0;
    while c1 != NUL && !aborted {
        ga_grow(&raw mut line_ga, 32 as ::core::ffi::c_int);
        if vgetorpeek(false_0 != 0) == NUL {
            emsg(gettext(
                &raw const e_cmd_mapping_must_end_with_cr as *const ::core::ffi::c_char,
            ));
            aborted = true_0 != 0;
            break;
        } else {
            c1 = vgetorpeek(true_0 != 0);
            if c1 == K_SPECIAL {
                c1 = vgetorpeek(true_0 != 0);
                let mut c2: ::core::ffi::c_int = vgetorpeek(true_0 != 0);
                if c1 == KS_MODIFIER {
                    cmod = c2;
                    continue;
                } else {
                    c1 = if c1 == KS_SPECIAL {
                        K_SPECIAL
                    } else if c1 == KS_ZERO {
                        K_ZERO
                    } else {
                        -(c1 + (c2 << 8 as ::core::ffi::c_int))
                    };
                }
            }
            if got_int {
                aborted = true_0 != 0;
            } else if c1 == '\r' as ::core::ffi::c_int || c1 == '\n' as ::core::ffi::c_int {
                c1 = NUL;
            } else if c1 == ESC {
                aborted = true_0 != 0;
            } else if c1
                == -(253 as ::core::ffi::c_int
                    + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                emsg(gettext(
                    &raw const e_cmd_mapping_must_end_with_cr_before_second_cmd
                        as *const ::core::ffi::c_char,
                ));
                aborted = true_0 != 0;
            } else if c1
                == -(253 as ::core::ffi::c_int
                    + ((KE_SNR as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                ga_concat_len(
                    &raw mut line_ga,
                    b"<SNR>\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                );
            } else {
                if cmod != 0 as ::core::ffi::c_int {
                    ga_append(&raw mut line_ga, K_SPECIAL as uint8_t);
                    ga_append(&raw mut line_ga, KS_MODIFIER as uint8_t);
                    ga_append(&raw mut line_ga, cmod as uint8_t);
                }
                if c1 < 0 as ::core::ffi::c_int {
                    ga_append(&raw mut line_ga, K_SPECIAL as uint8_t);
                    ga_append(
                        &raw mut line_ga,
                        (if c1 == K_SPECIAL {
                            KS_SPECIAL
                        } else if c1 == NUL {
                            KS_ZERO
                        } else {
                            -c1 & 0xff as ::core::ffi::c_int
                        }) as uint8_t,
                    );
                    ga_append(
                        &raw mut line_ga,
                        (if c1 == K_SPECIAL || c1 == NUL {
                            KE_FILLER as ::core::ffi::c_uint
                        } else {
                            -c1 as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int
                                & 0xff as ::core::ffi::c_uint
                        }) as uint8_t,
                    );
                } else {
                    ga_append(&raw mut line_ga, c1 as uint8_t);
                }
            }
            cmod = 0 as ::core::ffi::c_int;
        }
    }
    no_mapping -= 1;
    if aborted {
        ga_clear(&raw mut line_ga);
    }
    return line_ga.ga_data as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn map_execute_lua(mut may_repeat: bool, mut discard: bool) -> bool {
    let mut line_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut c1: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut aborted: bool = false_0 != 0;
    ga_init(
        &raw mut line_ga,
        1 as ::core::ffi::c_int,
        32 as ::core::ffi::c_int,
    );
    no_mapping += 1;
    got_int = false_0 != 0;
    while c1 != NUL && !aborted {
        ga_grow(&raw mut line_ga, 32 as ::core::ffi::c_int);
        c1 = vgetorpeek(true_0 != 0);
        if got_int {
            aborted = true_0 != 0;
        } else if c1 == '\r' as ::core::ffi::c_int || c1 == '\n' as ::core::ffi::c_int {
            c1 = NUL;
        } else {
            ga_append(&raw mut line_ga, c1 as uint8_t);
        }
    }
    no_mapping -= 1;
    if aborted as ::core::ffi::c_int != 0 || discard as ::core::ffi::c_int != 0 {
        ga_clear(&raw mut line_ga);
        return !aborted;
    }
    let mut ref_0: LuaRef = atoi(line_ga.ga_data as *const ::core::ffi::c_char);
    if may_repeat {
        repeat_luaref = ref_0;
    }
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut args: Array = ARRAY_DICT_INIT;
    nlua_call_ref(
        ref_0,
        ::core::ptr::null::<::core::ffi::c_char>(),
        args,
        kRetNilBool,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        semsg_multiline(
            b"emsg\0".as_ptr() as *const ::core::ffi::c_char,
            b"E5108: %s\0".as_ptr() as *const ::core::ffi::c_char,
            err.msg,
        );
        api_clear_error(&raw mut err);
    }
    ga_clear(&raw mut line_ga);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn paste_store(
    channel_id: uint64_t,
    state: TriState,
    str: String_0,
    crlf: bool,
) {
    if State & MODE_CMDLINE as ::core::ffi::c_int != 0 {
        return;
    }
    let need_redo: bool = !block_redo;
    let need_record: bool =
        reg_recording != 0 as ::core::ffi::c_int && !is_internal_call(channel_id);
    if !need_redo && !need_record {
        return;
    }
    if state as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
        let c: ::core::ffi::c_int = if state as ::core::ffi::c_int == kFalse as ::core::ffi::c_int {
            K_PASTE_START
        } else {
            K_PASTE_END
        };
        if need_redo {
            if state as ::core::ffi::c_int == kFalse as ::core::ffi::c_int
                && State & MODE_INSERT as ::core::ffi::c_int == 0
            {
                ResetRedobuff();
            }
            add_char_buff(&raw mut redobuff, c);
        }
        if need_record {
            add_char_buff(&raw mut recordbuff, c);
        }
        return;
    }
    let mut s: *const ::core::ffi::c_char = str.data;
    let str_end: *const ::core::ffi::c_char = str.data.offset(str.size as isize);
    while s < str_end {
        let mut start: *const ::core::ffi::c_char = s;
        while s < str_end
            && *s as uint8_t as ::core::ffi::c_int != K_SPECIAL
            && *s as ::core::ffi::c_int != NUL
            && *s as ::core::ffi::c_int != NL
            && !(crlf as ::core::ffi::c_int != 0 && *s as ::core::ffi::c_int == CAR)
        {
            s = s.offset(1);
        }
        if s > start {
            if need_redo {
                add_buff(&raw mut redobuff, start, s.offset_from(start));
            }
            if need_record {
                add_buff(&raw mut recordbuff, start, s.offset_from(start));
            }
        }
        if s < str_end {
            let c2rust_fresh17 = s;
            s = s.offset(1);
            let mut c_0: ::core::ffi::c_int = *c2rust_fresh17 as uint8_t as ::core::ffi::c_int;
            if crlf as ::core::ffi::c_int != 0 && c_0 == CAR {
                if s < str_end && *s as ::core::ffi::c_int == NL {
                    s = s.offset(1);
                }
                c_0 = NL;
            }
            if need_redo {
                add_byte_buff(&raw mut redobuff, c_0);
            }
            if need_record {
                add_byte_buff(&raw mut recordbuff, c_0);
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn paste_repeat(mut count: ::core::ffi::c_int) {
    let mut ga: garray_T = garray_T {
        ga_len: 0 as ::core::ffi::c_int,
        ga_maxlen: 0 as ::core::ffi::c_int,
        ga_itemsize: 1 as ::core::ffi::c_int,
        ga_growsize: 32 as ::core::ffi::c_int,
        ga_data: NULL_0,
    };
    let mut aborted: bool = false_0 != 0;
    no_mapping += 1;
    got_int = false_0 != 0;
    while !aborted {
        ga_grow(&raw mut ga, 32 as ::core::ffi::c_int);
        let mut c1: uint8_t = vgetorpeek(true_0 != 0) as uint8_t;
        if c1 as ::core::ffi::c_int == K_SPECIAL {
            c1 = vgetorpeek(true_0 != 0) as uint8_t;
            let mut c2: uint8_t = vgetorpeek(true_0 != 0) as uint8_t;
            let mut c: ::core::ffi::c_int = if c1 as ::core::ffi::c_int == KS_SPECIAL {
                K_SPECIAL
            } else if c1 as ::core::ffi::c_int == KS_ZERO {
                K_ZERO
            } else {
                -(c1 as ::core::ffi::c_int
                    + ((c2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            };
            if c == K_PASTE_END {
                break;
            }
            if c == K_ZERO {
                ga_append(&raw mut ga, NUL as uint8_t);
            } else if c == K_SPECIAL {
                ga_append(&raw mut ga, K_SPECIAL as uint8_t);
            } else {
                ga_append(&raw mut ga, K_SPECIAL as uint8_t);
                ga_append(&raw mut ga, c1);
                ga_append(&raw mut ga, c2);
            }
        } else {
            ga_append(&raw mut ga, c1);
        }
        aborted = got_int;
    }
    no_mapping -= 1;
    let mut str: String_0 = String_0 {
        data: ga.ga_data as *mut ::core::ffi::c_char,
        size: ga.ga_len as size_t,
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !aborted && i < count {
        nvim_paste(
            LUA_INTERNAL_CALL,
            str,
            false_0 != 0,
            -1 as Integer,
            &raw mut arena,
            &raw mut err,
        );
        aborted = err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int;
        i += 1;
    }
    api_clear_error(&raw mut err);
    arena_mem_free(arena_finish(&raw mut arena));
    ga_clear(&raw mut ga);
}
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const KS_ZERO: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
pub const K_SELECT_STRING: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"\x80\xF5X\0") };
pub const KE_FILLER: ::core::ffi::c_int = 'X' as ::core::ffi::c_int;
pub const K_ZERO: ::core::ffi::c_int =
    -(255 as ::core::ffi::c_int + (('X' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_UP: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('u' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KUP: ::core::ffi::c_int = -30027;
pub const K_DOWN: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('d' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KDOWN: ::core::ffi::c_int = -25675;
pub const K_LEFT: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('l' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KLEFT: ::core::ffi::c_int = -27723;
pub const K_RIGHT: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('r' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KRIGHT: ::core::ffi::c_int = -29259;
pub const K_S_HOME: ::core::ffi::c_int =
    -('#' as ::core::ffi::c_int + (('2' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_S_END: ::core::ffi::c_int =
    -('*' as ::core::ffi::c_int + (('7' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_XUP: ::core::ffi::c_int = -16893;
pub const K_XDOWN: ::core::ffi::c_int = -17149;
pub const K_XLEFT: ::core::ffi::c_int = -17405;
pub const K_XRIGHT: ::core::ffi::c_int = -17661;
pub const K_HOME: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('h' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_XHOME: ::core::ffi::c_int = -16381;
pub const K_ZHOME: ::core::ffi::c_int = -16637;
pub const K_END: ::core::ffi::c_int =
    -('@' as ::core::ffi::c_int + (('7' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_XEND: ::core::ffi::c_int = -15869;
pub const K_ZEND: ::core::ffi::c_int = -16125;
pub const K_KPLUS: ::core::ffi::c_int = -13899;
pub const K_KMINUS: ::core::ffi::c_int = -14155;
pub const K_KDIVIDE: ::core::ffi::c_int = -14411;
pub const K_KMULTIPLY: ::core::ffi::c_int = -14667;
pub const K_KENTER: ::core::ffi::c_int = -16715;
pub const K_KPOINT: ::core::ffi::c_int = -16971;
pub const K_PASTE_START: ::core::ffi::c_int =
    -('P' as ::core::ffi::c_int + (('S' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_PASTE_END: ::core::ffi::c_int =
    -('P' as ::core::ffi::c_int + (('E' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_K0: ::core::ffi::c_int = -17227;
pub const K_K1: ::core::ffi::c_int = -17483;
pub const K_K2: ::core::ffi::c_int = -17739;
pub const K_K3: ::core::ffi::c_int = -17995;
pub const K_K4: ::core::ffi::c_int = -18251;
pub const K_K5: ::core::ffi::c_int = -18507;
pub const K_K6: ::core::ffi::c_int = -18763;
pub const K_K7: ::core::ffi::c_int = -19019;
pub const K_K8: ::core::ffi::c_int = -19275;
pub const K_K9: ::core::ffi::c_int = -19531;
pub const K_KCOMMA: ::core::ffi::c_int = -19787;
pub const K_KEQUAL: ::core::ffi::c_int = -20043;
pub const K_VER_SCROLLBAR: ::core::ffi::c_int =
    -(249 as ::core::ffi::c_int + (('X' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_HOR_SCROLLBAR: ::core::ffi::c_int =
    -(248 as ::core::ffi::c_int + (('X' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const MOD_MASK_SHIFT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOD_MASK_CTRL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MOD_MASK_ALT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn utf_ptr2CharInfo(p_in: *const ::core::ffi::c_char) -> CharInfo {
    let p: *const uint8_t = p_in as *const uint8_t;
    let first: uint8_t = *p;
    if (first as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        return CharInfo {
            value: first as int32_t,
            len: 1 as ::core::ffi::c_int,
        };
    } else {
        let mut len: ::core::ffi::c_int = utf8len_tab[first as usize] as ::core::ffi::c_int;
        let code_point: int32_t = utf_ptr2CharInfo_impl(p, len as uintptr_t);
        if code_point < 0 as int32_t {
            len = 1 as ::core::ffi::c_int;
        }
        return CharInfo {
            value: code_point,
            len: len,
        };
    };
}
#[inline(always)]
unsafe extern "C" fn utfc_next(mut cur: StrCharInfo) -> StrCharInfo {
    let mut next: *mut uint8_t = cur.ptr.offset(cur.chr.len as isize) as *mut uint8_t;
    if ((*next as ::core::ffi::c_uint) < 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
        as ::core::ffi::c_long
        != 0
    {
        return StrCharInfo {
            ptr: next as *mut ::core::ffi::c_char,
            chr: CharInfo {
                value: *next as int32_t,
                len: 1 as ::core::ffi::c_int,
            },
        };
    }
    return utfc_next_impl(cur);
}
#[inline(always)]
unsafe extern "C" fn utf_ptr2StrCharInfo(mut ptr: *mut ::core::ffi::c_char) -> StrCharInfo {
    return StrCharInfo {
        ptr: ptr,
        chr: utf_ptr2CharInfo(ptr),
    };
}
#[inline(always)]
unsafe extern "C" fn win_charsize(
    mut cstype: CSType,
    mut vcol: ::core::ffi::c_int,
    mut ptr: *mut ::core::ffi::c_char,
    mut chr: int32_t,
    mut csarg: *mut CharsizeArg,
) -> CharSize {
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        return charsize_fast(csarg, ptr, vcol as colnr_T, chr);
    } else {
        return charsize_regular(csarg, ptr, vcol as colnr_T, chr);
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
unsafe extern "C" fn c2rust_run_static_initializers() {
    on_key_buf = C2Rust_Unnamed_31 {
        size: 0 as size_t,
        capacity: ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as size_t,
            ),
        items: &raw mut on_key_buf.init_array as *mut ::core::ffi::c_char,
        init_array: [0; 51],
    };
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [c2rust_run_static_initializers];
