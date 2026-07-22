use crate::src::nvim::channel::channel_from_connection;
use crate::src::nvim::eval::vars::{get_vim_var_str, set_vim_var_string};
use crate::src::nvim::event::libuv::{uv_freeaddrinfo, uv_strerror};
use crate::src::nvim::event::socket::{
    socket_watcher_close, socket_watcher_init, socket_watcher_start,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::main::{main_loop, IObuff, NameBuff};
use crate::src::nvim::memory::{strequal, xcalloc, xfree, xmalloc, xstrdup, xstrlcpy};
use crate::src::nvim::os::env::{os_env_exists, os_get_pid, os_getenv, os_unsetenv};
use crate::src::nvim::os::libc::{snprintf, strcmp, strlen, strstr};
use crate::src::nvim::os::stdpaths::{get_appname, stdpaths_get_xdg_var};
use crate::src::nvim::path::fix_fname;
pub use crate::src::nvim::types::{
    Loop, LuaRef, MultiQueue, Proc, ProcType, RStream, ScopeType, SocketWatcher, Stream, TriState,
    VarLockStatus, VimVarIndex, XDGVarType, __pthread_internal_list, __pthread_list_t,
    __pthread_mutex_s, __pthread_rwlock_arch_t, __socklen_t, addrinfo, dict_T, dictvar_S, garray_T,
    hash_T, hashitem_T, hashtab_T, int64_t, internal_proc_cb, loop_0,
    loop_0_children as C2Rust_Unnamed_11, multiqueue, proc, proc_exit_cb, proc_state_cb,
    pthread_mutex_t, pthread_rwlock_t, ptrdiff_t, queue, rstream, sa_family_t, size_t, sockaddr,
    socket_cb, socket_close_cb, socket_watcher, socket_watcher_uv as C2Rust_Unnamed_13,
    socket_watcher_uv_pipe as C2Rust_Unnamed_14, socket_watcher_uv_tcp as C2Rust_Unnamed_15,
    socklen_t, ssize_t, stream, stream_close_cb, stream_read_cb, stream_uv as C2Rust_Unnamed_12,
    stream_write_cb, uint32_t, uint64_t, uint8_t, uv__io_cb, uv__io_s, uv__io_t, uv__queue,
    uv_alloc_cb, uv_async_cb, uv_async_s, uv_async_s_u as C2Rust_Unnamed_3, uv_async_t, uv_buf_t,
    uv_close_cb, uv_connect_cb, uv_connect_s, uv_connect_t, uv_connection_cb, uv_file, uv_handle_s,
    uv_handle_s_u as C2Rust_Unnamed_0, uv_handle_t, uv_handle_type, uv_idle_cb, uv_idle_s,
    uv_idle_s_u as C2Rust_Unnamed_10, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_4, uv_loop_s_timer_heap as C2Rust_Unnamed_2, uv_loop_t,
    uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_7, uv_pipe_t, uv_read_cb, uv_req_type,
    uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t, uv_signal_cb, uv_signal_s,
    uv_signal_s_tree_entry as C2Rust_Unnamed, uv_signal_s_u as C2Rust_Unnamed_1, uv_signal_t,
    uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_5, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_6, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_8, uv_timer_s_u as C2Rust_Unnamed_9, uv_timer_t, QUEUE,
};
extern "C" {
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
}
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
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
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
pub const kXDGDataDirs: XDGVarType = 6;
pub const kXDGConfigDirs: XDGVarType = 5;
pub const kXDGRuntimeDir: XDGVarType = 4;
pub const kXDGStateHome: XDGVarType = 3;
pub const kXDGCacheHome: XDGVarType = 2;
pub const kXDGDataHome: XDGVarType = 1;
pub const kXDGConfigHome: XDGVarType = 0;
pub const kXDGNone: XDGVarType = -1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL,
};
pub const LOGLVL_WRN: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const MAX_CONNECTIONS: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const ENV_LISTEN: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"NVIM_LISTEN_ADDRESS\0")
};
static watchers: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);
#[no_mangle]
pub unsafe extern "C" fn server_init(mut listen_addr: *const ::core::ffi::c_char) -> bool {
    let mut ok: bool = true_0 != 0;
    let mut must_free: bool = false_0 != 0;
    let mut user_arg: TriState = kTrue;
    ga_init(
        watchers.ptr(),
        ::core::mem::size_of::<*mut SocketWatcher>() as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    if listen_addr.is_null()
        || *listen_addr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '\0' as ::core::ffi::c_int
    {
        if os_env_exists(ENV_LISTEN.as_ptr(), true_0 != 0) {
            user_arg = kFalse;
            listen_addr = os_getenv(ENV_LISTEN.as_ptr());
        } else {
            user_arg = kNone;
            listen_addr = server_address_new(::core::ptr::null::<::core::ffi::c_char>());
        }
        must_free = true_0 != 0;
    }
    let mut rv: ::core::ffi::c_int = server_start(listen_addr);
    if os_env_exists(
        b"__NVIM_TEST_LOG\0".as_ptr() as *const ::core::ffi::c_char,
        false_0 != 0,
    ) {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"server_init\0".as_ptr() as *const ::core::ffi::c_char,
            58 as ::core::ffi::c_int,
            true_0 != 0,
            b"test log message\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if !(rv == 0 as ::core::ffi::c_int
        || user_arg as ::core::ffi::c_int == kNone as ::core::ffi::c_int)
    {
        snprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            if user_arg as ::core::ffi::c_int == kTrue as ::core::ffi::c_int {
                b"Failed to --listen: %s: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"Failed $NVIM_LISTEN_ADDRESS: %s: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char
            },
            if rv < 0 as ::core::ffi::c_int {
                uv_strerror(rv)
            } else if rv == 1 as ::core::ffi::c_int {
                b"empty address\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"?\0".as_ptr() as *const ::core::ffi::c_char
            },
            listen_addr,
        );
        ok = false_0 != 0;
    }
    if os_env_exists(ENV_LISTEN.as_ptr(), false_0 != 0) {
        os_unsetenv(ENV_LISTEN.as_ptr());
    }
    if must_free {
        xfree(listen_addr as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void);
    }
    return ok;
}
unsafe extern "C" fn close_socket_watcher(mut watcher: *mut *mut SocketWatcher) {
    socket_watcher_close(
        *watcher,
        Some(
            free_server as unsafe extern "C" fn(*mut SocketWatcher, *mut ::core::ffi::c_void) -> (),
        ),
    );
}
unsafe extern "C" fn set_vservername(mut srvs: *mut garray_T) {
    let mut default_server: *mut ::core::ffi::c_char = if (*srvs).ga_len > 0 as ::core::ffi::c_int {
        &raw mut (**((*srvs).ga_data as *mut *mut SocketWatcher)
            .offset(0 as ::core::ffi::c_int as isize))
        .addr as *mut ::core::ffi::c_char
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    set_vim_var_string(VV_SEND_SERVER, default_server, -1 as ptrdiff_t);
}
#[no_mangle]
pub unsafe extern "C" fn server_teardown() {
    let mut _gap: *mut garray_T = watchers.ptr();
    if !(*_gap).ga_data.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*_gap).ga_len {
            let mut _item: *mut *mut SocketWatcher =
                ((*_gap).ga_data as *mut *mut SocketWatcher).offset(i as isize);
            close_socket_watcher(_item);
            i += 1;
        }
    }
    ga_clear(_gap);
}
#[no_mangle]
pub unsafe extern "C" fn server_address_new(
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    static count: GlobalCell<uint32_t> = GlobalCell::new(0 as uint32_t);
    let mut fmt: [::core::ffi::c_char; 256] = [0; 256];
    let mut dir: *mut ::core::ffi::c_char = stdpaths_get_xdg_var(kXDGRuntimeDir);
    get_appname(true_0 != 0);
    let c2rust_fresh1 = count.get();
    count.set((*count.ptr()).wrapping_add(1));
    let mut r: ::core::ffi::c_int = snprintf(
        &raw mut fmt as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
        b"%s/%s.%lu.%u\0".as_ptr() as *const ::core::ffi::c_char,
        dir,
        if !name.is_null() {
            name
        } else {
            NameBuff.ptr() as *mut ::core::ffi::c_char as *const ::core::ffi::c_char
        },
        os_get_pid(),
        c2rust_fresh1,
    );
    xfree(dir as *mut ::core::ffi::c_void);
    if r as size_t >= ::core::mem::size_of::<[::core::ffi::c_char; 256]>() {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"server_address_new\0".as_ptr() as *const ::core::ffi::c_char,
            133 as ::core::ffi::c_int,
            true_0 != 0,
            b"truncated server address: %.40s...\0".as_ptr() as *const ::core::ffi::c_char,
            &raw mut fmt as *mut ::core::ffi::c_char,
        );
    }
    return xstrdup(&raw mut fmt as *mut ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn server_owns_pipe_address(mut address: *const ::core::ffi::c_char) -> bool {
    let mut result: bool = false_0 != 0;
    let mut path: *mut ::core::ffi::c_char = fix_fname(address);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*watchers.ptr()).ga_len {
        let mut addr: *mut ::core::ffi::c_char = fix_fname(
            &raw mut (**((*watchers.ptr()).ga_data as *mut *mut SocketWatcher).offset(i as isize))
                .addr as *mut ::core::ffi::c_char,
        );
        result = strequal(path, addr);
        xfree(addr as *mut ::core::ffi::c_void);
        if result {
            break;
        }
        i += 1;
    }
    xfree(path as *mut ::core::ffi::c_void);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn server_start(mut addr: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if addr.is_null() || *addr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        logmsg(
            LOGLVL_WRN,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"server_start\0".as_ptr() as *const ::core::ffi::c_char,
            169 as ::core::ffi::c_int,
            true_0 != 0,
            b"Empty or NULL address\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return 1 as ::core::ffi::c_int;
    }
    let mut isname: bool = strstr(addr, b":\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        && strstr(addr, b"/\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        && strstr(addr, b"\\\0".as_ptr() as *const ::core::ffi::c_char).is_null();
    let mut addr_gen: *mut ::core::ffi::c_char = if isname as ::core::ffi::c_int != 0 {
        server_address_new(addr)
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    let mut watcher: *mut SocketWatcher =
        xmalloc(::core::mem::size_of::<SocketWatcher>()) as *mut SocketWatcher;
    let mut result: ::core::ffi::c_int = socket_watcher_init(
        main_loop.ptr(),
        watcher,
        if isname as ::core::ffi::c_int != 0 {
            addr_gen as *const ::core::ffi::c_char
        } else {
            addr
        },
    );
    xfree(addr_gen as *mut ::core::ffi::c_void);
    if result < 0 as ::core::ffi::c_int {
        xfree(watcher as *mut ::core::ffi::c_void);
        return result;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*watchers.ptr()).ga_len {
        if strcmp(
            &raw mut (*watcher).addr as *mut ::core::ffi::c_char,
            &raw mut (**((*watchers.ptr()).ga_data as *mut *mut SocketWatcher).offset(i as isize))
                .addr as *mut ::core::ffi::c_char,
        ) == 0
        {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"server_start\0".as_ptr() as *const ::core::ffi::c_char,
                186 as ::core::ffi::c_int,
                true_0 != 0,
                b"Already listening on %s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut (*watcher).addr as *mut ::core::ffi::c_char,
            );
            if (*(*watcher).stream).type_0 as ::core::ffi::c_uint
                == UV_TCP as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                uv_freeaddrinfo((*watcher).uv.tcp.addrinfo);
            }
            socket_watcher_close(
                watcher,
                Some(
                    free_server
                        as unsafe extern "C" fn(*mut SocketWatcher, *mut ::core::ffi::c_void) -> (),
                ),
            );
            return 2 as ::core::ffi::c_int;
        }
        i += 1;
    }
    result = socket_watcher_start(
        watcher,
        MAX_CONNECTIONS,
        Some(
            connection_cb
                as unsafe extern "C" fn(
                    *mut SocketWatcher,
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
    );
    if result < 0 as ::core::ffi::c_int {
        logmsg(
            LOGLVL_WRN,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"server_start\0".as_ptr() as *const ::core::ffi::c_char,
            197 as ::core::ffi::c_int,
            true_0 != 0,
            b"Failed to start server: %s: %s\0".as_ptr() as *const ::core::ffi::c_char,
            uv_strerror(result),
            &raw mut (*watcher).addr as *mut ::core::ffi::c_char,
        );
        socket_watcher_close(
            watcher,
            Some(
                free_server
                    as unsafe extern "C" fn(*mut SocketWatcher, *mut ::core::ffi::c_void) -> (),
            ),
        );
        return result;
    }
    ga_grow(watchers.ptr(), 1 as ::core::ffi::c_int);
    let c2rust_fresh0 = (*watchers.ptr()).ga_len;
    (*watchers.ptr()).ga_len = (*watchers.ptr()).ga_len + 1;
    let c2rust_lvalue_ptr = &raw mut *((*watchers.ptr()).ga_data as *mut *mut SocketWatcher)
        .offset(c2rust_fresh0 as isize);
    *c2rust_lvalue_ptr = watcher;
    if strlen(get_vim_var_str(VV_SEND_SERVER)) == 0 as size_t {
        set_vservername(watchers.ptr());
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn server_stop(
    mut endpoint: *const ::core::ffi::c_char,
    mut keep_vservername: bool,
) -> bool {
    let mut watcher: *mut SocketWatcher = ::core::ptr::null_mut::<SocketWatcher>();
    let mut watcher_found: bool = false_0 != 0;
    let mut addr: [::core::ffi::c_char; 256] = [0; 256];
    xstrlcpy(
        &raw mut addr as *mut ::core::ffi::c_char,
        endpoint,
        ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*watchers.ptr()).ga_len {
        watcher = *((*watchers.ptr()).ga_data as *mut *mut SocketWatcher).offset(i as isize);
        if strcmp(
            &raw mut addr as *mut ::core::ffi::c_char,
            &raw mut (*watcher).addr as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            watcher_found = true_0 != 0;
            break;
        } else {
            i += 1;
        }
    }
    if !watcher_found {
        logmsg(
            LOGLVL_WRN,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"server_stop\0".as_ptr() as *const ::core::ffi::c_char,
            236 as ::core::ffi::c_int,
            true_0 != 0,
            b"Not listening on %s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw mut addr as *mut ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    socket_watcher_close(
        watcher,
        Some(
            free_server as unsafe extern "C" fn(*mut SocketWatcher, *mut ::core::ffi::c_void) -> (),
        ),
    );
    if i != (*watchers.ptr()).ga_len - 1 as ::core::ffi::c_int {
        *((*watchers.ptr()).ga_data as *mut *mut SocketWatcher).offset(i as isize) =
            *((*watchers.ptr()).ga_data as *mut *mut SocketWatcher)
                .offset(((*watchers.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize);
    }
    (*watchers.ptr()).ga_len -= 1;
    if !keep_vservername
        && strequal(
            &raw mut addr as *mut ::core::ffi::c_char,
            get_vim_var_str(VV_SEND_SERVER),
        ) as ::core::ffi::c_int
            != 0
    {
        set_vservername(watchers.ptr());
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn server_address_list(
    mut size: *mut size_t,
) -> *mut *mut ::core::ffi::c_char {
    *size = (*watchers.ptr()).ga_len as size_t;
    if *size == 0 as size_t {
        return ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    }
    let mut addrs: *mut *mut ::core::ffi::c_char = xcalloc(
        (*watchers.ptr()).ga_len as size_t,
        ::core::mem::size_of::<*const ::core::ffi::c_char>(),
    ) as *mut *mut ::core::ffi::c_char;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*watchers.ptr()).ga_len {
        *addrs.offset(i as isize) = xstrdup(
            &raw mut (**((*watchers.ptr()).ga_data as *mut *mut SocketWatcher).offset(i as isize))
                .addr as *mut ::core::ffi::c_char,
        );
        i += 1;
    }
    return addrs;
}
unsafe extern "C" fn connection_cb(
    mut watcher: *mut SocketWatcher,
    mut result: ::core::ffi::c_int,
    mut _data: *mut ::core::ffi::c_void,
) {
    if result != 0 {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"connection_cb\0".as_ptr() as *const ::core::ffi::c_char,
            276 as ::core::ffi::c_int,
            true_0 != 0,
            b"Failed to accept connection: %s\0".as_ptr() as *const ::core::ffi::c_char,
            uv_strerror(result),
        );
        return;
    }
    channel_from_connection(watcher);
}
unsafe extern "C" fn free_server(
    mut watcher: *mut SocketWatcher,
    mut _data: *mut ::core::ffi::c_void,
) {
    xfree(watcher as *mut ::core::ffi::c_void);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
