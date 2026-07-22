//! Shared libuv `extern "C"` declarations (phase 5b).
//!
//! One declaration per symbol, `use`d by every consumer, instead of
//! the per-module copies c2rust emitted. Everything here resolves
//! against the static libuv at link time.

use crate::src::nvim::types::*;

extern "C" {
    pub fn uv_accept(server: *mut uv_stream_t, client: *mut uv_stream_t) -> ::core::ffi::c_int;
    pub fn uv_async_init(
        _: *mut uv_loop_t,
        async_0: *mut uv_async_t,
        async_cb_0: uv_async_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_async_send(async_0: *mut uv_async_t) -> ::core::ffi::c_int;
    pub fn uv_chdir(dir: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn uv_close(handle: *mut uv_handle_t, close_cb_0: uv_close_cb);
    pub fn uv_cwd(buffer: *mut ::core::ffi::c_char, size: *mut size_t) -> ::core::ffi::c_int;
    pub fn uv_disable_stdio_inheritance();
    pub fn uv_dlclose(lib: *mut uv_lib_t);
    pub fn uv_dlerror(lib: *const uv_lib_t) -> *const ::core::ffi::c_char;
    pub fn uv_dlopen(
        filename: *const ::core::ffi::c_char,
        lib: *mut uv_lib_t,
    ) -> ::core::ffi::c_int;
    pub fn uv_dlsym(
        lib: *mut uv_lib_t,
        name: *const ::core::ffi::c_char,
        ptr: *mut *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int;
    pub fn uv_err_name(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    pub fn uv_exepath(buffer: *mut ::core::ffi::c_char, size: *mut size_t) -> ::core::ffi::c_int;
    pub fn uv_freeaddrinfo(ai: *mut addrinfo);
    pub fn uv_fs_access(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        mode: ::core::ffi::c_int,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_chmod(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        mode: ::core::ffi::c_int,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_chown(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        uid: uv_uid_t,
        gid: uv_gid_t,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_close(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        file: uv_file,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_copyfile(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        new_path: *const ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_fchown(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        file: uv_file,
        uid: uv_uid_t,
        gid: uv_gid_t,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_fstat(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        file: uv_file,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_fsync(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        file: uv_file,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_lstat(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_mkdir(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        mode: ::core::ffi::c_int,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_mkdtemp(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        tpl: *const ::core::ffi::c_char,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_open(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        mode: ::core::ffi::c_int,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_read(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        file: uv_file,
        bufs: *const uv_buf_t,
        nbufs: ::core::ffi::c_uint,
        offset: int64_t,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_realpath(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_rename(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        new_path: *const ::core::ffi::c_char,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_req_cleanup(req: *mut uv_fs_t);
    pub fn uv_fs_rmdir(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_scandir(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_scandir_next(req: *mut uv_fs_t, ent: *mut uv_dirent_t) -> ::core::ffi::c_int;
    pub fn uv_fs_stat(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_unlink(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_utime(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        path: *const ::core::ffi::c_char,
        atime: ::core::ffi::c_double,
        mtime: ::core::ffi::c_double,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_fs_write(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        file: uv_file,
        bufs: *const uv_buf_t,
        nbufs: ::core::ffi::c_uint,
        offset: int64_t,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_get_total_memory() -> uint64_t;
    pub fn uv_guess_handle(file: uv_file) -> uv_handle_type;
    pub fn uv_hrtime() -> uint64_t;
    pub fn uv_idle_init(_: *mut uv_loop_t, idle: *mut uv_idle_t) -> ::core::ffi::c_int;
    pub fn uv_idle_start(idle: *mut uv_idle_t, cb: uv_idle_cb) -> ::core::ffi::c_int;
    pub fn uv_idle_stop(idle: *mut uv_idle_t) -> ::core::ffi::c_int;
    pub fn uv_is_closing(handle: *const uv_handle_t) -> ::core::ffi::c_int;
    pub fn uv_kill(pid: ::core::ffi::c_int, signum: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn uv_listen(
        stream: *mut uv_stream_t,
        backlog: ::core::ffi::c_int,
        cb: uv_connection_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_loop_close(loop_0: *mut uv_loop_t) -> ::core::ffi::c_int;
    pub fn uv_loop_init(loop_0: *mut uv_loop_t) -> ::core::ffi::c_int;
    pub fn uv_mutex_destroy(handle: *mut uv_mutex_t);
    pub fn uv_mutex_init(handle: *mut uv_mutex_t) -> ::core::ffi::c_int;
    pub fn uv_mutex_init_recursive(handle: *mut uv_mutex_t) -> ::core::ffi::c_int;
    pub fn uv_mutex_lock(handle: *mut uv_mutex_t);
    pub fn uv_mutex_unlock(handle: *mut uv_mutex_t);
    pub fn uv_now(_: *const uv_loop_t) -> uint64_t;
    pub fn uv_os_getenv(
        name: *const ::core::ffi::c_char,
        buffer: *mut ::core::ffi::c_char,
        size: *mut size_t,
    ) -> ::core::ffi::c_int;
    pub fn uv_os_homedir(buffer: *mut ::core::ffi::c_char, size: *mut size_t)
        -> ::core::ffi::c_int;
    pub fn uv_os_setenv(
        name: *const ::core::ffi::c_char,
        value: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn uv_os_unsetenv(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn uv_pipe(
        fds: *mut uv_file,
        read_flags: ::core::ffi::c_int,
        write_flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn uv_pipe_bind(
        handle: *mut uv_pipe_t,
        name: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn uv_pipe_connect(
        req: *mut uv_connect_t,
        handle: *mut uv_pipe_t,
        name: *const ::core::ffi::c_char,
        cb: uv_connect_cb,
    );
    pub fn uv_pipe_init(
        _: *mut uv_loop_t,
        handle: *mut uv_pipe_t,
        ipc: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn uv_pipe_open(_: *mut uv_pipe_t, file: uv_file) -> ::core::ffi::c_int;
    pub fn uv_print_all_handles(loop_0: *mut uv_loop_t, stream: *mut FILE);
    pub fn uv_read_start(
        _: *mut uv_stream_t,
        alloc_cb_0: uv_alloc_cb,
        read_cb_0: uv_read_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_read_stop(_: *mut uv_stream_t) -> ::core::ffi::c_int;
    pub fn uv_recv_buffer_size(
        handle: *mut uv_handle_t,
        value: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn uv_run(_: *mut uv_loop_t, mode: uv_run_mode) -> ::core::ffi::c_int;
    pub fn uv_signal_init(loop_0: *mut uv_loop_t, handle: *mut uv_signal_t) -> ::core::ffi::c_int;
    pub fn uv_signal_start(
        handle: *mut uv_signal_t,
        signal_cb: uv_signal_cb,
        signum: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn uv_signal_stop(handle: *mut uv_signal_t) -> ::core::ffi::c_int;
    pub fn uv_sleep(msec: ::core::ffi::c_uint);
    pub fn uv_spawn(
        loop_0: *mut uv_loop_t,
        handle: *mut uv_process_t,
        options: *const uv_process_options_t,
    ) -> ::core::ffi::c_int;
    pub fn uv_stop(_: *mut uv_loop_t);
    pub fn uv_stream_get_write_queue_size(stream: *const uv_stream_t) -> size_t;
    pub fn uv_stream_set_blocking(
        handle: *mut uv_stream_t,
        blocking: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn uv_strerror(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    pub fn uv_tcp_bind(
        handle: *mut uv_tcp_t,
        addr: *const sockaddr,
        flags: ::core::ffi::c_uint,
    ) -> ::core::ffi::c_int;
    pub fn uv_tcp_connect(
        req: *mut uv_connect_t,
        handle: *mut uv_tcp_t,
        addr: *const sockaddr,
        cb: uv_connect_cb,
    ) -> ::core::ffi::c_int;
    pub fn uv_tcp_getsockname(
        handle: *const uv_tcp_t,
        name: *mut sockaddr,
        namelen: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn uv_tcp_init(_: *mut uv_loop_t, handle: *mut uv_tcp_t) -> ::core::ffi::c_int;
    pub fn uv_tcp_nodelay(handle: *mut uv_tcp_t, enable: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn uv_timer_get_due_in(handle: *const uv_timer_t) -> uint64_t;
    pub fn uv_timer_init(_: *mut uv_loop_t, handle: *mut uv_timer_t) -> ::core::ffi::c_int;
    pub fn uv_timer_start(
        handle: *mut uv_timer_t,
        cb: uv_timer_cb,
        timeout: uint64_t,
        repeat: uint64_t,
    ) -> ::core::ffi::c_int;
    pub fn uv_timer_stop(handle: *mut uv_timer_t) -> ::core::ffi::c_int;
    pub fn uv_translate_sys_error(sys_errno: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn uv_tty_reset_mode() -> ::core::ffi::c_int;
    pub fn uv_unref(_: *mut uv_handle_t);
    pub fn uv_uptime(uptime: *mut ::core::ffi::c_double) -> ::core::ffi::c_int;
    pub fn uv_write(
        req: *mut uv_write_t,
        handle: *mut uv_stream_t,
        bufs: *const uv_buf_t,
        nbufs: ::core::ffi::c_uint,
        cb: uv_write_cb,
    ) -> ::core::ffi::c_int;
}
