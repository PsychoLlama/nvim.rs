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
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fopen(
        __filename: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
    ) -> *mut FILE;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn fscanf(__stream: *mut FILE, __format: *const ::core::ffi::c_char, ...)
        -> ::core::ffi::c_int;
    fn uv_kill(pid: ::core::ffi::c_int, signum: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
}
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
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
pub type C2Rust_Unnamed = ::core::ffi::c_int;
pub const UV_ERRNO_MAX: C2Rust_Unnamed = -4096;
pub const UV_ENOEXEC: C2Rust_Unnamed = -8;
pub const UV_EUNATCH: C2Rust_Unnamed = -49;
pub const UV_ENODATA: C2Rust_Unnamed = -61;
pub const UV_ESOCKTNOSUPPORT: C2Rust_Unnamed = -94;
pub const UV_EILSEQ: C2Rust_Unnamed = -84;
pub const UV_EFTYPE: C2Rust_Unnamed = -4028;
pub const UV_ENOTTY: C2Rust_Unnamed = -25;
pub const UV_EREMOTEIO: C2Rust_Unnamed = -121;
pub const UV_EHOSTDOWN: C2Rust_Unnamed = -112;
pub const UV_EMLINK: C2Rust_Unnamed = -31;
pub const UV_ENXIO: C2Rust_Unnamed = -6;
pub const UV_EOF: C2Rust_Unnamed = -4095;
pub const UV_UNKNOWN: C2Rust_Unnamed = -4094;
pub const UV_EXDEV: C2Rust_Unnamed = -18;
pub const UV_ETXTBSY: C2Rust_Unnamed = -26;
pub const UV_ETIMEDOUT: C2Rust_Unnamed = -110;
pub const UV_ESRCH: C2Rust_Unnamed = -3;
pub const UV_ESPIPE: C2Rust_Unnamed = -29;
pub const UV_ESHUTDOWN: C2Rust_Unnamed = -108;
pub const UV_EROFS: C2Rust_Unnamed = -30;
pub const UV_ERANGE: C2Rust_Unnamed = -34;
pub const UV_EPROTOTYPE: C2Rust_Unnamed = -91;
pub const UV_EPROTONOSUPPORT: C2Rust_Unnamed = -93;
pub const UV_EPROTO: C2Rust_Unnamed = -71;
pub const UV_EPIPE: C2Rust_Unnamed = -32;
pub const UV_EPERM: C2Rust_Unnamed = -1;
pub const UV_EOVERFLOW: C2Rust_Unnamed = -75;
pub const UV_ENOTSUP: C2Rust_Unnamed = -95;
pub const UV_ENOTSOCK: C2Rust_Unnamed = -88;
pub const UV_ENOTEMPTY: C2Rust_Unnamed = -39;
pub const UV_ENOTDIR: C2Rust_Unnamed = -20;
pub const UV_ENOTCONN: C2Rust_Unnamed = -107;
pub const UV_ENOSYS: C2Rust_Unnamed = -38;
pub const UV_ENOSPC: C2Rust_Unnamed = -28;
pub const UV_ENOPROTOOPT: C2Rust_Unnamed = -92;
pub const UV_ENONET: C2Rust_Unnamed = -64;
pub const UV_ENOMEM: C2Rust_Unnamed = -12;
pub const UV_ENOENT: C2Rust_Unnamed = -2;
pub const UV_ENODEV: C2Rust_Unnamed = -19;
pub const UV_ENOBUFS: C2Rust_Unnamed = -105;
pub const UV_ENFILE: C2Rust_Unnamed = -23;
pub const UV_ENETUNREACH: C2Rust_Unnamed = -101;
pub const UV_ENETDOWN: C2Rust_Unnamed = -100;
pub const UV_ENAMETOOLONG: C2Rust_Unnamed = -36;
pub const UV_EMSGSIZE: C2Rust_Unnamed = -90;
pub const UV_EMFILE: C2Rust_Unnamed = -24;
pub const UV_ELOOP: C2Rust_Unnamed = -40;
pub const UV_EISDIR: C2Rust_Unnamed = -21;
pub const UV_EISCONN: C2Rust_Unnamed = -106;
pub const UV_EIO: C2Rust_Unnamed = -5;
pub const UV_EINVAL: C2Rust_Unnamed = -22;
pub const UV_EINTR: C2Rust_Unnamed = -4;
pub const UV_EHOSTUNREACH: C2Rust_Unnamed = -113;
pub const UV_EFBIG: C2Rust_Unnamed = -27;
pub const UV_EFAULT: C2Rust_Unnamed = -14;
pub const UV_EEXIST: C2Rust_Unnamed = -17;
pub const UV_EDESTADDRREQ: C2Rust_Unnamed = -89;
pub const UV_ECONNRESET: C2Rust_Unnamed = -104;
pub const UV_ECONNREFUSED: C2Rust_Unnamed = -111;
pub const UV_ECONNABORTED: C2Rust_Unnamed = -103;
pub const UV_ECHARSET: C2Rust_Unnamed = -4080;
pub const UV_ECANCELED: C2Rust_Unnamed = -125;
pub const UV_EBUSY: C2Rust_Unnamed = -16;
pub const UV_EBADF: C2Rust_Unnamed = -9;
pub const UV_EALREADY: C2Rust_Unnamed = -114;
pub const UV_EAI_SOCKTYPE: C2Rust_Unnamed = -3011;
pub const UV_EAI_SERVICE: C2Rust_Unnamed = -3010;
pub const UV_EAI_PROTOCOL: C2Rust_Unnamed = -3014;
pub const UV_EAI_OVERFLOW: C2Rust_Unnamed = -3009;
pub const UV_EAI_NONAME: C2Rust_Unnamed = -3008;
pub const UV_EAI_NODATA: C2Rust_Unnamed = -3007;
pub const UV_EAI_MEMORY: C2Rust_Unnamed = -3006;
pub const UV_EAI_FAMILY: C2Rust_Unnamed = -3005;
pub const UV_EAI_FAIL: C2Rust_Unnamed = -3004;
pub const UV_EAI_CANCELED: C2Rust_Unnamed = -3003;
pub const UV_EAI_BADHINTS: C2Rust_Unnamed = -3013;
pub const UV_EAI_BADFLAGS: C2Rust_Unnamed = -3002;
pub const UV_EAI_AGAIN: C2Rust_Unnamed = -3001;
pub const UV_EAI_ADDRFAMILY: C2Rust_Unnamed = -3000;
pub const UV_EAGAIN: C2Rust_Unnamed = -11;
pub const UV_EAFNOSUPPORT: C2Rust_Unnamed = -97;
pub const UV_EADDRNOTAVAIL: C2Rust_Unnamed = -99;
pub const UV_EADDRINUSE: C2Rust_Unnamed = -98;
pub const UV_EACCES: C2Rust_Unnamed = -13;
pub const UV_E2BIG: C2Rust_Unnamed = -7;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 34] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"_Bool os_proc_tree_kill(int, int)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LOGLVL_INF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn os_proc_tree_kill(
    mut pid: ::core::ffi::c_int,
    mut sig: ::core::ffi::c_int,
) -> bool {
    '_c2rust_label: {
        if sig == 15 as ::core::ffi::c_int || sig == 9 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"sig == SIGTERM || sig == SIGKILL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/proc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                98 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    if pid == 0 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    logmsg(
        LOGLVL_INF,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"os_proc_tree_kill\0".as_ptr() as *const ::core::ffi::c_char,
        103 as ::core::ffi::c_int,
        true_0 != 0,
        b"sending %s to PID %d\0".as_ptr() as *const ::core::ffi::c_char,
        if sig == 15 as ::core::ffi::c_int {
            b"SIGTERM\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"SIGKILL\0".as_ptr() as *const ::core::ffi::c_char
        },
        -pid,
    );
    return uv_kill(-pid, sig) == 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn os_proc_children(
    mut ppid: ::core::ffi::c_int,
    mut proc_list: *mut *mut ::core::ffi::c_int,
    mut proc_count: *mut size_t,
) -> ::core::ffi::c_int {
    if ppid < 0 as ::core::ffi::c_int {
        return 2 as ::core::ffi::c_int;
    }
    let mut temp: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    *proc_list = ::core::ptr::null_mut::<::core::ffi::c_int>();
    *proc_count = 0 as size_t;
    let mut proc_p: [::core::ffi::c_char; 256] = [
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
    snprintf(
        &raw mut proc_p as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
        b"/proc/%d/task/%d/children\0".as_ptr() as *const ::core::ffi::c_char,
        ppid,
        ppid,
    );
    let mut fp: *mut FILE = fopen(
        &raw mut proc_p as *mut ::core::ffi::c_char,
        b"r\0".as_ptr() as *const ::core::ffi::c_char,
    ) as *mut FILE;
    if fp.is_null() {
        return 2 as ::core::ffi::c_int;
    }
    let mut match_pid: ::core::ffi::c_int = 0;
    while fscanf(
        fp,
        b"%d\0".as_ptr() as *const ::core::ffi::c_char,
        &raw mut match_pid,
    ) > 0 as ::core::ffi::c_int
    {
        temp = xrealloc(
            temp as *mut ::core::ffi::c_void,
            (*proc_count)
                .wrapping_add(1 as size_t)
                .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
        ) as *mut ::core::ffi::c_int;
        *temp.offset(*proc_count as isize) = match_pid;
        *proc_count = (*proc_count).wrapping_add(1);
    }
    fclose(fp);
    *proc_list = temp;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn os_proc_running(mut pid: ::core::ffi::c_int) -> bool {
    let mut err: ::core::ffi::c_int = uv_kill(pid, 0 as ::core::ffi::c_int);
    if err == 0 as ::core::ffi::c_int {
        return true_0 != 0;
    }
    if err == UV_ESRCH as ::core::ffi::c_int {
        return false_0 != 0;
    }
    return true_0 != 0;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
