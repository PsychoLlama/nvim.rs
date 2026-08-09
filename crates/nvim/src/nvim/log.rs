/// `#[macro_export]` publishes at the crate root; this re-export lets callers
/// name the macro where the rest of the logging API lives, and brings it into
/// scope here ahead of its own textual definition.
pub use crate::logmsg_c;
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
    __errno_location, fclose, fflush, fopen, fprintf, fputc, fputs, snprintf, stderr, stdout,
    strerror, strftime,
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
    let mut log_file: *mut FILE = fopen(fname, c"a".as_ptr()) as *mut FILE;
    if log_file.is_null() {
        return false_0 != 0;
    }
    fclose(log_file);
    return true_0 != 0;
}
unsafe extern "C" fn log_path_init() {
    let mut size: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 4097]>();
    expand_env(
        c"$NVIM_LOG_FILE".as_ptr() as *mut ::core::ffi::c_char,
        log_file_path.ptr() as *mut ::core::ffi::c_char,
        size as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
    );
    let mut user_set: bool = !strequal(
        c"$NVIM_LOG_FILE".as_ptr(),
        log_file_path.ptr() as *mut ::core::ffi::c_char,
    );
    if !user_set
        || (*log_file_path.ptr())[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == NUL
        || os_isdir(log_file_path.ptr() as *mut ::core::ffi::c_char) as ::core::ffi::c_int != 0
        || !log_try_create(log_file_path.ptr() as *mut ::core::ffi::c_char)
    {
        if user_set {
            os_setenv(
                c"__NVIM_LOG_FILE_WANT".as_ptr(),
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
        let mut defaultpath: *mut ::core::ffi::c_char =
            stdpaths_user_state_subpath(c"nvim.log".as_ptr(), 0 as size_t, true_0 != 0);
        let mut len: size_t = xstrlcpy(
            log_file_path.ptr() as *mut ::core::ffi::c_char,
            defaultpath,
            size,
        );
        xfree(defaultpath as *mut ::core::ffi::c_void);
        if len >= size || !log_try_create(log_file_path.ptr() as *mut ::core::ffi::c_char) {
            if !user_set {
                os_setenv(
                    c"__NVIM_LOG_FILE_WANT".as_ptr(),
                    log_file_path.ptr() as *mut ::core::ffi::c_char,
                    true_0,
                );
            }
            len = xstrlcpy(
                log_file_path.ptr() as *mut ::core::ffi::c_char,
                c"nvim.log".as_ptr(),
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
            logmsg_c!(
                LOGLVL_WRN,
                ::core::ptr::null::<::core::ffi::c_char>(),
                c"log_path_init".as_ptr(),
                106 as ::core::ffi::c_int,
                true_0 != 0,
                c"Failed to create directory %s for writing logs: %s".as_ptr(),
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
/// Set while a log line is being written, so a log call made from inside one
/// is refused rather than interleaved. `did_recursion_msg` keeps that refusal
/// to a single user-visible complaint per session.
static logging: GlobalCell<bool> = GlobalCell::new(false);
static did_recursion_msg: GlobalCell<bool> = GlobalCell::new(false);

/// The half of a log line that runs before its payload: the initialisation,
/// level and recursion guards, the log lock, the file handle, and the
/// date/level/name/location prefix.
///
/// Returns the open log file with the lock held and the prefix already
/// written, or null when the line is not going to be written at all — in
/// which case nothing is held and the caller is done. **Every non-null
/// return has to be paired with a [`logmsg_finish`]**; [`logmsg_c!`] is that
/// pairing, and is how this should be called.
///
/// # Safety
/// `context` and `func_name` are NUL-terminated or null, and outlive the
/// call.
pub unsafe fn logmsg_begin(
    log_level: ::core::ffi::c_int,
    context: *const ::core::ffi::c_char,
    func_name: *const ::core::ffi::c_char,
    line_num: ::core::ffi::c_int,
) -> *mut FILE {
    if !did_log_init.get() {
        (*g_stats.ptr()).log_skip += 1;
        return ::core::ptr::null_mut();
    }
    if log_level < g_min_log_level.get() {
        return ::core::ptr::null_mut();
    }
    log_lock();
    if logging.get() {
        if !did_recursion_msg.get() {
            did_recursion_msg.set(true);
            msg_schedule_semsg_c!(
                c"E5430: %s:%d: recursive log!".as_ptr(),
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
        return ::core::ptr::null_mut();
    }
    logging.set(true);
    let log_file: *mut FILE = open_log_file();
    if !log_write_prefix(log_file, log_level, context, func_name, line_num) {
        // The prefix is the head of the line; with none written there is
        // nothing to append to, so release everything here and report the
        // same failure the payload would have.
        logmsg_finish(log_file, false, false);
        return ::core::ptr::null_mut();
    }
    log_file
}

/// The half of a log line that runs after its payload: the end-of-line, the
/// flush, and releasing what [`logmsg_begin`] took.
///
/// `payload_ok` says whether the payload was written; a failed payload skips
/// the terminator and the flush but still releases. Returns whether the whole
/// line landed.
///
/// # Safety
/// `log_file` is the non-null handle a [`logmsg_begin`] call returned, and
/// this is its first `logmsg_finish`.
pub unsafe fn logmsg_finish(log_file: *mut FILE, eol: bool, payload_ok: bool) -> bool {
    let mut ret = payload_ok;
    if ret {
        if eol {
            fputc('\n' as ::core::ffi::c_int, log_file);
        }
        if fflush(log_file) == EOF {
            ret = false;
        }
    }
    if log_file != stderr && log_file != stdout {
        fclose(log_file);
    }
    logging.set(false);
    log_unlock();
    ret
}

/// Write one `printf`-formatted line to the log file, at `log_level`, tagged
/// with `context`/`func_name`/`line_num` and terminated by a newline when
/// `eol`. Evaluates to `bool`: whether the line landed.
///
/// This is `logmsg()` split at the seam it already had — [`logmsg_begin`]
/// takes the lock and writes the prefix, the expansion writes the payload
/// with a direct `fprintf`, [`logmsg_finish`] terminates it and releases.
/// Same handle, same order, same bytes as the C wrapper, without a C-variadic
/// definition. As with the function, the *call site* supplies the `unsafe`.
///
/// The payload arguments appear in both arms, so a log the guards refuse
/// still evaluates them — C evaluated every argument before the callee could
/// decide. They are evaluated *after* the guards rather than before, which
/// only a payload argument that itself logs could observe; none does.
#[macro_export]
macro_rules! logmsg_c {
    ($log_level:expr, $context:expr, $func_name:expr, $line_num:expr,
     $eol:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let log_level = $log_level;
        let context = $context;
        let func_name = $func_name;
        let line_num = $line_num;
        let eol = $eol;
        let fmt = $fmt;
        let log_file =
            $crate::src::nvim::log::logmsg_begin(log_level, context, func_name, line_num);
        if log_file.is_null() {
            $(let _ = $arg;)*
            false
        } else {
            let payload_ok =
                $crate::src::nvim::os::libc::fprintf(log_file, fmt $(, $arg)*) >= 0;
            $crate::src::nvim::log::logmsg_finish(log_file, eol, payload_ok)
        }
    }};
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
            c"a".as_ptr(),
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
        c"open_log_file".as_ptr(),
        234 as ::core::ffi::c_int,
    ) {
        fputs(msg.as_ptr() as *const ::core::ffi::c_char, stderr);
        fflush(stderr);
    }
    return stderr;
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
        c"DBG".as_ptr(),
        c"INF".as_ptr(),
        c"WRN".as_ptr(),
        c"ERR".as_ptr(),
    ]);
    debug_assert!(
        log_level >= 1 as ::core::ffi::c_int && log_level <= 4 as ::core::ffi::c_int,
        "log_level >= LOGLVL_DBG && log_level <= LOGLVL_ERR"
    );
    let mut local_time: tm = tm_zeroed();
    if !os_localtime(&mut local_time) {
        return false_0 != 0;
    }
    let mut date_time: [::core::ffi::c_char; 20] = [0; 20];
    if strftime(
        &raw mut date_time as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
        c"%Y-%m-%dT%H:%M:%S".as_ptr(),
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
                    c"ui/c/%s".as_ptr()
                } else {
                    c"c/%s".as_ptr()
                },
                parent,
            );
        } else if *serv.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
            snprintf(
                name.ptr() as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>(),
                if ui as ::core::ffi::c_int != 0 {
                    c"ui/%s".as_ptr()
                } else {
                    c"%s".as_ptr()
                },
                serv,
            );
        } else {
            let mut pid: int64_t = os_get_pid();
            snprintf(
                name.ptr() as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>(),
                c"%s.%-5ld".as_ptr(),
                if ui as ::core::ffi::c_int != 0 {
                    c"ui".as_ptr()
                } else {
                    c"?".as_ptr()
                },
                pid,
            );
        }
    }
    let mut rv: ::core::ffi::c_int = if line_num == -1 as ::core::ffi::c_int || func_name.is_null()
    {
        fprintf(
            log_file,
            c"%s %s.%03d %-10s %s".as_ptr(),
            (*log_levels.ptr())[log_level as usize],
            &raw mut date_time as *mut ::core::ffi::c_char,
            millis,
            name.ptr() as *mut ::core::ffi::c_char,
            if context.is_null() {
                c"?:".as_ptr()
            } else {
                context
            },
        )
    } else {
        fprintf(
            log_file,
            c"%s %s.%03d %-10s %s%s:%d: ".as_ptr(),
            (*log_levels.ptr())[log_level as usize],
            &raw mut date_time as *mut ::core::ffi::c_char,
            millis,
            name.ptr() as *mut ::core::ffi::c_char,
            if context.is_null() {
                c"".as_ptr()
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
