extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn fcntl(__fd: ::core::ffi::c_int, __cmd: ::core::ffi::c_int, ...) -> ::core::ffi::c_int;
    fn fdopen(__fd: ::core::ffi::c_int, __modes: *const ::core::ffi::c_char) -> *mut FILE;
    fn abort() -> !;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strerror(__errnum: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn uv_translate_sys_error(sys_errno: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn uv_strerror(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn uv_fs_req_cleanup(req: *mut uv_fs_t);
    fn uv_fs_close(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        file: uv_file,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_open(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        mode: ::core::ffi::c_int,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_unlink(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_copyfile(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        new_path: *const ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_mkdir(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        mode: ::core::ffi::c_int,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_mkdtemp(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        tpl: *const ::core::ffi::c_char,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_rmdir(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_scandir(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_scandir_next(req: *mut uv_fs_t, ent: *mut uv_dirent_t) -> ::core::ffi::c_int;
    fn uv_fs_stat(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_fstat(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        file: uv_file,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_rename(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        new_path: *const ::core::ffi::c_char,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_fsync(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        file: uv_file,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_access(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        mode: ::core::ffi::c_int,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_chmod(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        mode: ::core::ffi::c_int,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_utime(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        atime: ::core::ffi::c_double,
        mtime: ::core::ffi::c_double,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_lstat(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_realpath(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_chown(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        uid: uv_uid_t,
        gid: uv_gid_t,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_fchown(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        file: uv_file,
        uid: uv_uid_t,
        gid: uv_gid_t,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn uv_exepath(buffer: *mut ::core::ffi::c_char, size: *mut size_t) -> ::core::ffi::c_int;
    fn uv_cwd(buffer: *mut ::core::ffi::c_char, size: *mut size_t) -> ::core::ffi::c_int;
    fn uv_chdir(dir: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn read(__fd: ::core::ffi::c_int, __buf: *mut ::core::ffi::c_void, __nbytes: size_t)
        -> ssize_t;
    fn write(__fd: ::core::ffi::c_int, __buf: *const ::core::ffi::c_void, __n: size_t) -> ssize_t;
    fn dup(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn getuid() -> __uid_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xmemcpyz(
        dst: *mut ::core::ffi::c_void,
        src: *const ::core::ffi::c_void,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xstrchrnul(
        str: *const ::core::ffi::c_char,
        c: ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn memcnt(data: *const ::core::ffi::c_void, c: ::core::ffi::c_char, len: size_t) -> size_t;
    fn xstrlcpy(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static e_mkdir: [::core::ffi::c_char; 0];
    static e_noname: [::core::ffi::c_char; 0];
    fn setxattr(
        __path: *const ::core::ffi::c_char,
        __name: *const ::core::ffi::c_char,
        __value: *const ::core::ffi::c_void,
        __size: size_t,
        __flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn getxattr(
        __path: *const ::core::ffi::c_char,
        __name: *const ::core::ffi::c_char,
        __value: *mut ::core::ffi::c_void,
        __size: size_t,
    ) -> ssize_t;
    fn listxattr(
        __path: *const ::core::ffi::c_char,
        __list: *mut ::core::ffi::c_char,
        __size: size_t,
    ) -> ssize_t;
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    static mut g_stats: nvim_stats_s;
    static mut stdin_fd: ::core::ffi::c_int;
    fn smsg(hl_id: ::core::ffi::c_int, s: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn verbose_enter();
    fn verbose_leave();
    static mut p_verbose: OptInt;
    fn os_getenv(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn path_tail_with_sep(fname: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn get_past_head(path: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_ispathsep(c: ::core::ffi::c_int) -> bool;
    fn dir_of_file_exists(fname: *mut ::core::ffi::c_char) -> bool;
    fn FullName_save(fname: *const ::core::ffi::c_char, force: bool) -> *mut ::core::ffi::c_char;
    fn save_abs_path(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn gettail_dir(fname: *const ::core::ffi::c_char) -> *const ::core::ffi::c_char;
    fn append_path(
        path: *mut ::core::ffi::c_char,
        to_append: *const ::core::ffi::c_char,
        max_len: size_t,
    ) -> ::core::ffi::c_int;
    fn ui_call_chdir(path: String_0);
    fn readv(
        __fd: ::core::ffi::c_int,
        __iovec: *const iovec,
        __count: ::core::ffi::c_int,
    ) -> ssize_t;
}
pub type __uid_t = ::core::ffi::c_uint;
pub type __gid_t = ::core::ffi::c_uint;
pub type __mode_t = ::core::ffi::c_uint;
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct iovec {
    pub iov_base: *mut ::core::ffi::c_void,
    pub iov_len: size_t,
}
pub type mode_t = __mode_t;
pub type off_t = __off_t;
pub type ptrdiff_t = isize;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint64_t = u64;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__queue {
    pub next: *mut uv__queue,
    pub prev: *mut uv__queue,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__work {
    pub work: Option<unsafe extern "C" fn(*mut uv__work) -> ()>,
    pub done: Option<unsafe extern "C" fn(*mut uv__work, ::core::ffi::c_int) -> ()>,
    pub loop_0: *mut uv_loop_s,
    pub wq: uv__queue,
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
pub type C2Rust_Unnamed_5 = ::core::ffi::c_int;
pub const UV_ERRNO_MAX: C2Rust_Unnamed_5 = -4096;
pub const UV_ENOEXEC: C2Rust_Unnamed_5 = -8;
pub const UV_EUNATCH: C2Rust_Unnamed_5 = -49;
pub const UV_ENODATA: C2Rust_Unnamed_5 = -61;
pub const UV_ESOCKTNOSUPPORT: C2Rust_Unnamed_5 = -94;
pub const UV_EILSEQ: C2Rust_Unnamed_5 = -84;
pub const UV_EFTYPE: C2Rust_Unnamed_5 = -4028;
pub const UV_ENOTTY: C2Rust_Unnamed_5 = -25;
pub const UV_EREMOTEIO: C2Rust_Unnamed_5 = -121;
pub const UV_EHOSTDOWN: C2Rust_Unnamed_5 = -112;
pub const UV_EMLINK: C2Rust_Unnamed_5 = -31;
pub const UV_ENXIO: C2Rust_Unnamed_5 = -6;
pub const UV_EOF: C2Rust_Unnamed_5 = -4095;
pub const UV_UNKNOWN: C2Rust_Unnamed_5 = -4094;
pub const UV_EXDEV: C2Rust_Unnamed_5 = -18;
pub const UV_ETXTBSY: C2Rust_Unnamed_5 = -26;
pub const UV_ETIMEDOUT: C2Rust_Unnamed_5 = -110;
pub const UV_ESRCH: C2Rust_Unnamed_5 = -3;
pub const UV_ESPIPE: C2Rust_Unnamed_5 = -29;
pub const UV_ESHUTDOWN: C2Rust_Unnamed_5 = -108;
pub const UV_EROFS: C2Rust_Unnamed_5 = -30;
pub const UV_ERANGE: C2Rust_Unnamed_5 = -34;
pub const UV_EPROTOTYPE: C2Rust_Unnamed_5 = -91;
pub const UV_EPROTONOSUPPORT: C2Rust_Unnamed_5 = -93;
pub const UV_EPROTO: C2Rust_Unnamed_5 = -71;
pub const UV_EPIPE: C2Rust_Unnamed_5 = -32;
pub const UV_EPERM: C2Rust_Unnamed_5 = -1;
pub const UV_EOVERFLOW: C2Rust_Unnamed_5 = -75;
pub const UV_ENOTSUP: C2Rust_Unnamed_5 = -95;
pub const UV_ENOTSOCK: C2Rust_Unnamed_5 = -88;
pub const UV_ENOTEMPTY: C2Rust_Unnamed_5 = -39;
pub const UV_ENOTDIR: C2Rust_Unnamed_5 = -20;
pub const UV_ENOTCONN: C2Rust_Unnamed_5 = -107;
pub const UV_ENOSYS: C2Rust_Unnamed_5 = -38;
pub const UV_ENOSPC: C2Rust_Unnamed_5 = -28;
pub const UV_ENOPROTOOPT: C2Rust_Unnamed_5 = -92;
pub const UV_ENONET: C2Rust_Unnamed_5 = -64;
pub const UV_ENOMEM: C2Rust_Unnamed_5 = -12;
pub const UV_ENOENT: C2Rust_Unnamed_5 = -2;
pub const UV_ENODEV: C2Rust_Unnamed_5 = -19;
pub const UV_ENOBUFS: C2Rust_Unnamed_5 = -105;
pub const UV_ENFILE: C2Rust_Unnamed_5 = -23;
pub const UV_ENETUNREACH: C2Rust_Unnamed_5 = -101;
pub const UV_ENETDOWN: C2Rust_Unnamed_5 = -100;
pub const UV_ENAMETOOLONG: C2Rust_Unnamed_5 = -36;
pub const UV_EMSGSIZE: C2Rust_Unnamed_5 = -90;
pub const UV_EMFILE: C2Rust_Unnamed_5 = -24;
pub const UV_ELOOP: C2Rust_Unnamed_5 = -40;
pub const UV_EISDIR: C2Rust_Unnamed_5 = -21;
pub const UV_EISCONN: C2Rust_Unnamed_5 = -106;
pub const UV_EIO: C2Rust_Unnamed_5 = -5;
pub const UV_EINVAL: C2Rust_Unnamed_5 = -22;
pub const UV_EINTR: C2Rust_Unnamed_5 = -4;
pub const UV_EHOSTUNREACH: C2Rust_Unnamed_5 = -113;
pub const UV_EFBIG: C2Rust_Unnamed_5 = -27;
pub const UV_EFAULT: C2Rust_Unnamed_5 = -14;
pub const UV_EEXIST: C2Rust_Unnamed_5 = -17;
pub const UV_EDESTADDRREQ: C2Rust_Unnamed_5 = -89;
pub const UV_ECONNRESET: C2Rust_Unnamed_5 = -104;
pub const UV_ECONNREFUSED: C2Rust_Unnamed_5 = -111;
pub const UV_ECONNABORTED: C2Rust_Unnamed_5 = -103;
pub const UV_ECHARSET: C2Rust_Unnamed_5 = -4080;
pub const UV_ECANCELED: C2Rust_Unnamed_5 = -125;
pub const UV_EBUSY: C2Rust_Unnamed_5 = -16;
pub const UV_EBADF: C2Rust_Unnamed_5 = -9;
pub const UV_EALREADY: C2Rust_Unnamed_5 = -114;
pub const UV_EAI_SOCKTYPE: C2Rust_Unnamed_5 = -3011;
pub const UV_EAI_SERVICE: C2Rust_Unnamed_5 = -3010;
pub const UV_EAI_PROTOCOL: C2Rust_Unnamed_5 = -3014;
pub const UV_EAI_OVERFLOW: C2Rust_Unnamed_5 = -3009;
pub const UV_EAI_NONAME: C2Rust_Unnamed_5 = -3008;
pub const UV_EAI_NODATA: C2Rust_Unnamed_5 = -3007;
pub const UV_EAI_MEMORY: C2Rust_Unnamed_5 = -3006;
pub const UV_EAI_FAMILY: C2Rust_Unnamed_5 = -3005;
pub const UV_EAI_FAIL: C2Rust_Unnamed_5 = -3004;
pub const UV_EAI_CANCELED: C2Rust_Unnamed_5 = -3003;
pub const UV_EAI_BADHINTS: C2Rust_Unnamed_5 = -3013;
pub const UV_EAI_BADFLAGS: C2Rust_Unnamed_5 = -3002;
pub const UV_EAI_AGAIN: C2Rust_Unnamed_5 = -3001;
pub const UV_EAI_ADDRFAMILY: C2Rust_Unnamed_5 = -3000;
pub const UV_EAGAIN: C2Rust_Unnamed_5 = -11;
pub const UV_EAFNOSUPPORT: C2Rust_Unnamed_5 = -97;
pub const UV_EADDRNOTAVAIL: C2Rust_Unnamed_5 = -99;
pub const UV_EADDRINUSE: C2Rust_Unnamed_5 = -98;
pub const UV_EACCES: C2Rust_Unnamed_5 = -13;
pub const UV_E2BIG: C2Rust_Unnamed_5 = -7;
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
pub type uv_dirent_t = uv_dirent_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_dirent_s {
    pub name: *const ::core::ffi::c_char,
    pub type_0: uv_dirent_type_t,
}
pub type uv_dirent_type_t = ::core::ffi::c_uint;
pub const UV_DIRENT_BLOCK: uv_dirent_type_t = 7;
pub const UV_DIRENT_CHAR: uv_dirent_type_t = 6;
pub const UV_DIRENT_SOCKET: uv_dirent_type_t = 5;
pub const UV_DIRENT_FIFO: uv_dirent_type_t = 4;
pub const UV_DIRENT_LINK: uv_dirent_type_t = 3;
pub const UV_DIRENT_DIR: uv_dirent_type_t = 2;
pub const UV_DIRENT_FILE: uv_dirent_type_t = 1;
pub const UV_DIRENT_UNKNOWN: uv_dirent_type_t = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_fs_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub fs_type: uv_fs_type,
    pub loop_0: *mut uv_loop_t,
    pub cb: uv_fs_cb,
    pub result: ssize_t,
    pub ptr: *mut ::core::ffi::c_void,
    pub path: *const ::core::ffi::c_char,
    pub statbuf: uv_stat_t,
    pub new_path: *const ::core::ffi::c_char,
    pub file: uv_file,
    pub flags: ::core::ffi::c_int,
    pub mode: mode_t,
    pub nbufs: ::core::ffi::c_uint,
    pub bufs: *mut uv_buf_t,
    pub off: off_t,
    pub uid: uv_uid_t,
    pub gid: uv_gid_t,
    pub atime: ::core::ffi::c_double,
    pub mtime: ::core::ffi::c_double,
    pub work_req: uv__work,
    pub bufsml: [uv_buf_t; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_stat_t {
    pub st_dev: uint64_t,
    pub st_mode: uint64_t,
    pub st_nlink: uint64_t,
    pub st_uid: uint64_t,
    pub st_gid: uint64_t,
    pub st_rdev: uint64_t,
    pub st_ino: uint64_t,
    pub st_size: uint64_t,
    pub st_blksize: uint64_t,
    pub st_blocks: uint64_t,
    pub st_flags: uint64_t,
    pub st_gen: uint64_t,
    pub st_atim: uv_timespec_t,
    pub st_mtim: uv_timespec_t,
    pub st_ctim: uv_timespec_t,
    pub st_birthtim: uv_timespec_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_timespec_t {
    pub tv_sec: ::core::ffi::c_long,
    pub tv_nsec: ::core::ffi::c_long,
}
pub type uv_fs_cb = Option<unsafe extern "C" fn(*mut uv_fs_t) -> ()>;
pub type uv_fs_t = uv_fs_s;
pub type uv_fs_type = ::core::ffi::c_int;
pub const UV_FS_LUTIME: uv_fs_type = 36;
pub const UV_FS_MKSTEMP: uv_fs_type = 35;
pub const UV_FS_STATFS: uv_fs_type = 34;
pub const UV_FS_CLOSEDIR: uv_fs_type = 33;
pub const UV_FS_READDIR: uv_fs_type = 32;
pub const UV_FS_OPENDIR: uv_fs_type = 31;
pub const UV_FS_LCHOWN: uv_fs_type = 30;
pub const UV_FS_COPYFILE: uv_fs_type = 29;
pub const UV_FS_REALPATH: uv_fs_type = 28;
pub const UV_FS_FCHOWN: uv_fs_type = 27;
pub const UV_FS_CHOWN: uv_fs_type = 26;
pub const UV_FS_READLINK: uv_fs_type = 25;
pub const UV_FS_SYMLINK: uv_fs_type = 24;
pub const UV_FS_LINK: uv_fs_type = 23;
pub const UV_FS_SCANDIR: uv_fs_type = 22;
pub const UV_FS_RENAME: uv_fs_type = 21;
pub const UV_FS_MKDTEMP: uv_fs_type = 20;
pub const UV_FS_MKDIR: uv_fs_type = 19;
pub const UV_FS_RMDIR: uv_fs_type = 18;
pub const UV_FS_UNLINK: uv_fs_type = 17;
pub const UV_FS_FDATASYNC: uv_fs_type = 16;
pub const UV_FS_FSYNC: uv_fs_type = 15;
pub const UV_FS_FCHMOD: uv_fs_type = 14;
pub const UV_FS_CHMOD: uv_fs_type = 13;
pub const UV_FS_ACCESS: uv_fs_type = 12;
pub const UV_FS_FUTIME: uv_fs_type = 11;
pub const UV_FS_UTIME: uv_fs_type = 10;
pub const UV_FS_FTRUNCATE: uv_fs_type = 9;
pub const UV_FS_FSTAT: uv_fs_type = 8;
pub const UV_FS_LSTAT: uv_fs_type = 7;
pub const UV_FS_STAT: uv_fs_type = 6;
pub const UV_FS_SENDFILE: uv_fs_type = 5;
pub const UV_FS_WRITE: uv_fs_type = 4;
pub const UV_FS_READ: uv_fs_type = 3;
pub const UV_FS_CLOSE: uv_fs_type = 2;
pub const UV_FS_OPEN: uv_fs_type = 1;
pub const UV_FS_CUSTOM: uv_fs_type = 0;
pub const UV_FS_UNKNOWN: uv_fs_type = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FileInfo {
    pub stat: uv_stat_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FileID {
    pub inode: uint64_t,
    pub device_id: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Directory {
    pub request: uv_fs_t,
    pub ent: uv_dirent_t,
}
pub type vim_acl_T = *mut ::core::ffi::c_void;
pub type OptInt = int64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct nvim_stats_s {
    pub fsync: int64_t,
    pub redraw: int64_t,
    pub log_skip: int16_t,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const O_WRONLY: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const O_RDWR: ::core::ffi::c_int = 0o2 as ::core::ffi::c_int;
pub const O_CREAT: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const O_TRUNC: ::core::ffi::c_int = 0o1000 as ::core::ffi::c_int;
pub const O_APPEND: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;
pub const F_GETFD: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const F_SETFD: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const FD_CLOEXEC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NODE_NORMAL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NODE_WRITABLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NODE_OTHER: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static mut e_xattr_erange: [::core::ffi::c_char; 51] = unsafe {
    ::core::mem::transmute::<[u8; 51], [::core::ffi::c_char; 51]>(
        *b"E1506: Buffer too small to copy xattr value or key\0",
    )
};
static mut e_xattr_e2big: [::core::ffi::c_char; 84] = unsafe {
    ::core::mem::transmute::<[u8; 84], [::core::ffi::c_char; 84]>(
        *b"E1508: Size of the extended attribute value is larger than the maximum size allowed\0",
    )
};
static mut e_xattr_other: [::core::ffi::c_char; 65] = unsafe {
    ::core::mem::transmute::<[u8; 65], [::core::ffi::c_char; 65]>(
        *b"E1509: Error occurred when reading or writing extended attribute\0",
    )
};
static mut kLibuvSuccess: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn os_chdir(mut path: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if p_verbose >= 5 as OptInt {
        verbose_enter();
        smsg(
            0 as ::core::ffi::c_int,
            b"chdir(%s)\0".as_ptr() as *const ::core::ffi::c_char,
            path,
        );
        verbose_leave();
    }
    let mut err: ::core::ffi::c_int = uv_chdir(path);
    if err == 0 as ::core::ffi::c_int {
        ui_call_chdir(cstr_as_string(path));
    }
    return err;
}
#[no_mangle]
pub unsafe extern "C" fn os_dirname(
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut error_number: ::core::ffi::c_int = 0;
    error_number = uv_cwd(buf, &raw mut len);
    if error_number != kLibuvSuccess {
        xstrlcpy(buf, uv_strerror(error_number), len);
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn os_isrealdir(mut name: *const ::core::ffi::c_char) -> bool {
    let mut request: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    if uv_fs_lstat(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut request,
        name,
        None,
    ) != kLibuvSuccess
    {
        return false_0 != 0;
    }
    if request.statbuf.st_mode & __S_IFMT as uint64_t == 0o120000 as uint64_t {
        return false_0 != 0;
    }
    return request.statbuf.st_mode & __S_IFMT as uint64_t == 0o40000 as uint64_t;
}
#[no_mangle]
pub unsafe extern "C" fn os_isdir(mut name: *const ::core::ffi::c_char) -> bool {
    let mut mode: int32_t = os_getperm(name);
    if mode < 0 as int32_t {
        return false_0 != 0;
    }
    return mode & __S_IFMT as int32_t == 0o40000 as int32_t;
}
#[no_mangle]
pub unsafe extern "C" fn os_nodetype(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut statbuf: uv_stat_t = uv_stat_t {
        st_dev: 0,
        st_mode: 0,
        st_nlink: 0,
        st_uid: 0,
        st_gid: 0,
        st_rdev: 0,
        st_ino: 0,
        st_size: 0,
        st_blksize: 0,
        st_blocks: 0,
        st_flags: 0,
        st_gen: 0,
        st_atim: uv_timespec_t {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_mtim: uv_timespec_t {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_ctim: uv_timespec_t {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_birthtim: uv_timespec_t {
            tv_sec: 0,
            tv_nsec: 0,
        },
    };
    if 0 as ::core::ffi::c_int != os_stat(name, &raw mut statbuf) {
        return NODE_NORMAL;
    }
    if statbuf.st_mode & __S_IFMT as uint64_t == 0o100000 as uint64_t
        || statbuf.st_mode & __S_IFMT as uint64_t == 0o40000 as uint64_t
    {
        return NODE_NORMAL;
    }
    if statbuf.st_mode & __S_IFMT as uint64_t == 0o60000 as uint64_t {
        return NODE_OTHER;
    }
    return NODE_WRITABLE;
}
#[no_mangle]
pub unsafe extern "C" fn os_exepath(
    mut buffer: *mut ::core::ffi::c_char,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    return uv_exepath(buffer, size);
}
#[no_mangle]
pub unsafe extern "C" fn os_can_exe(
    mut name: *const ::core::ffi::c_char,
    mut abspath: *mut *mut ::core::ffi::c_char,
    mut use_path: bool,
) -> bool {
    if !use_path || gettail_dir(name) != name {
        return (use_path as ::core::ffi::c_int != 0 || gettail_dir(name) != name)
            && is_executable(name, abspath) as ::core::ffi::c_int != 0;
    }
    return is_executable_in_path(name, abspath);
}
unsafe extern "C" fn is_executable(
    mut name: *const ::core::ffi::c_char,
    mut abspath: *mut *mut ::core::ffi::c_char,
) -> bool {
    let mut mode: int32_t = os_getperm(name);
    if mode < 0 as int32_t {
        return false_0 != 0;
    }
    let mut r: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if mode & __S_IFMT as int32_t == 0o100000 as int32_t {
        let mut req: uv_fs_t = uv_fs_t {
            data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            type_0: UV_UNKNOWN_REQ,
            reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
            fs_type: UV_FS_CUSTOM,
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            cb: None,
            result: 0,
            ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            path: ::core::ptr::null::<::core::ffi::c_char>(),
            statbuf: uv_stat_t {
                st_dev: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_ino: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_flags: 0,
                st_gen: 0,
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
            new_path: ::core::ptr::null::<::core::ffi::c_char>(),
            file: 0,
            flags: 0,
            mode: 0,
            nbufs: 0,
            bufs: ::core::ptr::null_mut::<uv_buf_t>(),
            off: 0,
            uid: 0,
            gid: 0,
            atime: 0.,
            mtime: 0.,
            work_req: uv__work {
                work: None,
                done: None,
                loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
                wq: uv__queue {
                    next: ::core::ptr::null_mut::<uv__queue>(),
                    prev: ::core::ptr::null_mut::<uv__queue>(),
                },
            },
            bufsml: [uv_buf_t {
                base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                len: 0,
            }; 4],
        };
        r = uv_fs_access(
            ::core::ptr::null_mut::<uv_loop_t>(),
            &raw mut req,
            name,
            1 as ::core::ffi::c_int,
            None,
        );
        uv_fs_req_cleanup(&raw mut req);
    }
    let ok: bool = r == 0 as ::core::ffi::c_int;
    if ok as ::core::ffi::c_int != 0 && !abspath.is_null() {
        *abspath = save_abs_path(name);
    }
    return ok;
}
unsafe extern "C" fn is_executable_in_path(
    mut name: *const ::core::ffi::c_char,
    mut abspath: *mut *mut ::core::ffi::c_char,
) -> bool {
    let mut path_env: *mut ::core::ffi::c_char =
        os_getenv(b"PATH\0".as_ptr() as *const ::core::ffi::c_char);
    if path_env.is_null() {
        return false_0 != 0;
    }
    let mut path: *mut ::core::ffi::c_char = xstrdup(path_env);
    let bufsize: size_t = strlen(name)
        .wrapping_add(strlen(path))
        .wrapping_add(2 as size_t);
    let mut buf: *mut ::core::ffi::c_char = xmalloc(bufsize) as *mut ::core::ffi::c_char;
    let mut p: *mut ::core::ffi::c_char = path;
    let mut rv: bool = false_0 != 0;
    loop {
        let mut e: *mut ::core::ffi::c_char = xstrchrnul(p, ENV_SEPCHAR as ::core::ffi::c_char);
        xmemcpyz(
            buf as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            e.offset_from(p) as size_t,
        );
        append_path(buf, name, bufsize);
        if is_executable(buf, abspath) {
            rv = true_0 != 0;
            break;
        } else {
            if *e as ::core::ffi::c_int != ENV_SEPCHAR {
                break;
            }
            p = e.offset(1 as ::core::ffi::c_int as isize);
        }
    }
    xfree(buf as *mut ::core::ffi::c_void);
    xfree(path as *mut ::core::ffi::c_void);
    xfree(path_env as *mut ::core::ffi::c_void);
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn os_open(
    mut path: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut mode: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if path.is_null() {
        return UV_EINVAL as ::core::ffi::c_int;
    }
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    r = uv_fs_open(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        path,
        flags,
        mode,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn os_fopen(
    mut path: *const ::core::ffi::c_char,
    mut flags: *const ::core::ffi::c_char,
) -> *mut FILE {
    '_c2rust_label: {
        if !flags.is_null() && strlen(flags) > 0 as size_t && strlen(flags) <= 2 as size_t {
        } else {
            __assert_fail(
                b"flags != NULL && strlen(flags) > 0 && strlen(flags) <= 2\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/os/fs.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                439 as ::core::ffi::c_uint,
                b"FILE *os_fopen(const char *, const char *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut iflags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *flags.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        || *flags.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'b' as ::core::ffi::c_int
    {
        match *flags.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            114 => {
                iflags = O_RDONLY;
            }
            119 => {
                iflags = O_WRONLY | O_CREAT | O_TRUNC;
            }
            97 => {
                iflags = O_WRONLY | O_CREAT | O_APPEND;
            }
            _ => {
                abort();
            }
        }
    } else {
        '_c2rust_label_0: {
            if *flags.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '+' as ::core::ffi::c_int
            {
            } else {
                __assert_fail(
                    b"flags[1] == '+'\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/os/fs.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    464 as ::core::ffi::c_uint,
                    b"FILE *os_fopen(const char *, const char *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        match *flags.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            114 => {
                iflags = O_RDWR;
            }
            119 => {
                iflags = O_RDWR | O_CREAT | O_TRUNC;
            }
            97 => {
                iflags = O_RDWR | O_CREAT | O_APPEND;
            }
            _ => {
                abort();
            }
        }
    }
    let mut fd: ::core::ffi::c_int = os_open(path, iflags, 0o666 as ::core::ffi::c_int);
    if fd < 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<FILE>();
    }
    return fdopen(fd, flags);
}
#[no_mangle]
pub unsafe extern "C" fn os_set_cloexec(fd: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut e: ::core::ffi::c_int = 0;
    let mut fdflags: ::core::ffi::c_int = fcntl(fd, F_GETFD);
    if fdflags < 0 as ::core::ffi::c_int {
        e = *__errno_location();
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"os_set_cloexec\0".as_ptr() as *const ::core::ffi::c_char,
            497 as ::core::ffi::c_int,
            true_0 != 0,
            b"Failed to get flags on descriptor %d: %s\0".as_ptr() as *const ::core::ffi::c_char,
            fd,
            strerror(e),
        );
        *__errno_location() = e;
        return -1 as ::core::ffi::c_int;
    }
    if fdflags & FD_CLOEXEC == 0 as ::core::ffi::c_int
        && fcntl(fd, F_SETFD, fdflags | FD_CLOEXEC) == -1 as ::core::ffi::c_int
    {
        e = *__errno_location();
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"os_set_cloexec\0".as_ptr() as *const ::core::ffi::c_char,
            504 as ::core::ffi::c_int,
            true_0 != 0,
            b"Failed to set CLOEXEC on descriptor %d: %s\0".as_ptr() as *const ::core::ffi::c_char,
            fd,
            strerror(e),
        );
        *__errno_location() = e;
        return -1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn os_close(fd: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    r = uv_fs_close(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        fd as uv_file,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn os_dup(fd: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = 0;
    loop {
        ret = dup(fd);
        if ret < 0 as ::core::ffi::c_int {
            let error: ::core::ffi::c_int = uv_translate_sys_error(*__errno_location());
            *__errno_location() = 0 as ::core::ffi::c_int;
            if error == UV_EINTR as ::core::ffi::c_int {
                continue;
            }
            return error;
        } else {
            return ret;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn os_open_stdin_fd() -> ::core::ffi::c_int {
    let mut stdin_dup_fd: ::core::ffi::c_int = 0;
    if stdin_fd > 0 as ::core::ffi::c_int {
        stdin_dup_fd = stdin_fd;
    } else {
        stdin_dup_fd = os_dup(STDIN_FILENO);
    }
    return stdin_dup_fd;
}
#[no_mangle]
pub unsafe extern "C" fn os_read(
    fd: ::core::ffi::c_int,
    ret_eof: *mut bool,
    ret_buf: *mut ::core::ffi::c_char,
    size: size_t,
    non_blocking: bool,
) -> ptrdiff_t {
    *ret_eof = false_0 != 0;
    if ret_buf.is_null() {
        '_c2rust_label: {
            if size == 0 as size_t {
            } else {
                __assert_fail(
                    b"size == 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/os/fs.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    588 as ::core::ffi::c_uint,
                    b"ptrdiff_t os_read(const int, _Bool *const, char *const, const size_t, const _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        return 0 as ptrdiff_t;
    }
    let mut read_bytes: size_t = 0 as size_t;
    while read_bytes != size {
        '_c2rust_label_0: {
            if size >= read_bytes {
            } else {
                __assert_fail(
                    b"size >= read_bytes\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/os/fs.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    593 as ::core::ffi::c_uint,
                    b"ptrdiff_t os_read(const int, _Bool *const, char *const, const size_t, const _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let cur_read_bytes: ptrdiff_t = read(
            fd,
            ret_buf.offset(read_bytes as isize) as *mut ::core::ffi::c_void,
            size.wrapping_sub(read_bytes),
        ) as ptrdiff_t;
        if cur_read_bytes > 0 as ptrdiff_t {
            read_bytes = read_bytes.wrapping_add(cur_read_bytes as size_t);
        }
        if cur_read_bytes < 0 as ptrdiff_t {
            let error: ::core::ffi::c_int = uv_translate_sys_error(*__errno_location());
            *__errno_location() = 0 as ::core::ffi::c_int;
            if non_blocking as ::core::ffi::c_int != 0 && error == UV_EAGAIN as ::core::ffi::c_int {
                break;
            }
            if error == UV_EINTR as ::core::ffi::c_int || error == UV_EAGAIN as ::core::ffi::c_int {
                continue;
            }
            return error as ptrdiff_t;
        } else {
            if cur_read_bytes != 0 as ptrdiff_t {
                continue;
            }
            *ret_eof = true_0 != 0;
            break;
        }
    }
    return read_bytes as ptrdiff_t;
}
#[no_mangle]
pub unsafe extern "C" fn os_readv(
    fd: ::core::ffi::c_int,
    ret_eof: *mut bool,
    mut iov: *mut iovec,
    mut iov_size: size_t,
    non_blocking: bool,
) -> ptrdiff_t {
    *ret_eof = false_0 != 0;
    let mut read_bytes: size_t = 0 as size_t;
    let mut toread: size_t = 0 as size_t;
    let mut i: size_t = 0 as size_t;
    while i < iov_size {
        '_c2rust_label: {
            if toread
                <= (18446744073709551615 as size_t).wrapping_sub((*iov.offset(i as isize)).iov_len)
            {
            } else {
                __assert_fail(
                    b"toread <= SIZE_MAX - iov[i].iov_len\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/os/fs.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    642 as ::core::ffi::c_uint,
                    b"ptrdiff_t os_readv(const int, _Bool *const, struct iovec *, size_t, const _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        toread = toread.wrapping_add((*iov.offset(i as isize)).iov_len);
        i = i.wrapping_add(1);
    }
    while read_bytes < toread && iov_size != 0 && !*ret_eof {
        let mut cur_read_bytes: ptrdiff_t =
            readv(fd, iov, iov_size as ::core::ffi::c_int) as ptrdiff_t;
        if cur_read_bytes == 0 as ptrdiff_t {
            *ret_eof = true_0 != 0;
        }
        if cur_read_bytes > 0 as ptrdiff_t {
            read_bytes = read_bytes.wrapping_add(cur_read_bytes as size_t);
            while iov_size != 0 && cur_read_bytes != 0 {
                if cur_read_bytes < (*iov).iov_len as ptrdiff_t {
                    (*iov).iov_len = (*iov).iov_len.wrapping_sub(cur_read_bytes as size_t);
                    (*iov).iov_base = ((*iov).iov_base as *mut ::core::ffi::c_char)
                        .offset(cur_read_bytes as isize)
                        as *mut ::core::ffi::c_void;
                    cur_read_bytes = 0 as ptrdiff_t;
                } else {
                    cur_read_bytes -= (*iov).iov_len as ptrdiff_t;
                    iov_size = iov_size.wrapping_sub(1);
                    iov = iov.offset(1);
                }
            }
        } else {
            if cur_read_bytes >= 0 as ptrdiff_t {
                continue;
            }
            let error: ::core::ffi::c_int = uv_translate_sys_error(*__errno_location());
            *__errno_location() = 0 as ::core::ffi::c_int;
            if non_blocking as ::core::ffi::c_int != 0 && error == UV_EAGAIN as ::core::ffi::c_int {
                break;
            }
            if error == UV_EINTR as ::core::ffi::c_int || error == UV_EAGAIN as ::core::ffi::c_int {
                continue;
            }
            return error as ptrdiff_t;
        }
    }
    return read_bytes as ptrdiff_t;
}
#[no_mangle]
pub unsafe extern "C" fn os_write(
    fd: ::core::ffi::c_int,
    buf: *const ::core::ffi::c_char,
    size: size_t,
    non_blocking: bool,
) -> ptrdiff_t {
    if buf.is_null() {
        '_c2rust_label: {
            if size == 0 as size_t {
            } else {
                __assert_fail(
                    b"size == 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/os/fs.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    691 as ::core::ffi::c_uint,
                    b"ptrdiff_t os_write(const int, const char *const, const size_t, const _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        return 0 as ptrdiff_t;
    }
    let mut written_bytes: size_t = 0 as size_t;
    while written_bytes != size {
        '_c2rust_label_0: {
            if size >= written_bytes {
            } else {
                __assert_fail(
                    b"size >= written_bytes\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/os/fs.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    696 as ::core::ffi::c_uint,
                    b"ptrdiff_t os_write(const int, const char *const, const size_t, const _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let cur_written_bytes: ptrdiff_t = write(
            fd,
            buf.offset(written_bytes as isize) as *const ::core::ffi::c_void,
            size.wrapping_sub(written_bytes),
        ) as ptrdiff_t;
        if cur_written_bytes > 0 as ptrdiff_t {
            written_bytes = written_bytes.wrapping_add(cur_written_bytes as size_t);
        }
        if cur_written_bytes < 0 as ptrdiff_t {
            let error: ::core::ffi::c_int = uv_translate_sys_error(*__errno_location());
            *__errno_location() = 0 as ::core::ffi::c_int;
            if non_blocking as ::core::ffi::c_int != 0 && error == UV_EAGAIN as ::core::ffi::c_int {
                break;
            }
            if error == UV_EINTR as ::core::ffi::c_int || error == UV_EAGAIN as ::core::ffi::c_int {
                continue;
            }
            return error as ptrdiff_t;
        } else if cur_written_bytes == 0 as ptrdiff_t {
            return UV_UNKNOWN as ::core::ffi::c_int as ptrdiff_t;
        }
    }
    return written_bytes as ptrdiff_t;
}
#[no_mangle]
pub unsafe extern "C" fn os_copy(
    mut path: *const ::core::ffi::c_char,
    mut new_path: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    r = uv_fs_copyfile(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        path,
        new_path,
        flags,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn os_fsync(mut fd: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    r = uv_fs_fsync(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        fd as uv_file,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    g_stats.fsync += 1;
    return r;
}
unsafe extern "C" fn os_stat(
    mut name: *const ::core::ffi::c_char,
    mut statbuf: *mut uv_stat_t,
) -> ::core::ffi::c_int {
    if name.is_null() {
        return UV_EINVAL as ::core::ffi::c_int;
    }
    let mut request: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    let mut result: ::core::ffi::c_int = uv_fs_stat(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut request,
        name,
        None,
    );
    if result == kLibuvSuccess {
        *statbuf = request.statbuf;
    }
    uv_fs_req_cleanup(&raw mut request);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn os_getperm(mut name: *const ::core::ffi::c_char) -> int32_t {
    let mut statbuf: uv_stat_t = uv_stat_t {
        st_dev: 0,
        st_mode: 0,
        st_nlink: 0,
        st_uid: 0,
        st_gid: 0,
        st_rdev: 0,
        st_ino: 0,
        st_size: 0,
        st_blksize: 0,
        st_blocks: 0,
        st_flags: 0,
        st_gen: 0,
        st_atim: uv_timespec_t {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_mtim: uv_timespec_t {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_ctim: uv_timespec_t {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_birthtim: uv_timespec_t {
            tv_sec: 0,
            tv_nsec: 0,
        },
    };
    let mut stat_result: ::core::ffi::c_int = os_stat(name, &raw mut statbuf);
    if stat_result == kLibuvSuccess {
        return statbuf.st_mode as int32_t;
    }
    return stat_result as int32_t;
}
#[no_mangle]
pub unsafe extern "C" fn os_setperm(
    name: *const ::core::ffi::c_char,
    mut perm: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    r = uv_fs_chmod(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        name,
        perm,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return if r == kLibuvSuccess { OK } else { FAIL };
}
#[no_mangle]
pub unsafe extern "C" fn os_copy_xattr(
    mut from_file: *const ::core::ffi::c_char,
    mut to_file: *const ::core::ffi::c_char,
) {
    if from_file.is_null() {
        return;
    }
    let mut size: ssize_t = listxattr(
        from_file as *mut ::core::ffi::c_char,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        0 as size_t,
    );
    if size <= 0 as ssize_t {
        return;
    }
    let mut xattr_buf: *mut ::core::ffi::c_char =
        xmalloc(size as size_t) as *mut ::core::ffi::c_char;
    size = listxattr(from_file, xattr_buf, size as size_t);
    let mut tsize: ssize_t = size;
    *__errno_location() = 0 as ::core::ffi::c_int;
    let mut max_vallen: ssize_t = 0 as ssize_t;
    let mut val: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut errmsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut round: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    '_error_exit: while round < 2 as ::core::ffi::c_int {
        let mut key: *mut ::core::ffi::c_char = xattr_buf;
        if round == 1 as ::core::ffi::c_int {
            size = tsize;
        }
        while size > 0 as ssize_t {
            let mut vallen: ssize_t = getxattr(
                from_file,
                key,
                val as *mut ::core::ffi::c_void,
                if round != 0 {
                    max_vallen as size_t
                } else {
                    0 as size_t
                },
            );
            if !(vallen >= 0 as ssize_t
                && round != 0
                && setxattr(
                    to_file,
                    key,
                    val as *const ::core::ffi::c_void,
                    vallen as size_t,
                    0 as ::core::ffi::c_int,
                ) == 0 as ::core::ffi::c_int)
            {
                if *__errno_location() != 0 {
                    match *__errno_location() {
                        E2BIG => {
                            errmsg = &raw const e_xattr_e2big as *const ::core::ffi::c_char;
                            break '_error_exit;
                        }
                        ENOTSUP | EACCES | EPERM => {}
                        ERANGE => {
                            errmsg = &raw const e_xattr_erange as *const ::core::ffi::c_char;
                            break '_error_exit;
                        }
                        _ => {
                            errmsg = &raw const e_xattr_other as *const ::core::ffi::c_char;
                            break '_error_exit;
                        }
                    }
                }
            }
            if round == 0 as ::core::ffi::c_int && vallen > max_vallen {
                max_vallen = vallen;
            }
            let mut keylen: ssize_t = strlen(key) as ssize_t + 1 as ssize_t;
            size -= keylen;
            key = key.offset(keylen as isize);
        }
        if round != 0 {
            break;
        }
        val = xmalloc((max_vallen as size_t).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        round += 1;
    }
    xfree(xattr_buf as *mut ::core::ffi::c_void);
    xfree(val as *mut ::core::ffi::c_void);
    if !errmsg.is_null() {
        emsg(gettext(errmsg));
    }
}
#[no_mangle]
pub unsafe extern "C" fn os_get_acl(mut fname: *const ::core::ffi::c_char) -> vim_acl_T {
    let mut ret: vim_acl_T = NULL;
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn os_set_acl(mut fname: *const ::core::ffi::c_char, mut aclent: vim_acl_T) {
    if aclent.is_null() {
        return;
    }
}
#[no_mangle]
pub unsafe extern "C" fn os_free_acl(mut aclent: vim_acl_T) {
    if aclent.is_null() {
        return;
    }
}
#[no_mangle]
pub unsafe extern "C" fn os_file_owned(mut fname: *const ::core::ffi::c_char) -> bool {
    let mut uid: uid_t = getuid();
    let mut finfo: FileInfo = FileInfo {
        stat: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
    };
    let mut file_owned: bool = os_fileinfo(fname, &raw mut finfo) as ::core::ffi::c_int != 0
        && finfo.stat.st_uid == uid as uint64_t;
    let mut link_owned: bool = os_fileinfo_link(fname, &raw mut finfo) as ::core::ffi::c_int != 0
        && finfo.stat.st_uid == uid as uint64_t;
    return file_owned as ::core::ffi::c_int != 0 && link_owned as ::core::ffi::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn os_chown(
    mut path: *const ::core::ffi::c_char,
    mut owner: uv_uid_t,
    mut group: uv_gid_t,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    r = uv_fs_chown(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        path,
        owner,
        group,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn os_fchown(
    mut fd: ::core::ffi::c_int,
    mut owner: uv_uid_t,
    mut group: uv_gid_t,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    r = uv_fs_fchown(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        fd as uv_file,
        owner,
        group,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn os_path_exists(mut path: *const ::core::ffi::c_char) -> bool {
    let mut statbuf: uv_stat_t = uv_stat_t {
        st_dev: 0,
        st_mode: 0,
        st_nlink: 0,
        st_uid: 0,
        st_gid: 0,
        st_rdev: 0,
        st_ino: 0,
        st_size: 0,
        st_blksize: 0,
        st_blocks: 0,
        st_flags: 0,
        st_gen: 0,
        st_atim: uv_timespec_t {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_mtim: uv_timespec_t {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_ctim: uv_timespec_t {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_birthtim: uv_timespec_t {
            tv_sec: 0,
            tv_nsec: 0,
        },
    };
    return os_stat(path, &raw mut statbuf) == kLibuvSuccess;
}
#[no_mangle]
pub unsafe extern "C" fn os_file_settime(
    mut path: *const ::core::ffi::c_char,
    mut atime: ::core::ffi::c_double,
    mut mtime: ::core::ffi::c_double,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    r = uv_fs_utime(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        path,
        atime,
        mtime,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn os_file_is_readable(mut name: *const ::core::ffi::c_char) -> bool {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    r = uv_fs_access(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        name,
        4 as ::core::ffi::c_int,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r == 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn os_file_is_writable(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    r = uv_fs_access(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        name,
        2 as ::core::ffi::c_int,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    if r == 0 as ::core::ffi::c_int {
        return if os_isdir(name) as ::core::ffi::c_int != 0 {
            2 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn os_rename(
    mut path: *const ::core::ffi::c_char,
    mut new_path: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    r = uv_fs_rename(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        path,
        new_path,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return if r == kLibuvSuccess { OK } else { FAIL };
}
#[no_mangle]
pub unsafe extern "C" fn os_mkdir(
    mut path: *const ::core::ffi::c_char,
    mut mode: int32_t,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    r = uv_fs_mkdir(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        path,
        mode as ::core::ffi::c_int,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn os_mkdir_recurse(
    dir: *const ::core::ffi::c_char,
    mut mode: int32_t,
    failed_dir: *mut *mut ::core::ffi::c_char,
    created: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let dirlen: size_t = strlen(dir);
    let curdir: *mut ::core::ffi::c_char =
        xmemdupz(dir as *const ::core::ffi::c_void, dirlen) as *mut ::core::ffi::c_char;
    let past_head: *mut ::core::ffi::c_char = get_past_head(curdir);
    let mut e: *mut ::core::ffi::c_char = curdir.offset(dirlen as isize);
    let real_end: *const ::core::ffi::c_char = e;
    let past_head_save: ::core::ffi::c_char = *past_head;
    while !os_isdir(curdir) {
        e = path_tail_with_sep(curdir);
        if e <= past_head {
            *past_head = NUL as ::core::ffi::c_char;
            break;
        } else {
            *e = NUL as ::core::ffi::c_char;
        }
    }
    while e != real_end as *mut ::core::ffi::c_char {
        if e > past_head {
            *e = PATHSEP as ::core::ffi::c_char;
        } else {
            *past_head = past_head_save;
        }
        let component_len: size_t = strlen(e);
        e = e.offset(component_len as isize);
        if e == real_end as *mut ::core::ffi::c_char
            && memcnt(
                e.offset(-(component_len as isize)) as *const ::core::ffi::c_void,
                PATHSEP as ::core::ffi::c_char,
                component_len,
            ) == component_len
        {
            break;
        }
        let mut ret: ::core::ffi::c_int = 0;
        ret = os_mkdir(curdir, mode);
        if ret != 0 as ::core::ffi::c_int {
            *failed_dir = curdir;
            return ret;
        } else if !created.is_null() && (*created).is_null() {
            *created = FullName_save(curdir, false_0 != 0);
        }
    }
    xfree(curdir as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn os_file_mkdir(
    mut fname: *mut ::core::ffi::c_char,
    mut mode: int32_t,
) -> ::core::ffi::c_int {
    if !dir_of_file_exists(fname) {
        let mut tail: *mut ::core::ffi::c_char = path_tail_with_sep(fname);
        let mut last_char: *mut ::core::ffi::c_char = tail
            .offset(strlen(tail) as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
        if vim_ispathsep(*last_char as ::core::ffi::c_int) {
            emsg(gettext(&raw const e_noname as *const ::core::ffi::c_char));
            return -1 as ::core::ffi::c_int;
        }
        let mut c: ::core::ffi::c_char = *tail;
        *tail = NUL as ::core::ffi::c_char;
        let mut r: ::core::ffi::c_int = 0;
        let mut failed_dir: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        r = os_mkdir_recurse(
            fname,
            mode,
            &raw mut failed_dir,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        );
        if r < 0 as ::core::ffi::c_int {
            semsg(
                gettext(&raw const e_mkdir as *const ::core::ffi::c_char),
                failed_dir,
                uv_strerror(r),
            );
            xfree(failed_dir as *mut ::core::ffi::c_void);
        }
        *tail = c;
        return r;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn os_mkdtemp(
    mut templ: *const ::core::ffi::c_char,
    mut path: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut request: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    let mut result: ::core::ffi::c_int = uv_fs_mkdtemp(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut request,
        templ,
        None,
    );
    if result == kLibuvSuccess {
        xstrlcpy(path, request.path, TEMP_FILE_PATH_MAXLEN as size_t);
    }
    uv_fs_req_cleanup(&raw mut request);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn os_rmdir(mut path: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    r = uv_fs_rmdir(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        path,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn os_scandir(
    mut dir: *mut Directory,
    mut path: *const ::core::ffi::c_char,
) -> bool {
    let mut r: ::core::ffi::c_int = uv_fs_scandir(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut (*dir).request,
        path,
        0 as ::core::ffi::c_int,
        None,
    );
    if r < 0 as ::core::ffi::c_int {
        os_closedir(dir);
    }
    return r >= 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn os_scandir_next(mut dir: *mut Directory) -> *const ::core::ffi::c_char {
    let mut err: ::core::ffi::c_int =
        uv_fs_scandir_next(&raw mut (*dir).request, &raw mut (*dir).ent);
    return if err != UV_EOF as ::core::ffi::c_int {
        (*dir).ent.name
    } else {
        ::core::ptr::null::<::core::ffi::c_char>()
    };
}
#[no_mangle]
pub unsafe extern "C" fn os_closedir(mut dir: *mut Directory) {
    uv_fs_req_cleanup(&raw mut (*dir).request);
}
#[no_mangle]
pub unsafe extern "C" fn os_remove(mut path: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    r = uv_fs_unlink(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        path,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn os_fileinfo(
    mut path: *const ::core::ffi::c_char,
    mut file_info: *mut FileInfo,
) -> bool {
    memset(
        file_info as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<FileInfo>(),
    );
    return os_stat(path, &raw mut (*file_info).stat) == kLibuvSuccess;
}
#[no_mangle]
pub unsafe extern "C" fn os_fileinfo_link(
    mut path: *const ::core::ffi::c_char,
    mut file_info: *mut FileInfo,
) -> bool {
    memset(
        file_info as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<FileInfo>(),
    );
    if path.is_null() {
        return false_0 != 0;
    }
    let mut request: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    let mut ok: bool = uv_fs_lstat(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut request,
        path,
        None,
    ) == kLibuvSuccess;
    if ok {
        (*file_info).stat = request.statbuf;
    }
    uv_fs_req_cleanup(&raw mut request);
    return ok;
}
#[no_mangle]
pub unsafe extern "C" fn os_fileinfo_fd(
    mut file_descriptor: ::core::ffi::c_int,
    mut file_info: *mut FileInfo,
) -> bool {
    let mut request: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    memset(
        file_info as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<FileInfo>(),
    );
    let mut ok: bool = uv_fs_fstat(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut request,
        file_descriptor as uv_file,
        None,
    ) == kLibuvSuccess;
    if ok {
        (*file_info).stat = request.statbuf;
    }
    uv_fs_req_cleanup(&raw mut request);
    return ok;
}
#[no_mangle]
pub unsafe extern "C" fn os_fileinfo_id_equal(
    mut file_info_1: *const FileInfo,
    mut file_info_2: *const FileInfo,
) -> bool {
    return (*file_info_1).stat.st_ino == (*file_info_2).stat.st_ino
        && (*file_info_1).stat.st_dev == (*file_info_2).stat.st_dev;
}
#[no_mangle]
pub unsafe extern "C" fn os_fileinfo_id(mut file_info: *const FileInfo, mut file_id: *mut FileID) {
    (*file_id).inode = (*file_info).stat.st_ino;
    (*file_id).device_id = (*file_info).stat.st_dev;
}
#[no_mangle]
pub unsafe extern "C" fn os_fileinfo_inode(mut file_info: *const FileInfo) -> uint64_t {
    return (*file_info).stat.st_ino;
}
#[no_mangle]
pub unsafe extern "C" fn os_fileinfo_size(mut file_info: *const FileInfo) -> uint64_t {
    return (*file_info).stat.st_size;
}
#[no_mangle]
pub unsafe extern "C" fn os_fileinfo_hardlinks(mut file_info: *const FileInfo) -> uint64_t {
    return (*file_info).stat.st_nlink;
}
#[no_mangle]
pub unsafe extern "C" fn os_fileinfo_blocksize(mut file_info: *const FileInfo) -> uint64_t {
    return (*file_info).stat.st_blksize;
}
#[no_mangle]
pub unsafe extern "C" fn os_fileid(
    mut path: *const ::core::ffi::c_char,
    mut file_id: *mut FileID,
) -> bool {
    let mut statbuf: uv_stat_t = uv_stat_t {
        st_dev: 0,
        st_mode: 0,
        st_nlink: 0,
        st_uid: 0,
        st_gid: 0,
        st_rdev: 0,
        st_ino: 0,
        st_size: 0,
        st_blksize: 0,
        st_blocks: 0,
        st_flags: 0,
        st_gen: 0,
        st_atim: uv_timespec_t {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_mtim: uv_timespec_t {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_ctim: uv_timespec_t {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_birthtim: uv_timespec_t {
            tv_sec: 0,
            tv_nsec: 0,
        },
    };
    if os_stat(path, &raw mut statbuf) == kLibuvSuccess {
        (*file_id).inode = statbuf.st_ino;
        (*file_id).device_id = statbuf.st_dev;
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn os_fileid_equal(
    mut file_id_1: *const FileID,
    mut file_id_2: *const FileID,
) -> bool {
    return (*file_id_1).inode == (*file_id_2).inode
        && (*file_id_1).device_id == (*file_id_2).device_id;
}
#[no_mangle]
pub unsafe extern "C" fn os_fileid_equal_fileinfo(
    mut file_id: *const FileID,
    mut file_info: *const FileInfo,
) -> bool {
    return (*file_id).inode == (*file_info).stat.st_ino
        && (*file_id).device_id == (*file_info).stat.st_dev;
}
#[no_mangle]
pub unsafe extern "C" fn os_realpath(
    mut name: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> *mut ::core::ffi::c_char {
    let mut request: uv_fs_t = uv_fs_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        fs_type: UV_FS_CUSTOM,
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        cb: None,
        result: 0,
        ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        statbuf: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
        new_path: ::core::ptr::null::<::core::ffi::c_char>(),
        file: 0,
        flags: 0,
        mode: 0,
        nbufs: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        off: 0,
        uid: 0,
        gid: 0,
        atime: 0.,
        mtime: 0.,
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    let mut result: ::core::ffi::c_int = uv_fs_realpath(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut request,
        name,
        None,
    );
    if result == kLibuvSuccess {
        if buf.is_null() {
            buf = xmalloc(len) as *mut ::core::ffi::c_char;
        }
        xstrlcpy(buf, request.ptr as *const ::core::ffi::c_char, len);
    }
    uv_fs_req_cleanup(&raw mut request);
    return if result == kLibuvSuccess {
        buf
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const ENOTSUP: ::core::ffi::c_int = EOPNOTSUPP;
pub const EOPNOTSUPP: ::core::ffi::c_int = 95;
pub const EPERM: ::core::ffi::c_int = 1;
pub const E2BIG: ::core::ffi::c_int = 7;
pub const EACCES: ::core::ffi::c_int = 13;
pub const ERANGE: ::core::ffi::c_int = 34;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const TEMP_FILE_PATH_MAXLEN: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
