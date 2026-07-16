extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type terminal;
    pub type regprog;
    pub type undo_object;
    pub type qf_info_S;
    pub type multiqueue;
    pub type Unpacker;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn fcntl(__fd: ::core::ffi::c_int, __cmd: ::core::ffi::c_int, ...) -> ::core::ffi::c_int;
    static mut stderr: *mut FILE;
    fn freopen(
        __filename: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
        __stream: *mut FILE,
    ) -> *mut FILE;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn abort() -> !;
    fn qsort(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xmemdup(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn arena_finish(arena: *mut Arena) -> ArenaMem;
    fn arena_alloc(arena: *mut Arena, size: size_t, align: bool) -> *mut ::core::ffi::c_void;
    fn arena_mem_free(mem: ArenaMem);
    fn dup2(__fd: ::core::ffi::c_int, __fd2: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn object_to_vim(obj: Object, tv: *mut typval_T, err: *mut Error);
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
    fn map_del_uint64_t_ptr_t(
        map: *mut Map_uint64_t_ptr_t,
        key: uint64_t,
        key_alloc: *mut uint64_t,
    ) -> ptr_t;
    fn map_put_ref_uint64_t_ptr_t(
        map: *mut Map_uint64_t_ptr_t,
        key: uint64_t,
        key_alloc: *mut *mut uint64_t,
        new_item: *mut bool,
    ) -> *mut ptr_t;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn arena_array(arena: *mut Arena, max_size: size_t) -> Array;
    fn arena_dict(arena: *mut Arena, max_size: size_t) -> Dict;
    fn arena_string(arena: *mut Arena, str: String_0) -> String_0;
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn has_event(event: event_T) -> bool;
    fn libuv_proc_init(loop_0: *mut Loop, data: *mut ::core::ffi::c_void) -> LibuvProc;
    static mut channels: Map_uint64_t_ptr_t;
    fn pty_proc_tty_name(ptyproc: *mut PtyProc) -> *const ::core::ffi::c_char;
    fn pty_proc_resize(ptyproc: *mut PtyProc, width: uint16_t, height: uint16_t);
    fn pty_proc_resume(ptyproc: *mut PtyProc);
    fn pty_proc_close_master(ptyproc: *mut PtyProc);
    fn pty_proc_init(loop_0: *mut Loop, data: *mut ::core::ffi::c_void) -> PtyProc;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static e_invarg2: [::core::ffi::c_char; 0];
    static e_invchan: [::core::ffi::c_char; 0];
    static e_jobspawn: [::core::ffi::c_char; 0];
    static e_invstream: [::core::ffi::c_char; 0];
    static e_invstreamrpc: [::core::ffi::c_char; 0];
    static e_streamkey: [::core::ffi::c_char; 0];
    fn get_v_event(sve: *mut save_v_event_T) -> *mut dict_T;
    fn restore_v_event(v_event: *mut dict_T, sve: *mut save_v_event_T);
    fn callback_call(
        callback: *mut Callback,
        argcount_in: ::core::ffi::c_int,
        argvars_in: *mut typval_T,
        rettv: *mut typval_T,
    ) -> bool;
    fn eval_fmt_source_name_line(buf: *mut ::core::ffi::c_char, bufsize: size_t);
    fn encode_list_write(
        data: *mut ::core::ffi::c_void,
        buf: *const ::core::ffi::c_char,
        len: size_t,
    );
    fn encode_tv2json(tv: *mut typval_T, len: *mut size_t) -> *mut ::core::ffi::c_char;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_unref(l: *mut list_T);
    fn tv_list_append_string(l: *mut list_T, str: *const ::core::ffi::c_char, len: ssize_t);
    fn callback_free(callback: *mut Callback);
    fn tv_dict_free(d: *mut dict_T);
    fn tv_dict_find(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        len: ptrdiff_t,
    ) -> *mut dictitem_T;
    fn tv_dict_add_list(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        list: *mut list_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_add_dict(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        dict: *mut dict_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_set_keys_readonly(dict: *mut dict_T);
    fn tv_clear(tv: *mut typval_T);
    fn multiqueue_new_child(parent: *mut MultiQueue) -> *mut MultiQueue;
    fn multiqueue_free(self_0: *mut MultiQueue);
    fn multiqueue_put_event(self_0: *mut MultiQueue, event: Event);
    fn rstream_init_fd(loop_0: *mut Loop, stream: *mut RStream, fd: ::core::ffi::c_int);
    fn rstream_init(stream: *mut RStream);
    fn rstream_start_inner(stream: *mut RStream);
    fn rstream_start(stream: *mut RStream, cb: stream_read_cb, data: *mut ::core::ffi::c_void);
    fn rstream_stop_inner(stream: *mut RStream);
    fn rstream_may_close(stream: *mut RStream);
    fn proc_spawn(proc: *mut Proc, in_0: bool, out: bool, err: bool) -> ::core::ffi::c_int;
    fn proc_stop(proc: *mut Proc);
    fn proc_free(proc: *mut Proc);
    fn exit_on_closed_chan(status: ::core::ffi::c_int);
    fn socket_watcher_accept(
        watcher: *mut SocketWatcher,
        stream: *mut RStream,
    ) -> ::core::ffi::c_int;
    fn socket_connect(
        loop_0: *mut Loop,
        stream: *mut RStream,
        is_tcp: bool,
        address: *const ::core::ffi::c_char,
        timeout: ::core::ffi::c_int,
        error: *mut *const ::core::ffi::c_char,
    ) -> bool;
    fn wstream_init_fd(
        loop_0: *mut Loop,
        stream: *mut Stream,
        fd: ::core::ffi::c_int,
        maxmem: size_t,
    );
    fn wstream_init(stream: *mut Stream, maxmem: size_t);
    fn wstream_write(stream: *mut Stream, buffer: *mut WBuffer) -> ::core::ffi::c_int;
    fn wstream_new_buffer(
        data: *mut ::core::ffi::c_char,
        size: size_t,
        refcount: size_t,
        cb: wbuffer_data_finalizer,
    ) -> *mut WBuffer;
    fn stream_may_close(stream: *mut Stream);
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_concat_len(gap: *mut garray_T, s: *const ::core::ffi::c_char, len: size_t);
    static mut curbuf: *mut buf_T;
    static mut exiting: bool;
    static mut IObuff: [::core::ffi::c_char; 1025];
    static mut embedded_mode: bool;
    static mut headless_mode: bool;
    fn uv_strerror(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn api_free_luaref(ref_0: LuaRef);
    static mut main_loop: Loop;
    fn rpc_init();
    fn rpc_start(channel: *mut Channel);
    fn rpc_close(channel: *mut Channel);
    fn rpc_free(channel: *mut Channel);
    fn server_owns_pipe_address(address: *const ::core::ffi::c_char) -> bool;
    fn os_write(
        fd: ::core::ffi::c_int,
        buf: *const ::core::ffi::c_char,
        size: size_t,
        non_blocking: bool,
    ) -> ptrdiff_t;
    fn shell_free_argv(argv: *mut *mut ::core::ffi::c_char);
    static mut ui_client_channel_id: uint64_t;
    fn terminal_alloc(buf: *mut buf_T, opts: TerminalOptions) -> *mut Terminal;
    fn terminal_close(termpp: *mut *mut Terminal, status: ::core::ffi::c_int);
    fn terminal_set_state(term: *mut Terminal, suspended: bool);
    fn terminal_destroy(termpp: *mut *mut Terminal);
    fn terminal_receive(term: *mut Terminal, data: *const ::core::ffi::c_char, len: size_t);
    fn terminal_buf(term: *const Terminal) -> Buffer;
    fn ui_client_attach_to_restarted_server();
}
pub type __uid_t = ::core::ffi::c_uint;
pub type __gid_t = ::core::ffi::c_uint;
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type __time_t = ::core::ffi::c_long;
pub type __socklen_t = ::core::ffi::c_uint;
pub type size_t = usize;
pub type time_t = __time_t;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type intptr_t = isize;
pub type ptrdiff_t = isize;
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
pub type gid_t = __gid_t;
pub type uid_t = __uid_t;
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
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
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
pub type socklen_t = __socklen_t;
pub type sa_family_t = ::core::ffi::c_ushort;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr {
    pub sa_family: sa_family_t,
    pub sa_data: [::core::ffi::c_char; 14],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct addrinfo {
    pub ai_flags: ::core::ffi::c_int,
    pub ai_family: ::core::ffi::c_int,
    pub ai_socktype: ::core::ffi::c_int,
    pub ai_protocol: ::core::ffi::c_int,
    pub ai_addrlen: socklen_t,
    pub ai_addr: *mut sockaddr,
    pub ai_canonname: *mut ::core::ffi::c_char,
    pub ai_next: *mut addrinfo,
}
pub type schar_T = uint32_t;
pub type sattr_T = int32_t;
pub type handle_T = ::core::ffi::c_int;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
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
pub type Buffer = handle_T;
pub type ListLenSpecials = ::core::ffi::c_int;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictitem_T {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 0],
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
    pub u: C2Rust_Unnamed_27,
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
pub union C2Rust_Unnamed_27 {
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
    pub data: C2Rust_Unnamed_28,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_28 {
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
pub type argv_callback = Option<unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Event {
    pub handler: argv_callback,
    pub argv: [*mut ::core::ffi::c_void; 10],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wbuffer {
    pub size: size_t,
    pub refcount: size_t,
    pub data: *mut ::core::ffi::c_char,
    pub cb: wbuffer_data_finalizer,
}
pub type wbuffer_data_finalizer = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub type WBuffer = wbuffer;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct socket_watcher {
    pub addr: [::core::ffi::c_char; 256],
    pub uv: C2Rust_Unnamed_29,
    pub stream: *mut uv_stream_t,
    pub data: *mut ::core::ffi::c_void,
    pub cb: socket_cb,
    pub close_cb: socket_close_cb,
    pub events: *mut MultiQueue,
}
pub type socket_close_cb =
    Option<unsafe extern "C" fn(*mut SocketWatcher, *mut ::core::ffi::c_void) -> ()>;
pub type SocketWatcher = socket_watcher;
pub type socket_cb = Option<
    unsafe extern "C" fn(*mut SocketWatcher, ::core::ffi::c_int, *mut ::core::ffi::c_void) -> (),
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_29 {
    pub tcp: C2Rust_Unnamed_31,
    pub pipe: C2Rust_Unnamed_30,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_30 {
    pub handle: uv_pipe_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_31 {
    pub handle: uv_tcp_t,
    pub addrinfo: *mut addrinfo,
}
pub type ChannelStreamType = ::core::ffi::c_uint;
pub const kChannelStreamInternal: ChannelStreamType = 4;
pub const kChannelStreamStderr: ChannelStreamType = 3;
pub const kChannelStreamStdio: ChannelStreamType = 2;
pub const kChannelStreamSocket: ChannelStreamType = 1;
pub const kChannelStreamProc: ChannelStreamType = 0;
pub type ChannelPart = ::core::ffi::c_uint;
pub const kChannelPartAll: ChannelPart = 4;
pub const kChannelPartRpc: ChannelPart = 3;
pub const kChannelPartStderr: ChannelPart = 2;
pub const kChannelPartStdout: ChannelPart = 1;
pub const kChannelPartStdin: ChannelPart = 0;
pub type ChannelStdinMode = ::core::ffi::c_uint;
pub const kChannelStdinNull: ChannelStdinMode = 1;
pub const kChannelStdinPipe: ChannelStdinMode = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StdioPair {
    pub in_0: RStream,
    pub out: Stream,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StderrState {
    pub closed: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct InternalState {
    pub cb: LuaRef,
    pub closed: bool,
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
pub struct LibuvProc {
    pub proc: Proc,
    pub uv: uv_process_t,
    pub uvopts: uv_process_options_t,
    pub uvstdio: [uv_stdio_container_t; 4],
}
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
pub type PackerBuffer = packer_buffer_t;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Channel {
    pub id: uint64_t,
    pub refcount: size_t,
    pub events: *mut MultiQueue,
    pub streamtype: ChannelStreamType,
    pub stream: C2Rust_Unnamed_33,
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
pub struct RpcState {
    pub closed: bool,
    pub unpacker: *mut Unpacker,
    pub ui: *mut RemoteUI,
    pub next_request_id: uint32_t,
    pub call_stack: C2Rust_Unnamed_32,
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
pub struct C2Rust_Unnamed_32 {
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
pub union C2Rust_Unnamed_33 {
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
pub struct save_v_event_T {
    pub sve_did_save: bool,
    pub sve_hashtab: hashtab_T,
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_34 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut int64_t,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const F_DUPFD_CLOEXEC: ::core::ffi::c_int = 1030 as ::core::ffi::c_int;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const LOGLVL_INF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static mut value_init_ptr_t: ptr_t = NULL;
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_put_uint64_t_ptr_t(
    mut map: *mut Map_uint64_t_ptr_t,
    mut key: uint64_t,
    mut value: ptr_t,
) {
    let mut val: *mut ptr_t = map_put_ref_uint64_t_ptr_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut uint64_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
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
pub const CHAN_STDIO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CHAN_STDERR: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn callback_reader_set(mut reader: CallbackReader) -> bool {
    return reader.cb.type_0 as ::core::ffi::c_uint
        != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
        || !reader.self_0.is_null();
}
#[inline]
unsafe extern "C" fn find_channel(mut id: uint64_t) -> *mut Channel {
    return map_get_uint64_t_ptr_t(&raw mut channels, id) as *mut Channel;
}
#[inline]
unsafe extern "C" fn channel_instream(mut chan: *mut Channel) -> *mut Stream {
    match (*chan).streamtype as ::core::ffi::c_uint {
        0 => return &raw mut (*chan).stream.proc.in_0,
        1 => return &raw mut (*chan).stream.socket.s,
        2 => return &raw mut (*chan).stream.stdio.out,
        4 | 3 => {
            abort();
        }
        _ => {}
    }
    abort();
}
static mut did_stdio: bool = false_0 != 0;
static mut next_chan_id: uint64_t = (CHAN_STDERR + 1 as ::core::ffi::c_int) as uint64_t;
#[no_mangle]
pub unsafe extern "C" fn channel_teardown() {
    let mut chan: *mut Channel = ::core::ptr::null_mut::<Channel>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < channels.set.h.n_keys {
        chan = *channels.values.offset(__i as isize) as *mut Channel;
        channel_close(
            (*chan).id,
            kChannelPartAll,
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
        );
        __i = __i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn channel_close(
    mut id: uint64_t,
    mut part: ChannelPart,
    mut error: *mut *const ::core::ffi::c_char,
) -> bool {
    let mut chan: *mut Channel = ::core::ptr::null_mut::<Channel>();
    let mut proc: *mut Proc = ::core::ptr::null_mut::<Proc>();
    let mut dummy: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if error.is_null() {
        error = &raw mut dummy;
    }
    chan = find_channel(id);
    if chan.is_null() {
        if id < next_chan_id {
            return true_0 != 0;
        }
        *error = &raw const e_invchan as *const ::core::ffi::c_char;
        return false_0 != 0;
    }
    let mut close_main: bool = false_0 != 0;
    if part as ::core::ffi::c_uint == kChannelPartRpc as ::core::ffi::c_int as ::core::ffi::c_uint
        || part as ::core::ffi::c_uint
            == kChannelPartAll as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        close_main = true_0 != 0;
        if (*chan).is_rpc {
            rpc_close(chan);
        } else if part as ::core::ffi::c_uint
            == kChannelPartRpc as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            *error = &raw const e_invstream as *const ::core::ffi::c_char;
            return false_0 != 0;
        }
    } else if (part as ::core::ffi::c_uint
        == kChannelPartStdin as ::core::ffi::c_int as ::core::ffi::c_uint
        || part as ::core::ffi::c_uint
            == kChannelPartStdout as ::core::ffi::c_int as ::core::ffi::c_uint)
        && (*chan).is_rpc as ::core::ffi::c_int != 0
    {
        *error = &raw const e_invstreamrpc as *const ::core::ffi::c_char;
        return false_0 != 0;
    }
    match (*chan).streamtype as ::core::ffi::c_uint {
        1 => {
            if !close_main {
                *error = &raw const e_invstream as *const ::core::ffi::c_char;
                return false_0 != 0;
            }
            rstream_may_close(&raw mut (*chan).stream.socket);
        }
        0 => {
            proc = &raw mut (*chan).stream.proc;
            if part as ::core::ffi::c_uint
                == kChannelPartStdin as ::core::ffi::c_int as ::core::ffi::c_uint
                || close_main as ::core::ffi::c_int != 0
            {
                stream_may_close(&raw mut (*proc).in_0);
            }
            if part as ::core::ffi::c_uint
                == kChannelPartStdout as ::core::ffi::c_int as ::core::ffi::c_uint
                || close_main as ::core::ffi::c_int != 0
            {
                rstream_may_close(&raw mut (*proc).out);
            }
            if part as ::core::ffi::c_uint
                == kChannelPartStderr as ::core::ffi::c_int as ::core::ffi::c_uint
                || part as ::core::ffi::c_uint
                    == kChannelPartAll as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                rstream_may_close(&raw mut (*proc).err);
            }
            if (*proc).type_0 as ::core::ffi::c_uint
                == kProcTypePty as ::core::ffi::c_int as ::core::ffi::c_uint
                && part as ::core::ffi::c_uint
                    == kChannelPartAll as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                pty_proc_close_master(&raw mut (*chan).stream.pty);
            }
        }
        2 => {
            if part as ::core::ffi::c_uint
                == kChannelPartStdin as ::core::ffi::c_int as ::core::ffi::c_uint
                || close_main as ::core::ffi::c_int != 0
            {
                rstream_may_close(&raw mut (*chan).stream.stdio.in_0);
            }
            if part as ::core::ffi::c_uint
                == kChannelPartStdout as ::core::ffi::c_int as ::core::ffi::c_uint
                || close_main as ::core::ffi::c_int != 0
            {
                stream_may_close(&raw mut (*chan).stream.stdio.out);
            }
            if part as ::core::ffi::c_uint
                == kChannelPartStderr as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                *error = &raw const e_invstream as *const ::core::ffi::c_char;
                return false_0 != 0;
            }
        }
        3 => {
            if part as ::core::ffi::c_uint
                != kChannelPartAll as ::core::ffi::c_int as ::core::ffi::c_uint
                && part as ::core::ffi::c_uint
                    != kChannelPartStderr as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                *error = &raw const e_invstream as *const ::core::ffi::c_char;
                return false_0 != 0;
            }
            if !(*chan).stream.err.closed {
                (*chan).stream.err.closed = true_0 != 0;
                if !exiting {
                    freopen(
                        b"/dev/null\0".as_ptr() as *const ::core::ffi::c_char,
                        b"w\0".as_ptr() as *const ::core::ffi::c_char,
                        stderr,
                    );
                }
                channel_decref(chan);
            }
        }
        4 => {
            if !close_main {
                *error = &raw const e_invstream as *const ::core::ffi::c_char;
                return false_0 != 0;
            }
            if !(*chan).term.is_null() {
                api_free_luaref((*chan).stream.internal.cb);
                (*chan).stream.internal.cb = LUA_NOREF as LuaRef;
                (*chan).stream.internal.closed = true_0 != 0;
                terminal_close(&raw mut (*chan).term, 0 as ::core::ffi::c_int);
                (*chan).exit_status = 0 as ::core::ffi::c_int;
            } else {
                channel_decref(chan);
            }
        }
        _ => {}
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn channel_init() {
    channel_alloc(kChannelStreamStderr);
    rpc_init();
}
#[no_mangle]
pub unsafe extern "C" fn channel_alloc(mut type_0: ChannelStreamType) -> *mut Channel {
    let mut chan: *mut Channel =
        xcalloc(1 as size_t, ::core::mem::size_of::<Channel>()) as *mut Channel;
    if type_0 as ::core::ffi::c_uint
        == kChannelStreamStdio as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*chan).id = CHAN_STDIO as uint64_t;
    } else if type_0 as ::core::ffi::c_uint
        == kChannelStreamStderr as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*chan).id = CHAN_STDERR as uint64_t;
    } else {
        let c2rust_fresh0 = next_chan_id;
        next_chan_id = next_chan_id.wrapping_add(1);
        (*chan).id = c2rust_fresh0;
    }
    (*chan).events = multiqueue_new_child(main_loop.events);
    (*chan).refcount = 1 as size_t;
    (*chan).exit_status = -1 as ::core::ffi::c_int;
    (*chan).streamtype = type_0;
    (*chan).detach = false_0 != 0;
    '_c2rust_label: {
        if (*chan).id <= 9223372036854775807 as uint64_t {
        } else {
            __assert_fail(
                b"chan->id <= VARNUMBER_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/channel.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                230 as ::core::ffi::c_uint,
                b"Channel *channel_alloc(ChannelStreamType)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    map_put_uint64_t_ptr_t(&raw mut channels, (*chan).id, chan as ptr_t);
    return chan;
}
#[no_mangle]
pub unsafe extern "C" fn channel_create_event(
    mut chan: *mut Channel,
    mut ext_source: *const ::core::ffi::c_char,
) {
    let mut source: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !ext_source.is_null() {
        source = ext_source;
    } else {
        eval_fmt_source_name_line(
            &raw mut IObuff as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
        );
        source = &raw mut IObuff as *mut ::core::ffi::c_char;
    }
    '_c2rust_label: {
        if (*chan).id <= 9223372036854775807 as uint64_t {
        } else {
            __assert_fail(
                b"chan->id <= VARNUMBER_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/channel.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                249 as ::core::ffi::c_uint,
                b"void channel_create_event(Channel *, const char *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut info: Dict = channel_info((*chan).id, &raw mut arena);
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    object_to_vim(
        object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: info },
        },
        &raw mut tv,
        ::core::ptr::null_mut::<Error>(),
    );
    '_c2rust_label_0: {
        if tv.v_type as ::core::ffi::c_uint == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"tv.v_type == VAR_DICT\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/channel.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                256 as ::core::ffi::c_uint,
                b"void channel_create_event(Channel *, const char *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut str: *mut ::core::ffi::c_char =
        encode_tv2json(&raw mut tv, ::core::ptr::null_mut::<size_t>());
    logmsg(
        LOGLVL_INF,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"channel_create_event\0".as_ptr() as *const ::core::ffi::c_char,
        258 as ::core::ffi::c_int,
        true_0 != 0,
        b"new channel %lu (%s) : %s\0".as_ptr() as *const ::core::ffi::c_char,
        (*chan).id,
        source,
        str,
    );
    xfree(str as *mut ::core::ffi::c_void);
    arena_mem_free(arena_finish(&raw mut arena));
    channel_info_changed(chan, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn channel_incref(mut chan: *mut Channel) {
    (*chan).refcount = (*chan).refcount.wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn channel_decref(mut chan: *mut Channel) {
    (*chan).refcount = (*chan).refcount.wrapping_sub(1);
    if (*chan).refcount == 0 {
        multiqueue_put_event(
            main_loop.events,
            Event {
                handler: Some(
                    free_channel_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    chan as *mut ::core::ffi::c_void,
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ],
            },
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn callback_reader_free(mut reader: *mut CallbackReader) {
    callback_free(&raw mut (*reader).cb);
    ga_clear(&raw mut (*reader).buffer);
}
#[no_mangle]
pub unsafe extern "C" fn callback_reader_start(
    mut reader: *mut CallbackReader,
    mut type_0: *const ::core::ffi::c_char,
) {
    ga_init(
        &raw mut (*reader).buffer,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        32 as ::core::ffi::c_int,
    );
    (*reader).type_0 = type_0;
}
unsafe extern "C" fn channel_destroy(mut chan: *mut Channel) {
    if (*chan).is_rpc {
        rpc_free(chan);
    }
    if (*chan).streamtype as ::core::ffi::c_uint
        == kChannelStreamProc as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        proc_free(&raw mut (*chan).stream.proc);
    }
    callback_reader_free(&raw mut (*chan).on_data);
    callback_reader_free(&raw mut (*chan).on_stderr);
    callback_free(&raw mut (*chan).on_exit);
    multiqueue_free((*chan).events);
    xfree(chan as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn free_channel_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut chan: *mut Channel = *argv.offset(0 as ::core::ffi::c_int as isize) as *mut Channel;
    map_del_uint64_t_ptr_t(
        &raw mut channels,
        (*chan).id,
        ::core::ptr::null_mut::<uint64_t>(),
    );
    channel_destroy(chan);
}
unsafe extern "C" fn channel_destroy_early(mut chan: *mut Channel) {
    next_chan_id = next_chan_id.wrapping_sub(1);
    if (*chan).id != next_chan_id {
        abort();
    }
    map_del_uint64_t_ptr_t(
        &raw mut channels,
        (*chan).id,
        ::core::ptr::null_mut::<uint64_t>(),
    );
    (*chan).id = 0 as uint64_t;
    (*chan).refcount = (*chan).refcount.wrapping_sub(1);
    if (*chan).refcount != 0 as size_t {
        abort();
    }
    multiqueue_put_event(
        main_loop.events,
        Event {
            handler: Some(
                free_channel_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                chan as *mut ::core::ffi::c_void,
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ],
        },
    );
}
unsafe extern "C" fn close_cb(mut stream: *mut Stream, mut data: *mut ::core::ffi::c_void) {
    channel_decref(data as *mut Channel);
}
#[no_mangle]
pub unsafe extern "C" fn channel_job_start(
    mut argv: *mut *mut ::core::ffi::c_char,
    mut exepath: *const ::core::ffi::c_char,
    mut on_stdout: CallbackReader,
    mut on_stderr: CallbackReader,
    mut on_exit: Callback,
    mut pty: bool,
    mut rpc: bool,
    mut overlapped: bool,
    mut detach: bool,
    mut stdin_mode: ChannelStdinMode,
    mut cwd: *const ::core::ffi::c_char,
    mut pty_width: uint16_t,
    mut pty_height: uint16_t,
    mut env: *mut dict_T,
    mut status_out: *mut varnumber_T,
) -> *mut Channel {
    let mut chan: *mut Channel = channel_alloc(kChannelStreamProc);
    (*chan).on_data = on_stdout;
    (*chan).on_stderr = on_stderr;
    (*chan).on_exit = on_exit;
    if pty {
        if detach {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                b"terminal/pty job cannot be detached\0".as_ptr() as *const ::core::ffi::c_char,
            );
            shell_free_argv(argv);
            if !env.is_null() {
                tv_dict_free(env);
            }
            channel_destroy_early(chan);
            *status_out = 0 as varnumber_T;
            return ::core::ptr::null_mut::<Channel>();
        }
        (*chan).stream.pty = pty_proc_init(&raw mut main_loop, chan as *mut ::core::ffi::c_void);
        if pty_width as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
            (*chan).stream.pty.width = pty_width;
        }
        if pty_height as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
            (*chan).stream.pty.height = pty_height;
        }
    } else {
        (*chan).stream.uv = libuv_proc_init(&raw mut main_loop, chan as *mut ::core::ffi::c_void);
    }
    let mut proc: *mut Proc = &raw mut (*chan).stream.proc;
    (*proc).argv = argv;
    (*proc).exepath = exepath;
    (*proc).cb = Some(
        channel_proc_exit_cb
            as unsafe extern "C" fn(*mut Proc, ::core::ffi::c_int, *mut ::core::ffi::c_void) -> (),
    ) as proc_exit_cb;
    (*proc).state_cb = Some(
        channel_proc_state_cb
            as unsafe extern "C" fn(*mut Proc, bool, *mut ::core::ffi::c_void) -> (),
    ) as proc_state_cb;
    (*proc).events = (*chan).events;
    (*proc).detach = detach;
    (*proc).cwd = cwd;
    (*proc).env = env;
    (*proc).overlapped = overlapped;
    let mut cmd: *mut ::core::ffi::c_char = xstrdup(proc_get_exepath(proc));
    let mut has_out: bool = false;
    let mut has_err: bool = false;
    if (*proc).type_0 as ::core::ffi::c_uint
        == kProcTypePty as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        has_out = true_0 != 0;
        has_err = false_0 != 0;
    } else {
        has_out = rpc as ::core::ffi::c_int != 0
            || callback_reader_set((*chan).on_data) as ::core::ffi::c_int != 0;
        has_err = callback_reader_set((*chan).on_stderr);
        (*proc).fwd_err = (*chan).on_stderr.fwd_err;
    }
    let mut has_in: bool = stdin_mode as ::core::ffi::c_uint
        == kChannelStdinPipe as ::core::ffi::c_int as ::core::ffi::c_uint;
    let mut status: ::core::ffi::c_int = proc_spawn(proc, has_in, has_out, has_err);
    if status != 0 {
        semsg(
            gettext(&raw const e_jobspawn as *const ::core::ffi::c_char),
            uv_strerror(status),
            cmd,
        );
        xfree(cmd as *mut ::core::ffi::c_void);
        if !(*proc).env.is_null() {
            tv_dict_free((*proc).env);
        }
        channel_destroy_early(chan);
        *status_out = (*proc).status as varnumber_T;
        return ::core::ptr::null_mut::<Channel>();
    }
    xfree(cmd as *mut ::core::ffi::c_void);
    if !(*proc).env.is_null() {
        tv_dict_free((*proc).env);
    }
    if has_in {
        wstream_init(&raw mut (*proc).in_0, 0 as size_t);
    }
    if has_out {
        rstream_init(&raw mut (*proc).out);
    }
    if rpc {
        rpc_start(chan);
    } else if has_out {
        callback_reader_start(
            &raw mut (*chan).on_data,
            b"stdout\0".as_ptr() as *const ::core::ffi::c_char,
        );
        rstream_start(
            &raw mut (*proc).out,
            Some(
                on_channel_data
                    as unsafe extern "C" fn(
                        *mut RStream,
                        *const ::core::ffi::c_char,
                        size_t,
                        *mut ::core::ffi::c_void,
                        bool,
                    ) -> size_t,
            ),
            chan as *mut ::core::ffi::c_void,
        );
    }
    if has_err {
        callback_reader_start(
            &raw mut (*chan).on_stderr,
            b"stderr\0".as_ptr() as *const ::core::ffi::c_char,
        );
        rstream_init(&raw mut (*proc).err);
        rstream_start(
            &raw mut (*proc).err,
            Some(
                on_job_stderr
                    as unsafe extern "C" fn(
                        *mut RStream,
                        *const ::core::ffi::c_char,
                        size_t,
                        *mut ::core::ffi::c_void,
                        bool,
                    ) -> size_t,
            ),
            chan as *mut ::core::ffi::c_void,
        );
    }
    *status_out = (*chan).id as varnumber_T;
    return chan;
}
#[no_mangle]
pub unsafe extern "C" fn channel_connect(
    mut tcp: bool,
    mut address: *const ::core::ffi::c_char,
    mut rpc: bool,
    mut on_output: CallbackReader,
    mut timeout: ::core::ffi::c_int,
    mut error: *mut *const ::core::ffi::c_char,
) -> uint64_t {
    let mut channel: *mut Channel = ::core::ptr::null_mut::<Channel>();
    '_end: {
        if !tcp && rpc as ::core::ffi::c_int != 0 {
            if server_owns_pipe_address(address) {
                channel = channel_alloc(kChannelStreamInternal);
                (*channel).stream.internal.cb = LUA_NOREF as LuaRef;
                rpc_start(channel);
                break '_end;
            }
        }
        channel = channel_alloc(kChannelStreamSocket);
        if !socket_connect(
            &raw mut main_loop,
            &raw mut (*channel).stream.socket,
            tcp,
            address,
            timeout,
            error,
        ) {
            channel_decref(channel);
            return 0 as uint64_t;
        }
        (*channel).stream.socket.s.internal_close_cb =
            Some(close_cb as unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> ())
                as stream_close_cb;
        (*channel).stream.socket.s.internal_data = channel as *mut ::core::ffi::c_void;
        wstream_init(&raw mut (*channel).stream.socket.s, 0 as size_t);
        rstream_init(&raw mut (*channel).stream.socket);
        if rpc {
            rpc_start(channel);
        } else {
            (*channel).on_data = on_output;
            callback_reader_start(
                &raw mut (*channel).on_data,
                b"data\0".as_ptr() as *const ::core::ffi::c_char,
            );
            rstream_start(
                &raw mut (*channel).stream.socket,
                Some(
                    on_channel_data
                        as unsafe extern "C" fn(
                            *mut RStream,
                            *const ::core::ffi::c_char,
                            size_t,
                            *mut ::core::ffi::c_void,
                            bool,
                        ) -> size_t,
                ),
                channel as *mut ::core::ffi::c_void,
            );
        }
    }
    channel_create_event(channel, address);
    return (*channel).id;
}
#[no_mangle]
pub unsafe extern "C" fn channel_from_connection(mut watcher: *mut SocketWatcher) {
    let mut channel: *mut Channel = channel_alloc(kChannelStreamSocket);
    socket_watcher_accept(watcher, &raw mut (*channel).stream.socket);
    (*channel).stream.socket.s.internal_close_cb =
        Some(close_cb as unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> ())
            as stream_close_cb;
    (*channel).stream.socket.s.internal_data = channel as *mut ::core::ffi::c_void;
    wstream_init(&raw mut (*channel).stream.socket.s, 0 as size_t);
    rstream_init(&raw mut (*channel).stream.socket);
    rpc_start(channel);
    channel_create_event(
        channel,
        &raw mut (*watcher).addr as *mut ::core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn channel_from_stdio(
    mut rpc: bool,
    mut on_output: CallbackReader,
    mut error: *mut *const ::core::ffi::c_char,
) -> uint64_t {
    if !headless_mode && !embedded_mode {
        *error = gettext(
            b"can only be opened in headless mode\0".as_ptr() as *const ::core::ffi::c_char
        );
        return 0 as uint64_t;
    }
    if did_stdio {
        *error = gettext(b"channel was already open\0".as_ptr() as *const ::core::ffi::c_char);
        return 0 as uint64_t;
    }
    did_stdio = true_0 != 0;
    let mut channel: *mut Channel = channel_alloc(kChannelStreamStdio);
    let mut stdin_dup_fd: ::core::ffi::c_int = STDIN_FILENO;
    let mut stdout_dup_fd: ::core::ffi::c_int = STDOUT_FILENO;
    if embedded_mode {
        stdin_dup_fd = fcntl(
            STDIN_FILENO,
            F_DUPFD_CLOEXEC,
            STDERR_FILENO + 1 as ::core::ffi::c_int,
        );
        stdout_dup_fd = fcntl(
            STDOUT_FILENO,
            F_DUPFD_CLOEXEC,
            STDERR_FILENO + 1 as ::core::ffi::c_int,
        );
        dup2(STDERR_FILENO, STDOUT_FILENO);
        dup2(STDERR_FILENO, STDIN_FILENO);
    }
    rstream_init_fd(
        &raw mut main_loop,
        &raw mut (*channel).stream.stdio.in_0,
        stdin_dup_fd,
    );
    wstream_init_fd(
        &raw mut main_loop,
        &raw mut (*channel).stream.stdio.out,
        stdout_dup_fd,
        0 as size_t,
    );
    if rpc {
        rpc_start(channel);
    } else {
        (*channel).on_data = on_output;
        callback_reader_start(
            &raw mut (*channel).on_data,
            b"stdin\0".as_ptr() as *const ::core::ffi::c_char,
        );
        rstream_start(
            &raw mut (*channel).stream.stdio.in_0,
            Some(
                on_channel_data
                    as unsafe extern "C" fn(
                        *mut RStream,
                        *const ::core::ffi::c_char,
                        size_t,
                        *mut ::core::ffi::c_void,
                        bool,
                    ) -> size_t,
            ),
            channel as *mut ::core::ffi::c_void,
        );
    }
    return (*channel).id;
}
#[no_mangle]
pub unsafe extern "C" fn channel_send(
    mut id: uint64_t,
    mut data: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut data_owned: bool,
    mut error: *mut *const ::core::ffi::c_char,
) -> size_t {
    let mut in_0: *mut Stream = ::core::ptr::null_mut::<Stream>();
    let mut buf: *mut WBuffer = ::core::ptr::null_mut::<WBuffer>();
    let mut chan: *mut Channel = find_channel(id);
    let mut written: size_t = 0 as size_t;
    if chan.is_null() {
        *error = gettext(&raw const e_invchan as *const ::core::ffi::c_char);
    } else if (*chan).streamtype as ::core::ffi::c_uint
        == kChannelStreamStderr as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*chan).stream.err.closed {
            *error = gettext(
                b"Can't send data to closed stream\0".as_ptr() as *const ::core::ffi::c_char
            );
        } else {
            let mut wres: ptrdiff_t = os_write(STDERR_FILENO, data, len, false_0 != 0);
            if wres >= 0 as ptrdiff_t {
                written = wres as size_t;
            }
        }
    } else if (*chan).streamtype as ::core::ffi::c_uint
        == kChannelStreamInternal as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*chan).is_rpc {
            *error = gettext(
                b"Can't send raw data to rpc channel\0".as_ptr() as *const ::core::ffi::c_char
            );
        } else if (*chan).term.is_null()
            || (*chan).stream.internal.closed as ::core::ffi::c_int != 0
        {
            *error = gettext(
                b"Can't send data to closed stream\0".as_ptr() as *const ::core::ffi::c_char
            );
        } else {
            terminal_receive((*chan).term, data, len);
            written = len;
        }
    } else {
        in_0 = channel_instream(chan);
        if (*in_0).closed {
            *error = gettext(
                b"Can't send data to closed stream\0".as_ptr() as *const ::core::ffi::c_char
            );
        } else if (*chan).is_rpc {
            *error = gettext(
                b"Can't send raw data to rpc channel\0".as_ptr() as *const ::core::ffi::c_char
            );
        } else {
            buf = wstream_new_buffer(
                (if data_owned as ::core::ffi::c_int != 0 {
                    data as *mut ::core::ffi::c_void
                } else {
                    xmemdup(data as *const ::core::ffi::c_void, len)
                }) as *mut ::core::ffi::c_char,
                len,
                1 as size_t,
                Some(xfree as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
            );
            return if wstream_write(in_0, buf) == 0 as ::core::ffi::c_int {
                len
            } else {
                0 as size_t
            };
        }
    }
    if data_owned {
        xfree(data as *mut ::core::ffi::c_void);
    }
    return written;
}
#[inline(always)]
unsafe extern "C" fn buffer_to_tv_list(
    buf: *const ::core::ffi::c_char,
    len: size_t,
) -> *mut list_T {
    let l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    tv_list_append_string(
        l,
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        0 as ssize_t,
    );
    if len > 0 as size_t {
        encode_list_write(l as *mut ::core::ffi::c_void, buf, len);
    }
    return l;
}
#[no_mangle]
pub unsafe extern "C" fn on_channel_data(
    mut stream: *mut RStream,
    mut buf: *const ::core::ffi::c_char,
    mut count: size_t,
    mut data: *mut ::core::ffi::c_void,
    mut eof: bool,
) -> size_t {
    let mut chan: *mut Channel = data as *mut Channel;
    return on_channel_output(stream, chan, buf, count, eof, &raw mut (*chan).on_data);
}
#[no_mangle]
pub unsafe extern "C" fn on_job_stderr(
    mut stream: *mut RStream,
    mut buf: *const ::core::ffi::c_char,
    mut count: size_t,
    mut data: *mut ::core::ffi::c_void,
    mut eof: bool,
) -> size_t {
    let mut chan: *mut Channel = data as *mut Channel;
    return on_channel_output(stream, chan, buf, count, eof, &raw mut (*chan).on_stderr);
}
unsafe extern "C" fn on_channel_output(
    mut stream: *mut RStream,
    mut chan: *mut Channel,
    mut buf: *const ::core::ffi::c_char,
    mut count: size_t,
    mut eof: bool,
    mut reader: *mut CallbackReader,
) -> size_t {
    if !(*chan).term.is_null() {
        terminal_receive((*chan).term, buf, count);
    }
    if eof {
        (*reader).eof = true_0 != 0;
    }
    if callback_reader_set(*reader) {
        ga_concat_len(&raw mut (*reader).buffer, buf, count);
        schedule_channel_event(chan);
    }
    return count;
}
unsafe extern "C" fn schedule_channel_event(mut chan: *mut Channel) {
    if !(*chan).callback_scheduled {
        if !(*chan).callback_busy {
            multiqueue_put_event(
                (*chan).events,
                Event {
                    handler: Some(
                        on_channel_event
                            as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                    ),
                    argv: [
                        chan as *mut ::core::ffi::c_void,
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ],
                },
            );
            channel_incref(chan);
        }
        (*chan).callback_scheduled = true_0 != 0;
    }
}
unsafe extern "C" fn on_channel_event(mut args: *mut *mut ::core::ffi::c_void) {
    let mut chan: *mut Channel = *args.offset(0 as ::core::ffi::c_int as isize) as *mut Channel;
    (*chan).callback_busy = true_0 != 0;
    (*chan).callback_scheduled = false_0 != 0;
    let mut exit_status: ::core::ffi::c_int = (*chan).exit_status;
    channel_reader_callbacks(chan, &raw mut (*chan).on_data);
    channel_reader_callbacks(chan, &raw mut (*chan).on_stderr);
    if exit_status > -1 as ::core::ffi::c_int {
        channel_callback_call(chan, ::core::ptr::null_mut::<CallbackReader>());
        (*chan).exit_status = -1 as ::core::ffi::c_int;
    }
    (*chan).callback_busy = false_0 != 0;
    if (*chan).callback_scheduled {
        multiqueue_put_event(
            (*chan).events,
            Event {
                handler: Some(
                    on_channel_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    chan as *mut ::core::ffi::c_void,
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ],
            },
        );
        channel_incref(chan);
    }
    channel_decref(chan);
}
#[no_mangle]
pub unsafe extern "C" fn channel_reader_callbacks(
    mut chan: *mut Channel,
    mut reader: *mut CallbackReader,
) {
    if (*reader).buffered {
        if (*reader).eof {
            if !(*reader).self_0.is_null() {
                if tv_dict_find((*reader).self_0, (*reader).type_0, -1 as ptrdiff_t).is_null() {
                    let mut data: *mut list_T = buffer_to_tv_list(
                        (*reader).buffer.ga_data as *const ::core::ffi::c_char,
                        (*reader).buffer.ga_len as size_t,
                    );
                    tv_dict_add_list(
                        (*reader).self_0,
                        (*reader).type_0,
                        strlen((*reader).type_0),
                        data,
                    );
                } else {
                    semsg(
                        gettext(&raw const e_streamkey as *const ::core::ffi::c_char),
                        (*reader).type_0,
                        (*chan).id,
                    );
                }
            } else {
                channel_callback_call(chan, reader);
            }
            (*reader).eof = false_0 != 0;
        }
    } else {
        let mut is_eof: bool = (*reader).eof;
        if (*reader).buffer.ga_len > 0 as ::core::ffi::c_int {
            channel_callback_call(chan, reader);
        }
        if is_eof {
            channel_callback_call(chan, reader);
            (*reader).eof = false_0 != 0;
        }
    };
}
unsafe extern "C" fn channel_proc_exit_cb(
    mut proc: *mut Proc,
    mut status: ::core::ffi::c_int,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut chan: *mut Channel = data as *mut Channel;
    if !(*chan).term.is_null() {
        terminal_close(&raw mut (*chan).term, status);
    }
    if !exiting && ui_client_channel_id == (*chan).id {
        ui_client_attach_to_restarted_server();
        if ui_client_channel_id == (*chan).id {
            exit_on_closed_chan(status);
        }
    }
    let mut exited: bool = status >= 0 as ::core::ffi::c_int;
    if exited as ::core::ffi::c_int != 0
        && (*chan).on_exit.type_0 as ::core::ffi::c_uint
            != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        schedule_channel_event(chan);
    }
    (*chan).exit_status = if exited as ::core::ffi::c_int != 0 {
        status
    } else {
        (*chan).exit_status
    };
    channel_decref(chan);
}
unsafe extern "C" fn channel_proc_state_cb(
    mut proc: *mut Proc,
    mut suspended: bool,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut chan: *mut Channel = data as *mut Channel;
    if !(*chan).term.is_null() {
        terminal_set_state((*chan).term, suspended);
    }
}
unsafe extern "C" fn channel_callback_call(
    mut chan: *mut Channel,
    mut reader: *mut CallbackReader,
) {
    let mut cb: *mut Callback = ::core::ptr::null_mut::<Callback>();
    let mut argv: [typval_T; 4] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 4];
    argv[0 as ::core::ffi::c_int as usize].v_type = VAR_NUMBER;
    argv[0 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
    argv[0 as ::core::ffi::c_int as usize].vval.v_number = (*chan).id as varnumber_T;
    if !reader.is_null() {
        argv[1 as ::core::ffi::c_int as usize].v_type = VAR_LIST;
        argv[1 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
        argv[1 as ::core::ffi::c_int as usize].vval.v_list = buffer_to_tv_list(
            (*reader).buffer.ga_data as *const ::core::ffi::c_char,
            (*reader).buffer.ga_len as size_t,
        );
        tv_list_ref(argv[1 as ::core::ffi::c_int as usize].vval.v_list);
        ga_clear(&raw mut (*reader).buffer);
        cb = &raw mut (*reader).cb;
        argv[2 as ::core::ffi::c_int as usize].vval.v_string =
            (*reader).type_0 as *mut ::core::ffi::c_char;
    } else {
        argv[1 as ::core::ffi::c_int as usize].v_type = VAR_NUMBER;
        argv[1 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
        argv[1 as ::core::ffi::c_int as usize].vval.v_number = (*chan).exit_status as varnumber_T;
        cb = &raw mut (*chan).on_exit;
        argv[2 as ::core::ffi::c_int as usize].vval.v_string =
            b"exit\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    argv[2 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
    argv[2 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    callback_call(
        cb,
        3 as ::core::ffi::c_int,
        &raw mut argv as *mut typval_T,
        &raw mut rettv,
    );
    tv_clear(&raw mut rettv);
    if !reader.is_null() {
        tv_list_unref(argv[1 as ::core::ffi::c_int as usize].vval.v_list);
    }
}
#[no_mangle]
pub unsafe extern "C" fn channel_terminal_alloc(mut buf: *mut buf_T, mut chan: *mut Channel) {
    let mut topts: TerminalOptions = TerminalOptions {
        data: chan as *mut ::core::ffi::c_void,
        width: (*chan).stream.pty.width,
        height: (*chan).stream.pty.height,
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
        force_crlf: false_0 != 0,
    };
    (*buf).b_p_channel = (*chan).id as OptInt;
    channel_incref(chan);
    (*chan).term = terminal_alloc(buf, topts);
}
unsafe extern "C" fn term_read_pause(mut pause: bool, mut data: *mut ::core::ffi::c_void) {
    let mut chan: *mut Channel = data as *mut Channel;
    if (*chan).stream.proc.out.s.closed {
        return;
    }
    if pause {
        rstream_stop_inner(&raw mut (*chan).stream.proc.out);
    } else {
        rstream_start_inner(&raw mut (*chan).stream.proc.out);
    };
}
unsafe extern "C" fn term_write(
    mut buf: *const ::core::ffi::c_char,
    mut size: size_t,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut chan: *mut Channel = data as *mut Channel;
    if (*chan).stream.proc.in_0.closed {
        logmsg(
            LOGLVL_INF,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"term_write\0".as_ptr() as *const ::core::ffi::c_char,
            918 as ::core::ffi::c_int,
            true_0 != 0,
            b"write failed: stream is closed\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut wbuf: *mut WBuffer = wstream_new_buffer(
        xmemdup(buf as *const ::core::ffi::c_void, size) as *mut ::core::ffi::c_char,
        size,
        1 as size_t,
        Some(xfree as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
    );
    wstream_write(&raw mut (*chan).stream.proc.in_0, wbuf);
}
unsafe extern "C" fn term_resize(
    mut width: uint16_t,
    mut height: uint16_t,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut chan: *mut Channel = data as *mut Channel;
    pty_proc_resize(&raw mut (*chan).stream.pty, width, height);
}
unsafe extern "C" fn term_resume(mut data: *mut ::core::ffi::c_void) {
    let mut chan: *mut Channel = data as *mut Channel;
    pty_proc_resume(&raw mut (*chan).stream.pty);
}
#[inline]
unsafe extern "C" fn term_delayed_free(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut chan: *mut Channel = *argv.offset(0 as ::core::ffi::c_int as isize) as *mut Channel;
    if (*chan).stream.proc.in_0.pending_reqs != 0 || (*chan).stream.proc.out.s.pending_reqs != 0 {
        multiqueue_put_event(
            (*chan).events,
            Event {
                handler: Some(
                    term_delayed_free as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    chan as *mut ::core::ffi::c_void,
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ],
            },
        );
        return;
    }
    if !(*chan).term.is_null() {
        terminal_destroy(&raw mut (*chan).term);
    }
    channel_decref(chan);
}
unsafe extern "C" fn term_close(mut data: *mut ::core::ffi::c_void) {
    let mut chan: *mut Channel = data as *mut Channel;
    proc_stop(&raw mut (*chan).stream.proc);
    multiqueue_put_event(
        (*chan).events,
        Event {
            handler: Some(
                term_delayed_free as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                data,
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ],
        },
    );
}
#[no_mangle]
pub unsafe extern "C" fn channel_info_changed(mut chan: *mut Channel, mut new_chan: bool) {
    let mut event: event_T = (if new_chan as ::core::ffi::c_int != 0 {
        EVENT_CHANOPEN as ::core::ffi::c_int
    } else {
        EVENT_CHANINFO as ::core::ffi::c_int
    }) as event_T;
    if has_event(event) {
        channel_incref(chan);
        multiqueue_put_event(
            main_loop.events,
            Event {
                handler: Some(
                    set_info_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    chan as *mut ::core::ffi::c_void,
                    ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                        event as intptr_t as usize,
                    ),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ],
            },
        );
    }
}
unsafe extern "C" fn set_info_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut chan: *mut Channel = *argv.offset(0 as ::core::ffi::c_int as isize) as *mut Channel;
    let mut event: event_T =
        (*argv.offset(1 as ::core::ffi::c_int as isize)).expose_addr() as ptrdiff_t as event_T;
    let mut save_v_event: save_v_event_T = save_v_event_T {
        sve_did_save: false,
        sve_hashtab: hashtab_T {
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
        },
    };
    let mut dict: *mut dict_T = get_v_event(&raw mut save_v_event);
    let mut arena: Arena = ARENA_EMPTY;
    let mut info: Dict = channel_info((*chan).id, &raw mut arena);
    let mut retval: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    object_to_vim(
        object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: info },
        },
        &raw mut retval,
        ::core::ptr::null_mut::<Error>(),
    );
    '_c2rust_label: {
        if retval.v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"retval.v_type == VAR_DICT\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/channel.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                978 as ::core::ffi::c_uint,
                b"void set_info_event(void **)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    tv_dict_add_dict(
        dict,
        b"info\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        retval.vval.v_dict,
    );
    tv_dict_set_keys_readonly(dict);
    apply_autocmds(
        event,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0 != 0,
        curbuf,
    );
    restore_v_event(dict, &raw mut save_v_event);
    arena_mem_free(arena_finish(&raw mut arena));
    channel_decref(chan);
}
#[no_mangle]
pub unsafe extern "C" fn channel_job_running(mut id: uint64_t) -> bool {
    let mut chan: *mut Channel = find_channel(id);
    return !chan.is_null()
        && (*chan).streamtype as ::core::ffi::c_uint
            == kChannelStreamProc as ::core::ffi::c_int as ::core::ffi::c_uint
        && !proc_is_stopped(&raw mut (*chan).stream.proc);
}
#[no_mangle]
pub unsafe extern "C" fn channel_info(mut id: uint64_t, mut arena: *mut Arena) -> Dict {
    let mut chan: *mut Channel = find_channel(id);
    if chan.is_null() {
        return Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
    }
    let mut info: Dict = arena_dict(arena, 9 as size_t);
    let c2rust_fresh1 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh1 as isize) = key_value_pair {
        key: cstr_as_string(b"id\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*chan).id as Integer,
            },
        },
    };
    let mut stream_desc: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut mode_desc: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    's_131: {
        match (*chan).streamtype as ::core::ffi::c_uint {
            0 => {
                stream_desc = b"job\0".as_ptr() as *const ::core::ffi::c_char;
                if (*chan).stream.proc.type_0 as ::core::ffi::c_uint
                    == kProcTypePty as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut name: *const ::core::ffi::c_char =
                        pty_proc_tty_name(&raw mut (*chan).stream.pty);
                    let c2rust_fresh2 = info.size;
                    info.size = info.size.wrapping_add(1);
                    *info.items.offset(c2rust_fresh2 as isize) = key_value_pair {
                        key: cstr_as_string(b"pty\0".as_ptr() as *const ::core::ffi::c_char),
                        value: object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed {
                                string: arena_string(arena, cstr_as_string(name)),
                            },
                        },
                    };
                }
                let mut args: *mut *mut ::core::ffi::c_char = (*chan).stream.proc.argv;
                let mut argv: Array = Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
                if !args.is_null() {
                    let mut n: size_t = 0;
                    n = 0 as size_t;
                    while !(*args.offset(n as isize)).is_null() {
                        n = n.wrapping_add(1);
                    }
                    argv = arena_array(arena, n);
                    let mut i: size_t = 0 as size_t;
                    while i < n {
                        let c2rust_fresh3 = argv.size;
                        argv.size = argv.size.wrapping_add(1);
                        *argv.items.offset(c2rust_fresh3 as isize) = object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed {
                                string: cstr_as_string(*args.offset(i as isize)),
                            },
                        };
                        i = i.wrapping_add(1);
                    }
                }
                let c2rust_fresh4 = info.size;
                info.size = info.size.wrapping_add(1);
                *info.items.offset(c2rust_fresh4 as isize) = key_value_pair {
                    key: cstr_as_string(b"argv\0".as_ptr() as *const ::core::ffi::c_char),
                    value: object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed { array: argv },
                    },
                };
                break 's_131;
            }
            2 => {
                stream_desc = b"stdio\0".as_ptr() as *const ::core::ffi::c_char;
                break 's_131;
            }
            3 => {
                stream_desc = b"stderr\0".as_ptr() as *const ::core::ffi::c_char;
                break 's_131;
            }
            4 => {
                let c2rust_fresh5 = info.size;
                info.size = info.size.wrapping_add(1);
                *info.items.offset(c2rust_fresh5 as isize) = key_value_pair {
                    key: cstr_as_string(b"internal\0".as_ptr() as *const ::core::ffi::c_char),
                    value: object {
                        type_0: kObjectTypeBoolean,
                        data: C2Rust_Unnamed { boolean: true },
                    },
                };
            }
            1 => {}
            _ => {
                break 's_131;
            }
        }
        stream_desc = b"socket\0".as_ptr() as *const ::core::ffi::c_char;
    }
    let c2rust_fresh6 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh6 as isize) = key_value_pair {
        key: cstr_as_string(b"stream\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(stream_desc),
            },
        },
    };
    if (*chan).is_rpc {
        mode_desc = b"rpc\0".as_ptr() as *const ::core::ffi::c_char;
        let c2rust_fresh7 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh7 as isize) = key_value_pair {
            key: cstr_as_string(b"client\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed {
                    dict: (*chan).rpc.info,
                },
            },
        };
    } else if !(*chan).term.is_null() {
        mode_desc = b"terminal\0".as_ptr() as *const ::core::ffi::c_char;
        let c2rust_fresh8 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh8 as isize) = key_value_pair {
            key: cstr_as_string(b"buf\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBuffer,
                data: C2Rust_Unnamed {
                    integer: terminal_buf((*chan).term) as Integer,
                },
            },
        };
        let c2rust_fresh9 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh9 as isize) = key_value_pair {
            key: cstr_as_string(b"buffer\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBuffer,
                data: C2Rust_Unnamed {
                    integer: terminal_buf((*chan).term) as Integer,
                },
            },
        };
        let c2rust_fresh10 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh10 as isize) = key_value_pair {
            key: cstr_as_string(b"exitcode\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*chan).exit_status as Integer,
                },
            },
        };
    } else {
        mode_desc = b"bytes\0".as_ptr() as *const ::core::ffi::c_char;
    }
    let c2rust_fresh11 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh11 as isize) = key_value_pair {
        key: cstr_as_string(b"mode\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(mode_desc),
            },
        },
    };
    return info;
}
unsafe extern "C" fn int64_t_cmp(
    mut pa: *const ::core::ffi::c_void,
    mut pb: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let a: int64_t = *(pa as *const int64_t);
    let b: int64_t = *(pb as *const int64_t);
    return if a == b {
        0 as ::core::ffi::c_int
    } else if a > b {
        1 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn channel_all_info(mut arena: *mut Arena) -> Array {
    let mut ids: C2Rust_Unnamed_34 = C2Rust_Unnamed_34 {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<int64_t>(),
    };
    ids.capacity = channels.set.h.size as size_t;
    ids.items = arena_alloc(
        arena,
        ::core::mem::size_of::<int64_t>().wrapping_mul(ids.capacity),
        true_0 != 0,
    ) as *mut int64_t;
    let mut id: uint64_t = 0;
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < channels.set.h.n_keys {
        id = *channels.set.keys.offset(__i as isize);
        if ids.size == ids.capacity {
            ids.capacity = (if ids.capacity != 0 {
                ids.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            });
            ids.items = xrealloc(
                ids.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<int64_t>().wrapping_mul(ids.capacity),
            ) as *mut int64_t;
        } else {
        };
        let c2rust_fresh12 = ids.size;
        ids.size = ids.size.wrapping_add(1);
        *ids.items.offset(c2rust_fresh12 as isize) = id as int64_t;
        __i = __i.wrapping_add(1);
    }
    qsort(
        ids.items as *mut ::core::ffi::c_void,
        ids.size,
        ::core::mem::size_of::<int64_t>(),
        Some(
            int64_t_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    let mut ret: Array = arena_array(arena, ids.size);
    let mut i: size_t = 0 as size_t;
    while i < ids.size {
        let c2rust_fresh13 = ret.size;
        ret.size = ret.size.wrapping_add(1);
        *ret.items.offset(c2rust_fresh13 as isize) = object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed {
                dict: channel_info(*ids.items.offset(i as isize) as uint64_t, arena),
            },
        };
        i = i.wrapping_add(1);
    }
    return ret;
}
#[inline(always)]
unsafe extern "C" fn tv_list_ref(l: *mut list_T) {
    if l.is_null() {
        return;
    }
    (*l).lv_refcount += 1;
}
#[inline]
unsafe extern "C" fn proc_get_exepath(mut proc: *mut Proc) -> *const ::core::ffi::c_char {
    return if !(*proc).exepath.is_null() {
        (*proc).exepath
    } else {
        *(*proc).argv.offset(0 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_char
    };
}
#[inline]
unsafe extern "C" fn proc_is_stopped(mut proc: *mut Proc) -> bool {
    let mut exited: bool = (*proc).status >= 0 as ::core::ffi::c_int;
    return exited as ::core::ffi::c_int != 0 || (*proc).stopped_time != 0 as uint64_t;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
