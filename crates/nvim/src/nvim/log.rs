use crate::msg_schedule_semsg_c;
use crate::src::nvim::eval::vars::get_vim_var_str;
use crate::src::nvim::event::libuv::{
    uv_mutex_init_recursive, uv_mutex_lock, uv_mutex_unlock, uv_print_all_handles, uv_strerror,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{g_min_log_level, g_stats, ui_client_channel_id};
use crate::src::nvim::memory::{strequal, xfree, xstrlcpy};
use crate::src::nvim::os::env::{expand_env, os_get_pid, os_getenv_buf, os_setenv};
use crate::src::nvim::os::fs::{os_isdir, os_mkdir_recurse};
use crate::src::nvim::os::libc::{
    __assert_fail, __errno_location, fclose, fflush, fopen, fprintf, fputc, fputs, snprintf,
    stderr, stdout, strerror, strftime, vfprintf,
};
use crate::src::nvim::os::stdpaths::{get_xdg_home, stdpaths_user_state_subpath};
use crate::src::nvim::os::time::{os_localtime, tm_zeroed};
use crate::src::nvim::path::path_tail;
use crate::src::nvim::types::{
    __pthread_internal_list, __pthread_list_t, __pthread_mutex_s, FILE, VV_SEND_SERVER, XDGVarType,
    int32_t, int64_t, pthread_mutex_t, size_t, tm, uv_loop_t, uv_mutex_t,
};
unsafe extern "C" {
    fn uv_gettimeofday(tv: *mut uv_timeval64_t) -> ::core::ffi::c_int;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_timeval64_t {
    pub tv_sec: int64_t,
    pub tv_usec: int32_t,
}
pub const kXDGStateHome: XDGVarType = 3;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const EOF: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
static log_file_path: GlobalCell<[::core::ffi::c_char; 4097]> = GlobalCell::new([0; 4097]);
static did_log_init: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static mutex: GlobalCell<uv_mutex_t> = GlobalCell::new(pthread_mutex_t {
    __data: __pthread_mutex_s {
        __lock: 0,
        __count: 0,
        __owner: 0,
        __nusers: 0,
        __kind: 0,
        __spins: 0,
        __elision: 0,
        __list: __pthread_list_t {
            __prev: ::core::ptr::null_mut::<__pthread_internal_list>(),
            __next: ::core::ptr::null_mut::<__pthread_internal_list>(),
        },
    },
});
unsafe extern "C" fn log_try_create(mut fname: *mut ::core::ffi::c_char) -> bool {
    if fname.is_null()
        || *fname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        return false_0 != 0;
    }
    let mut log_file: *mut FILE =
        fopen(fname, b"a\0".as_ptr() as *const ::core::ffi::c_char) as *mut FILE;
    if log_file.is_null() {
        return false_0 != 0;
    }
    fclose(log_file);
    return true_0 != 0;
}
unsafe extern "C" fn log_path_init() {
    let mut size: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 4097]>();
    expand_env(
        b"$NVIM_LOG_FILE\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        log_file_path.ptr() as *mut ::core::ffi::c_char,
        size as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
    );
    let mut user_set: bool = !strequal(
        b"$NVIM_LOG_FILE\0".as_ptr() as *const ::core::ffi::c_char,
        log_file_path.ptr() as *mut ::core::ffi::c_char,
    );
    if !user_set
        || (*log_file_path.ptr())[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == NUL
        || os_isdir(log_file_path.ptr() as *mut ::core::ffi::c_char) as ::core::ffi::c_int != 0
        || !log_try_create(log_file_path.ptr() as *mut ::core::ffi::c_char)
    {
        if user_set {
            os_setenv(
                b"__NVIM_LOG_FILE_WANT\0".as_ptr() as *const ::core::ffi::c_char,
                log_file_path.ptr() as *mut ::core::ffi::c_char,
                true_0,
            );
        }
        let mut loghome: *mut ::core::ffi::c_char = get_xdg_home(kXDGStateHome);
        let mut failed_dir: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut log_dir_failure: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if !os_isdir(loghome) {
            log_dir_failure = os_mkdir_recurse(
                loghome,
                0o700 as int32_t,
                &raw mut failed_dir,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            );
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut loghome as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        let mut defaultpath: *mut ::core::ffi::c_char = stdpaths_user_state_subpath(
            b"nvim.log\0".as_ptr() as *const ::core::ffi::c_char,
            0 as size_t,
            true_0 != 0,
        );
        let mut len: size_t = xstrlcpy(
            log_file_path.ptr() as *mut ::core::ffi::c_char,
            defaultpath,
            size,
        );
        xfree(defaultpath as *mut ::core::ffi::c_void);
        if len >= size || !log_try_create(log_file_path.ptr() as *mut ::core::ffi::c_char) {
            if !user_set {
                os_setenv(
                    b"__NVIM_LOG_FILE_WANT\0".as_ptr() as *const ::core::ffi::c_char,
                    log_file_path.ptr() as *mut ::core::ffi::c_char,
                    true_0,
                );
            }
            len = xstrlcpy(
                log_file_path.ptr() as *mut ::core::ffi::c_char,
                b"nvim.log\0".as_ptr() as *const ::core::ffi::c_char,
                size,
            );
        }
        if len >= size || !log_try_create(log_file_path.ptr() as *mut ::core::ffi::c_char) {
            (*log_file_path.ptr())[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
            return;
        }
        os_setenv(
            ENV_LOGFILE.as_ptr(),
            log_file_path.ptr() as *mut ::core::ffi::c_char,
            true_0,
        );
        if log_dir_failure != 0 {
            logmsg(
                LOGLVL_WRN,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"log_path_init\0".as_ptr() as *const ::core::ffi::c_char,
                106 as ::core::ffi::c_int,
                true_0 != 0,
                b"Failed to create directory %s for writing logs: %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
                failed_dir,
                uv_strerror(log_dir_failure),
            );
        }
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut failed_dir as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL;
        let _ = *ptr__0;
    }
}
pub unsafe extern "C" fn log_init() {
    uv_mutex_init_recursive(mutex.ptr());
    log_path_init();
    did_log_init.set(true_0 != 0);
}
pub unsafe extern "C" fn log_lock() {
    uv_mutex_lock(mutex.ptr());
}
pub unsafe extern "C" fn log_unlock() {
    uv_mutex_unlock(mutex.ptr());
}
pub unsafe extern "C" fn logmsg(
    mut log_level: ::core::ffi::c_int,
    mut context: *const ::core::ffi::c_char,
    mut func_name: *const ::core::ffi::c_char,
    mut line_num: ::core::ffi::c_int,
    mut eol: bool,
    mut fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> bool {
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    static did_msg: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if !did_log_init.get() {
        (*g_stats.ptr()).log_skip += 1;
        return false_0 != 0;
    }
    if log_level < g_min_log_level.get() {
        return false_0 != 0;
    }
    log_lock();
    if recursive.get() {
        if !did_msg.get() {
            did_msg.set(true_0 != 0);
            msg_schedule_semsg_c!(
                b"E5430: %s:%d: recursive log!\0".as_ptr() as *const ::core::ffi::c_char,
                if !func_name.is_null() {
                    func_name
                } else {
                    context
                },
                line_num,
            );
        }
        (*g_stats.ptr()).log_skip += 1;
        log_unlock();
        return false_0 != 0;
    }
    recursive.set(true_0 != 0);
    let mut ret: bool = false_0 != 0;
    let mut log_file: *mut FILE = open_log_file();
    let mut args: ::core::ffi::VaList;
    args = c2rust_args.clone();
    ret = v_do_log_to_file(
        log_file, log_level, context, func_name, line_num, eol, fmt, args,
    );
    if log_file != stderr && log_file != stdout {
        fclose(log_file);
    }
    recursive.set(false_0 != 0);
    log_unlock();
    return ret;
}
pub unsafe extern "C" fn log_uv_handles(mut loop_0: *mut ::core::ffi::c_void) {
    let mut l: *mut uv_loop_t = loop_0 as *mut uv_loop_t;
    log_lock();
    let mut log_file: *mut FILE = open_log_file();
    uv_print_all_handles(l, log_file);
    if log_file != stderr && log_file != stdout {
        fclose(log_file);
    }
    log_unlock();
}
pub unsafe extern "C" fn open_log_file() -> *mut FILE {
    *__errno_location() = 0 as ::core::ffi::c_int;
    if (*log_file_path.ptr())[0 as ::core::ffi::c_int as usize] != 0 {
        let mut f: *mut FILE = fopen(
            log_file_path.ptr() as *mut ::core::ffi::c_char,
            b"a\0".as_ptr() as *const ::core::ffi::c_char,
        ) as *mut FILE;
        if !f.is_null() {
            return f;
        }
    }
    // strerror before the prefix writer can clobber fopen's errno; the
    // trailing newline stands in for the old eol=true.
    let msg = format!(
        "failed to open $NVIM_LOG_FILE ({}): {}\n\0",
        ::core::ffi::CStr::from_ptr(strerror(*__errno_location())).to_string_lossy(),
        ::core::ffi::CStr::from_ptr(log_file_path.ptr() as *const ::core::ffi::c_char)
            .to_string_lossy(),
    );
    if log_write_prefix(
        stderr,
        LOGLVL_ERR,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"open_log_file\0".as_ptr() as *const ::core::ffi::c_char,
        234 as ::core::ffi::c_int,
    ) {
        fputs(msg.as_ptr() as *const ::core::ffi::c_char, stderr);
        fflush(stderr);
    }
    return stderr;
}
unsafe extern "C" fn v_do_log_to_file(
    mut log_file: *mut FILE,
    mut log_level: ::core::ffi::c_int,
    mut context: *const ::core::ffi::c_char,
    mut func_name: *const ::core::ffi::c_char,
    mut line_num: ::core::ffi::c_int,
    mut eol: bool,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ::core::ffi::VaList,
) -> bool {
    if !log_write_prefix(log_file, log_level, context, func_name, line_num) {
        return false_0 != 0;
    }
    if vfprintf(log_file, fmt, args) < 0 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    if eol {
        fputc('\n' as ::core::ffi::c_int, log_file);
    }
    if fflush(log_file) == EOF {
        return false_0 != 0;
    }
    return true_0 != 0;
}
/// The date/level/name/source-location head of a log line, up to where the
/// payload starts. Split out of `v_do_log_to_file` so a preformatted message
/// (`open_log_file`'s fallback) can log without a variadic hop.
unsafe extern "C" fn log_write_prefix(
    mut log_file: *mut FILE,
    mut log_level: ::core::ffi::c_int,
    mut context: *const ::core::ffi::c_char,
    mut func_name: *const ::core::ffi::c_char,
    mut line_num: ::core::ffi::c_int,
) -> bool {
    static name: GlobalCell<[::core::ffi::c_char; 32]> = GlobalCell::new([0; 32]);
    static log_levels: GlobalCell<[*const ::core::ffi::c_char; 5]> = GlobalCell::new([
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"DBG\0".as_ptr() as *const ::core::ffi::c_char,
        b"INF\0".as_ptr() as *const ::core::ffi::c_char,
        b"WRN\0".as_ptr() as *const ::core::ffi::c_char,
        b"ERR\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    '_c2rust_label: {
        if log_level >= 1 as ::core::ffi::c_int && log_level <= 4 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"log_level >= LOGLVL_DBG && log_level <= LOGLVL_ERR\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/log.rs\0".as_ptr() as *const ::core::ffi::c_char,
                313 as ::core::ffi::c_uint,
                b"_Bool log_write_prefix(FILE *, int, const char *, const char *, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut local_time: tm = tm_zeroed();
    if !os_localtime(&mut local_time) {
        return false_0 != 0;
    }
    let mut date_time: [::core::ffi::c_char; 20] = [0; 20];
    if strftime(
        &raw mut date_time as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
        b"%Y-%m-%dT%H:%M:%S\0".as_ptr() as *const ::core::ffi::c_char,
        &raw mut local_time,
    ) == 0 as size_t
    {
        return false_0 != 0;
    }
    let mut millis: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut curtime: uv_timeval64_t = uv_timeval64_t {
        tv_sec: 0,
        tv_usec: 0,
    };
    if uv_gettimeofday(&raw mut curtime) == 0 as ::core::ffi::c_int {
        millis = curtime.tv_usec as ::core::ffi::c_int / 1000 as ::core::ffi::c_int;
    }
    let mut ui: bool = ui_client_channel_id.get() != 0;
    let mut regen: bool = ui as ::core::ffi::c_int != 0
        || (*name.ptr())[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == NUL
        || (*name.ptr())[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            == '?' as ::core::ffi::c_int;
    if regen {
        let mut parent_buf: [::core::ffi::c_char; 4096] = [0; 4096];
        let mut parent: *const ::core::ffi::c_char = path_tail(os_getenv_buf(
            ENV_NVIM.as_ptr(),
            &raw mut parent_buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        ));
        let mut serv: *const ::core::ffi::c_char = path_tail(get_vim_var_str(VV_SEND_SERVER));
        if *parent.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
            snprintf(
                name.ptr() as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>(),
                if ui as ::core::ffi::c_int != 0 {
                    b"ui/c/%s\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"c/%s\0".as_ptr() as *const ::core::ffi::c_char
                },
                parent,
            );
        } else if *serv.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
            snprintf(
                name.ptr() as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>(),
                if ui as ::core::ffi::c_int != 0 {
                    b"ui/%s\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char
                },
                serv,
            );
        } else {
            let mut pid: int64_t = os_get_pid();
            snprintf(
                name.ptr() as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>(),
                b"%s.%-5ld\0".as_ptr() as *const ::core::ffi::c_char,
                if ui as ::core::ffi::c_int != 0 {
                    b"ui\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"?\0".as_ptr() as *const ::core::ffi::c_char
                },
                pid,
            );
        }
    }
    let mut rv: ::core::ffi::c_int = if line_num == -1 as ::core::ffi::c_int || func_name.is_null()
    {
        fprintf(
            log_file,
            b"%s %s.%03d %-10s %s\0".as_ptr() as *const ::core::ffi::c_char,
            (*log_levels.ptr())[log_level as usize],
            &raw mut date_time as *mut ::core::ffi::c_char,
            millis,
            name.ptr() as *mut ::core::ffi::c_char,
            if context.is_null() {
                b"?:\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                context
            },
        )
    } else {
        fprintf(
            log_file,
            b"%s %s.%03d %-10s %s%s:%d: \0".as_ptr() as *const ::core::ffi::c_char,
            (*log_levels.ptr())[log_level as usize],
            &raw mut date_time as *mut ::core::ffi::c_char,
            millis,
            name.ptr() as *mut ::core::ffi::c_char,
            if context.is_null() {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                context
            },
            func_name,
            line_num,
        )
    };
    if rv < 0 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    return true_0 != 0;
}
/// The levels `log_message` takes, and 'verbose' compares against.
pub const LOGLVL_DBG: ::core::ffi::c_int = 1;
pub const LOGLVL_INF: ::core::ffi::c_int = 2;
pub const LOGLVL_WRN: ::core::ffi::c_int = 3;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4;
pub const ENV_LOGFILE: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"NVIM_LOG_FILE\0") };
pub const ENV_NVIM: [::core::ffi::c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"NVIM\0") };
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
