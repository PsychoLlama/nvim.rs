use crate::src::nvim::charset::{skipwhite, vim_isIDc, vim_isfilec};
use crate::src::nvim::cmdexpand::{ExpandInit, ExpandOne};
use crate::src::nvim::eval::fs::modify_fname;
use crate::src::nvim::eval::skip_expr;
use crate::src::nvim::eval::vars::get_vim_var_str;
use crate::src::nvim::event::libuv::{
    uv_err_name, uv_os_getenv, uv_os_homedir, uv_os_setenv, uv_os_unsetenv, uv_strerror,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::{LOGLVL_ERR, logmsg};
use crate::src::nvim::main::{
    IObuff, NameBuff, didset_vim, didset_vimruntime, nvim_testing, os_buf, p_hf,
};
use crate::src::nvim::memory::{
    xfree, xmalloc, xmemcpyz, xmemdupz, xmemrchr, xstrdup, xstrlcat, xstrlcpy,
};
use crate::src::nvim::message::internal_error;
use crate::src::nvim::os::fs::{os_dirname, os_isdir, os_realpath};
use crate::src::nvim::os::libc::{
    __assert_fail, environ, getpid, memcpy, strcasecmp, strchr, strcmp, strcpy, strlen, strncmp,
    strpbrk,
};
use crate::src::nvim::os::users::os_get_userdir;
use crate::src::nvim::path::{
    after_pathsep, append_path, concat_fnames, path_fnamencmp, path_is_absolute, path_tail,
    path_tail_with_sep, vim_ispathsep,
};
use crate::src::nvim::strings::{striequal, vim_strchr, vim_strsave_escaped};
use crate::src::nvim::types::{
    Direction, VV_PROGPATH, buf_T, evalarg_T, expand_T, int64_t, pos_T, ptrdiff_t, sctx_T, size_t,
    uint8_t, xp_prefix_T,
};
unsafe extern "C" {
    fn uname(__name: *mut utsname) -> ::core::ffi::c_int;
}
pub type C2Rust_Unnamed = ::core::ffi::c_int;
pub const UV_EMLINK: C2Rust_Unnamed = -31;
pub const UV_UNKNOWN: C2Rust_Unnamed = -4094;
pub const UV_ENOENT: C2Rust_Unnamed = -2;
pub const UV_ENOBUFS: C2Rust_Unnamed = -105;
pub const UV_ELOOP: C2Rust_Unnamed = -40;
pub const UV_EISDIR: C2Rust_Unnamed = -21;
pub const UV_EINVAL: C2Rust_Unnamed = -22;
pub const UV_EEXIST: C2Rust_Unnamed = -17;
pub const UV_EBADF: C2Rust_Unnamed = -9;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const EXPAND_BUF_LEN: C2Rust_Unnamed_13 = 256;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const EXPAND_FILES: C2Rust_Unnamed_14 = 2;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const WILD_ALL_KEEP: C2Rust_Unnamed_15 = 8;
pub const WILD_ALL: C2Rust_Unnamed_15 = 6;
pub const WILD_EXPAND_FREE: C2Rust_Unnamed_15 = 2;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const WILD_ICASE: C2Rust_Unnamed_16 = 256;
pub const WILD_SILENT: C2Rust_Unnamed_16 = 64;
pub const WILD_KEEP_ALL: C2Rust_Unnamed_16 = 32;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_16 = 16;
pub const WILD_USE_NL: C2Rust_Unnamed_16 = 4;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_16 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct utsname {
    pub sysname: [::core::ffi::c_char; 65],
    pub nodename: [::core::ffi::c_char; 65],
    pub release: [::core::ffi::c_char; 65],
    pub version: [::core::ffi::c_char; 65],
    pub machine: [::core::ffi::c_char; 65],
    pub domainname: [::core::ffi::c_char; 65],
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const RUNTIME_DIRNAME: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"runtime\0") };
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub static default_vim_dir: GlobalCell<*mut ::core::ffi::c_char> = GlobalCell::new(
    b"/usr/local/share/nvim\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
);
pub static default_vimruntime_dir: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(concat!(env!("NVIM_DEFAULT_VIMRUNTIME_DIR"), "\0").as_ptr()
        as *const ::core::ffi::c_char as *mut ::core::ffi::c_char);
pub static default_lib_dir: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(concat!(env!("NVIM_DEFAULT_LIB_DIR"), "\0").as_ptr()
        as *const ::core::ffi::c_char as *mut ::core::ffi::c_char);
pub unsafe extern "C" fn env_init() {
    nvim_testing.set(os_env_exists(
        b"NVIM_TEST\0".as_ptr() as *const ::core::ffi::c_char,
        false_0 != 0,
    ));
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_getenv(
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut e: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut size: size_t = INIT_SIZE as size_t;
    let mut buf: [::core::ffi::c_char; 64] = [0; 64];
    r = uv_os_getenv(
        name,
        &raw mut buf as *mut ::core::ffi::c_char,
        &raw mut size,
    );
    if r == UV_ENOBUFS as ::core::ffi::c_int {
        e = xmalloc(size) as *mut ::core::ffi::c_char;
        r = uv_os_getenv(name, e, &raw mut size);
        if r != 0 as ::core::ffi::c_int
            || size == 0 as size_t
            || *e.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut e as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
        }
    } else if r != 0 as ::core::ffi::c_int
        || size == 0 as size_t
        || buf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == NUL
    {
        e = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        e = xmemdupz(
            &raw mut buf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            size,
        ) as *mut ::core::ffi::c_char;
    }
    if r != 0 as ::core::ffi::c_int
        && r != UV_ENOENT as ::core::ffi::c_int
        && r != UV_UNKNOWN as ::core::ffi::c_int
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"os_getenv\0".as_ptr() as *const ::core::ffi::c_char,
            98 as ::core::ffi::c_int,
            true_0 != 0,
            b"uv_os_getenv(%s) failed: %d %s\0".as_ptr() as *const ::core::ffi::c_char,
            name,
            r,
            uv_err_name(r),
        );
    }
    return e;
}
pub const INIT_SIZE: ::core::ffi::c_int = 64 as ::core::ffi::c_int;
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_getenv_buf(
    name: *const ::core::ffi::c_char,
    buf: *mut ::core::ffi::c_char,
    bufsize: size_t,
) -> *mut ::core::ffi::c_char {
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut size: size_t = bufsize;
    let mut r: ::core::ffi::c_int = uv_os_getenv(name, buf, &raw mut size);
    if r == UV_ENOBUFS as ::core::ffi::c_int {
        let mut e: *mut ::core::ffi::c_char = xmalloc(size) as *mut ::core::ffi::c_char;
        r = uv_os_getenv(name, e, &raw mut size);
        if r == 0 as ::core::ffi::c_int
            && size != 0 as size_t
            && *e.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            xmemcpyz(
                buf as *mut ::core::ffi::c_void,
                e as *const ::core::ffi::c_void,
                (if bufsize < size { bufsize } else { size }).wrapping_sub(1 as size_t),
            );
        }
        xfree(e as *mut ::core::ffi::c_void);
    }
    if r != 0 as ::core::ffi::c_int
        || size == 0 as size_t
        || *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        if r != 0 as ::core::ffi::c_int
            && r != UV_ENOENT as ::core::ffi::c_int
            && r != UV_UNKNOWN as ::core::ffi::c_int
        {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"os_getenv_buf\0".as_ptr() as *const ::core::ffi::c_char,
                129 as ::core::ffi::c_int,
                true_0 != 0,
                b"uv_os_getenv(%s) failed: %d %s\0".as_ptr() as *const ::core::ffi::c_char,
                name,
                r,
                uv_err_name(r),
            );
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return buf;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_getenv_noalloc(
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return os_getenv_buf(
        name,
        NameBuff.ptr() as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
    );
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_env_exists(
    mut name: *const ::core::ffi::c_char,
    mut nonempty: bool,
) -> bool {
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        return false_0 != 0;
    }
    let mut buf: [::core::ffi::c_char; 2] = [0; 2];
    let mut size: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 2]>();
    let mut r: ::core::ffi::c_int = uv_os_getenv(
        name,
        &raw mut buf as *mut ::core::ffi::c_char,
        &raw mut size,
    );
    '_c2rust_label: {
        if r != UV_EINVAL as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"r != UV_EINVAL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                165 as ::core::ffi::c_uint,
                b"_Bool os_env_exists(const char *, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if r != 0 as ::core::ffi::c_int
        && r != UV_ENOENT as ::core::ffi::c_int
        && r != UV_ENOBUFS as ::core::ffi::c_int
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"os_env_exists\0".as_ptr() as *const ::core::ffi::c_char,
            167 as ::core::ffi::c_int,
            true_0 != 0,
            b"uv_os_getenv(%s) failed: %d %s\0".as_ptr() as *const ::core::ffi::c_char,
            name,
            r,
            uv_err_name(r),
        );
    }
    return r == 0 as ::core::ffi::c_int && (!nonempty || size > 0 as size_t)
        || r == UV_ENOBUFS as ::core::ffi::c_int;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_setenv(
    mut name: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut overwrite: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        return -1 as ::core::ffi::c_int;
    }
    if overwrite == 0 && os_env_exists(name, false_0 != 0) as ::core::ffi::c_int != 0 {
        return 0 as ::core::ffi::c_int;
    }
    let mut r: ::core::ffi::c_int = 0;
    r = uv_os_setenv(name, value);
    '_c2rust_label: {
        if r != UV_EINVAL as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"r != UV_EINVAL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                204 as ::core::ffi::c_uint,
                b"int os_setenv(const char *, const char *, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if r != 0 as ::core::ffi::c_int {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"os_setenv\0".as_ptr() as *const ::core::ffi::c_char,
            206 as ::core::ffi::c_int,
            true_0 != 0,
            b"uv_os_setenv(%s) failed: %d %s\0".as_ptr() as *const ::core::ffi::c_char,
            name,
            r,
            uv_err_name(r),
        );
    }
    return if r == 0 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_unsetenv(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        return -1 as ::core::ffi::c_int;
    }
    let mut r: ::core::ffi::c_int = uv_os_unsetenv(name);
    if r != 0 as ::core::ffi::c_int {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"os_unsetenv\0".as_ptr() as *const ::core::ffi::c_char,
            220 as ::core::ffi::c_int,
            true_0 != 0,
            b"uv_os_unsetenv(%s) failed: %d %s\0".as_ptr() as *const ::core::ffi::c_char,
            name,
            r,
            uv_err_name(r),
        );
    }
    return if r == 0 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
}
pub unsafe extern "C" fn os_get_fullenv_size() -> size_t {
    let mut len: size_t = 0 as size_t;
    unsafe extern "C" {
        #[link_name = "environ"]
        static mut environ_0: *mut *mut ::core::ffi::c_char;
    }
    while !(*environ.offset(len as isize)).is_null() {
        len = len.wrapping_add(1);
    }
    return len;
}
pub unsafe extern "C" fn os_free_fullenv(mut env: *mut *mut ::core::ffi::c_char) {
    if env.is_null() {
        return;
    }
    let mut it: *mut *mut ::core::ffi::c_char = env;
    while !(*it).is_null() {
        let mut ptr_: *mut *mut ::core::ffi::c_void = it as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        it = it.offset(1);
    }
    xfree(env as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn os_copy_fullenv(
    mut env: *mut *mut ::core::ffi::c_char,
    mut env_size: size_t,
) {
    unsafe extern "C" {
        #[link_name = "environ"]
        static mut environ_0: *mut *mut ::core::ffi::c_char;
    }
    let mut i: size_t = 0 as size_t;
    while i < env_size && !(*environ.offset(i as isize)).is_null() {
        *env.offset(i as isize) = xstrdup(*environ.offset(i as isize));
        i = i.wrapping_add(1);
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_getenvname_at_index(mut index: size_t) -> *mut ::core::ffi::c_char {
    unsafe extern "C" {
        #[link_name = "environ"]
        static mut environ_0: *mut *mut ::core::ffi::c_char;
    }
    let mut i: size_t = 0 as size_t;
    while i <= index {
        if (*environ.offset(i as isize)).is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        i = i.wrapping_add(1);
    }
    let mut str: *mut ::core::ffi::c_char = *environ.offset(index as isize);
    '_c2rust_label: {
        if !str.is_null() {
        } else {
            __assert_fail(
                b"str != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                375 as ::core::ffi::c_uint,
                b"char *os_getenvname_at_index(size_t)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let end: *const ::core::ffi::c_char = strchr(str, '=' as ::core::ffi::c_int);
    '_c2rust_label_0: {
        if !end.is_null() {
        } else {
            __assert_fail(
                b"end != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                377 as ::core::ffi::c_uint,
                b"char *os_getenvname_at_index(size_t)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut len: ptrdiff_t = end.offset_from(str);
    '_c2rust_label_1: {
        if len > 0 as ptrdiff_t {
        } else {
            __assert_fail(
                b"len > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                379 as ::core::ffi::c_uint,
                b"char *os_getenvname_at_index(size_t)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return xmemdupz(str as *const ::core::ffi::c_void, len as size_t) as *mut ::core::ffi::c_char;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_get_pid() -> int64_t {
    return getpid() as int64_t;
}
pub unsafe extern "C" fn os_hint_priority() {}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_get_hostname(mut hostname: *mut ::core::ffi::c_char, mut size: size_t) {
    let mut vutsname: utsname = utsname {
        sysname: [0; 65],
        nodename: [0; 65],
        release: [0; 65],
        version: [0; 65],
        machine: [0; 65],
        domainname: [0; 65],
    };
    if uname(&raw mut vutsname) < 0 as ::core::ffi::c_int {
        *hostname = NUL as ::core::ffi::c_char;
    } else {
        xstrlcpy(
            hostname,
            &raw mut vutsname.nodename as *mut ::core::ffi::c_char,
            size,
        );
    };
}
static homedir: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub unsafe extern "C" fn init_homedir() {
    xfree(homedir.get() as *mut ::core::ffi::c_void);
    homedir.set(::core::ptr::null_mut::<::core::ffi::c_char>());
    let mut var: *mut ::core::ffi::c_char =
        os_getenv(b"HOME\0".as_ptr() as *const ::core::ffi::c_char);
    let mut tofree: *mut ::core::ffi::c_char = var;
    if var.is_null() {
        var = os_uv_homedir();
    }
    if !var.is_null()
        && !os_realpath(
            var,
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
        )
        .is_null()
    {
        var = IObuff.ptr() as *mut ::core::ffi::c_char;
    }
    if (var.is_null() || *var as ::core::ffi::c_int == NUL)
        && os_dirname(
            os_buf.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        ) == OK
    {
        var = os_buf.ptr() as *mut ::core::ffi::c_char;
    }
    if !var.is_null() {
        homedir.set(xstrdup(var));
    }
    xfree(tofree as *mut ::core::ffi::c_void);
}
static homedir_buf: GlobalCell<[::core::ffi::c_char; 4096]> = GlobalCell::new([0; 4096]);
unsafe extern "C" fn os_uv_homedir() -> *mut ::core::ffi::c_char {
    (*homedir_buf.ptr())[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    let mut homedir_size: size_t = MAXPATHL as size_t;
    let mut ret_value: ::core::ffi::c_int = uv_os_homedir(
        homedir_buf.ptr() as *mut ::core::ffi::c_char,
        &raw mut homedir_size,
    );
    if ret_value == 0 as ::core::ffi::c_int && homedir_size < MAXPATHL as size_t {
        return homedir_buf.ptr() as *mut ::core::ffi::c_char;
    }
    logmsg(
        LOGLVL_ERR,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"os_uv_homedir\0".as_ptr() as *const ::core::ffi::c_char,
        570 as ::core::ffi::c_int,
        true_0 != 0,
        b"uv_os_homedir() failed %d: %s\0".as_ptr() as *const ::core::ffi::c_char,
        ret_value,
        uv_strerror(ret_value),
    );
    (*homedir_buf.ptr())[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn expand_env_save(
    mut src: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return expand_env_save_opt(src, false_0 != 0);
}
pub unsafe extern "C" fn expand_env_save_opt(
    mut src: *mut ::core::ffi::c_char,
    mut one: bool,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    expand_env_esc(
        src,
        p,
        MAXPATHL,
        false_0 != 0,
        one,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
    return p;
}
pub unsafe extern "C" fn expand_env(
    mut src: *mut ::core::ffi::c_char,
    mut dst: *mut ::core::ffi::c_char,
    mut dstlen: ::core::ffi::c_int,
) -> size_t {
    return expand_env_esc(
        src,
        dst,
        dstlen,
        false_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn expand_env_esc(
    mut srcp: *const ::core::ffi::c_char,
    mut dst: *mut ::core::ffi::c_char,
    mut dstlen: ::core::ffi::c_int,
    mut esc: bool,
    mut one: bool,
    mut prefix: *mut ::core::ffi::c_char,
) -> size_t {
    let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut var: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut copy_char: bool = false;
    let mut mustfree: bool = false;
    let mut at_start: bool = true_0 != 0;
    let dst_start: *mut ::core::ffi::c_char = dst;
    let mut prefix_len: ::core::ffi::c_int = if prefix.is_null() {
        0 as ::core::ffi::c_int
    } else {
        strlen(prefix) as ::core::ffi::c_int
    };
    let mut src: *mut ::core::ffi::c_char = skipwhite(srcp);
    dstlen -= 1;
    while *src as ::core::ffi::c_int != 0 && dstlen > 0 as ::core::ffi::c_int {
        if *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '`' as ::core::ffi::c_int
            && *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '=' as ::core::ffi::c_int
        {
            var = src;
            src = src.offset(2 as ::core::ffi::c_int as isize);
            skip_expr(&raw mut src, ::core::ptr::null_mut::<evalarg_T>());
            if *src as ::core::ffi::c_int == '`' as ::core::ffi::c_int {
                src = src.offset(1);
            }
            let mut len: size_t = src.offset_from(var) as size_t;
            if len > dstlen as size_t {
                len = dstlen as size_t;
            }
            memcpy(
                dst as *mut ::core::ffi::c_void,
                var as *const ::core::ffi::c_void,
                len,
            );
            dst = dst.offset(len as isize);
            dstlen -= len as ::core::ffi::c_int;
        } else {
            copy_char = true_0 != 0;
            if *src as ::core::ffi::c_int == '$' as ::core::ffi::c_int
                || *src as ::core::ffi::c_int == '~' as ::core::ffi::c_int
                    && at_start as ::core::ffi::c_int != 0
            {
                mustfree = false_0 != 0;
                if *src as ::core::ffi::c_int != '~' as ::core::ffi::c_int {
                    tail = src.offset(1 as ::core::ffi::c_int as isize);
                    var = dst;
                    let mut c: ::core::ffi::c_int = dstlen - 1 as ::core::ffi::c_int;
                    if *tail as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                        && !vim_isIDc('{' as ::core::ffi::c_int)
                    {
                        tail = tail.offset(1);
                        loop {
                            let c2rust_fresh0 = c;
                            c = c - 1;
                            if !(c2rust_fresh0 > 0 as ::core::ffi::c_int
                                && *tail as ::core::ffi::c_int != NUL
                                && *tail as ::core::ffi::c_int != '}' as ::core::ffi::c_int)
                            {
                                break;
                            }
                            let c2rust_fresh1 = tail;
                            tail = tail.offset(1);
                            let c2rust_fresh2 = var;
                            var = var.offset(1);
                            *c2rust_fresh2 = *c2rust_fresh1;
                        }
                    } else {
                        loop {
                            let c2rust_fresh3 = c;
                            c = c - 1;
                            if !(c2rust_fresh3 > 0 as ::core::ffi::c_int
                                && *tail as ::core::ffi::c_int != NUL
                                && vim_isIDc(*tail as uint8_t as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0)
                            {
                                break;
                            }
                            let c2rust_fresh4 = tail;
                            tail = tail.offset(1);
                            let c2rust_fresh5 = var;
                            var = var.offset(1);
                            *c2rust_fresh5 = *c2rust_fresh4;
                        }
                    }
                    if *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '{' as ::core::ffi::c_int
                        && *tail as ::core::ffi::c_int != '}' as ::core::ffi::c_int
                    {
                        var = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    } else {
                        if *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '{' as ::core::ffi::c_int
                        {
                            tail = tail.offset(1);
                        }
                        *var = NUL as ::core::ffi::c_char;
                        var = vim_getenv(dst);
                        mustfree = true_0 != 0;
                    }
                } else if *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || vim_ispathsep(
                        *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                    || !vim_strchr(
                        b" ,\t\n\0".as_ptr() as *const ::core::ffi::c_char,
                        *src.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int,
                    )
                    .is_null()
                {
                    var = homedir.get();
                    tail = src.offset(1 as ::core::ffi::c_int as isize);
                } else {
                    tail = src;
                    var = dst;
                    let mut c_0: ::core::ffi::c_int = dstlen - 1 as ::core::ffi::c_int;
                    loop {
                        let c2rust_fresh6 = c_0;
                        c_0 = c_0 - 1;
                        if !(c2rust_fresh6 > 0 as ::core::ffi::c_int
                            && *tail as ::core::ffi::c_int != 0
                            && vim_isfilec(*tail as uint8_t as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                                != 0
                            && !vim_ispathsep(*tail as ::core::ffi::c_int))
                        {
                            break;
                        }
                        let c2rust_fresh7 = tail;
                        tail = tail.offset(1);
                        let c2rust_fresh8 = var;
                        var = var.offset(1);
                        *c2rust_fresh8 = *c2rust_fresh7;
                    }
                    *var = NUL as ::core::ffi::c_char;
                    var = if *dst as ::core::ffi::c_int == NUL {
                        ::core::ptr::null_mut::<::core::ffi::c_char>()
                    } else {
                        os_get_userdir(dst.offset(1 as ::core::ffi::c_int as isize))
                    };
                    mustfree = true_0 != 0;
                    if var.is_null() {
                        let mut xpc: expand_T = expand_T {
                            xp_pattern: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            xp_context: 0,
                            xp_pattern_len: 0,
                            xp_prefix: XP_PREFIX_NONE,
                            xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            xp_luaref: 0,
                            xp_script_ctx: sctx_T {
                                sc_sid: 0,
                                sc_seq: 0,
                                sc_lnum: 0,
                                sc_chan: 0,
                            },
                            xp_backslash: 0,
                            xp_shell: false,
                            xp_numfiles: 0,
                            xp_col: 0,
                            xp_selected: 0,
                            xp_orig: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            xp_files: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                            xp_line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            xp_buf: [0; 256],
                            xp_search_dir: kDirectionNotSet,
                            xp_pre_incsearch_pos: pos_T {
                                lnum: 0,
                                col: 0,
                                coladd: 0,
                            },
                        };
                        ExpandInit(&raw mut xpc);
                        xpc.xp_context = EXPAND_FILES as ::core::ffi::c_int;
                        var = ExpandOne(
                            &raw mut xpc,
                            dst,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            WILD_ADD_SLASH as ::core::ffi::c_int
                                | WILD_SILENT as ::core::ffi::c_int,
                            WILD_EXPAND_FREE as ::core::ffi::c_int,
                        );
                        mustfree = true_0 != 0;
                    }
                }
                if esc as ::core::ffi::c_int != 0
                    && !var.is_null()
                    && !strpbrk(var, b" \t\0".as_ptr() as *const ::core::ffi::c_char).is_null()
                {
                    let mut p: *mut ::core::ffi::c_char =
                        vim_strsave_escaped(var, b" \t\0".as_ptr() as *const ::core::ffi::c_char);
                    if mustfree {
                        xfree(var as *mut ::core::ffi::c_void);
                    }
                    var = p;
                    mustfree = true_0 != 0;
                }
                if !var.is_null() && *var as ::core::ffi::c_int != NUL {
                    let mut c_1: ::core::ffi::c_int = strlen(var) as ::core::ffi::c_int;
                    if (c_1 as size_t)
                        .wrapping_add(strlen(tail))
                        .wrapping_add(1 as size_t)
                        < dstlen as ::core::ffi::c_uint as size_t
                    {
                        strcpy(dst, var);
                        dstlen -= c_1;
                        if after_pathsep(dst, dst.offset(c_1 as isize)) != 0
                            && vim_ispathsep(*tail as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                        {
                            tail = tail.offset(1);
                        }
                        dst = dst.offset(c_1 as isize);
                        src = tail;
                        copy_char = false_0 != 0;
                    }
                }
                if mustfree {
                    xfree(var as *mut ::core::ffi::c_void);
                }
            }
            if copy_char {
                at_start = false_0 != 0;
                if *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    let c2rust_fresh9 = src;
                    src = src.offset(1);
                    let c2rust_fresh10 = dst;
                    dst = dst.offset(1);
                    *c2rust_fresh10 = *c2rust_fresh9;
                    dstlen -= 1;
                } else if (*src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ' ' as ::core::ffi::c_int
                    || *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ',' as ::core::ffi::c_int)
                    && !one
                {
                    at_start = true_0 != 0;
                }
                if dstlen > 0 as ::core::ffi::c_int {
                    let c2rust_fresh11 = src;
                    src = src.offset(1);
                    let c2rust_fresh12 = dst;
                    dst = dst.offset(1);
                    *c2rust_fresh12 = *c2rust_fresh11;
                    dstlen -= 1;
                    if !prefix.is_null()
                        && src.offset(-(prefix_len as isize)) >= srcp as *mut ::core::ffi::c_char
                        && strncmp(
                            src.offset(-(prefix_len as isize)),
                            prefix,
                            prefix_len as size_t,
                        ) == 0 as ::core::ffi::c_int
                    {
                        at_start = true_0 != 0;
                    }
                }
            }
        }
    }
    *dst = NUL as ::core::ffi::c_char;
    return dst.offset_from(dst_start) as size_t;
}
unsafe extern "C" fn vim_runtime_dir(
    mut vimdir: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if vimdir.is_null() || *vimdir as ::core::ffi::c_int == NUL {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut p: *mut ::core::ffi::c_char =
        concat_fnames(vimdir, RUNTIME_DIRNAME.as_ptr(), true_0 != 0);
    if os_isdir(p) {
        return p;
    }
    xfree(p as *mut ::core::ffi::c_void);
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn remove_tail(
    mut path: *mut ::core::ffi::c_char,
    mut pend: *mut ::core::ffi::c_char,
    mut dirname: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut len: size_t = strlen(dirname);
    let mut new_tail: *mut ::core::ffi::c_char = pend
        .offset(-(len as isize))
        .offset(-(1 as ::core::ffi::c_int as isize));
    if new_tail >= path
        && path_fnamencmp(new_tail, dirname, len) == 0 as ::core::ffi::c_int
        && (new_tail == path || after_pathsep(path, new_tail) != 0)
    {
        return new_tail;
    }
    return pend;
}
pub unsafe extern "C" fn vim_env_iter(
    delim: ::core::ffi::c_char,
    val: *const ::core::ffi::c_char,
    iter: *const ::core::ffi::c_void,
    dir: *mut *const ::core::ffi::c_char,
    len: *mut size_t,
) -> *const ::core::ffi::c_void {
    let mut varval: *const ::core::ffi::c_char = iter as *const ::core::ffi::c_char;
    if varval.is_null() {
        varval = val;
    }
    *dir = varval;
    let dirend: *const ::core::ffi::c_char = strchr(varval, delim as ::core::ffi::c_int);
    if dirend.is_null() {
        *len = strlen(varval);
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
    *len = dirend.offset_from(varval) as size_t;
    return dirend.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void;
}
pub unsafe extern "C" fn vim_env_iter_rev(
    delim: ::core::ffi::c_char,
    val: *const ::core::ffi::c_char,
    iter: *const ::core::ffi::c_void,
    dir: *mut *const ::core::ffi::c_char,
    len: *mut size_t,
) -> *const ::core::ffi::c_void {
    let mut varend: *const ::core::ffi::c_char = iter as *const ::core::ffi::c_char;
    if varend.is_null() {
        varend = val
            .offset(strlen(val) as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
    }
    let varlen: size_t = (varend.offset_from(val) as size_t).wrapping_add(1 as size_t);
    let colon: *const ::core::ffi::c_char =
        xmemrchr(val as *const ::core::ffi::c_void, delim as uint8_t, varlen)
            as *const ::core::ffi::c_char;
    if colon.is_null() {
        *len = varlen;
        *dir = val;
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
    *dir = colon.offset(1 as ::core::ffi::c_int as isize);
    *len = varend.offset_from(colon) as size_t;
    return colon.offset(-(1 as ::core::ffi::c_int as isize)) as *const ::core::ffi::c_void;
}
pub unsafe extern "C" fn vim_get_prefix_from_exepath(mut exe_name: *mut ::core::ffi::c_char) {
    xstrlcpy(
        exe_name,
        get_vim_var_str(VV_PROGPATH),
        (MAXPATHL as size_t).wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
    );
    let mut path_end: *mut ::core::ffi::c_char = path_tail_with_sep(exe_name);
    *path_end = NUL as ::core::ffi::c_char;
    path_end = path_tail(exe_name);
    *path_end = NUL as ::core::ffi::c_char;
}
pub unsafe extern "C" fn vim_getenv(
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    '_c2rust_label: {
        if *get_vim_var_str(VV_PROGPATH).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != '\0' as ::core::ffi::c_int
        {
        } else {
            __assert_fail(
                b"get_vim_var_str(VV_PROGPATH)[0] != NUL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                956 as ::core::ffi::c_uint,
                b"char *vim_getenv(const char *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut kos_env_path: *mut ::core::ffi::c_char = os_getenv(name);
    if !kos_env_path.is_null() {
        return kos_env_path;
    }
    let mut vimruntime: bool = strcmp(name, b"VIMRUNTIME\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int;
    if !vimruntime
        && strcmp(name, b"VIM\0".as_ptr() as *const ::core::ffi::c_char) != 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut vim_path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if vimruntime as ::core::ffi::c_int != 0
        && *default_vimruntime_dir.get() as ::core::ffi::c_int == NUL
    {
        kos_env_path = os_getenv(b"VIM\0".as_ptr() as *const ::core::ffi::c_char);
        if !kos_env_path.is_null() {
            vim_path = vim_runtime_dir(kos_env_path);
            if vim_path.is_null() {
                vim_path = kos_env_path;
            } else {
                xfree(kos_env_path as *mut ::core::ffi::c_void);
            }
        }
    }
    if vim_path.is_null() {
        if !(*p_hf.ptr()).is_null() && vim_strchr(p_hf.get(), '$' as ::core::ffi::c_int).is_null() {
            vim_path = p_hf.get();
        }
        let mut exe_name: [::core::ffi::c_char; 4096] = [0; 4096];
        if vim_path.is_null() {
            vim_get_prefix_from_exepath(&raw mut exe_name as *mut ::core::ffi::c_char);
            if append_path(
                &raw mut exe_name as *mut ::core::ffi::c_char,
                b"share/nvim/runtime/\0".as_ptr() as *const ::core::ffi::c_char,
                MAXPATHL as size_t,
            ) == OK
            {
                vim_path = &raw mut exe_name as *mut ::core::ffi::c_char;
            }
        }
        if !vim_path.is_null() {
            let mut vim_path_end: *mut ::core::ffi::c_char = path_tail(vim_path);
            if vim_path == p_hf.get() {
                vim_path_end = remove_tail(
                    vim_path,
                    vim_path_end,
                    b"doc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
            }
            if !vimruntime {
                vim_path_end = remove_tail(
                    vim_path,
                    vim_path_end,
                    RUNTIME_DIRNAME.as_ptr() as *mut ::core::ffi::c_char,
                );
            }
            if vim_path_end > vim_path && after_pathsep(vim_path, vim_path_end) != 0 {
                vim_path_end = vim_path_end.offset(-1);
            }
            '_c2rust_label_0: {
                if vim_path_end >= vim_path {
                } else {
                    __assert_fail(
                        b"vim_path_end >= vim_path\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1027 as ::core::ffi::c_uint,
                        b"char *vim_getenv(const char *)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            vim_path = xmemdupz(
                vim_path as *const ::core::ffi::c_void,
                vim_path_end.offset_from(vim_path) as size_t,
            ) as *mut ::core::ffi::c_char;
            if !os_isdir(vim_path) {
                xfree(vim_path as *mut ::core::ffi::c_void);
                vim_path = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        }
        '_c2rust_label_1: {
            if vim_path != &raw mut exe_name as *mut ::core::ffi::c_char {
            } else {
                __assert_fail(
                    b"vim_path != exe_name\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1035 as ::core::ffi::c_uint,
                    b"char *vim_getenv(const char *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
    }
    if vim_path.is_null() {
        if vimruntime as ::core::ffi::c_int != 0
            && *default_vimruntime_dir.get() as ::core::ffi::c_int != NUL
        {
            vim_path = xstrdup(default_vimruntime_dir.get());
        } else if *default_vim_dir.get() as ::core::ffi::c_int != NUL {
            if vimruntime as ::core::ffi::c_int != 0 && {
                vim_path = vim_runtime_dir(default_vim_dir.get());
                vim_path.is_null()
            } {
                vim_path = xstrdup(default_vim_dir.get());
            }
        }
    }
    if !vim_path.is_null() {
        if vimruntime {
            os_setenv(
                b"VIMRUNTIME\0".as_ptr() as *const ::core::ffi::c_char,
                vim_path,
                1 as ::core::ffi::c_int,
            );
            didset_vimruntime.set(true_0 != 0);
        } else {
            os_setenv(
                b"VIM\0".as_ptr() as *const ::core::ffi::c_char,
                vim_path,
                1 as ::core::ffi::c_int,
            );
            didset_vim.set(true_0 != 0);
        }
    }
    return vim_path;
}
pub unsafe extern "C" fn home_replace(
    buf: *const buf_T,
    mut src: *const ::core::ffi::c_char,
    dst: *mut ::core::ffi::c_char,
    mut dstlen: size_t,
    one: bool,
) -> size_t {
    let mut dirlen: size_t = 0 as size_t;
    let mut envlen: size_t = 0 as size_t;
    if src.is_null() {
        *dst = NUL as ::core::ffi::c_char;
        return 0 as size_t;
    }
    if !buf.is_null() && (*buf).b_help as ::core::ffi::c_int != 0 {
        let dlen: size_t = xstrlcpy(dst, path_tail(src), dstlen);
        return if dlen < dstlen.wrapping_sub(1 as size_t) {
            dlen
        } else {
            dstlen.wrapping_sub(1 as size_t)
        };
    }
    if !(*homedir.ptr()).is_null() {
        dirlen = strlen(homedir.get());
    }
    let mut homedir_env: *mut ::core::ffi::c_char =
        os_getenv(b"HOME\0".as_ptr() as *const ::core::ffi::c_char);
    let mut homedir_env_mod: *mut ::core::ffi::c_char = homedir_env;
    let mut must_free: bool = false_0 != 0;
    if !homedir_env_mod.is_null()
        && *homedir_env_mod as ::core::ffi::c_int == '~' as ::core::ffi::c_int
    {
        must_free = true_0 != 0;
        let mut usedlen: size_t = 0 as size_t;
        let mut flen: size_t = strlen(homedir_env_mod);
        let mut fbuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        modify_fname(
            b":p\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            false_0 != 0,
            &raw mut usedlen,
            &raw mut homedir_env_mod,
            &raw mut fbuf,
            &raw mut flen,
        );
        flen = strlen(homedir_env_mod);
        '_c2rust_label: {
            if homedir_env_mod != homedir_env {
            } else {
                __assert_fail(
                    b"homedir_env_mod != homedir_env\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/os/env.rs\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    1123 as ::core::ffi::c_uint,
                    b"size_t home_replace(const buf_T *const, const char *, char *const, size_t, const _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if vim_ispathsep(
            *homedir_env_mod.offset(flen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int,
        ) {
            *homedir_env_mod.offset(flen.wrapping_sub(1 as size_t) as isize) =
                NUL as ::core::ffi::c_char;
        }
    }
    if !homedir_env_mod.is_null() {
        envlen = strlen(homedir_env_mod);
    }
    if !one {
        src = skipwhite(src);
    }
    let mut dst_p: *mut ::core::ffi::c_char = dst;
    while *src as ::core::ffi::c_int != 0 && dstlen > 0 as size_t {
        let mut p: *mut ::core::ffi::c_char = homedir.get();
        let mut len: size_t = dirlen;
        loop {
            if len != 0
                && path_fnamencmp(src, p, len) == 0 as ::core::ffi::c_int
                && (vim_ispathsep(*src.offset(len as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
                    || !one
                        && (*src.offset(len as isize) as ::core::ffi::c_int
                            == ',' as ::core::ffi::c_int
                            || *src.offset(len as isize) as ::core::ffi::c_int
                                == ' ' as ::core::ffi::c_int)
                    || *src.offset(len as isize) as ::core::ffi::c_int == NUL)
            {
                src = src.offset(len as isize);
                dstlen = dstlen.wrapping_sub(1);
                if dstlen > 0 as size_t {
                    let c2rust_fresh13 = dst_p;
                    dst_p = dst_p.offset(1);
                    *c2rust_fresh13 = '~' as ::core::ffi::c_char;
                }
                break;
            } else {
                if p == homedir_env_mod {
                    break;
                }
                p = homedir_env_mod;
                len = envlen;
            }
        }
        if dstlen == 0 as size_t {
            break;
        } else {
            while *src as ::core::ffi::c_int != 0
                && (one as ::core::ffi::c_int != 0
                    || *src as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                        && *src as ::core::ffi::c_int != ' ' as ::core::ffi::c_int)
                && {
                    dstlen = dstlen.wrapping_sub(1);
                    dstlen > 0 as size_t
                }
            {
                let c2rust_fresh14 = src;
                src = src.offset(1);
                let c2rust_fresh15 = dst_p;
                dst_p = dst_p.offset(1);
                *c2rust_fresh15 = *c2rust_fresh14;
            }
            if dstlen == 0 as size_t {
                break;
            }
            while (*src as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                || *src as ::core::ffi::c_int == ',' as ::core::ffi::c_int)
                && {
                    dstlen = dstlen.wrapping_sub(1);
                    dstlen > 0 as size_t
                }
            {
                let c2rust_fresh16 = src;
                src = src.offset(1);
                let c2rust_fresh17 = dst_p;
                dst_p = dst_p.offset(1);
                *c2rust_fresh17 = *c2rust_fresh16;
            }
        }
    }
    *dst_p = NUL as ::core::ffi::c_char;
    xfree(homedir_env as *mut ::core::ffi::c_void);
    if must_free {
        xfree(homedir_env_mod as *mut ::core::ffi::c_void);
    }
    return dst_p.offset_from(dst) as size_t;
}
pub unsafe extern "C" fn home_replace_save(
    mut buf: *mut buf_T,
    mut src: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut len: size_t = 3 as size_t;
    if !src.is_null() {
        len = len.wrapping_add(strlen(src));
    }
    let mut dst: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    home_replace(buf, src, dst, len, true_0 != 0);
    return dst;
}
pub unsafe extern "C" fn get_env_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    '_c2rust_label: {
        if idx >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"idx >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1216 as ::core::ffi::c_uint,
                b"char *get_env_name(expand_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut envname: *mut ::core::ffi::c_char = os_getenvname_at_index(idx as size_t);
    if !envname.is_null() {
        xstrlcpy(
            &raw mut (*xp).xp_buf as *mut ::core::ffi::c_char,
            envname,
            EXPAND_BUF_LEN as ::core::ffi::c_int as size_t,
        );
        xfree(envname as *mut ::core::ffi::c_void);
        return &raw mut (*xp).xp_buf as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_setenv_append_path(mut fname: *const ::core::ffi::c_char) -> bool {
    if !path_is_absolute(fname) {
        internal_error(b"os_setenv_append_path()\0".as_ptr() as *const ::core::ffi::c_char);
        return false_0 != 0;
    }
    let mut tail: *const ::core::ffi::c_char =
        path_tail_with_sep(fname as *mut ::core::ffi::c_char);
    let mut dirlen: size_t = tail.offset_from(fname) as size_t;
    '_c2rust_label: {
        if tail >= fname
            && dirlen.wrapping_add(1 as size_t)
                < ::core::mem::size_of::<[::core::ffi::c_char; 4096]>()
        {
        } else {
            __assert_fail(
                b"tail >= fname && dirlen + 1 < sizeof(os_buf)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1247 as ::core::ffi::c_uint,
                b"_Bool os_setenv_append_path(const char *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    xmemcpyz(
        os_buf.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        fname as *const ::core::ffi::c_void,
        dirlen,
    );
    let mut path: *mut ::core::ffi::c_char =
        os_getenv(b"PATH\0".as_ptr() as *const ::core::ffi::c_char);
    let pathlen: size_t = if !path.is_null() {
        strlen(path)
    } else {
        0 as size_t
    };
    let newlen: size_t = pathlen.wrapping_add(dirlen).wrapping_add(2 as size_t);
    let mut retval: bool = false_0 != 0;
    if newlen < MAX_ENVPATHLEN as size_t {
        let mut temp: *mut ::core::ffi::c_char = xmalloc(newlen) as *mut ::core::ffi::c_char;
        if pathlen == 0 as size_t {
            *temp.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        } else {
            xstrlcpy(temp, path, newlen);
            if ENV_SEPCHAR
                != *path.offset(pathlen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            {
                xstrlcat(temp, ENV_SEPSTR.as_ptr(), newlen);
            }
        }
        xstrlcat(temp, os_buf.ptr() as *mut ::core::ffi::c_char, newlen);
        os_setenv(
            b"PATH\0".as_ptr() as *const ::core::ffi::c_char,
            temp,
            1 as ::core::ffi::c_int,
        );
        xfree(temp as *mut ::core::ffi::c_void);
        retval = true_0 != 0;
    }
    xfree(path as *mut ::core::ffi::c_void);
    return retval;
}
pub const MAX_ENVPATHLEN: ::core::ffi::c_int = INT_MAX;
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_shell_is_cmdexe(mut sh: *const ::core::ffi::c_char) -> bool {
    if *sh as ::core::ffi::c_int == NUL {
        return false_0 != 0;
    }
    if striequal(sh, b"$COMSPEC\0".as_ptr() as *const ::core::ffi::c_char) {
        let mut comspec: *mut ::core::ffi::c_char =
            os_getenv_noalloc(b"COMSPEC\0".as_ptr() as *const ::core::ffi::c_char);
        return striequal(
            b"cmd.exe\0".as_ptr() as *const ::core::ffi::c_char,
            path_tail(comspec),
        );
    }
    if striequal(sh, b"cmd.exe\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int != 0
        || striequal(sh, b"cmd\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int != 0
    {
        return true_0 != 0;
    }
    return striequal(
        b"cmd.exe\0".as_ptr() as *const ::core::ffi::c_char,
        path_tail(sh),
    );
}
pub unsafe extern "C" fn vim_unsetenv_ext(mut var: *const ::core::ffi::c_char) {
    os_unsetenv(var);
    if strcasecmp(
        var as *mut ::core::ffi::c_char,
        b"VIM\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        didset_vim.set(false_0 != 0);
    } else if strcasecmp(
        var as *mut ::core::ffi::c_char,
        b"VIMRUNTIME\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        didset_vimruntime.set(false_0 != 0);
    }
}
pub unsafe extern "C" fn vim_setenv_ext(
    mut name: *const ::core::ffi::c_char,
    mut val: *const ::core::ffi::c_char,
) {
    os_setenv(name, val, 1 as ::core::ffi::c_int);
    if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"HOME\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        init_homedir();
    } else if didset_vim.get() as ::core::ffi::c_int != 0
        && strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"VIM\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        didset_vim.set(false_0 != 0);
    } else if didset_vimruntime.get() as ::core::ffi::c_int != 0
        && strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"VIMRUNTIME\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        didset_vimruntime.set(false_0 != 0);
    }
}
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
pub const ENV_SEPSTR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b":\0") };
