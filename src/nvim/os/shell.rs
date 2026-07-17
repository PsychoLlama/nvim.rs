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
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fopen(
        __filename: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
    ) -> *mut FILE;
    fn fread(
        __ptr: *mut ::core::ffi::c_void,
        __size: size_t,
        __n: size_t,
        __stream: *mut FILE,
    ) -> ::core::ffi::c_ulong;
    fn fseek(
        __stream: *mut FILE,
        __off: ::core::ffi::c_long,
        __whence: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn ftell(__stream: *mut FILE) -> ::core::ffi::c_long;
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
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strcat(
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
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn uv_strerror(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn uv_err_name(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xstrlcpy(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrlcat(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn read_buffer_into(buf: *mut buf_T, start: linenr_T, end: linenr_T, sb: *mut StringBuilder);
    static mut p_sh: *mut ::core::ffi::c_char;
    static mut p_shcf: *mut ::core::ffi::c_char;
    static mut p_sxq: *mut ::core::ffi::c_char;
    static mut p_sxe: *mut ::core::ffi::c_char;
    static mut p_verbose: OptInt;
    fn vim_strsave_escaped_ext(
        string: *const ::core::ffi::c_char,
        esc_chars: *const ::core::ffi::c_char,
        cc: ::core::ffi::c_char,
        bsl: bool,
    ) -> *mut ::core::ffi::c_char;
    fn vim_strnsave_unquoted(
        string: *const ::core::ffi::c_char,
        length: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn vim_snprintf(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn backslash_halve(p: *mut ::core::ffi::c_char);
    static e_notmp: [::core::ffi::c_char; 0];
    static e_cant_read_file_str: [::core::ffi::c_char; 0];
    static e_shellempty: [::core::ffi::c_char; 0];
    static e_wildexpand: [::core::ffi::c_char; 0];
    static e_cannot_read_from_str_2: [::core::ffi::c_char; 0];
    fn set_vim_var_nr(idx: VimVarIndex, val: varnumber_T);
    fn libuv_proc_init(loop_0: *mut Loop, data: *mut ::core::ffi::c_void) -> LibuvProc;
    fn os_hrtime() -> uint64_t;
    fn os_delay(ms: uint64_t, ignoreinput: bool);
    fn multiqueue_new_child(parent: *mut MultiQueue) -> *mut MultiQueue;
    fn multiqueue_free(self_0: *mut MultiQueue);
    fn multiqueue_put_event(self_0: *mut MultiQueue, event: Event);
    fn multiqueue_empty(self_0: *mut MultiQueue) -> bool;
    fn loop_poll_events(loop_0: *mut Loop, ms: int64_t) -> bool;
    fn proc_spawn(proc: *mut Proc, in_0: bool, out: bool, err: bool) -> ::core::ffi::c_int;
    fn proc_wait(
        proc: *mut Proc,
        ms: ::core::ffi::c_int,
        events: *mut MultiQueue,
    ) -> ::core::ffi::c_int;
    fn proc_stop(proc: *mut Proc);
    fn stream_may_close(stream: *mut Stream);
    fn wstream_init(stream: *mut Stream, maxmem: size_t);
    fn wstream_set_write_cb(
        stream: *mut Stream,
        cb: stream_write_cb,
        data: *mut ::core::ffi::c_void,
    );
    fn wstream_write(stream: *mut Stream, buffer: *mut WBuffer) -> ::core::ffi::c_int;
    fn wstream_new_buffer(
        data: *mut ::core::ffi::c_char,
        size: size_t,
        refcount: size_t,
        cb: wbuffer_data_finalizer,
    ) -> *mut WBuffer;
    fn rstream_init(stream: *mut RStream);
    fn rstream_start(stream: *mut RStream, cb: stream_read_cb, data: *mut ::core::ffi::c_void);
    fn make_filter_cmd(
        cmd: *mut ::core::ffi::c_char,
        itmp: *mut ::core::ffi::c_char,
        otmp: *mut ::core::ffi::c_char,
        do_in: bool,
    ) -> *mut ::core::ffi::c_char;
    fn check_secure() -> bool;
    fn vim_tempname() -> *mut ::core::ffi::c_char;
    static mut Rows: ::core::ffi::c_int;
    static mut cmdline_row: ::core::ffi::c_int;
    static mut no_wait_return: ::core::ffi::c_int;
    static mut lines_left: ::core::ffi::c_int;
    static mut msg_no_more: bool;
    static mut do_profiling: ::core::ffi::c_int;
    static mut no_check_timestamps: ::core::ffi::c_int;
    static mut curwin: *mut win_T;
    static mut curbuf: *mut buf_T;
    static mut secure: ::core::ffi::c_int;
    static mut sandbox: ::core::ffi::c_int;
    static mut State: ::core::ffi::c_int;
    static mut emsg_silent: ::core::ffi::c_int;
    static mut got_int: bool;
    static mut main_loop: Loop;
    fn utfc_ptr2len_len(
        p: *const ::core::ffi::c_char,
        size: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    static utf8len_tab_zero: [uint8_t; 256];
    fn ml_append(
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        len: colnr_T,
        newfile: bool,
    ) -> ::core::ffi::c_int;
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn msg_multiline(
        str: String_0,
        hl_id: ::core::ffi::c_int,
        check_int: bool,
        hist: bool,
        need_clear: *mut bool,
    );
    fn smsg(hl_id: ::core::ffi::c_int, s: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn msg_schedule_semsg(fmt: *const ::core::ffi::c_char, ...);
    fn msg_ext_set_kind(msg_kind: *const ::core::ffi::c_char);
    fn msg_ext_set_append(append: bool);
    fn msg_start();
    fn msg_putchar(c: ::core::ffi::c_int);
    fn msg_outnum(n: ::core::ffi::c_int);
    fn msg_outtrans(
        str: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    ) -> ::core::ffi::c_int;
    fn msg_puts(s: *const ::core::ffi::c_char);
    fn msg_sb_eol();
    fn msg_end() -> bool;
    fn verbose_enter();
    fn verbose_leave();
    fn os_isdir(name: *const ::core::ffi::c_char) -> bool;
    fn os_can_exe(
        name: *const ::core::ffi::c_char,
        abspath: *mut *mut ::core::ffi::c_char,
        use_path: bool,
    ) -> bool;
    fn os_fopen(path: *const ::core::ffi::c_char, flags: *const ::core::ffi::c_char) -> *mut FILE;
    fn os_path_exists(path: *const ::core::ffi::c_char) -> bool;
    fn os_remove(path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn signal_reject_deadly();
    fn signal_accept_deadly();
    fn prof_child_enter(tm: *mut proftime_T);
    fn prof_child_exit(tm: *mut proftime_T);
    fn path_tail(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn invocation_path_tail(
        invocation: *const ::core::ffi::c_char,
        len: *mut size_t,
    ) -> *const ::core::ffi::c_char;
    fn add_pathsep(p: *mut ::core::ffi::c_char) -> bool;
    fn path_has_wildcard(p: *const ::core::ffi::c_char) -> bool;
    fn tag_freematch();
    fn ui_busy_start();
    fn ui_busy_stop();
    fn ui_flush();
    fn ui_has(ext: UIExtension) -> bool;
}
pub type __uid_t = ::core::ffi::c_uint;
pub type __gid_t = ::core::ffi::c_uint;
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
pub type intptr_t = isize;
pub type size_t = usize;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__queue {
    pub next: *mut uv__queue,
    pub prev: *mut uv__queue,
}
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
pub struct uv_loop_s {
    pub data: *mut ::core::ffi::c_void,
    pub active_handles: ::core::ffi::c_uint,
    pub handle_queue: uv__queue,
    pub active_reqs: C2Rust_Unnamed_4,
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
    pub timer_heap: C2Rust_Unnamed_2,
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
    pub u: C2Rust_Unnamed_1,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub signal_cb: uv_signal_cb,
    pub signum: ::core::ffi::c_int,
    pub tree_entry: C2Rust_Unnamed,
    pub caught_signals: ::core::ffi::c_uint,
    pub dispatched_signals: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed {
    pub rbe_left: *mut uv_signal_s,
    pub rbe_right: *mut uv_signal_s,
    pub rbe_parent: *mut uv_signal_s,
    pub rbe_color: ::core::ffi::c_int,
}
pub type uv_signal_cb = Option<unsafe extern "C" fn(*mut uv_signal_t, ::core::ffi::c_int) -> ()>;
pub type uv_handle_t = uv_handle_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_handle_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_0,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_0 {
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
pub union C2Rust_Unnamed_1 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_2 {
    pub min: *mut ::core::ffi::c_void,
    pub nelts: ::core::ffi::c_uint,
}
pub type uv_rwlock_t = pthread_rwlock_t;
pub type uv_async_t = uv_async_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_async_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_3,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub async_cb: uv_async_cb,
    pub queue: uv__queue,
    pub pending: ::core::ffi::c_int,
}
pub type uv_async_cb = Option<unsafe extern "C" fn(*mut uv_async_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_3 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_mutex_t = pthread_mutex_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_4 {
    pub unused: *mut ::core::ffi::c_void,
    pub count: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_buf_t {
    pub base: *mut ::core::ffi::c_char,
    pub len: size_t,
}
pub type uv_file = ::core::ffi::c_int;
pub type uv_gid_t = gid_t;
pub type uv_uid_t = uid_t;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_stream_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_5,
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
pub type uv_stream_t = uv_stream_s;
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
pub union C2Rust_Unnamed_5 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_tcp_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_6,
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
pub union C2Rust_Unnamed_6 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_tcp_t = uv_tcp_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_pipe_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_7,
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
pub union C2Rust_Unnamed_7 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_pipe_t = uv_pipe_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_timer_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_9,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub timer_cb: uv_timer_cb,
    pub node: C2Rust_Unnamed_8,
    pub timeout: uint64_t,
    pub repeat: uint64_t,
    pub start_id: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_8 {
    pub heap: [*mut ::core::ffi::c_void; 3],
    pub queue: uv__queue,
}
pub type uv_timer_cb = Option<unsafe extern "C" fn(*mut uv_timer_t) -> ()>;
pub type uv_timer_t = uv_timer_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_9 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_idle_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_10,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub idle_cb: uv_idle_cb,
    pub queue: uv__queue,
}
pub type uv_idle_cb = Option<unsafe extern "C" fn(*mut uv_idle_t) -> ()>;
pub type uv_idle_t = uv_idle_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_10 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_process_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_11,
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
pub union C2Rust_Unnamed_11 {
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
    pub data: C2Rust_Unnamed_12,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_12 {
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
    pub b_wininfo: C2Rust_Unnamed_24,
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
    pub b_signcols: C2Rust_Unnamed_16,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_14,
    pub update_callbacks: C2Rust_Unnamed_13,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_13 {
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
pub struct C2Rust_Unnamed_14 {
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
    pub data: C2Rust_Unnamed_15,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_15 {
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
pub struct C2Rust_Unnamed_16 {
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
    pub sst_union: C2Rust_Unnamed_17,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_17 {
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
    pub data: C2Rust_Unnamed_18,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_18 {
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
    pub fc_fixvar: [C2Rust_Unnamed_19; 12],
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
pub struct C2Rust_Unnamed_19 {
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
    pub uh_next: C2Rust_Unnamed_23,
    pub uh_prev: C2Rust_Unnamed_22,
    pub uh_alt_next: C2Rust_Unnamed_21,
    pub uh_alt_prev: C2Rust_Unnamed_20,
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
pub union C2Rust_Unnamed_20 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_21 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_22 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_23 {
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
pub struct C2Rust_Unnamed_24 {
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
    pub type_0: C2Rust_Unnamed_25,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_25 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_25 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_25 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_25 = 0;
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
    pub children: C2Rust_Unnamed_26,
    pub children_watcher: uv_signal_t,
    pub children_kill_timer: uv_timer_t,
    pub poll_timer: uv_timer_t,
    pub exit_delay_timer: uv_timer_t,
    pub async_0: uv_async_t,
    pub mutex: uv_mutex_t,
    pub recursive: ::core::ffi::c_int,
    pub closing: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_26 {
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
pub type Stream = stream;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stream {
    pub closed: bool,
    pub uv: C2Rust_Unnamed_27,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_27 {
    pub pipe: uv_pipe_t,
    pub tcp: uv_tcp_t,
    pub idle: uv_idle_t,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
}
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_28 = 76;
pub const HLF_PRE: C2Rust_Unnamed_28 = 75;
pub const HLF_OK: C2Rust_Unnamed_28 = 74;
pub const HLF_SO: C2Rust_Unnamed_28 = 73;
pub const HLF_SE: C2Rust_Unnamed_28 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_28 = 71;
pub const HLF_TS: C2Rust_Unnamed_28 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_28 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_28 = 68;
pub const HLF_CU: C2Rust_Unnamed_28 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_28 = 66;
pub const HLF_WBR: C2Rust_Unnamed_28 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_28 = 64;
pub const HLF_MSG: C2Rust_Unnamed_28 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_28 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_28 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_28 = 60;
pub const HLF_0: C2Rust_Unnamed_28 = 59;
pub const HLF_QFL: C2Rust_Unnamed_28 = 58;
pub const HLF_MC: C2Rust_Unnamed_28 = 57;
pub const HLF_CUL: C2Rust_Unnamed_28 = 56;
pub const HLF_CUC: C2Rust_Unnamed_28 = 55;
pub const HLF_TPF: C2Rust_Unnamed_28 = 54;
pub const HLF_TPS: C2Rust_Unnamed_28 = 53;
pub const HLF_TP: C2Rust_Unnamed_28 = 52;
pub const HLF_PBR: C2Rust_Unnamed_28 = 51;
pub const HLF_PST: C2Rust_Unnamed_28 = 50;
pub const HLF_PSB: C2Rust_Unnamed_28 = 49;
pub const HLF_PSX: C2Rust_Unnamed_28 = 48;
pub const HLF_PNX: C2Rust_Unnamed_28 = 47;
pub const HLF_PSK: C2Rust_Unnamed_28 = 46;
pub const HLF_PNK: C2Rust_Unnamed_28 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_28 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_28 = 43;
pub const HLF_PSI: C2Rust_Unnamed_28 = 42;
pub const HLF_PNI: C2Rust_Unnamed_28 = 41;
pub const HLF_SPL: C2Rust_Unnamed_28 = 40;
pub const HLF_SPR: C2Rust_Unnamed_28 = 39;
pub const HLF_SPC: C2Rust_Unnamed_28 = 38;
pub const HLF_SPB: C2Rust_Unnamed_28 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_28 = 36;
pub const HLF_SC: C2Rust_Unnamed_28 = 35;
pub const HLF_TXA: C2Rust_Unnamed_28 = 34;
pub const HLF_TXD: C2Rust_Unnamed_28 = 33;
pub const HLF_DED: C2Rust_Unnamed_28 = 32;
pub const HLF_CHD: C2Rust_Unnamed_28 = 31;
pub const HLF_ADD: C2Rust_Unnamed_28 = 30;
pub const HLF_FC: C2Rust_Unnamed_28 = 29;
pub const HLF_FL: C2Rust_Unnamed_28 = 28;
pub const HLF_WM: C2Rust_Unnamed_28 = 27;
pub const HLF_W: C2Rust_Unnamed_28 = 26;
pub const HLF_VNC: C2Rust_Unnamed_28 = 25;
pub const HLF_V: C2Rust_Unnamed_28 = 24;
pub const HLF_T: C2Rust_Unnamed_28 = 23;
pub const HLF_VSP: C2Rust_Unnamed_28 = 22;
pub const HLF_C: C2Rust_Unnamed_28 = 21;
pub const HLF_SNC: C2Rust_Unnamed_28 = 20;
pub const HLF_S: C2Rust_Unnamed_28 = 19;
pub const HLF_R: C2Rust_Unnamed_28 = 18;
pub const HLF_CLF: C2Rust_Unnamed_28 = 17;
pub const HLF_CLS: C2Rust_Unnamed_28 = 16;
pub const HLF_CLN: C2Rust_Unnamed_28 = 15;
pub const HLF_LNB: C2Rust_Unnamed_28 = 14;
pub const HLF_LNA: C2Rust_Unnamed_28 = 13;
pub const HLF_N: C2Rust_Unnamed_28 = 12;
pub const HLF_CM: C2Rust_Unnamed_28 = 11;
pub const HLF_M: C2Rust_Unnamed_28 = 10;
pub const HLF_LC: C2Rust_Unnamed_28 = 9;
pub const HLF_L: C2Rust_Unnamed_28 = 8;
pub const HLF_I: C2Rust_Unnamed_28 = 7;
pub const HLF_E: C2Rust_Unnamed_28 = 6;
pub const HLF_D: C2Rust_Unnamed_28 = 5;
pub const HLF_AT: C2Rust_Unnamed_28 = 4;
pub const HLF_TERM: C2Rust_Unnamed_28 = 3;
pub const HLF_EOB: C2Rust_Unnamed_28 = 2;
pub const HLF_8: C2Rust_Unnamed_28 = 1;
pub const HLF_NONE: C2Rust_Unnamed_28 = 0;
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
pub struct LibuvProc {
    pub proc: Proc,
    pub uv: uv_process_t,
    pub uvopts: uv_process_options_t,
    pub uvstdio: [uv_stdio_container_t; 4],
}
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_29 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_29 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_29 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_29 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_29 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_29 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_29 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_29 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_29 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_29 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_29 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_29 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_29 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_29 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_29 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_29 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_29 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_29 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_29 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_29 = 1;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const kShellOptHideMess: C2Rust_Unnamed_30 = 64;
pub const kShellOptWrite: C2Rust_Unnamed_30 = 32;
pub const kShellOptRead: C2Rust_Unnamed_30 = 16;
pub const kShellOptSilent: C2Rust_Unnamed_30 = 8;
pub const kShellOptDoOut: C2Rust_Unnamed_30 = 4;
pub const kShellOptExpand: C2Rust_Unnamed_30 = 2;
pub const kShellOptFilter: C2Rust_Unnamed_30 = 1;
pub const EW_NOTFOUND: C2Rust_Unnamed_31 = 4;
pub const EW_SHELLCMD: C2Rust_Unnamed_31 = 8192;
pub const EW_EXEC: C2Rust_Unnamed_31 = 64;
pub const EW_FILE: C2Rust_Unnamed_31 = 2;
pub const EW_DIR: C2Rust_Unnamed_31 = 1;
pub const EW_SILENT: C2Rust_Unnamed_31 = 32;
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
pub const EW_KEEPDOLLAR: C2Rust_Unnamed_31 = 2048;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const EW_NOBREAK: C2Rust_Unnamed_31 = 262144;
pub const EW_CDPATH: C2Rust_Unnamed_31 = 131072;
pub const EW_NOTENV: C2Rust_Unnamed_31 = 65536;
pub const EW_EMPTYOK: C2Rust_Unnamed_31 = 32768;
pub const EW_DODOT: C2Rust_Unnamed_31 = 16384;
pub const EW_ALLLINKS: C2Rust_Unnamed_31 = 4096;
pub const EW_NOTWILD: C2Rust_Unnamed_31 = 1024;
pub const EW_NOERROR: C2Rust_Unnamed_31 = 512;
pub const EW_ICASE: C2Rust_Unnamed_31 = 256;
pub const EW_PATH: C2Rust_Unnamed_31 = 128;
pub const EW_KEEPALL: C2Rust_Unnamed_31 = 16;
pub const EW_ADDSLASH: C2Rust_Unnamed_31 = 8;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const KV_INITIAL_VALUE: StringBuilder = StringBuilder {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PROF_YES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NS_1_SECOND: ::core::ffi::c_uint = 1000000000 as ::core::ffi::c_uint;
pub const OUT_DATA_THRESHOLD: ::core::ffi::c_uint =
    (1024 as ::core::ffi::c_uint).wrapping_mul(10 as ::core::ffi::c_uint);
pub const SHELL_SPECIAL: [::core::ffi::c_char; 15] = unsafe {
    ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\t \"&'$;<>()\\|\n\0")
};
unsafe extern "C" fn save_patterns(
    mut num_pat: ::core::ffi::c_int,
    mut pat: *mut *mut ::core::ffi::c_char,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
) {
    *file = xmalloc(
        (num_pat as size_t).wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
    ) as *mut *mut ::core::ffi::c_char;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_pat {
        let mut s: *mut ::core::ffi::c_char = xstrdup(*pat.offset(i as isize));
        backslash_halve(s);
        *(*file).offset(i as isize) = s;
        i += 1;
    }
    *num_file = num_pat;
}
unsafe extern "C" fn have_wildcard(
    mut num: ::core::ffi::c_int,
    mut file: *mut *mut ::core::ffi::c_char,
) -> bool {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num {
        if path_has_wildcard(*file.offset(i as isize)) {
            return true_0 != 0;
        }
        i += 1;
    }
    return false_0 != 0;
}
unsafe extern "C" fn have_dollars(
    mut num: ::core::ffi::c_int,
    mut file: *mut *mut ::core::ffi::c_char,
) -> bool {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num {
        if !vim_strchr(*file.offset(i as isize), '$' as ::core::ffi::c_int).is_null() {
            return true_0 != 0;
        }
        i += 1;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn os_expand_wildcards(
    mut num_pat: ::core::ffi::c_int,
    mut pat: *mut *mut ::core::ffi::c_char,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut fseek_res: ::core::ffi::c_int = 0;
    let mut templen: int64_t = 0;
    let mut buffer: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut readlen: size_t = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut len: size_t = 0;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut extra_shell_arg: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut shellopts: ::core::ffi::c_int =
        kShellOptExpand as ::core::ffi::c_int | kShellOptSilent as ::core::ffi::c_int;
    let mut j: ::core::ffi::c_int = 0;
    let mut tempname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut shell_style: ::core::ffi::c_int = STYLE_ECHO;
    let mut check_spaces: ::core::ffi::c_int = 0;
    static mut did_find_nul: bool = false_0 != 0;
    let mut ampersand: bool = false_0 != 0;
    static mut sh_vimglob_func: *mut ::core::ffi::c_char =
        b"vimglob() { while [ $# -ge 1 ]; do echo \"$1\"; shift; done }; vimglob >\0".as_ptr()
            as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    static mut sh_globstar_opt: *mut ::core::ffi::c_char =
        b"[[ ${BASH_VERSINFO[0]} -ge 4 ]] && shopt -s globstar; \0".as_ptr()
            as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    let mut is_fish_shell: bool = strncmp(
        invocation_path_tail(p_sh, ::core::ptr::null_mut::<size_t>()),
        b"fish\0".as_ptr() as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int;
    *num_file = 0 as ::core::ffi::c_int;
    *file = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    if !have_wildcard(num_pat, pat) {
        save_patterns(num_pat, pat, num_file, file);
        return OK;
    }
    if sandbox != 0 as ::core::ffi::c_int && check_secure() as ::core::ffi::c_int != 0 {
        return FAIL;
    }
    if secure != 0 {
        i = 0 as ::core::ffi::c_int;
        while i < num_pat {
            if !vim_strchr(*pat.offset(i as isize), '`' as ::core::ffi::c_int).is_null()
                && check_secure() as ::core::ffi::c_int != 0
            {
                return FAIL;
            }
            i += 1;
        }
    }
    tempname = vim_tempname();
    if tempname.is_null() {
        emsg(gettext(&raw const e_notmp as *const ::core::ffi::c_char));
        return FAIL;
    }
    if num_pat == 1 as ::core::ffi::c_int
        && **pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '`' as ::core::ffi::c_int
        && {
            len = strlen(*pat.offset(0 as ::core::ffi::c_int as isize));
            len > 2 as size_t
        }
        && *(*pat.offset(0 as ::core::ffi::c_int as isize))
            .offset(len as isize)
            .offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
            == '`' as ::core::ffi::c_int
    {
        shell_style = STYLE_BT;
    } else {
        len = strlen(p_sh);
        if len >= 3 as size_t {
            if strcmp(
                p_sh.offset(len as isize)
                    .offset(-(3 as ::core::ffi::c_int as isize)),
                b"csh\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                shell_style = STYLE_GLOB;
            } else if strcmp(
                p_sh.offset(len as isize)
                    .offset(-(3 as ::core::ffi::c_int as isize)),
                b"zsh\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                shell_style = STYLE_PRINT;
            }
        }
    }
    if shell_style == STYLE_ECHO {
        if !strstr(
            path_tail(p_sh),
            b"bash\0".as_ptr() as *const ::core::ffi::c_char,
        )
        .is_null()
        {
            shell_style = STYLE_GLOBSTAR;
        } else if !strstr(
            path_tail(p_sh),
            b"sh\0".as_ptr() as *const ::core::ffi::c_char,
        )
        .is_null()
        {
            shell_style = STYLE_VIMGLOB;
        }
    }
    len = strlen(tempname).wrapping_add(29 as size_t);
    if shell_style == STYLE_VIMGLOB {
        len = len.wrapping_add(strlen(sh_vimglob_func));
    } else if shell_style == STYLE_GLOBSTAR {
        len = len.wrapping_add(strlen(sh_vimglob_func).wrapping_add(strlen(sh_globstar_opt)));
    }
    i = 0 as ::core::ffi::c_int;
    while i < num_pat {
        len = len.wrapping_add(1);
        j = 0 as ::core::ffi::c_int;
        while *(*pat.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int != NUL {
            if !vim_strchr(
                SHELL_SPECIAL.as_ptr(),
                *(*pat.offset(i as isize)).offset(j as isize) as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
            {
                len = len.wrapping_add(1);
            }
            len = len.wrapping_add(1);
            j += 1;
        }
        i += 1;
    }
    if is_fish_shell {
        len = (len as ::core::ffi::c_ulong).wrapping_add(
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize)
                as ::core::ffi::c_ulong,
        ) as size_t;
    }
    let mut command: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    if shell_style == STYLE_BT {
        if is_fish_shell {
            strcpy(
                command,
                b"begin; \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
        } else {
            strcpy(
                command,
                b"(\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
        }
        strcat(
            command,
            (*pat.offset(0 as ::core::ffi::c_int as isize))
                .offset(1 as ::core::ffi::c_int as isize),
        );
        p = command
            .offset(strlen(command) as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
        if is_fish_shell {
            let c2rust_fresh0 = p;
            p = p.offset(-1);
            *c2rust_fresh0 = ';' as ::core::ffi::c_char;
            strcat(command, b" end\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            let c2rust_fresh1 = p;
            p = p.offset(-1);
            *c2rust_fresh1 = ')' as ::core::ffi::c_char;
        }
        while p > command && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
            p = p.offset(-1);
        }
        if *p as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
            ampersand = true_0 != 0;
            *p = ' ' as ::core::ffi::c_char;
        }
        strcat(command, b">\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        strcpy(
            command,
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if shell_style == STYLE_GLOB {
            if flags & EW_NOTFOUND as ::core::ffi::c_int != 0 {
                strcat(
                    command,
                    b"set nonomatch; \0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                strcat(
                    command,
                    b"unset nonomatch; \0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        }
        if shell_style == STYLE_GLOB {
            strcat(command, b"glob >\0".as_ptr() as *const ::core::ffi::c_char);
        } else if shell_style == STYLE_PRINT {
            strcat(
                command,
                b"print -N >\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else if shell_style == STYLE_VIMGLOB {
            strcat(command, sh_vimglob_func);
        } else if shell_style == STYLE_GLOBSTAR {
            strcat(command, sh_globstar_opt);
            strcat(command, sh_vimglob_func);
        } else {
            strcat(command, b"echo >\0".as_ptr() as *const ::core::ffi::c_char);
        }
    }
    strcat(command, tempname);
    if shell_style != STYLE_BT {
        i = 0 as ::core::ffi::c_int;
        while i < num_pat {
            let mut intick: bool = false_0 != 0;
            p = command.offset(strlen(command) as isize);
            let c2rust_fresh2 = p;
            p = p.offset(1);
            *c2rust_fresh2 = ' ' as ::core::ffi::c_char;
            j = 0 as ::core::ffi::c_int;
            while *(*pat.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int != NUL {
                if *(*pat.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int
                    == '`' as ::core::ffi::c_int
                {
                    intick = !intick;
                } else if *(*pat.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *(*pat.offset(i as isize)).offset((j + 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        != NUL
                {
                    if intick as ::core::ffi::c_int != 0
                        || !vim_strchr(
                            SHELL_SPECIAL.as_ptr(),
                            *(*pat.offset(i as isize))
                                .offset((j + 1 as ::core::ffi::c_int) as isize)
                                as uint8_t as ::core::ffi::c_int,
                        )
                        .is_null()
                        || *(*pat.offset(i as isize)).offset((j + 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            == '`' as ::core::ffi::c_int
                    {
                        let c2rust_fresh3 = p;
                        p = p.offset(1);
                        *c2rust_fresh3 = '\\' as ::core::ffi::c_char;
                    }
                    j += 1;
                } else if !intick
                    && (flags & EW_KEEPDOLLAR as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                        || *(*pat.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int
                            != '$' as ::core::ffi::c_int)
                    && !vim_strchr(
                        SHELL_SPECIAL.as_ptr(),
                        *(*pat.offset(i as isize)).offset(j as isize) as uint8_t
                            as ::core::ffi::c_int,
                    )
                    .is_null()
                {
                    let c2rust_fresh4 = p;
                    p = p.offset(1);
                    *c2rust_fresh4 = '\\' as ::core::ffi::c_char;
                }
                let c2rust_fresh5 = p;
                p = p.offset(1);
                *c2rust_fresh5 = *(*pat.offset(i as isize)).offset(j as isize);
                j += 1;
            }
            *p = NUL as ::core::ffi::c_char;
            i += 1;
        }
    }
    if flags & EW_SILENT as ::core::ffi::c_int != 0 {
        shellopts |= kShellOptHideMess as ::core::ffi::c_int;
    }
    if ampersand {
        strcat(command, b"&\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if shell_style == STYLE_PRINT {
        extra_shell_arg =
            b"-G\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else if shell_style == STYLE_GLOB && !have_dollars(num_pat, pat) {
        extra_shell_arg =
            b"-f\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    i = call_shell(command, shellopts, extra_shell_arg);
    if ampersand {
        os_delay(10 as uint64_t, true_0 != 0);
    }
    xfree(command as *mut ::core::ffi::c_void);
    if i != 0 {
        os_remove(tempname);
        xfree(tempname as *mut ::core::ffi::c_void);
        if flags & EW_SILENT as ::core::ffi::c_int == 0 {
            msg_putchar('\n' as ::core::ffi::c_int);
            cmdline_row = Rows - 1 as ::core::ffi::c_int;
            msg(
                gettext(&raw const e_wildexpand as *const ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
            );
            msg_start();
        }
        if shell_style == STYLE_BT {
            return FAIL;
        }
    } else {
        fd = fopen(tempname, READBIN.as_ptr()) as *mut FILE;
        if fd.is_null() {
            if flags & EW_SILENT as ::core::ffi::c_int == 0 {
                msg(
                    gettext(&raw const e_wildexpand as *const ::core::ffi::c_char),
                    0 as ::core::ffi::c_int,
                );
                msg_start();
            }
            xfree(tempname as *mut ::core::ffi::c_void);
        } else {
            fseek_res = fseek(fd, 0 as ::core::ffi::c_long, SEEK_END);
            if fseek_res < 0 as ::core::ffi::c_int {
                xfree(tempname as *mut ::core::ffi::c_void);
                fclose(fd);
                return FAIL;
            }
            templen = ftell(fd) as int64_t;
            if templen < 0 as int64_t {
                xfree(tempname as *mut ::core::ffi::c_void);
                fclose(fd);
                return FAIL;
            }
            len = templen as size_t;
            fseek(fd, 0 as ::core::ffi::c_long, SEEK_SET);
            buffer = xmalloc(len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
            readlen = fread(buffer as *mut ::core::ffi::c_void, 1 as size_t, len, fd) as size_t;
            fclose(fd);
            os_remove(tempname);
            if readlen != len {
                semsg(
                    gettext(&raw const e_cant_read_file_str as *const ::core::ffi::c_char),
                    tempname,
                );
                xfree(tempname as *mut ::core::ffi::c_void);
                xfree(buffer as *mut ::core::ffi::c_void);
                return FAIL;
            }
            xfree(tempname as *mut ::core::ffi::c_void);
            if shell_style == STYLE_ECHO {
                *buffer.offset(len as isize) = '\n' as ::core::ffi::c_char;
                p = buffer;
                i = 0 as ::core::ffi::c_int;
                while *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int {
                    while *p as ::core::ffi::c_int != ' ' as ::core::ffi::c_int
                        && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                    {
                        p = p.offset(1);
                    }
                    p = skipwhite(p);
                    i += 1;
                }
            } else if shell_style == STYLE_BT
                || shell_style == STYLE_VIMGLOB
                || shell_style == STYLE_GLOBSTAR
            {
                *buffer.offset(len as isize) = NUL as ::core::ffi::c_char;
                p = buffer;
                i = 0 as ::core::ffi::c_int;
                while *p as ::core::ffi::c_int != NUL {
                    while *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                        && *p as ::core::ffi::c_int != NUL
                    {
                        p = p.offset(1);
                    }
                    if *p as ::core::ffi::c_int != NUL {
                        p = p.offset(1);
                    }
                    p = skipwhite(p);
                    i += 1;
                }
            } else {
                check_spaces = false_0;
                if shell_style == STYLE_PRINT && !did_find_nul {
                    *buffer.offset(len as isize) = NUL as ::core::ffi::c_char;
                    if len != 0
                        && (strlen(buffer) as ::core::ffi::c_int) < len as ::core::ffi::c_int
                    {
                        did_find_nul = true_0 != 0;
                    } else {
                        check_spaces = true_0;
                    }
                }
                if len != 0
                    && *buffer.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                        == NUL
                {
                    len = len.wrapping_sub(1);
                } else {
                    *buffer.offset(len as isize) = NUL as ::core::ffi::c_char;
                }
                p = buffer;
                while p < buffer.offset(len as isize) {
                    if *p as ::core::ffi::c_int == NUL
                        || *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                            && check_spaces != 0
                    {
                        i += 1;
                        *p = NUL as ::core::ffi::c_char;
                    }
                    p = p.offset(1);
                }
                if len != 0 {
                    i += 1;
                }
            }
            '_c2rust_label: {
                if *buffer.offset(len as isize) as ::core::ffi::c_int == '\0' as ::core::ffi::c_int
                    || *buffer.offset(len as isize) as ::core::ffi::c_int
                        == '\n' as ::core::ffi::c_int
                {
                } else {
                    __assert_fail(
                        b"buffer[len] == NUL || buffer[len] == '\\n'\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/os/shell.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        487 as ::core::ffi::c_uint,
                        b"int os_expand_wildcards(int, char **, int *, char ***, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            if i == 0 as ::core::ffi::c_int {
                xfree(buffer as *mut ::core::ffi::c_void);
            } else {
                *num_file = i;
                *file = xmalloc(
                    ::core::mem::size_of::<*mut ::core::ffi::c_char>().wrapping_mul(i as size_t),
                ) as *mut *mut ::core::ffi::c_char;
                p = buffer;
                i = 0 as ::core::ffi::c_int;
                while i < *num_file {
                    *(*file).offset(i as isize) = p;
                    if shell_style == STYLE_ECHO
                        || shell_style == STYLE_BT
                        || shell_style == STYLE_VIMGLOB
                        || shell_style == STYLE_GLOBSTAR
                    {
                        while !(shell_style == STYLE_ECHO
                            && *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int)
                            && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                            && *p as ::core::ffi::c_int != NUL
                        {
                            p = p.offset(1);
                        }
                        if p == buffer.offset(len as isize) {
                            *p = NUL as ::core::ffi::c_char;
                        } else {
                            let c2rust_fresh6 = p;
                            p = p.offset(1);
                            *c2rust_fresh6 = NUL as ::core::ffi::c_char;
                            p = skipwhite(p);
                        }
                    } else {
                        while *p as ::core::ffi::c_int != 0 && p < buffer.offset(len as isize) {
                            p = p.offset(1);
                        }
                        p = p.offset(1);
                    }
                    i += 1;
                }
                j = 0 as ::core::ffi::c_int;
                i = 0 as ::core::ffi::c_int;
                while i < *num_file {
                    if !(flags & EW_NOTFOUND as ::core::ffi::c_int == 0
                        && !os_path_exists(*(*file).offset(i as isize)))
                    {
                        let mut dir: bool = os_isdir(*(*file).offset(i as isize));
                        if !(dir as ::core::ffi::c_int != 0
                            && flags & EW_DIR as ::core::ffi::c_int == 0
                            || !dir && flags & EW_FILE as ::core::ffi::c_int == 0)
                        {
                            if !(!dir
                                && flags & EW_EXEC as ::core::ffi::c_int != 0
                                && !os_can_exe(
                                    *(*file).offset(i as isize),
                                    ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                                    flags & EW_SHELLCMD as ::core::ffi::c_int == 0,
                                ))
                            {
                                p = xmalloc(
                                    strlen(*(*file).offset(i as isize))
                                        .wrapping_add(1 as size_t)
                                        .wrapping_add(dir as size_t),
                                ) as *mut ::core::ffi::c_char;
                                strcpy(p, *(*file).offset(i as isize));
                                if dir {
                                    add_pathsep(p);
                                }
                                let c2rust_fresh7 = j;
                                j = j + 1;
                                let c2rust_lvalue_ptr =
                                    &raw mut *(*file).offset(c2rust_fresh7 as isize);
                                *c2rust_lvalue_ptr = p;
                            }
                        }
                    }
                    i += 1;
                }
                xfree(buffer as *mut ::core::ffi::c_void);
                *num_file = j;
                if *num_file == 0 as ::core::ffi::c_int {
                    let mut ptr_: *mut *mut ::core::ffi::c_void =
                        file as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL;
                    *ptr_;
                } else {
                    return OK;
                }
            }
        }
    }
    if flags & EW_NOTFOUND as ::core::ffi::c_int != 0 {
        save_patterns(num_pat, pat, num_file, file);
        return OK;
    }
    return FAIL;
}
pub const STYLE_ECHO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const STYLE_GLOB: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STYLE_VIMGLOB: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const STYLE_PRINT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const STYLE_BT: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const STYLE_GLOBSTAR: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn shell_build_argv(
    mut cmd: *const ::core::ffi::c_char,
    mut extra_args: *const ::core::ffi::c_char,
) -> *mut *mut ::core::ffi::c_char {
    let mut argc: size_t = tokenize(p_sh, ::core::ptr::null_mut::<*mut ::core::ffi::c_char>())
        .wrapping_add(if !cmd.is_null() {
            tokenize(p_shcf, ::core::ptr::null_mut::<*mut ::core::ffi::c_char>())
        } else {
            0 as size_t
        });
    let mut rv: *mut *mut ::core::ffi::c_char = xmalloc(
        argc.wrapping_add(4 as size_t)
            .wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
    ) as *mut *mut ::core::ffi::c_char;
    let mut i: size_t = tokenize(p_sh, rv);
    if !extra_args.is_null() {
        let c2rust_fresh8 = i;
        i = i.wrapping_add(1);
        let c2rust_lvalue_ptr = &raw mut *rv.offset(c2rust_fresh8 as isize);
        *c2rust_lvalue_ptr = xstrdup(extra_args);
    }
    if !cmd.is_null() {
        i = i.wrapping_add(tokenize(p_shcf, rv.offset(i as isize)));
        let c2rust_fresh9 = i;
        i = i.wrapping_add(1);
        let c2rust_lvalue_ptr_0 = &raw mut *rv.offset(c2rust_fresh9 as isize);
        *c2rust_lvalue_ptr_0 = shell_xescape_xquote(cmd);
    }
    *rv.offset(i as isize) = ::core::ptr::null_mut::<::core::ffi::c_char>();
    '_c2rust_label: {
        if !(*rv.offset(0 as ::core::ffi::c_int as isize)).is_null() {
        } else {
            __assert_fail(
                b"rv[0]\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/shell.rs\0".as_ptr() as *const ::core::ffi::c_char,
                596 as ::core::ffi::c_uint,
                b"char **shell_build_argv(const char *, const char *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn shell_free_argv(mut argv: *mut *mut ::core::ffi::c_char) {
    let mut p: *mut *mut ::core::ffi::c_char = argv;
    if p.is_null() {
        return;
    }
    while !(*p).is_null() {
        xfree(*p as *mut ::core::ffi::c_void);
        p = p.offset(1);
    }
    xfree(argv as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn shell_argv_to_str(
    argv: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut n: size_t = 0 as size_t;
    let mut p: *mut *mut ::core::ffi::c_char = argv;
    let mut rv: *mut ::core::ffi::c_char =
        xcalloc(256 as size_t, ::core::mem::size_of::<::core::ffi::c_char>())
            as *mut ::core::ffi::c_char;
    let maxsize: size_t =
        (256 as size_t).wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>());
    if (*p).is_null() {
        return rv;
    }
    while !(*p).is_null() {
        xstrlcat(rv, b"'\0".as_ptr() as *const ::core::ffi::c_char, maxsize);
        xstrlcat(rv, *p, maxsize);
        n = xstrlcat(rv, b"' \0".as_ptr() as *const ::core::ffi::c_char, maxsize);
        if n >= maxsize {
            break;
        }
        p = p.offset(1);
    }
    if n < maxsize {
        *rv.offset(n.wrapping_sub(1 as size_t) as isize) = NUL as ::core::ffi::c_char;
    } else {
        *rv.offset(maxsize.wrapping_sub(4 as size_t) as isize) = '.' as ::core::ffi::c_char;
        *rv.offset(maxsize.wrapping_sub(3 as size_t) as isize) = '.' as ::core::ffi::c_char;
        *rv.offset(maxsize.wrapping_sub(2 as size_t) as isize) = '.' as ::core::ffi::c_char;
        *rv.offset(maxsize.wrapping_sub(1 as size_t) as isize) = NUL as ::core::ffi::c_char;
    }
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn os_call_shell(
    mut cmd: *mut ::core::ffi::c_char,
    mut opts: ::core::ffi::c_int,
    mut extra_args: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut input: StringBuilder = KV_INITIAL_VALUE;
    let mut output: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut output_ptr: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut current_state: ::core::ffi::c_int = State;
    let mut forward_output: bool = true_0 != 0;
    signal_reject_deadly();
    if opts & (kShellOptHideMess as ::core::ffi::c_int | kShellOptExpand as ::core::ffi::c_int) != 0
    {
        forward_output = false_0 != 0;
    } else {
        State = MODE_EXTERNCMD as ::core::ffi::c_int;
        if opts & kShellOptWrite as ::core::ffi::c_int != 0 {
            read_input(&raw mut input);
        }
        if opts & kShellOptRead as ::core::ffi::c_int != 0 {
            output_ptr = &raw mut output;
            forward_output = false_0 != 0;
        } else if opts & kShellOptDoOut as ::core::ffi::c_int != 0 {
            forward_output = false_0 != 0;
        }
    }
    let mut nread: size_t = 0;
    let mut exitcode: ::core::ffi::c_int = do_os_system(
        shell_build_argv(cmd, extra_args),
        input.items,
        input.size,
        output_ptr,
        &raw mut nread,
        emsg_silent != 0,
        forward_output,
    );
    xfree(input.items as *mut ::core::ffi::c_void);
    input.capacity = 0 as size_t;
    input.size = input.capacity;
    input.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !output.is_null() {
        write_output(output, nread, true_0 != 0);
        xfree(output as *mut ::core::ffi::c_void);
    }
    if emsg_silent == 0
        && exitcode != 0 as ::core::ffi::c_int
        && opts & kShellOptSilent as ::core::ffi::c_int == 0
    {
        msg_ext_set_kind(b"shell_ret\0".as_ptr() as *const ::core::ffi::c_char);
        if !ui_has(kUIMessages) {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        msg_puts(gettext(
            b"shell returned \0".as_ptr() as *const ::core::ffi::c_char
        ));
        msg_outnum(exitcode);
    }
    State = current_state;
    signal_accept_deadly();
    return exitcode;
}
#[no_mangle]
pub unsafe extern "C" fn call_shell(
    mut cmd: *mut ::core::ffi::c_char,
    mut opts: ::core::ffi::c_int,
    mut extra_shell_arg: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = 0;
    let mut wait_time: proftime_T = 0;
    if p_verbose > 3 as OptInt {
        verbose_enter();
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"Executing command: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
            if cmd.is_null() { p_sh } else { cmd },
        );
        msg_putchar('\n' as ::core::ffi::c_int);
        verbose_leave();
    }
    if do_profiling == PROF_YES {
        prof_child_enter(&raw mut wait_time);
    }
    if *p_sh as ::core::ffi::c_int == NUL {
        emsg(gettext(
            &raw const e_shellempty as *const ::core::ffi::c_char,
        ));
        retval = -1 as ::core::ffi::c_int;
    } else {
        tag_freematch();
        retval = os_call_shell(cmd, opts, extra_shell_arg);
    }
    set_vim_var_nr(VV_SHELL_ERROR, retval as varnumber_T);
    if do_profiling == PROF_YES {
        prof_child_exit(&raw mut wait_time);
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn get_cmd_output(
    mut cmd: *mut ::core::ffi::c_char,
    mut infile: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut ret_len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut len: size_t = 0;
    let mut i: size_t = 0;
    let mut buffer: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if check_secure() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut tempname: *mut ::core::ffi::c_char = vim_tempname();
    if tempname.is_null() {
        emsg(gettext(&raw const e_notmp as *const ::core::ffi::c_char));
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut command: *mut ::core::ffi::c_char =
        make_filter_cmd(cmd, infile, tempname, false_0 != 0);
    no_check_timestamps += 1;
    call_shell(
        command,
        kShellOptDoOut as ::core::ffi::c_int | kShellOptExpand as ::core::ffi::c_int | flags,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
    no_check_timestamps -= 1;
    xfree(command as *mut ::core::ffi::c_void);
    let mut fd: *mut FILE = os_fopen(tempname, READBIN.as_ptr());
    let mut len_l: ::core::ffi::c_long = 0;
    if fd.is_null()
        || fseek(fd, 0 as ::core::ffi::c_long, SEEK_END) == -1 as ::core::ffi::c_int
        || {
            len_l = ftell(fd);
            len_l == -1 as ::core::ffi::c_long
        }
        || fseek(fd, 0 as ::core::ffi::c_long, SEEK_SET) == -1 as ::core::ffi::c_int
    {
        semsg(
            gettext(&raw const e_cannot_read_from_str_2 as *const ::core::ffi::c_char),
            tempname,
        );
        if !fd.is_null() {
            fclose(fd);
        }
    } else {
        len = len_l as size_t;
        buffer = xmalloc(len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        i = fread(buffer as *mut ::core::ffi::c_void, 1 as size_t, len, fd) as size_t;
        fclose(fd);
        os_remove(tempname);
        if i != len {
            semsg(
                gettext(&raw const e_cant_read_file_str as *const ::core::ffi::c_char),
                tempname,
            );
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut buffer as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            *ptr_;
        } else if ret_len.is_null() {
            i = 0 as size_t;
            while i < len {
                if *buffer.offset(i as isize) as ::core::ffi::c_int == NUL {
                    *buffer.offset(i as isize) = 1 as ::core::ffi::c_char;
                }
                i = i.wrapping_add(1);
            }
            *buffer.offset(len as isize) = NUL as ::core::ffi::c_char;
        } else {
            *ret_len = len;
        }
    }
    xfree(tempname as *mut ::core::ffi::c_void);
    return buffer;
}
#[no_mangle]
pub unsafe extern "C" fn os_system(
    mut argv: *mut *mut ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
    mut len: size_t,
    mut output: *mut *mut ::core::ffi::c_char,
    mut nread: *mut size_t,
) -> ::core::ffi::c_int {
    return do_os_system(argv, input, len, output, nread, true_0 != 0, false_0 != 0);
}
unsafe extern "C" fn do_os_system(
    mut argv: *mut *mut ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
    mut len: size_t,
    mut output: *mut *mut ::core::ffi::c_char,
    mut nread: *mut size_t,
    mut silent: bool,
    mut forward_output: bool,
) -> ::core::ffi::c_int {
    let mut exitcode: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    out_data_decide_throttle(0 as size_t);
    out_data_ring(::core::ptr::null::<::core::ffi::c_char>(), 0 as size_t);
    let mut has_input: bool = !input.is_null() && len > 0 as size_t;
    let mut buf: StringBuilder = KV_INITIAL_VALUE;
    let mut data_cb: stream_read_cb = Some(
        system_data_cb
            as unsafe extern "C" fn(
                *mut RStream,
                *const ::core::ffi::c_char,
                size_t,
                *mut ::core::ffi::c_void,
                bool,
            ) -> size_t,
    );
    if !nread.is_null() {
        *nread = 0 as size_t;
    }
    if forward_output {
        data_cb = Some(
            out_data_cb
                as unsafe extern "C" fn(
                    *mut RStream,
                    *const ::core::ffi::c_char,
                    size_t,
                    *mut ::core::ffi::c_void,
                    bool,
                ) -> size_t,
        ) as stream_read_cb;
    } else if output.is_null() {
        data_cb = None;
    }
    let mut prog: [::core::ffi::c_char; 4096] = [0; 4096];
    xstrlcpy(
        &raw mut prog as *mut ::core::ffi::c_char,
        *argv.offset(0 as ::core::ffi::c_int as isize),
        MAXPATHL as size_t,
    );
    let mut uvproc: LibuvProc =
        libuv_proc_init(&raw mut main_loop, &raw mut buf as *mut ::core::ffi::c_void);
    let mut proc: *mut Proc = &raw mut uvproc.proc;
    let mut events: *mut MultiQueue = multiqueue_new_child(main_loop.events);
    (*proc).events = events;
    (*proc).argv = argv;
    let mut status: ::core::ffi::c_int = proc_spawn(proc, has_input, true_0 != 0, true_0 != 0);
    '_end: {
        if status != 0 {
            loop_poll_events(&raw mut main_loop, 0 as int64_t);
            if !silent {
                msg_puts(gettext(
                    b"\nshell failed to start: \0".as_ptr() as *const ::core::ffi::c_char
                ));
                msg_outtrans(uv_strerror(status), 0 as ::core::ffi::c_int, false_0 != 0);
                msg_puts(b": \0".as_ptr() as *const ::core::ffi::c_char);
                msg_outtrans(
                    &raw mut prog as *mut ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
                msg_putchar('\n' as ::core::ffi::c_int);
            }
        } else {
            if has_input {
                wstream_init(&raw mut (*proc).in_0, 0 as size_t);
            }
            rstream_init(&raw mut (*proc).out);
            rstream_start(
                &raw mut (*proc).out,
                data_cb,
                &raw mut buf as *mut ::core::ffi::c_void,
            );
            rstream_init(&raw mut (*proc).err);
            rstream_start(
                &raw mut (*proc).err,
                data_cb,
                &raw mut buf as *mut ::core::ffi::c_void,
            );
            if has_input {
                let mut input_buffer: *mut WBuffer =
                    wstream_new_buffer(input as *mut ::core::ffi::c_char, len, 1 as size_t, None);
                if wstream_write(&raw mut (*proc).in_0, input_buffer) != 0 as ::core::ffi::c_int {
                    proc_stop(proc);
                    break '_end;
                } else {
                    wstream_set_write_cb(
                        &raw mut (*proc).in_0,
                        Some(
                            shell_write_cb
                                as unsafe extern "C" fn(
                                    *mut Stream,
                                    *mut ::core::ffi::c_void,
                                    ::core::ffi::c_int,
                                ) -> (),
                        ),
                        NULL,
                    );
                }
            }
            ui_busy_start();
            ui_flush();
            if forward_output {
                msg_sb_eol();
                msg_start();
                msg_no_more = true_0 != 0;
                lines_left = -1 as ::core::ffi::c_int;
            }
            exitcode = proc_wait(
                proc,
                -1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<MultiQueue>(),
            );
            if !got_int && out_data_decide_throttle(0 as size_t) as ::core::ffi::c_int != 0 {
                out_data_ring(
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    SIZE_MAX as size_t,
                );
            }
            if forward_output {
                no_wait_return += 1;
                msg_end();
                no_wait_return -= 1;
                msg_no_more = false_0 != 0;
            }
            ui_busy_stop();
            if !output.is_null() {
                '_c2rust_label: {
                    if !nread.is_null() {
                    } else {
                        __assert_fail(
                            b"nread\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/os/shell.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            971 as ::core::ffi::c_uint,
                            b"int do_os_system(char **, const char *, size_t, char **, size_t *, _Bool, _Bool)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if buf.size == 0 as size_t {
                    *output = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    *nread = 0 as size_t;
                    xfree(buf.items as *mut ::core::ffi::c_void);
                    buf.capacity = 0 as size_t;
                    buf.size = buf.capacity;
                    buf.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
                } else {
                    *nread = buf.size;
                    if buf.size == buf.capacity {
                        buf.capacity = if buf.capacity != 0 {
                            buf.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        buf.items = xrealloc(
                            buf.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<::core::ffi::c_char>()
                                .wrapping_mul(buf.capacity),
                        ) as *mut ::core::ffi::c_char;
                    } else {
                    };
                    let c2rust_fresh10 = buf.size;
                    buf.size = buf.size.wrapping_add(1);
                    *buf.items.offset(c2rust_fresh10 as isize) = '\0' as ::core::ffi::c_char;
                    *output = buf.items;
                }
            }
            '_c2rust_label_0: {
                if multiqueue_empty(events) {
                } else {
                    __assert_fail(
                        b"multiqueue_empty(events)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/os/shell.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        985 as ::core::ffi::c_uint,
                        b"int do_os_system(char **, const char *, size_t, char **, size_t *, _Bool, _Bool)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
        }
    }
    multiqueue_free(events);
    return exitcode;
}
unsafe extern "C" fn system_data_cb(
    mut _stream: *mut RStream,
    mut buf: *const ::core::ffi::c_char,
    mut count: size_t,
    mut data: *mut ::core::ffi::c_void,
    mut _eof: bool,
) -> size_t {
    let mut dbuf: *mut StringBuilder = data as *mut StringBuilder;
    if count > 0 as size_t {
        if (*dbuf).capacity < (*dbuf).size.wrapping_add(count) {
            (*dbuf).capacity = (*dbuf).size.wrapping_add(count);
            (*dbuf).capacity = (*dbuf).capacity.wrapping_sub(1);
            (*dbuf).capacity |= (*dbuf).capacity >> 1 as ::core::ffi::c_int;
            (*dbuf).capacity |= (*dbuf).capacity >> 2 as ::core::ffi::c_int;
            (*dbuf).capacity |= (*dbuf).capacity >> 4 as ::core::ffi::c_int;
            (*dbuf).capacity |= (*dbuf).capacity >> 8 as ::core::ffi::c_int;
            (*dbuf).capacity |= (*dbuf).capacity >> 16 as ::core::ffi::c_int;
            (*dbuf).capacity = (*dbuf).capacity.wrapping_add(1);
            (*dbuf).capacity = (*dbuf).capacity;
            (*dbuf).items = xrealloc(
                (*dbuf).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*dbuf).capacity),
            ) as *mut ::core::ffi::c_char;
        }
        '_c2rust_label: {
            if !(*dbuf).items.is_null() {
            } else {
                __assert_fail(
                    b"(*dbuf).items\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/os/shell.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1002 as ::core::ffi::c_uint,
                    b"size_t system_data_cb(RStream *, const char *, size_t, void *, _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        memcpy(
            (*dbuf).items.offset((*dbuf).size as isize) as *mut ::core::ffi::c_void,
            buf as *const ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(count),
        );
        (*dbuf).size = (*dbuf).size.wrapping_add(count);
    }
    return count;
}
unsafe extern "C" fn out_data_decide_throttle(mut size: size_t) -> bool {
    static mut started: uint64_t = 0 as uint64_t;
    static mut received: size_t = 0 as size_t;
    static mut visit: size_t = 0 as size_t;
    static mut pulse_msg: [::core::ffi::c_char; 4] = [
        ' ' as ::core::ffi::c_char,
        ' ' as ::core::ffi::c_char,
        ' ' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
    ];
    if size == 0 {
        let mut previous_decision: bool = visit > 0 as size_t;
        visit = 0 as size_t;
        received = visit;
        started = received as uint64_t;
        return previous_decision;
    }
    received = received.wrapping_add(size);
    if received < OUT_DATA_THRESHOLD as size_t
        || started == 0 && received < size.wrapping_add(1000 as size_t)
    {
        return false_0 != 0;
    } else if visit == 0 {
        started = os_hrtime();
    } else {
        let mut since: uint64_t = os_hrtime().wrapping_sub(started);
        if since
            < (visit as uint64_t)
                .wrapping_mul(NS_1_SECOND.wrapping_div(10 as ::core::ffi::c_uint) as uint64_t)
        {
            return true_0 != 0;
        }
        if since > (3 as ::core::ffi::c_uint).wrapping_mul(NS_1_SECOND) as uint64_t {
            visit = 0 as size_t;
            received = visit;
            return false_0 != 0;
        }
    }
    visit = visit.wrapping_add(1);
    let mut tick: size_t = visit.wrapping_rem(4 as size_t);
    pulse_msg[0 as ::core::ffi::c_int as usize] = (if tick > 0 as size_t {
        '.' as ::core::ffi::c_int
    } else {
        ' ' as ::core::ffi::c_int
    }) as ::core::ffi::c_char;
    pulse_msg[1 as ::core::ffi::c_int as usize] = (if tick > 1 as size_t {
        '.' as ::core::ffi::c_int
    } else {
        ' ' as ::core::ffi::c_int
    }) as ::core::ffi::c_char;
    pulse_msg[2 as ::core::ffi::c_int as usize] = (if tick > 2 as size_t {
        '.' as ::core::ffi::c_int
    } else {
        ' ' as ::core::ffi::c_int
    }) as ::core::ffi::c_char;
    if visit == 1 as size_t {
        msg_puts(b"...\n\0".as_ptr() as *const ::core::ffi::c_char);
    }
    msg_putchar('\r' as ::core::ffi::c_int);
    msg_puts(&raw mut pulse_msg as *mut ::core::ffi::c_char);
    msg_putchar('\r' as ::core::ffi::c_int);
    ui_flush();
    return true_0 != 0;
}
unsafe extern "C" fn out_data_ring(mut output: *const ::core::ffi::c_char, mut size: size_t) {
    static mut last_skipped: [::core::ffi::c_char; 5120] = [0; 5120];
    static mut last_skipped_len: size_t = 0 as size_t;
    '_c2rust_label: {
        if !output.is_null() || (size == 0 as size_t || size == 18446744073709551615 as size_t) {
        } else {
            __assert_fail(
                b"output != NULL || (size == 0 || size == SIZE_MAX)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/os/shell.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1092 as ::core::ffi::c_uint,
                b"void out_data_ring(const char *, size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if output.is_null() && size == 0 as size_t {
        last_skipped_len = 0 as size_t;
        return;
    }
    if output.is_null() && size == SIZE_MAX as size_t {
        out_data_append_to_screen(
            &raw mut last_skipped as *mut ::core::ffi::c_char,
            &raw mut last_skipped_len,
            STDOUT_FILENO,
            true_0 != 0,
        );
        return;
    }
    if size >= MAX_CHUNK_SIZE as size_t {
        let mut start: size_t = size.wrapping_sub(MAX_CHUNK_SIZE as size_t);
        memcpy(
            &raw mut last_skipped as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            output.offset(start as isize) as *const ::core::ffi::c_void,
            MAX_CHUNK_SIZE as size_t,
        );
        last_skipped_len = MAX_CHUNK_SIZE as size_t;
    } else if size > 0 as size_t {
        let mut keep_len: size_t = if last_skipped_len
            < ((1024 as ::core::ffi::c_uint)
                .wrapping_mul(10 as ::core::ffi::c_uint)
                .wrapping_div(2 as ::core::ffi::c_uint) as size_t)
                .wrapping_sub(size)
        {
            last_skipped_len
        } else {
            ((1024 as ::core::ffi::c_uint)
                .wrapping_mul(10 as ::core::ffi::c_uint)
                .wrapping_div(2 as ::core::ffi::c_uint) as size_t)
                .wrapping_sub(size)
        };
        let mut keep_start: size_t = last_skipped_len.wrapping_sub(keep_len);
        if keep_start != 0 {
            memmove(
                &raw mut last_skipped as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                (&raw mut last_skipped as *mut ::core::ffi::c_char).offset(keep_start as isize)
                    as *const ::core::ffi::c_void,
                keep_len,
            );
        }
        memcpy(
            (&raw mut last_skipped as *mut ::core::ffi::c_char).offset(keep_len as isize)
                as *mut ::core::ffi::c_void,
            output as *const ::core::ffi::c_void,
            size,
        );
        last_skipped_len = keep_len.wrapping_add(size);
    }
}
pub const MAX_CHUNK_SIZE: ::core::ffi::c_uint =
    OUT_DATA_THRESHOLD.wrapping_div(2 as ::core::ffi::c_uint);
unsafe extern "C" fn out_data_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut need_clear: bool = true_0 != 0;
    let mut hl: ::core::ffi::c_int = if (*argv.offset(2 as ::core::ffi::c_int as isize))
        .expose_addr() as intptr_t as ::core::ffi::c_int
        == STDERR_FILENO
    {
        HLF_SE as ::core::ffi::c_int
    } else {
        HLF_SO as ::core::ffi::c_int
    };
    msg_ext_set_kind(
        if (*argv.offset(2 as ::core::ffi::c_int as isize)).expose_addr() as intptr_t
            as ::core::ffi::c_int
            == STDERR_FILENO
        {
            b"shell_err\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"shell_out\0".as_ptr() as *const ::core::ffi::c_char
        },
    );
    msg_ext_set_append(true_0 != 0);
    msg_multiline(
        String_0 {
            data: *argv.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char,
            size: (*argv.offset(1 as ::core::ffi::c_int as isize)).expose_addr() as size_t,
        },
        hl,
        false_0 != 0,
        false_0 != 0,
        &raw mut need_clear,
    );
    xfree(*argv.offset(0 as ::core::ffi::c_int as isize));
    ui_flush();
}
unsafe extern "C" fn out_data_append_to_screen(
    mut output: *const ::core::ffi::c_char,
    mut count: *mut size_t,
    mut fd: ::core::ffi::c_int,
    mut eof: bool,
) {
    let mut p: *const ::core::ffi::c_char = output;
    let mut end: *const ::core::ffi::c_char = output.offset(*count as isize);
    while p < end {
        let mut i: ::core::ffi::c_int = if *p as ::core::ffi::c_int != 0 {
            utfc_ptr2len_len(
                p,
                *count as ::core::ffi::c_int - p.offset_from(output) as ::core::ffi::c_int,
            )
        } else {
            1 as ::core::ffi::c_int
        };
        if !eof
            && i == 1 as ::core::ffi::c_int
            && utf8len_tab_zero[*(p as *mut uint8_t) as usize] as isize > end.offset_from(p)
        {
            *count = p.offset_from(output) as size_t;
            break;
        } else {
            p = p.offset(i as isize);
        }
    }
    let mut str: *mut ::core::ffi::c_char =
        xmemdupz(output as *const ::core::ffi::c_void, *count) as *mut ::core::ffi::c_char;
    if ui_has(kUIMessages) {
        multiqueue_put_event(
            main_loop.fast_events,
            Event {
                handler: Some(
                    out_data_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    str as *mut ::core::ffi::c_void,
                    ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(*count as usize),
                    ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                        fd as intptr_t as usize,
                    ),
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
    } else {
        let mut c2rust_lvalue: [*mut ::core::ffi::c_void; 3] = [
            str as *mut ::core::ffi::c_void,
            ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(*count as usize),
            ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(fd as intptr_t as usize),
        ];
        out_data_event(&raw mut c2rust_lvalue as *mut *mut ::core::ffi::c_void);
    };
}
unsafe extern "C" fn out_data_cb(
    mut stream: *mut RStream,
    mut ptr: *const ::core::ffi::c_char,
    mut count: size_t,
    mut _data: *mut ::core::ffi::c_void,
    mut eof: bool,
) -> size_t {
    if count > 0 as size_t && out_data_decide_throttle(count) as ::core::ffi::c_int != 0 {
        out_data_ring(ptr, count);
    } else if count > 0 as size_t {
        out_data_append_to_screen(
            ptr,
            &raw mut count,
            (*stream).s.fd as ::core::ffi::c_int,
            eof,
        );
    }
    return count;
}
unsafe extern "C" fn tokenize(
    str: *const ::core::ffi::c_char,
    argv: *mut *mut ::core::ffi::c_char,
) -> size_t {
    let mut argc: size_t = 0 as size_t;
    let mut p: *const ::core::ffi::c_char = str;
    while *p as ::core::ffi::c_int != NUL {
        let len: size_t = word_length(p);
        if !argv.is_null() {
            *argv.offset(argc as isize) = vim_strnsave_unquoted(p, len);
        }
        argc = argc.wrapping_add(1);
        p = skipwhite(p.offset(len as isize));
    }
    return argc;
}
unsafe extern "C" fn word_length(mut str: *const ::core::ffi::c_char) -> size_t {
    let mut p: *const ::core::ffi::c_char = str;
    let mut inquote: bool = false_0 != 0;
    let mut length: size_t = 0 as size_t;
    while *p as ::core::ffi::c_int != 0
        && (inquote as ::core::ffi::c_int != 0
            || *p as ::core::ffi::c_int != ' ' as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != TAB)
    {
        if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
            inquote = !inquote;
        } else if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && inquote as ::core::ffi::c_int != 0
        {
            p = p.offset(1);
            length = length.wrapping_add(1);
        }
        p = p.offset(1);
        length = length.wrapping_add(1);
    }
    return length;
}
unsafe extern "C" fn read_input(mut buf: *mut StringBuilder) {
    read_buffer_into(
        curbuf,
        (*curbuf).b_op_start.lnum,
        (*curbuf).b_op_end.lnum,
        buf,
    );
}
unsafe extern "C" fn write_output(
    mut output: *mut ::core::ffi::c_char,
    mut remaining: size_t,
    mut eof: bool,
) -> size_t {
    if output.is_null() {
        return 0 as size_t;
    }
    let mut start: *mut ::core::ffi::c_char = output;
    let mut off: size_t = 0 as size_t;
    while off < remaining {
        if *output.offset(off as isize) as ::core::ffi::c_int == CAR
            && *output.offset(off.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int == NL
            && (*curbuf).b_p_bin == 0
        {
            *output.offset(off as isize) = NUL as ::core::ffi::c_char;
            let c2rust_fresh11 = (*curwin).w_cursor.lnum;
            (*curwin).w_cursor.lnum = (*curwin).w_cursor.lnum + 1;
            ml_append(
                c2rust_fresh11,
                output,
                off as colnr_T + 1 as colnr_T,
                false_0 != 0,
            );
            let mut skip: size_t = off.wrapping_add(2 as size_t);
            output = output.offset(skip as isize);
            remaining = remaining.wrapping_sub(skip);
            off = 0 as size_t;
        } else if *output.offset(off as isize) as ::core::ffi::c_int == CAR
            && (*curbuf).b_p_bin == 0
            || *output.offset(off as isize) as ::core::ffi::c_int == NL
        {
            *output.offset(off as isize) = NUL as ::core::ffi::c_char;
            let c2rust_fresh12 = (*curwin).w_cursor.lnum;
            (*curwin).w_cursor.lnum = (*curwin).w_cursor.lnum + 1;
            ml_append(
                c2rust_fresh12,
                output,
                off as colnr_T + 1 as colnr_T,
                false_0 != 0,
            );
            let mut skip_0: size_t = off.wrapping_add(1 as size_t);
            output = output.offset(skip_0 as isize);
            remaining = remaining.wrapping_sub(skip_0);
            off = 0 as size_t;
        } else {
            if *output.offset(off as isize) as ::core::ffi::c_int == NUL {
                *output.offset(off as isize) = NL as ::core::ffi::c_char;
            }
            off = off.wrapping_add(1);
        }
    }
    if eof {
        if remaining != 0 {
            let c2rust_fresh13 = (*curwin).w_cursor.lnum;
            (*curwin).w_cursor.lnum = (*curwin).w_cursor.lnum + 1;
            ml_append(c2rust_fresh13, output, 0 as colnr_T, false_0 != 0);
            (*curbuf).b_no_eol_lnum = (*curwin).w_cursor.lnum;
            output = output.offset(remaining as isize);
        } else {
            (*curbuf).b_no_eol_lnum = 0 as ::core::ffi::c_int as linenr_T;
        }
    }
    ui_flush();
    return output.offset_from(start) as size_t;
}
unsafe extern "C" fn shell_write_cb(
    mut stream: *mut Stream,
    mut _data: *mut ::core::ffi::c_void,
    mut status: ::core::ffi::c_int,
) {
    if status != 0 {
        msg_schedule_semsg(
            gettext(
                b"E5677: Error writing input to shell-command: %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            uv_err_name(status),
        );
    }
    stream_may_close(stream);
}
unsafe extern "C" fn shell_xescape_xquote(
    mut cmd: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if *p_sxq as ::core::ffi::c_int == NUL {
        return xstrdup(cmd);
    }
    let mut ecmd: *const ::core::ffi::c_char = cmd;
    if *p_sxe as ::core::ffi::c_int != NUL
        && strcmp(p_sxq, b"(\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        ecmd = vim_strsave_escaped_ext(cmd, p_sxe, '^' as ::core::ffi::c_char, false_0 != 0);
    }
    let mut ncmd_size: size_t = strlen(ecmd)
        .wrapping_add(strlen(p_sxq).wrapping_mul(2 as size_t))
        .wrapping_add(1 as size_t);
    let mut ncmd: *mut ::core::ffi::c_char = xmalloc(ncmd_size) as *mut ::core::ffi::c_char;
    if strcmp(p_sxq, b"(\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        vim_snprintf(
            ncmd,
            ncmd_size,
            b"(%s)\0".as_ptr() as *const ::core::ffi::c_char,
            ecmd,
        );
    } else if strcmp(p_sxq, b"\"(\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        vim_snprintf(
            ncmd,
            ncmd_size,
            b"\"(%s)\"\0".as_ptr() as *const ::core::ffi::c_char,
            ecmd,
        );
    } else {
        vim_snprintf(
            ncmd,
            ncmd_size,
            b"%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
            p_sxq,
            ecmd,
            p_sxq,
        );
    }
    if ecmd != cmd {
        xfree(ecmd as *mut ::core::ffi::c_void);
    }
    return ncmd;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const READBIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"rb\0") };
