//! The editor's global state: upstream's `globals.h` has no translation unit
//! of its own, so the transpiler parked every `EXTERN` declaration here beside
//! `main()`. What follows is that header — roughly a thousand `GlobalCell`s
//! read from all over the tree. The startup path lives in the submodules.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::global_cell::{GlobalCell, SharedCell};
use crate::options::{
    kOptArabic, kOptCbFlagUnnamed, kOptCbFlagUnnamedplus, kOptErrorfile, kOptKeymap, kOptRightleft,
    kOptShadafile, kOptShortmess, kOptVerbosefile, kOptWindow,
};
use crate::profile::time_msg;
use crate::registry::{IdSet, SlotTable, id_set};
use crate::types::{
    Array, Channel, EStack, EStackType, EstackInfo, FILE, FileComparison, Handle, Loop, LuaRetMode,
    MultiQueue, NluaRefState, Object, Proc, ProfTime, ScriptCtx, UV_MUTEX_INIT, UV_RWLOCK_INIT,
    XDGVarType, caller_scope, int16_t, int64_t, nvim_stats_s, size_t, uint32_t, uint64_t, uv__io_t,
    uv__queue, uv_async_s_u, uv_async_t, uv_handle_t, uv_handle_type, uv_loop_s_active_reqs,
    uv_loop_s_timer_heap, uv_loop_t, uv_signal_s, uv_signal_s_tree_entry, uv_signal_s_u,
    uv_signal_t, uv_timer_s_node, uv_timer_s_u, uv_timer_t,
};
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};

mod entry;
pub use self::entry::*;
mod args;
mod buffers;
mod config;
mod exit;
mod remote;
mod usage;
pub use self::exit::*;

/// A C string literal as the fixed-size `c_char` array a global holds.
///
/// c2rust spelled every `static char foo[] = "…"` as a `transmute` from the
/// byte string, which is one `unsafe` block per message and 194 of them in
/// this file alone. Copying the bytes is const-evaluable, needs no `unsafe`,
/// and unlike `transmute` it works with a const generic length.
pub(crate) const fn c_bytes<const N: usize>(bytes: &[u8; N]) -> [c_char; N] {
    let mut out = [0 as c_char; N];
    let mut i = 0;
    while i < N {
        out[i] = bytes[i] as c_char;
        i += 1;
    }
    out
}

/// Record a startup-timing message, if `--startuptime` asked for one.
///
/// The C spells this as the `TIME_MSG` macro. Safe: no raw pointer crosses
/// the boundary, and the `time_fd` test is the whole of it.
pub(crate) fn time_msg_at(what: &CStr) {
    if !time_fd.get().is_null() {
        // SAFETY: `time_fd` is the startup-timing file, opened once by
        // `init_startuptime` and closed by `time_finish`; `what` outlives the
        // call and the second argument is the "no elapsed time" null.
        unsafe { time_msg(what.as_ptr(), ::core::ptr::null::<ProfTime>()) };
    }
}

pub(crate) const UV_UNKNOWN_HANDLE: uv_handle_type = 0;
pub(crate) const kXDGConfigDirs: XDGVarType = 5;
pub(crate) const READ_STDIN: c_uint = 4;
pub(crate) const READ_NEW: c_uint = 1;
pub(crate) const ETYPE_ENV: EStackType = 7;
pub(crate) const ETYPE_ARGS: EStackType = 6;
pub(crate) const ETYPE_TOP: EStackType = 0;
pub(crate) const kRetObject: LuaRetMode = 0;
#[derive(Clone)] // not `Copy`: it owns several of its strings
pub struct MainParams {
    pub argc: c_int,
    pub argv: *mut *mut c_char,
    pub use_vimrc: *mut c_char,
    pub clean: bool,
    pub n_commands: c_int,
    pub commands: [*mut c_char; 10],
    pub cmds_tofree: [c_char; 10],
    pub n_pre_commands: c_int,
    pub pre_commands: [*mut c_char; 10],
    pub luaf: *mut c_char,
    pub lua_arg0: c_int,
    pub edit_type: c_int,
    pub tagname: *mut c_char,
    pub use_ef: *mut c_char,
    pub input_istext: bool,
    pub no_swap_file: c_int,
    pub use_debug_break_level: c_int,
    pub window_count: c_int,
    pub window_layout: c_int,
    pub diff_mode: c_int,
    pub listen_addr: *mut c_char,
    pub remote: c_int,
    pub server_addr: *mut c_char,
    pub scriptin: *mut c_char,
    pub scriptout: *mut c_char,
    pub scriptout_append: bool,
    pub had_stdin_file: bool,
}
pub(crate) const EDIT_QF: c_uint = 4;
pub(crate) const WIN_TABS: c_uint = 3;
pub(crate) const WIN_VER: c_uint = 2;
pub(crate) const WIN_HOR: c_uint = 1;
pub(crate) const EDIT_STDIN: c_uint = 2;
pub(crate) const kEqualFiles: FileComparison = 1;
pub(crate) const DOSO_VIMRC: c_uint = 1;
pub(crate) const DOSO_NONE: c_uint = 0;
pub(crate) const EDIT_FILE: c_uint = 1;
pub(crate) const EDIT_TAG: c_uint = 3;
pub(crate) const EDIT_NONE: c_uint = 0;
pub static arena_alloc_count: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
pub(crate) const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub(crate) const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub static g_min_log_level: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) const SESSION_FILE: &CStr = c"Session.vim";
/// Namespace name -> id, in creation order.
///
/// khash, which this was, is insertion-ordered with a swap-remove; nothing
/// ever removes a namespace, so the order `nvim_get_namespaces` renders and
/// `describe_ns` searches is creation order. [`SlotTable`] keeps it.
pub(crate) static namespace_ids: GlobalCell<SlotTable<Box<[u8]>, Handle>> =
    GlobalCell::new(SlotTable::new());
/// The namespaces that are window-local rather than visible everywhere.
/// Membership only; never walked.
pub(crate) static namespace_localscope: GlobalCell<IdSet<uint32_t>> = GlobalCell::new(id_set());
pub static next_namespace_id: GlobalCell<Handle> = GlobalCell::new(1 as Handle);
pub(crate) const PATHSEP: c_int = '/' as c_int;
/// Every open channel, by id. See [`crate::registry`] for the order this
/// keeps and the reentrancy rule it answers: a channel's callback runs Lua
/// and Vimscript, which can open and close channels, so nothing holds a
/// borrow of this across one.
pub(crate) static channels: GlobalCell<SlotTable<uint64_t, *mut Channel>> =
    GlobalCell::new(SlotTable::new());
// TV_CSTRING (SIZE_MAX - 1): c2rust dropped the initializer expression and
// left 0, which is a valid pointer-sentinel value and would corrupt any
// caller comparing against it (the unit tests do, via FFI).
pub static kTVCstring: GlobalCell<size_t> = GlobalCell::new(18446744073709551614);
pub(crate) const SYS_VIMRC_FILE: &CStr = c"$VIM/sysinit.vim";
pub(crate) const VIMRC_FILE: &CStr = c".nvimrc";
pub static g_stats: GlobalCell<nvim_stats_s> = GlobalCell::new(nvim_stats_s {
    fsync: 0 as int64_t,
    redraw: 0 as int64_t,
    log_skip: 0 as int16_t,
});
pub(crate) const NO_BUFFERS: c_int = 1 as c_int;
pub static ex_exitval: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static didset_vim: GlobalCell<bool> = GlobalCell::new(false);
pub static didset_vimruntime: GlobalCell<bool> = GlobalCell::new(false);
pub static do_profiling: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static may_garbage_collect: GlobalCell<bool> = GlobalCell::new(false);
pub static want_garbage_collect: GlobalCell<bool> = GlobalCell::new(false);
pub static garbage_collect_at_exit: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) const SID_CMDARG: c_int = -2 as c_int;
pub(crate) const SID_CARG: c_int = -3 as c_int;
pub(crate) const SID_ENV: c_int = -4 as c_int;
pub static current_sctx: GlobalCell<ScriptCtx> = GlobalCell::new(ScriptCtx::NONE);
pub static did_source_packages: GlobalCell<bool> = GlobalCell::new(false);
pub static provider_caller_scope: GlobalCell<caller_scope> = GlobalCell::new(caller_scope {
    script_ctx: ScriptCtx::NONE,
    es_entry: EStack {
        es_lnum: 0,
        es_name: ::core::ptr::null_mut::<c_char>(),
        es_type: ETYPE_TOP,
        es_info: EstackInfo::None,
    },
    autocmd_fname: ::core::ptr::null_mut::<c_char>(),
    autocmd_match: ::core::ptr::null_mut::<c_char>(),
    autocmd_fname_full: false,
    autocmd_bufnr: 0,
    funccalp: ::core::ptr::null_mut::<c_void>(),
});
pub static provider_call_nesting: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static need_check_timestamps: GlobalCell<bool> = GlobalCell::new(false);
pub static did_check_timestamps: GlobalCell<bool> = GlobalCell::new(false);
pub static no_check_timestamps: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
#[unsafe(no_mangle)]
pub static starting: GlobalCell<c_int> = GlobalCell::new(2 as c_int);
pub static exiting: GlobalCell<bool> = GlobalCell::new(false);
pub static v_dying: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static stdin_isatty: GlobalCell<bool> = GlobalCell::new(true);
pub static stdout_isatty: GlobalCell<bool> = GlobalCell::new(true);
pub static stderr_isatty: GlobalCell<bool> = GlobalCell::new(true);
pub static stdin_fd: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static full_screen: GlobalCell<bool> = GlobalCell::new(false);
pub static silent_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static inhibit_delete_count: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static fenc_default: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static readonlymode: GlobalCell<bool> = GlobalCell::new(false);
pub static recoverymode: GlobalCell<bool> = GlobalCell::new(false);
pub static did_outofmem_msg: GlobalCell<bool> = GlobalCell::new(false);
pub static did_swapwrite_msg: GlobalCell<bool> = GlobalCell::new(false);
pub static globaldir: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static last_chdir_reason: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static time_fd: GlobalCell<*mut FILE> = GlobalCell::new(::core::ptr::null_mut::<FILE>());
pub static vim_ignored: GlobalCell<c_int> = GlobalCell::new(0);
pub static embedded_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static headless_mode: GlobalCell<bool> = GlobalCell::new(false);
/// The Windows release `windowsversion()` reports. Nothing here writes it,
/// so it stays the empty string a non-Windows build always answered.
pub static windowsVersion: [c_char; 20] = [0 as c_char; 20];
pub(crate) const LUA_GLOBALSINDEX: c_int = -10002 as c_int;
pub static nlua_global_refs: GlobalCell<*mut NluaRefState> =
    GlobalCell::new(::core::ptr::null_mut::<NluaRefState>());
pub static nlua_disable_preload: SharedCell<bool> = SharedCell::new(false);
pub static main_loop: SharedCell<Loop> = SharedCell::new(Loop {
    uv: uv_loop_t {
        data: ::core::ptr::null_mut::<c_void>(),
        active_handles: 0,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        active_reqs: uv_loop_s_active_reqs {
            unused: ::core::ptr::null_mut::<c_void>(),
        },
        internal_fields: ::core::ptr::null_mut::<c_void>(),
        stop_flag: 0,
        flags: 0,
        backend_fd: 0,
        pending_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        watcher_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        watchers: ::core::ptr::null_mut::<*mut uv__io_t>(),
        nwatchers: 0,
        nfds: 0,
        wq: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        wq_mutex: UV_MUTEX_INIT,
        wq_async: uv_async_t {
            data: ::core::ptr::null_mut::<c_void>(),
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            type_0: UV_UNKNOWN_HANDLE,
            close_cb: None,
            handle_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            u: uv_async_s_u { fd: 0 },
            next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
            flags: 0,
            async_cb: None,
            queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            pending: 0,
        },
        cloexec_lock: UV_RWLOCK_INIT,
        closing_handles: ::core::ptr::null_mut::<uv_handle_t>(),
        process_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        prepare_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        check_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        idle_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        async_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        async_unused: None,
        async_io_watcher: uv__io_t {
            cb: None,
            pending_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            watcher_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            pevents: 0,
            events: 0,
            fd: 0,
        },
        async_wfd: 0,
        timer_heap: uv_loop_s_timer_heap {
            min: ::core::ptr::null_mut::<c_void>(),
            nelts: 0,
        },
        timer_counter: 0,
        time: 0,
        signal_pipefd: [0; 2],
        signal_io_watcher: uv__io_t {
            cb: None,
            pending_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            watcher_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            pevents: 0,
            events: 0,
            fd: 0,
        },
        child_watcher: uv_signal_t {
            data: ::core::ptr::null_mut::<c_void>(),
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            type_0: UV_UNKNOWN_HANDLE,
            close_cb: None,
            handle_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            u: uv_signal_s_u { fd: 0 },
            next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
            flags: 0,
            signal_cb: None,
            signum: 0,
            tree_entry: uv_signal_s_tree_entry {
                rbe_left: ::core::ptr::null_mut::<uv_signal_s>(),
                rbe_right: ::core::ptr::null_mut::<uv_signal_s>(),
                rbe_parent: ::core::ptr::null_mut::<uv_signal_s>(),
                rbe_color: 0,
            },
            caught_signals: 0,
            dispatched_signals: 0,
        },
        emfile_fd: 0,
        inotify_read_watcher: uv__io_t {
            cb: None,
            pending_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            watcher_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            pevents: 0,
            events: 0,
            fd: 0,
        },
        inotify_watchers: ::core::ptr::null_mut::<c_void>(),
        inotify_fd: 0,
    },
    events: ::core::ptr::null_mut::<MultiQueue>(),
    thread_events: ::core::ptr::null_mut::<MultiQueue>(),
    fast_events: ::core::ptr::null_mut::<MultiQueue>(),
    children: ::core::ptr::null_mut::<Vec<*mut Proc>>(),
    children_watcher: uv_signal_t {
        data: ::core::ptr::null_mut::<c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: uv_signal_s_u { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        signal_cb: None,
        signum: 0,
        tree_entry: uv_signal_s_tree_entry {
            rbe_left: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_right: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_parent: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_color: 0,
        },
        caught_signals: 0,
        dispatched_signals: 0,
    },
    children_kill_timer: uv_timer_t {
        data: ::core::ptr::null_mut::<c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: uv_timer_s_u { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        timer_cb: None,
        node: uv_timer_s_node {
            heap: [::core::ptr::null_mut::<c_void>(); 3],
        },
        timeout: 0,
        repeat: 0,
        start_id: 0,
    },
    poll_timer: uv_timer_t {
        data: ::core::ptr::null_mut::<c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: uv_timer_s_u { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        timer_cb: None,
        node: uv_timer_s_node {
            heap: [::core::ptr::null_mut::<c_void>(); 3],
        },
        timeout: 0,
        repeat: 0,
        start_id: 0,
    },
    exit_delay_timer: uv_timer_t {
        data: ::core::ptr::null_mut::<c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: uv_timer_s_u { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        timer_cb: None,
        node: uv_timer_s_node {
            heap: [::core::ptr::null_mut::<c_void>(); 3],
        },
        timeout: 0,
        repeat: 0,
        start_id: 0,
    },
    async_0: uv_async_t {
        data: ::core::ptr::null_mut::<c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: uv_async_s_u { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        async_cb: None,
        queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        pending: 0,
    },
    mutex: UV_MUTEX_INIT,
    recursive: 0,
    closing: false,
});
static argv0: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
static err_arg_missing: GlobalCell<*const c_char> =
    GlobalCell::new(c"Argument missing after".as_ptr());
static err_opt_garbage: GlobalCell<*const c_char> =
    GlobalCell::new(c"Garbage after option argument".as_ptr());
static err_opt_unknown: GlobalCell<*const c_char> =
    GlobalCell::new(c"Unknown option argument".as_ptr());
static err_too_many_args: GlobalCell<*const c_char> =
    GlobalCell::new(c"Too many edit arguments".as_ptr());
static err_extra_cmd: GlobalCell<*const c_char> = GlobalCell::new(
    c"Too many \"+command\", \"-c command\" or \"--cmd command\" arguments".as_ptr(),
);
pub static tslua_query_parse_count: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub(crate) const MAX_ARG_CMDS: c_int = 10 as c_int;
pub static ch_before_blocking_events: GlobalCell<*mut MultiQueue> =
    GlobalCell::new(::core::ptr::null_mut::<MultiQueue>());
pub static used_stdin: GlobalCell<bool> = GlobalCell::new(false);
pub static nvim_testing: GlobalCell<bool> = GlobalCell::new(false);
pub static ui_client_channel_id: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub static ui_client_error_exit: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static ui_client_exit_status: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static ui_client_attached: GlobalCell<bool> = GlobalCell::new(false);
pub static ui_client_forward_stdin: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) const WRITEBIN: &CStr = c"wb";
pub(crate) const APPENDBIN: &CStr = c"ab";
