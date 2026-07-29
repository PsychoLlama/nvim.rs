use crate::src::nvim::api::private::helpers::{api_free_object, api_metadata_raw, cstr_as_string};
use crate::src::nvim::api::ui::remote_ui_wait_for_attach;
use crate::src::nvim::arglist::{alist_add, alist_init, alist_name};
use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::autocmd::{
    apply_autocmds, autocmd_init, block_autocmds, is_autocmd_blocked, unblock_autocmds,
};
use crate::src::nvim::buffer::buf_get_changedtick;
use crate::src::nvim::buffer::{
    buf_is_empty, buf_set_changedtick, buf_valid, buflist_new, bufref_valid, do_autochdir,
    do_modelines, handle_swap_exists, open_buffer, set_buflisted, set_bufref, set_curbuf, setfname,
};
use crate::src::nvim::channel::{
    channel_connect, channel_from_stdio, channel_init, channel_teardown,
};
use crate::src::nvim::diff::{diff_win_options, diffopt_horizontal};
use crate::src::nvim::drawscreen::{
    default_grid_alloc, redraw_all_later, redraw_later, screenclear,
};
use crate::src::nvim::eval::typval::tv_list_set_lock;
use crate::src::nvim::eval::typval::{tv_list_alloc, tv_list_append_string};
use crate::src::nvim::eval::userfunc::invoke_all_defer;
use crate::src::nvim::eval::vars::{
    get_vim_var_list, get_vim_var_str, set_reg_var, set_vim_var_list, set_vim_var_nr,
    set_vim_var_string, set_vim_var_type,
};
use crate::src::nvim::eval::{
    eval_has_provider, eval_init, garbage_collect, set_argv_var, timer_teardown,
};
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::event::r#loop::{loop_close, loop_init, loop_poll_events};
use crate::src::nvim::event::multiqueue::{multiqueue_new_child, multiqueue_process_events};
use crate::src::nvim::event::proc::proc_teardown;
use crate::src::nvim::event::socket::socket_address_is_tcp;
use crate::src::nvim::event::stream::stream_set_blocking;
use crate::src::nvim::ex_cmds::do_ecmd;
use crate::src::nvim::ex_docmd::{do_cmdline_cmd, filetype_maybe_enable, filetype_plugin_enable};
use crate::src::nvim::ex_getln::cmdline_init;
use crate::src::nvim::fileio::{readfile, shorten_fnames};
use crate::src::nvim::garray::ga_grow;
use crate::src::nvim::getchar::{open_scriptin, stuffcharReadbuff, vgetc};
use crate::src::nvim::global_cell::{GlobalCell, SharedCell};
use crate::src::nvim::highlight::highlight_init;
use crate::src::nvim::highlight_group::init_highlight;
use crate::src::nvim::log::{log_init, logmsg};
use crate::src::nvim::lua::executor::{
    get_global_lstate, nlua_exec, nlua_exec_file, nlua_init, nlua_init_defaults, nlua_pcall,
    nlua_run_script,
};
use crate::src::nvim::lua::ffi::{lua_getfield, lua_pushstring, lua_tolstring};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::memline::{
    ml_close_all, ml_close_notmod, ml_recover, ml_sync_all, recover_names,
};
use crate::src::nvim::memory::{strequal, xfree, xmalloc, xrealloc, xstrdup};
use crate::src::nvim::message::{msg_putchar, semsg, wait_return};
use crate::src::nvim::mouse::setmouse;
use crate::src::nvim::r#move::update_topline;
use crate::src::nvim::msgpack_rpc::server::{server_init, server_teardown};
use crate::src::nvim::normal::{check_scrollbind, init_normal_cmds, normal_enter};
use crate::src::nvim::option::{
    reset_modifiable, set_init_1, set_init_2, set_init_3, set_init_tablocal, set_option_direct,
    set_option_value_give_err, set_options_bin,
};
use crate::src::nvim::options::{
    kOptArabic, kOptCbFlagUnnamed, kOptCbFlagUnnamedplus, kOptErrorfile, kOptKeymap, kOptRightleft,
    kOptShadafile, kOptShortmess, kOptVerbosefile, kOptWindow,
};
use crate::src::nvim::os::env::{
    env_init, init_homedir, os_getenv, os_getenv_noalloc, os_hint_priority, vim_env_iter,
};
use crate::src::nvim::os::fs::{os_exepath, os_fopen, os_isdir, os_path_exists, os_write};
use crate::src::nvim::os::input::{input_start, input_stop, os_breakcheck, os_isatty};
use crate::src::nvim::os::lang::{init_locale, set_lang_var};
use crate::src::nvim::os::libc::{
    __assert_fail, abort, atoi, exit, fprintf, gettext, memcpy, memset, printf, setbuf, snprintf,
    stderr, stdout, strcasecmp, strlen, strncasecmp, tcdrain,
};
use crate::src::nvim::os::signal::{
    signal_init, signal_reject_deadly, signal_stop, signal_teardown,
};
use crate::src::nvim::os::stdpaths::{
    appname_is_valid, get_appname, stdpaths_get_xdg_var, stdpaths_user_conf_subpath,
};
use crate::src::nvim::os::time::os_realtime;
use crate::src::nvim::path::{
    concat_fnames, path_full_compare, path_guess_exepath, path_tail, vim_FullName,
};
use crate::src::nvim::profile::{profile_dump, time_finish, time_init, time_msg, time_start};
use crate::src::nvim::quickfix::qf_init_stack;
use crate::src::nvim::register::get_default_register_name;
use crate::src::nvim::runtime::{
    do_source, estack_init, estack_pop, estack_push, load_plugins, runtime_init,
};
use crate::src::nvim::shada::{shada_read_everything, shada_write_file};
use crate::src::nvim::strings::vim_snprintf;
use crate::src::nvim::syntax::syn_maybe_enable;
use crate::src::nvim::terminal::{terminal_init, terminal_teardown};
use crate::src::nvim::types::{
    __pthread_internal_list, __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t,
    AdditionalData, Arena, Array, Callback, Callback_data, CallbackReader, CallbackType,
    DecorRangeSlot, DecorSignHighlight, DecorState, DecorState_ranges_i, DecorState_slots, Error,
    ErrorType, FILE, GridView, Integer, ListLenSpecials, Loop, LuaRef, LuaRetMode, MTNode, MTPos,
    Map_String_int, Map_int_ptr_t, Map_uint64_t_ptr_t, MapHash, MarkTreeIter, MarkTreeIter_s,
    MultiQueue, NS, Object, ObjectType, OptInt, OptVal, OptValData, OptValType, RgbValue,
    ScreenGrid, Set_String, Set_int, Set_uint32_t, Set_uint64_t, StlClickDefinition, String_0,
    TriState, VarLockStatus, VarType, VimVarIndex, WinExtmark, XDGVarType, aentry_T, alist_T,
    aucmdwin_T, auto_event, bln_values, buf_T, bufref_T, caller_scope, cmdmod_T, colnr_T, dict_T,
    disptick_T, estack_T, estack_T_es_info, etype_T, evalarg_T, exarg_T, except_T, file_comparison,
    fmark_T, fmarkv_T, frame_T, garray_T, handle_T, hlf_T, int16_t, int32_t, int64_t, key_extra,
    linenr_T, list_T, lpos_T, lua_State, match_T, msglist_T, nlua_ref_state_t, nvim_stats_s,
    object, object_data as C2Rust_Unnamed, optmagic_T, pos_T, proftime_T, pthread_mutex_t,
    pthread_rwlock_t, ptr_t, ptrdiff_t, qf_info_T, reg_extmatch_T, regmatch_T, regmmatch_T,
    regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t, ssize_t, tabpage_T, typebuf_T, uint8_t,
    uint32_t, uint64_t, uv__io_t, uv__queue, uv_async_s_u, uv_async_t, uv_handle_t, uv_handle_type,
    uv_loop_s_active_reqs, uv_loop_s_timer_heap, uv_loop_t, uv_signal_s, uv_signal_s_tree_entry,
    uv_signal_s_u, uv_signal_t, uv_timer_s_node, uv_timer_s_u, uv_timer_t, varnumber_T, vimmenu_T,
    win_T, xfmark_T,
};
use crate::src::nvim::ui::{
    do_autocmd_uienter_all, ui_call_error_exit, ui_call_set_title, ui_call_stop, ui_flush, ui_init,
};
use crate::src::nvim::ui_client::{ui_client_run, ui_client_start_server, ui_client_stop};
use crate::src::nvim::ui_compositor::ui_comp_syn_init;
use crate::src::nvim::version::list_version;
use crate::src::nvim::window::{
    goto_tabpage, make_tabpages, make_windows, only_one_window, win_alloc_first, win_close,
    win_count, win_enter, win_equal, win_init_size, win_new_screensize,
};
use core::ffi::{CStr, c_char, c_int, c_long, c_uint, c_void};
unsafe extern "C" {
    fn qf_init(
        wp: *mut win_T,
        efile: *const c_char,
        errorformat: *mut c_char,
        newlist: c_int,
        qf_title: *const c_char,
        enc: *mut c_char,
    ) -> c_int;
    fn qf_jump(qi: *mut qf_info_T, dir: c_int, errornr: c_int, forceit: c_int);
}
pub const kErrorTypeNone: ErrorType = -1;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeArray: ObjectType = 5;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub const kCallbackNone: CallbackType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_NUMBER: VarType = 1;
pub const UV_UNKNOWN_HANDLE: uv_handle_type = 0;
pub const MAXLNUM: c_uint = 2147483647;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const HLF_NONE: hlf_T = 0;
pub const OPTION_MAGIC_OFF: optmagic_T = 2;
pub const OPTION_MAGIC_ON: optmagic_T = 1;
pub const OPTION_MAGIC_NOT_SET: optmagic_T = 0;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const EVENT_VIMLEAVEPRE: auto_event = 134;
pub const EVENT_VIMLEAVE: auto_event = 133;
pub const EVENT_VIMENTER: auto_event = 132;
pub const EVENT_BUFWINLEAVE: auto_event = 17;
pub const EVENT_BUFUNLOAD: auto_event = 15;
pub const EVENT_BUFENTER: auto_event = 3;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AucmdWinVec {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut aucmdwin_T,
}
pub const BLN_LISTED: bln_values = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorSignHighlightVec {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut DecorSignHighlight,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WinExtmarkVec {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut WinExtmark,
}
pub const UPD_NOT_VALID: c_uint = 40;
pub const UPD_VALID: c_uint = 10;
pub const VV_EXITREASON: VimVarIndex = 105;
pub const VV_STARTTIME: VimVarIndex = 104;
pub const VV_VIM_DID_INIT: VimVarIndex = 94;
pub const VV_EXITING: VimVarIndex = 91;
pub const VV_ARGF: VimVarIndex = 88;
pub const VV_VIM_DID_ENTER: VimVarIndex = 75;
pub const VV_PROGPATH: VimVarIndex = 60;
pub const VV_OLDFILES: VimVarIndex = 58;
pub const VV_SWAPCOMMAND: VimVarIndex = 49;
pub const VV_PROGNAME: VimVarIndex = 27;
pub const kXDGConfigDirs: XDGVarType = 5;
pub const EVAL_EVALUATE: c_uint = 1;
pub const ECMD_HIDE: c_uint = 1;
pub const ECMD_LASTL: c_int = 0;
pub const READ_STDIN: c_uint = 4;
pub const READ_NEW: c_uint = 1;
pub const ETYPE_ENV: etype_T = 7;
pub const ETYPE_ARGS: etype_T = 6;
pub const ETYPE_TOP: etype_T = 0;
pub const MODE_NORMAL: c_uint = 1;
pub const KE_NOP: key_extra = 97;
pub const kRetObject: LuaRetMode = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mparm_T {
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
pub const EDIT_QF: c_uint = 4;
pub const WIN_TABS: c_uint = 3;
pub const WIN_VER: c_uint = 2;
pub const WIN_HOR: c_uint = 1;
pub const EDIT_STDIN: c_uint = 2;
pub const kEqualFiles: file_comparison = 1;
pub const DOSO_VIMRC: c_uint = 1;
pub const DOSO_NONE: c_uint = 0;
pub const EDIT_FILE: c_uint = 1;
pub const EDIT_TAG: c_uint = 3;
pub const EDIT_NONE: c_uint = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PumWant {
    pub active: bool,
    pub item: c_int,
    pub insert: bool,
    pub finish: bool,
}
pub const NULL_0: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub static arena_alloc_count: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
pub const STDIN_FILENO: c_int = 0 as c_int;
pub const STDOUT_FILENO: c_int = 1 as c_int;
pub const STDERR_FILENO: c_int = 2 as c_int;
pub const DEFAULT_MAXPATHL: c_int = 4096 as c_int;
pub const MAXPATHL: c_int = DEFAULT_MAXPATHL;
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as c_int,
    ga_maxlen: 0 as c_int,
    ga_itemsize: 0 as c_int,
    ga_growsize: 1 as c_int,
    ga_data: NULL_0,
};
pub const LOGLVL_DBG: c_int = 1 as c_int;
pub const LOGLVL_INF: c_int = 2 as c_int;
pub static g_min_log_level: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub const SESSION_FILE: [c_char; 12] =
    unsafe { ::core::mem::transmute::<[u8; 12], [c_char; 12]>(*b"Session.vim\0") };
pub const OK: c_int = 1 as c_int;
pub const FAIL: c_int = 0 as c_int;
pub static namespace_ids: GlobalCell<Map_String_int> = GlobalCell::new(Map_String_int {
    set: Set_String {
        h: MapHash {
            n_buckets: 0 as uint32_t,
            size: 0 as uint32_t,
            n_occupied: 0 as uint32_t,
            upper_bound: 0 as uint32_t,
            n_keys: 0 as uint32_t,
            keys_capacity: 0 as uint32_t,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<String_0>(),
    },
    values: ::core::ptr::null_mut::<c_int>(),
});
pub static namespace_localscope: GlobalCell<Set_uint32_t> = GlobalCell::new(Set_uint32_t {
    h: MapHash {
        n_buckets: 0 as uint32_t,
        size: 0 as uint32_t,
        n_occupied: 0 as uint32_t,
        upper_bound: 0 as uint32_t,
        n_keys: 0 as uint32_t,
        keys_capacity: 0 as uint32_t,
        hash: ::core::ptr::null_mut::<uint32_t>(),
    },
    keys: ::core::ptr::null_mut::<uint32_t>(),
});
pub static next_namespace_id: GlobalCell<handle_T> = GlobalCell::new(1 as handle_T);
pub static buffer_handles: GlobalCell<Map_int_ptr_t> = GlobalCell::new(Map_int_ptr_t {
    set: Set_int {
        h: MapHash {
            n_buckets: 0 as uint32_t,
            size: 0 as uint32_t,
            n_occupied: 0 as uint32_t,
            upper_bound: 0 as uint32_t,
            n_keys: 0 as uint32_t,
            keys_capacity: 0 as uint32_t,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<c_int>(),
    },
    values: ::core::ptr::null_mut::<ptr_t>(),
});
pub static window_handles: GlobalCell<Map_int_ptr_t> = GlobalCell::new(Map_int_ptr_t {
    set: Set_int {
        h: MapHash {
            n_buckets: 0 as uint32_t,
            size: 0 as uint32_t,
            n_occupied: 0 as uint32_t,
            upper_bound: 0 as uint32_t,
            n_keys: 0 as uint32_t,
            keys_capacity: 0 as uint32_t,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<c_int>(),
    },
    values: ::core::ptr::null_mut::<ptr_t>(),
});
pub static tabpage_handles: GlobalCell<Map_int_ptr_t> = GlobalCell::new(Map_int_ptr_t {
    set: Set_int {
        h: MapHash {
            n_buckets: 0 as uint32_t,
            size: 0 as uint32_t,
            n_occupied: 0 as uint32_t,
            upper_bound: 0 as uint32_t,
            n_keys: 0 as uint32_t,
            keys_capacity: 0 as uint32_t,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<c_int>(),
    },
    values: ::core::ptr::null_mut::<ptr_t>(),
});
pub static ui_ext_names: GlobalCell<[*const c_char; 10]> = GlobalCell::new([
    b"ext_cmdline\0".as_ptr() as *const c_char,
    b"ext_popupmenu\0".as_ptr() as *const c_char,
    b"ext_tabline\0".as_ptr() as *const c_char,
    b"ext_wildmenu\0".as_ptr() as *const c_char,
    b"ext_messages\0".as_ptr() as *const c_char,
    b"ext_linegrid\0".as_ptr() as *const c_char,
    b"ext_multigrid\0".as_ptr() as *const c_char,
    b"ext_hlstate\0".as_ptr() as *const c_char,
    b"ext_termcolors\0".as_ptr() as *const c_char,
    b"_debug_float\0".as_ptr() as *const c_char,
]);
pub const NUL: c_int = '\0' as c_int;
pub const PATHSEP: c_int = '/' as c_int;
pub static last_cursormoved_win: GlobalCell<*mut win_T> =
    GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static last_cursormoved: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0 as linenr_T,
    col: 0 as colnr_T,
    coladd: 0 as colnr_T,
});
pub static autocmd_busy: GlobalCell<bool> = GlobalCell::new(false);
pub static autocmd_no_enter: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static autocmd_no_leave: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static au_new_curbuf: GlobalCell<bufref_T> = GlobalCell::new(bufref_T {
    br_buf: ::core::ptr::null_mut::<buf_T>(),
    br_fnum: 0 as c_int,
    br_buf_free_count: 0 as c_int,
});
pub static au_pending_free_buf: GlobalCell<*mut buf_T> =
    GlobalCell::new(::core::ptr::null_mut::<buf_T>());
pub static au_pending_free_win: GlobalCell<*mut win_T> =
    GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static autocmd_fname: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static autocmd_fname_full: GlobalCell<bool> = GlobalCell::new(false);
pub static autocmd_bufnr: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static autocmd_match: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static did_cursorhold: GlobalCell<bool> = GlobalCell::new(true);
#[unsafe(no_mangle)]
pub static aucmd_win_vec: GlobalCell<AucmdWinVec> = GlobalCell::new(AucmdWinVec {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<aucmdwin_T>(),
});
pub static deferred_events: GlobalCell<*mut MultiQueue> =
    GlobalCell::new(::core::ptr::null_mut::<MultiQueue>());
pub static msg_loclist: GlobalCell<*mut c_char> =
    GlobalCell::new(b"[Location List]\0".as_ptr() as *const c_char as *mut c_char);
pub static msg_qflist: GlobalCell<*mut c_char> =
    GlobalCell::new(b"[Quickfix List]\0".as_ptr() as *const c_char as *mut c_char);
pub static channels: GlobalCell<Map_uint64_t_ptr_t> = GlobalCell::new(Map_uint64_t_ptr_t {
    set: Set_uint64_t {
        h: MapHash {
            n_buckets: 0 as uint32_t,
            size: 0 as uint32_t,
            n_occupied: 0 as uint32_t,
            upper_bound: 0 as uint32_t,
            n_keys: 0 as uint32_t,
            keys_capacity: 0 as uint32_t,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<uint64_t>(),
    },
    values: ::core::ptr::null_mut::<ptr_t>(),
});
pub static on_print: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: Callback_data {
        funcref: ::core::ptr::null_mut::<c_char>(),
    },
    type_0: kCallbackNone,
});
pub static virt_text_pos_str: GlobalCell<[*const c_char; 6]> = GlobalCell::new([
    b"eol\0".as_ptr() as *const c_char,
    b"eol_right_align\0".as_ptr() as *const c_char,
    b"inline\0".as_ptr() as *const c_char,
    b"overlay\0".as_ptr() as *const c_char,
    b"right_align\0".as_ptr() as *const c_char,
    b"win_col\0".as_ptr() as *const c_char,
]);
pub static hl_mode_str: GlobalCell<[*const c_char; 4]> = GlobalCell::new([
    b"\0".as_ptr() as *const c_char,
    b"replace\0".as_ptr() as *const c_char,
    b"combine\0".as_ptr() as *const c_char,
    b"blend\0".as_ptr() as *const c_char,
]);
pub static decor_state: GlobalCell<DecorState> = GlobalCell::new(DecorState {
    itr: [MarkTreeIter {
        pos: MTPos {
            row: 0 as int32_t,
            col: 0,
        },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [MarkTreeIter_s { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }],
    slots: DecorState_slots {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<DecorRangeSlot>(),
    },
    ranges_i: DecorState_ranges_i {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<c_int>(),
    },
    current_end: 0,
    future_begin: 0,
    free_slot_i: 0,
    new_range_ordering: 0,
    win: ::core::ptr::null_mut::<win_T>(),
    top_row: 0,
    row: 0,
    col_last: 0,
    current: 0,
    eol_col: 0,
    conceal: 0,
    conceal_char: 0,
    conceal_attr: 0,
    spell: kFalse,
    running_decor_provider: false,
    itr_valid: false,
});
#[unsafe(no_mangle)]
pub static decor_items: GlobalCell<DecorSignHighlightVec> =
    GlobalCell::new(DecorSignHighlightVec {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<DecorSignHighlight>(),
    });
pub static diff_context: GlobalCell<c_int> = GlobalCell::new(6 as c_int);
pub static diff_foldcolumn: GlobalCell<c_int> = GlobalCell::new(2 as c_int);
pub static diff_need_scrollbind: GlobalCell<bool> = GlobalCell::new(false);
pub static need_diff_redraw: GlobalCell<bool> = GlobalCell::new(false);
#[unsafe(no_mangle)]
pub static win_extmark_arr: GlobalCell<WinExtmarkVec> = GlobalCell::new(WinExtmarkVec {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<WinExtmark>(),
});
pub static updating_screen: GlobalCell<bool> = GlobalCell::new(false);
pub static redraw_not_allowed: GlobalCell<bool> = GlobalCell::new(false);
pub static screen_search_hl: GlobalCell<match_T> = GlobalCell::new(match_T {
    rm: regmmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    },
    buf: ::core::ptr::null_mut::<buf_T>(),
    lnum: 0,
    attr: 0,
    attr_cur: 0,
    first_lnum: 0,
    startcol: 0,
    endcol: 0,
    is_addpos: false,
    has_cursor: false,
    tm: 0,
});
pub static search_hl_has_cursor_lnum: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
pub static e_api_spawn_failed: GlobalCell<[c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [c_char; 30]>(*b"E903: Could not spawn API job\0")
});
pub static e_argreq: GlobalCell<[c_char; 24]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], [c_char; 24]>(*b"E471: Argument required\0")
});
pub static e_backslash: GlobalCell<[c_char; 39]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 39], [c_char; 39]>(*b"E10: \\ should be followed by /, ? or &\0")
});
pub static e_cmdwin: GlobalCell<[c_char; 65]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 65], [c_char; 65]>(
        *b"E11: Invalid in command-line window; <CR> executes, CTRL-C quits\0",
    )
});
pub static e_curdir: GlobalCell<[c_char; 69]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 69], [c_char; 69]>(
        *b"E12: Command not allowed in secure mode in current dir or tag search\0",
    )
});
pub static e_invalid_buffer_name_str: GlobalCell<[c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [c_char; 30]>(*b"E158: Invalid buffer name: %s\0")
});
pub static e_command_too_recursive: GlobalCell<[c_char; 28]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 28], [c_char; 28]>(*b"E169: Command too recursive\0")
});
pub static e_buffer_is_not_loaded: GlobalCell<[c_char; 27]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 27], [c_char; 27]>(*b"E681: Buffer is not loaded\0")
});
pub static e_endif: GlobalCell<[c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [c_char; 21]>(*b"E171: Missing :endif\0")
});
pub static e_endtry: GlobalCell<[c_char; 22]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 22], [c_char; 22]>(*b"E600: Missing :endtry\0")
});
pub static e_endwhile: GlobalCell<[c_char; 24]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], [c_char; 24]>(*b"E170: Missing :endwhile\0")
});
pub static e_endfor: GlobalCell<[c_char; 22]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 22], [c_char; 22]>(*b"E170: Missing :endfor\0")
});
pub static e_while: GlobalCell<[c_char; 31]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 31], [c_char; 31]>(*b"E588: :endwhile without :while\0")
});
pub static e_for: GlobalCell<[c_char; 27]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 27], [c_char; 27]>(*b"E588: :endfor without :for\0")
});
pub static e_exists: GlobalCell<[c_char; 37]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 37], [c_char; 37]>(*b"E13: File exists (add ! to override)\0")
});
pub static e_failed: GlobalCell<[c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [c_char; 21]>(*b"E472: Command failed\0")
});
pub static e_intern2: GlobalCell<[c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [c_char; 25]>(*b"E685: Internal error: %s\0")
});
pub static e_interr: GlobalCell<[c_char; 12]> =
    GlobalCell::new(unsafe { ::core::mem::transmute::<[u8; 12], [c_char; 12]>(*b"Interrupted\0") });
pub static e_invarg: GlobalCell<[c_char; 23]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 23], [c_char; 23]>(*b"E474: Invalid argument\0")
});
pub static e_invarg2: GlobalCell<[c_char; 27]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 27], [c_char; 27]>(*b"E475: Invalid argument: %s\0")
});
pub static e_invargval: GlobalCell<[c_char; 36]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 36], [c_char; 36]>(*b"E475: Invalid value for argument %s\0")
});
pub static e_invargNval: GlobalCell<[c_char; 40]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 40], [c_char; 40]>(*b"E475: Invalid value for argument %s: %s\0")
});
pub static e_duparg2: GlobalCell<[c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [c_char; 29]>(*b"E983: Duplicate argument: %s\0")
});
pub static e_invexpr2: GlobalCell<[c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [c_char; 30]>(*b"E15: Invalid expression: \"%s\"\0")
});
pub static e_invrange: GlobalCell<[c_char; 19]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 19], [c_char; 19]>(*b"E16: Invalid range\0")
});
pub static e_invcmd: GlobalCell<[c_char; 22]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 22], [c_char; 22]>(*b"E476: Invalid command\0")
});
pub static e_isadir2: GlobalCell<[c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [c_char; 25]>(*b"E17: \"%s\" is a directory\0")
});
pub static e_no_spell: GlobalCell<[c_char; 37]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 37], [c_char; 37]>(*b"E756: Spell checking is not possible\0")
});
pub static e_invchan: GlobalCell<[c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [c_char; 25]>(*b"E900: Invalid channel id\0")
});
pub static e_invchanjob: GlobalCell<[c_char; 36]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 36], [c_char; 36]>(*b"E900: Invalid channel id: not a job\0")
});
pub static e_jobspawn: GlobalCell<[c_char; 40]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 40], [c_char; 40]>(
        *b"E903: Process failed to start: %s: \"%s\"\0",
    )
});
pub static e_channotpty: GlobalCell<[c_char; 27]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 27], [c_char; 27]>(*b"E904: channel is not a pty\0")
});
pub static e_stdiochan2: GlobalCell<[c_char; 38]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 38], [c_char; 38]>(*b"E905: Couldn't open stdio channel: %s\0")
});
pub static e_invstream: GlobalCell<[c_char; 33]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 33], [c_char; 33]>(*b"E906: invalid stream for channel\0")
});
pub static e_invstreamrpc: GlobalCell<[c_char; 48]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 48], [c_char; 48]>(
        *b"E906: invalid stream for rpc channel, use 'rpc'\0",
    )
});
pub static e_streamkey: GlobalCell<[c_char; 68]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 68], [c_char; 68]>(
        *b"E5210: dict key '%s' already set for buffered stream in channel %lu\0",
    )
});
pub static e_libcall: GlobalCell<[c_char; 37]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 37], [c_char; 37]>(*b"E364: Library call failed for \"%s()\"\0")
});
pub static e_fsync: GlobalCell<[c_char; 23]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 23], [c_char; 23]>(*b"E667: Fsync failed: %s\0")
});
pub static e_mkdir: GlobalCell<[c_char; 37]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 37], [c_char; 37]>(*b"E739: Cannot create directory %s: %s\0")
});
pub static e_markinval: GlobalCell<[c_char; 34]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 34], [c_char; 34]>(*b"E19: Mark has invalid line number\0")
});
pub static e_marknotset: GlobalCell<[c_char; 18]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 18], [c_char; 18]>(*b"E20: Mark not set\0")
});
pub static e_modifiable: GlobalCell<[c_char; 46]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 46], [c_char; 46]>(
        *b"E21: Cannot make changes, 'modifiable' is off\0",
    )
});
pub static e_nesting: GlobalCell<[c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [c_char; 29]>(*b"E22: Scripts nested too deep\0")
});
pub static e_noalt: GlobalCell<[c_char; 23]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 23], [c_char; 23]>(*b"E23: No alternate file\0")
});
pub static e_noabbr: GlobalCell<[c_char; 26]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 26], [c_char; 26]>(*b"E24: No such abbreviation\0")
});
pub static e_nobang: GlobalCell<[c_char; 19]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 19], [c_char; 19]>(*b"E477: No ! allowed\0")
});
pub static e_nogroup: GlobalCell<[c_char; 38]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 38], [c_char; 38]>(*b"E28: No such highlight group name: %s\0")
});
pub static e_noinstext: GlobalCell<[c_char; 26]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 26], [c_char; 26]>(*b"E29: No inserted text yet\0")
});
pub static e_nolastcmd: GlobalCell<[c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [c_char; 30]>(*b"E30: No previous command line\0")
});
pub static e_nomap: GlobalCell<[c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [c_char; 21]>(*b"E31: No such mapping\0")
});
pub static e_noident: GlobalCell<[c_char; 33]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 33], [c_char; 33]>(*b"E349: No identifier under cursor\0")
});
pub static e_nomatch: GlobalCell<[c_char; 15]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 15], [c_char; 15]>(*b"E479: No match\0")
});
pub static e_nomatch2: GlobalCell<[c_char; 19]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 19], [c_char; 19]>(*b"E480: No match: %s\0")
});
pub static e_noname: GlobalCell<[c_char; 18]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 18], [c_char; 18]>(*b"E32: No file name\0")
});
pub static e_nopresub: GlobalCell<[c_char; 47]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 47], [c_char; 47]>(
        *b"E33: No previous substitute regular expression\0",
    )
});
pub static e_noprev: GlobalCell<[c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [c_char; 25]>(*b"E34: No previous command\0")
});
pub static e_noprevre: GlobalCell<[c_char; 36]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 36], [c_char; 36]>(*b"E35: No previous regular expression\0")
});
pub static e_norange: GlobalCell<[c_char; 23]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 23], [c_char; 23]>(*b"E481: No range allowed\0")
});
pub static e_noroom: GlobalCell<[c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [c_char; 21]>(*b"E36: Not enough room\0")
});
pub static e_notmp: GlobalCell<[c_char; 31]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 31], [c_char; 31]>(*b"E483: Can't get temp file name\0")
});
pub static e_notopen: GlobalCell<[c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [c_char; 25]>(*b"E484: Can't open file %s\0")
});
pub static e_notopen_2: GlobalCell<[c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [c_char; 29]>(*b"E484: Can't open file %s: %s\0")
});
pub static e_cant_read_file_str: GlobalCell<[c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [c_char; 25]>(*b"E485: Can't read file %s\0")
});
pub static e_null: GlobalCell<[c_char; 19]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 19], [c_char; 19]>(*b"E38: Null argument\0")
});
pub static e_number_exp: GlobalCell<[c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [c_char; 21]>(*b"E39: Number expected\0")
});
pub static e_openerrf: GlobalCell<[c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [c_char; 29]>(*b"E40: Can't open errorfile %s\0")
});
pub static e_outofmem: GlobalCell<[c_char; 20]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 20], [c_char; 20]>(*b"E41: Out of memory!\0")
});
pub static e_patnotf: GlobalCell<[c_char; 18]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 18], [c_char; 18]>(*b"Pattern not found\0")
});
pub static e_patnotf2: GlobalCell<[c_char; 28]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 28], [c_char; 28]>(*b"E486: Pattern not found: %s\0")
});
pub static e_positive: GlobalCell<[c_char; 32]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 32], [c_char; 32]>(*b"E487: Argument must be positive\0")
});
pub static e_prev_dir: GlobalCell<[c_char; 43]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 43], [c_char; 43]>(
        *b"E459: Cannot go back to previous directory\0",
    )
});
pub static e_no_errors: GlobalCell<[c_char; 15]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 15], [c_char; 15]>(*b"E42: No Errors\0")
});
pub static e_loclist: GlobalCell<[c_char; 23]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 23], [c_char; 23]>(*b"E776: No location list\0")
});
pub static e_re_damg: GlobalCell<[c_char; 26]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 26], [c_char; 26]>(*b"E43: Damaged match string\0")
});
pub static e_re_corr: GlobalCell<[c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [c_char; 30]>(*b"E44: Corrupted regexp program\0")
});
pub static e_readonly: GlobalCell<[c_char; 50]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 50], [c_char; 50]>(
        *b"E45: 'readonly' option is set (add ! to override)\0",
    )
});
pub static e_letwrong: GlobalCell<[c_char; 34]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 34], [c_char; 34]>(*b"E734: Wrong variable type for %s=\0")
});
pub static e_illvar: GlobalCell<[c_char; 32]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 32], [c_char; 32]>(*b"E461: Illegal variable name: %s\0")
});
pub static e_cannot_mod: GlobalCell<[c_char; 38]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 38], [c_char; 38]>(*b"E995: Cannot modify existing variable\0")
});
pub static e_cannot_change_readonly_variable_str: GlobalCell<[c_char; 45]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 45], [c_char; 45]>(
            *b"E46: Cannot change read-only variable \"%.*s\"\0",
        )
    });
pub static e_dictreq: GlobalCell<[c_char; 26]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 26], [c_char; 26]>(*b"E715: Dictionary required\0")
});
pub static e_blobidx: GlobalCell<[c_char; 35]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 35], [c_char; 35]>(*b"E979: Blob index out of range: %ld\0")
});
pub static e_invalblob: GlobalCell<[c_char; 33]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 33], [c_char; 33]>(*b"E978: Invalid operation for Blob\0")
});
pub static e_toomanyarg: GlobalCell<[c_char; 42]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 42], [c_char; 42]>(
        *b"E118: Too many arguments for function: %s\0",
    )
});
pub static e_toofewarg: GlobalCell<[c_char; 44]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 44], [c_char; 44]>(
        *b"E119: Not enough arguments for function: %s\0",
    )
});
pub static e_dictkey: GlobalCell<[c_char; 42]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 42], [c_char; 42]>(
        *b"E716: Key not present in Dictionary: \"%s\"\0",
    )
});
pub static e_dictkey_len: GlobalCell<[c_char; 44]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 44], [c_char; 44]>(
        *b"E716: Key not present in Dictionary: \"%.*s\"\0",
    )
});
pub static e_listreq: GlobalCell<[c_char; 20]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 20], [c_char; 20]>(*b"E714: List required\0")
});
pub static e_listblobreq: GlobalCell<[c_char; 28]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 28], [c_char; 28]>(*b"E897: List or Blob required\0")
});
pub static e_listblobarg: GlobalCell<[c_char; 44]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 44], [c_char; 44]>(
        *b"E899: Argument of %s must be a List or Blob\0",
    )
});
pub static e_listdictarg: GlobalCell<[c_char; 50]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 50], [c_char; 50]>(
        *b"E712: Argument of %s must be a List or Dictionary\0",
    )
});
pub static e_listdictblobarg: GlobalCell<[c_char; 56]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 56], [c_char; 56]>(
        *b"E896: Argument of %s must be a List, Dictionary or Blob\0",
    )
});
pub static e_readerrf: GlobalCell<[c_char; 35]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 35], [c_char; 35]>(*b"E47: Error while reading errorfile\0")
});
pub static e_sandbox: GlobalCell<[c_char; 28]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 28], [c_char; 28]>(*b"E48: Not allowed in sandbox\0")
});
pub static e_secure: GlobalCell<[c_char; 23]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 23], [c_char; 23]>(*b"E523: Not allowed here\0")
});
pub static e_textlock: GlobalCell<[c_char; 50]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 50], [c_char; 50]>(
        *b"E565: Not allowed to change text or change window\0",
    )
});
pub static e_screenmode: GlobalCell<[c_char; 40]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 40], [c_char; 40]>(*b"E359: Screen mode setting not supported\0")
});
pub static e_scroll: GlobalCell<[c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [c_char; 25]>(*b"E49: Invalid scroll size\0")
});
pub static e_shellempty: GlobalCell<[c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [c_char; 29]>(*b"E91: 'shell' option is empty\0")
});
pub static e_swapclose: GlobalCell<[c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [c_char; 30]>(*b"E72: Close error on swap file\0")
});
pub static e_toocompl: GlobalCell<[c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [c_char; 25]>(*b"E74: Command too complex\0")
});
pub static e_longname: GlobalCell<[c_char; 19]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 19], [c_char; 19]>(*b"E75: Name too long\0")
});
pub static e_toomany: GlobalCell<[c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [c_char; 25]>(*b"E77: Too many file names\0")
});
pub static e_trailing: GlobalCell<[c_char; 26]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 26], [c_char; 26]>(*b"E488: Trailing characters\0")
});
pub static e_trailing_arg: GlobalCell<[c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [c_char; 30]>(*b"E488: Trailing characters: %s\0")
});
pub static e_umark: GlobalCell<[c_char; 18]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 18], [c_char; 18]>(*b"E78: Unknown mark\0")
});
pub static e_wildexpand: GlobalCell<[c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [c_char; 29]>(*b"E79: Cannot expand wildcards\0")
});
pub static e_winheight: GlobalCell<[c_char; 56]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 56], [c_char; 56]>(
        *b"E591: 'winheight' cannot be smaller than 'winminheight'\0",
    )
});
pub static e_winwidth: GlobalCell<[c_char; 54]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 54], [c_char; 54]>(
        *b"E592: 'winwidth' cannot be smaller than 'winminwidth'\0",
    )
});
pub static e_write: GlobalCell<[c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [c_char; 25]>(*b"E80: Error while writing\0")
});
pub static e_zerocount: GlobalCell<[c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [c_char; 30]>(*b"E939: Positive count required\0")
});
pub static e_usingsid: GlobalCell<[c_char; 41]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 41], [c_char; 41]>(*b"E81: Using <SID> not in a script context\0")
});
pub static e_missingparen: GlobalCell<[c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [c_char; 30]>(*b"E107: Missing parentheses: %s\0")
});
pub static e_empty_buffer: GlobalCell<[c_char; 19]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 19], [c_char; 19]>(*b"E749: Empty buffer\0")
});
pub static e_nobufnr: GlobalCell<[c_char; 31]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 31], [c_char; 31]>(*b"E86: Buffer %ld does not exist\0")
});
pub static e_no_write_since_last_change: GlobalCell<[c_char; 32]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 32], [c_char; 32]>(*b"E37: No write since last change\0")
});
pub static e_no_write_since_last_change_add_bang_to_override: GlobalCell<[c_char; 52]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 52], [c_char; 52]>(
            *b"E37: No write since last change (add ! to override)\0",
        )
    });
pub static e_no_write_since_last_change_for_buffer_nr_add_bang_to_override: GlobalCell<
    [c_char; 66],
> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 66], [c_char; 66]>(
        *b"E89: No write since last change for buffer %d (add ! to override)\0",
    )
});
pub static e_buffer_nr_not_found: GlobalCell<[c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [c_char; 25]>(*b"E92: Buffer %d not found\0")
});
pub static e_unknown_function_str: GlobalCell<[c_char; 27]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 27], [c_char; 27]>(*b"E117: Unknown function: %s\0")
});
pub static e_str_not_inside_function: GlobalCell<[c_char; 31]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 31], [c_char; 31]>(*b"E193: %s not inside a function\0")
});
pub static e_job_still_running: GlobalCell<[c_char; 24]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], [c_char; 24]>(*b"E948: Job still running\0")
});
pub static e_job_still_running_add_bang_to_end_the_job: GlobalCell<[c_char; 47]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 47], [c_char; 47]>(
            *b"E948: Job still running (add ! to end the job)\0",
        )
    });
pub static e_invalpat: GlobalCell<[c_char; 42]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 42], [c_char; 42]>(
        *b"E682: Invalid search pattern or delimiter\0",
    )
});
pub static e_bufloaded: GlobalCell<[c_char; 39]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 39], [c_char; 39]>(*b"E139: File is loaded in another buffer\0")
});
pub static e_notset: GlobalCell<[c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [c_char; 29]>(*b"E764: Option '%s' is not set\0")
});
pub static e_dirnotf: GlobalCell<[c_char; 40]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 40], [c_char; 40]>(
        *b"E919: Directory not found in '%s': \"%s\"\0",
    )
});
pub static e_au_recursive: GlobalCell<[c_char; 44]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 44], [c_char; 44]>(
        *b"E952: Autocommand caused recursive behavior\0",
    )
});
pub static e_menu_only_exists_in_another_mode: GlobalCell<[c_char; 39]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 39], [c_char; 39]>(*b"E328: Menu only exists in another mode\0")
});
pub static e_autocmd_close: GlobalCell<[c_char; 34]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 34], [c_char; 34]>(*b"E813: Cannot close autocmd window\0")
});
pub static e_list_index_out_of_range_nr: GlobalCell<[c_char; 35]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 35], [c_char; 35]>(*b"E684: List index out of range: %ld\0")
});
pub static e_listarg: GlobalCell<[c_char; 36]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 36], [c_char; 36]>(*b"E686: Argument of %s must be a List\0")
});
pub static e_unsupportedoption: GlobalCell<[c_char; 27]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 27], [c_char; 27]>(*b"E519: Option not supported\0")
});
pub static e_fnametoolong: GlobalCell<[c_char; 24]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], [c_char; 24]>(*b"E856: Filename too long\0")
});
pub static e_using_float_as_string: GlobalCell<[c_char; 32]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 32], [c_char; 32]>(*b"E806: Using a Float as a String\0")
});
pub static e_cannot_edit_other_buf: GlobalCell<[c_char; 45]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 45], [c_char; 45]>(
        *b"E788: Not allowed to edit another buffer now\0",
    )
});
pub static e_using_number_as_bool_nr: GlobalCell<[c_char; 36]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 36], [c_char; 36]>(*b"E1023: Using a Number as a Bool: %d\0")
});
pub static e_not_callable_type_str: GlobalCell<[c_char; 31]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 31], [c_char; 31]>(*b"E1085: Not a callable type: %s\0")
});
pub static e_auabort: GlobalCell<[c_char; 43]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 43], [c_char; 43]>(
        *b"E855: Autocommands caused command to abort\0",
    )
});
pub static e_api_error: GlobalCell<[c_char; 20]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 20], [c_char; 20]>(*b"E5555: API call: %s\0")
});
pub static e_fast_api_disabled: GlobalCell<[c_char; 53]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 53], [c_char; 53]>(
        *b"E5560: %s must not be called in a fast event context\0",
    )
});
pub static e_floatonly: GlobalCell<[c_char; 62]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 62], [c_char; 62]>(
        *b"E5601: Cannot close window, only floating window would remain\0",
    )
});
pub static e_floatexchange: GlobalCell<[c_char; 39]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 39], [c_char; 39]>(*b"E5602: Cannot exchange or rotate float\0")
});
pub static e_cant_find_directory_str_in_cdpath: GlobalCell<[c_char; 42]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 42], [c_char; 42]>(
            *b"E344: Can't find directory \"%s\" in cdpath\0",
        )
    });
pub static e_cant_find_file_str_in_path: GlobalCell<[c_char; 35]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 35], [c_char; 35]>(*b"E345: Can't find file \"%s\" in path\0")
});
pub static e_no_more_directory_str_found_in_cdpath: GlobalCell<[c_char; 45]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 45], [c_char; 45]>(
            *b"E346: No more directory \"%s\" found in cdpath\0",
        )
    });
pub static e_no_more_file_str_found_in_path: GlobalCell<[c_char; 38]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 38], [c_char; 38]>(*b"E347: No more file \"%s\" found in path\0")
});
pub static e_value_is_locked: GlobalCell<[c_char; 22]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 22], [c_char; 22]>(*b"E741: Value is locked\0")
});
pub static e_value_is_locked_str: GlobalCell<[c_char; 28]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 28], [c_char; 28]>(*b"E741: Value is locked: %.*s\0")
});
pub static e_cannot_change_value: GlobalCell<[c_char; 26]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 26], [c_char; 26]>(*b"E742: Cannot change value\0")
});
pub static e_cannot_change_value_of_str: GlobalCell<[c_char; 34]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 34], [c_char; 34]>(*b"E742: Cannot change value of %.*s\0")
});
pub static e_cannot_set_variable_in_sandbox_str: GlobalCell<[c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [c_char; 49]>(
            *b"E794: Cannot set variable in the sandbox: \"%.*s\"\0",
        )
    });
pub static e_cannot_delete_variable_str: GlobalCell<[c_char; 34]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 34], [c_char; 34]>(*b"E795: Cannot delete variable %.*s\0")
});
pub static e_invalwindow: GlobalCell<[c_char; 28]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 28], [c_char; 28]>(*b"E957: Invalid window number\0")
});
pub static e_problem_creating_internal_diff: GlobalCell<[c_char; 41]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 41], [c_char; 41]>(*b"E960: Problem creating the internal diff\0")
});
pub static e_cannot_define_autocommands_for_all_events: GlobalCell<[c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [c_char; 49]>(
            *b"E1155: Cannot define autocommands for ALL events\0",
        )
    });
pub static e_cannot_change_arglist_recursively: GlobalCell<[c_char; 51]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 51], [c_char; 51]>(
            *b"E1156: Cannot change the argument list recursively\0",
        )
    });
pub static e_resulting_text_too_long: GlobalCell<[c_char; 31]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 31], [c_char; 31]>(*b"E1240: Resulting text too long\0")
});
pub static e_line_number_out_of_range: GlobalCell<[c_char; 32]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 32], [c_char; 32]>(*b"E1247: Line number out of range\0")
});
pub static e_highlight_group_name_invalid_char: GlobalCell<[c_char; 39]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 39], [c_char; 39]>(
            *b"E5248: Invalid character in group name\0",
        )
    });
pub static e_highlight_group_name_too_long: GlobalCell<[c_char; 37]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 37], [c_char; 37]>(*b"E1249: Highlight group name too long\0")
});
pub static e_string_required: GlobalCell<[c_char; 22]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 22], [c_char; 22]>(*b"E928: String required\0")
});
pub static e_invalid_column_number_nr: GlobalCell<[c_char; 33]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 33], [c_char; 33]>(*b"E964: Invalid column number: %ld\0")
});
pub static e_invalid_line_number_nr: GlobalCell<[c_char; 31]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 31], [c_char; 31]>(*b"E966: Invalid line number: %ld\0")
});
pub static e_reduce_of_an_empty_str_with_no_initial_value: GlobalCell<[c_char; 50]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 50], [c_char; 50]>(
            *b"E998: Reduce of an empty %s with no initial value\0",
        )
    });
pub static e_invalid_value_for_blob_nr: GlobalCell<[c_char; 36]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 36], [c_char; 36]>(*b"E1239: Invalid value for blob: 0xlX\0")
});
pub static e_stray_closing_curly_str: GlobalCell<[c_char; 44]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 44], [c_char; 44]>(
        *b"E1278: Stray '}' without a matching '{': %s\0",
    )
});
pub static e_missing_close_curly_str: GlobalCell<[c_char; 23]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 23], [c_char; 23]>(*b"E1279: Missing '}': %s\0")
});
pub static e_cannot_change_menus_while_listing: GlobalCell<[c_char; 41]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 41], [c_char; 41]>(
            *b"E1310: Cannot change menus while listing\0",
        )
    });
pub static e_not_allowed_to_change_window_layout_in_this_autocmd: GlobalCell<[c_char; 63]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 63], [c_char; 63]>(
            *b"E1312: Not allowed to change the window layout in this autocmd\0",
        )
    });
pub static e_val_too_large_len: GlobalCell<[c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [c_char; 29]>(*b"E1510: Value too large: %.*s\0")
});
pub static e_undobang_cannot_redo_or_move_branch: GlobalCell<[c_char; 68]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 68], [c_char; 68]>(
            *b"E5767: Cannot use :undo! to redo or move to a different undo branch\0",
        )
    });
pub static e_winfixbuf_cannot_go_to_buffer: GlobalCell<[c_char; 52]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 52], [c_char; 52]>(
        *b"E1513: Cannot switch buffer. 'winfixbuf' is enabled\0",
    )
});
pub static e_invalid_return_type_from_findfunc: GlobalCell<[c_char; 45]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 45], [c_char; 45]>(
            *b"E1514: 'findfunc' did not return a List type\0",
        )
    });
pub static e_cannot_switch_to_a_closing_buffer: GlobalCell<[c_char; 41]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 41], [c_char; 41]>(
            *b"E1546: Cannot switch to a closing buffer\0",
        )
    });
pub static e_cannot_have_more_than_nr_diff_anchors: GlobalCell<[c_char; 45]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 45], [c_char; 45]>(
            *b"E1549: Cannot have more than %d diff anchors\0",
        )
    });
pub static e_failed_to_find_all_diff_anchors: GlobalCell<[c_char; 39]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 39], [c_char; 39]>(*b"E1550: Failed to find all diff anchors\0")
});
pub static e_diff_anchors_with_hidden_windows: GlobalCell<[c_char; 60]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 60], [c_char; 60]>(
        *b"E1562: Diff anchors cannot be used with hidden diff windows\0",
    )
});
pub static e_leadtab_requires_tab: GlobalCell<[c_char; 66]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 66], [c_char; 66]>(
        *b"E1572: 'listchars' field \"leadtab\" requires \"tab\" to be specified\0",
    )
});
pub static e_invalid_format_string_single_percent_s: GlobalCell<[c_char; 55]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 55], [c_char; 55]>(
            *b"E1577: Invalid format string, only one \"%s\" is allowed\0",
        )
    });
pub static e_cannot_read_from_str_2: GlobalCell<[c_char; 28]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 28], [c_char; 28]>(*b"E282: Cannot read from \"%s\"\0")
});
pub static e_conflicting_configs: GlobalCell<[c_char; 38]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 38], [c_char; 38]>(
        *b"E5422: Conflicting configs: \"%s\" \"%s\"\0",
    )
});
pub static e_unknown_option2: GlobalCell<[c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [c_char; 25]>(*b"E355: Unknown option: %s\0")
});
pub static top_bot_msg: GlobalCell<[c_char; 37]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 37], [c_char; 37]>(*b"search hit TOP, continuing at BOTTOM\0")
});
pub static bot_top_msg: GlobalCell<[c_char; 37]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 37], [c_char; 37]>(*b"search hit BOTTOM, continuing at TOP\0")
});
pub static line_msg: GlobalCell<[c_char; 7]> =
    GlobalCell::new(unsafe { ::core::mem::transmute::<[u8; 7], [c_char; 7]>(*b" line \0") });
pub static EVALARG_EVALUATE: GlobalCell<evalarg_T> = GlobalCell::new(evalarg_T {
    eval_flags: EVAL_EVALUATE as c_int,
    eval_getline: None,
    eval_cookie: ::core::ptr::null_mut::<c_void>(),
    eval_tofree: ::core::ptr::null_mut::<c_char>(),
});
pub static msg_ext_skip_flush: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_ext_overwrite: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_ext_skip_verbose: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_grid: GlobalCell<ScreenGrid> = GlobalCell::new(ScreenGrid {
    handle: 0 as handle_T,
    chars: ::core::ptr::null_mut::<schar_T>(),
    attrs: ::core::ptr::null_mut::<sattr_T>(),
    vcols: ::core::ptr::null_mut::<colnr_T>(),
    line_offset: ::core::ptr::null_mut::<size_t>(),
    dirty_col: ::core::ptr::null_mut::<c_int>(),
    rows: 0 as c_int,
    cols: 0 as c_int,
    valid: false,
    throttled: false,
    blending: false,
    mouse_enabled: true,
    zindex: 0 as c_int,
    comp_row: 0 as c_int,
    comp_col: 0 as c_int,
    comp_width: 0 as c_int,
    comp_height: 0 as c_int,
    comp_index: 0 as size_t,
    comp_disabled: false,
    pending_comp_index_update: true,
});
pub static msg_grid_pos: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static msg_grid_adj: GlobalCell<GridView> = GlobalCell::new(GridView {
    target: ::core::ptr::null_mut::<ScreenGrid>(),
    row_offset: 0,
    col_offset: 0,
});
pub static msg_scrolled_at_flush: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static msg_grid_scroll_discount: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static msg_listdo_overwrite: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
// TV_CSTRING (SIZE_MAX - 1): c2rust dropped the initializer expression and
// left 0, which is a valid pointer-sentinel value and would corrupt any
// caller comparing against it (the unit tests do, via FFI).
#[unsafe(no_mangle)]
pub static kTVCstring: GlobalCell<size_t> = GlobalCell::new(18446744073709551614);
pub static disable_fold_update: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
#[unsafe(no_mangle)]
pub static test_disable_char_avail: GlobalCell<bool> = GlobalCell::new(false);
pub const IOSIZE: c_int = 1024 as c_int + 1 as c_int;
pub const SYS_VIMRC_FILE: [c_char; 17] =
    unsafe { ::core::mem::transmute::<[u8; 17], [c_char; 17]>(*b"$VIM/sysinit.vim\0") };
pub const VIMRC_FILE: [c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [c_char; 8]>(*b".nvimrc\0") };
pub static g_stats: GlobalCell<nvim_stats_s> = GlobalCell::new(nvim_stats_s {
    fsync: 0 as int64_t,
    redraw: 0 as int64_t,
    log_skip: 0 as int16_t,
});
pub const NO_BUFFERS: c_int = 1 as c_int;
pub static Rows: GlobalCell<c_int> = GlobalCell::new(24 as c_int);
pub static Columns: GlobalCell<c_int> = GlobalCell::new(80 as c_int);
pub static mod_mask: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static vgetc_mod_mask: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static vgetc_char: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static cmdline_row: GlobalCell<c_int> = GlobalCell::new(0);
pub static redraw_cmdline: GlobalCell<bool> = GlobalCell::new(false);
pub static redraw_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static clear_cmdline: GlobalCell<bool> = GlobalCell::new(false);
pub static mode_displayed: GlobalCell<bool> = GlobalCell::new(false);
pub static cmdline_star: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static redrawing_cmdline: GlobalCell<bool> = GlobalCell::new(false);
pub static cmdline_was_last_drawn: GlobalCell<bool> = GlobalCell::new(false);
pub static exec_from_reg: GlobalCell<bool> = GlobalCell::new(false);
pub static dollar_vcol: GlobalCell<colnr_T> = GlobalCell::new(-1 as colnr_T);
pub static edit_submode: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static edit_submode_pre: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static edit_submode_extra: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static edit_submode_highl: GlobalCell<hlf_T> = GlobalCell::new(HLF_NONE);
pub static cmdmsg_rl: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_col: GlobalCell<c_int> = GlobalCell::new(0);
pub static msg_row: GlobalCell<c_int> = GlobalCell::new(0);
pub static msg_scrolled: GlobalCell<c_int> = GlobalCell::new(0);
pub static msg_scrolled_ign: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_did_scroll: GlobalCell<bool> = GlobalCell::new(false);
pub static keep_msg: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static keep_msg_hl_id: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static need_fileinfo: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_scroll: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static msg_didout: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_didany: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_nowait: GlobalCell<bool> = GlobalCell::new(false);
pub static emsg_off: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static info_message: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_hist_off: GlobalCell<bool> = GlobalCell::new(false);
pub static need_clr_eos: GlobalCell<bool> = GlobalCell::new(false);
#[unsafe(no_mangle)]
pub static emsg_skip: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static emsg_severe: GlobalCell<bool> = GlobalCell::new(false);
pub static emsg_assert_fails_msg: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static emsg_assert_fails_lnum: GlobalCell<c_long> = GlobalCell::new(0 as c_long);
pub static emsg_assert_fails_context: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static did_endif: GlobalCell<bool> = GlobalCell::new(false);
pub static did_emsg: GlobalCell<c_int> = GlobalCell::new(0);
pub static called_vim_beep: GlobalCell<bool> = GlobalCell::new(false);
pub static did_emsg_syntax: GlobalCell<bool> = GlobalCell::new(false);
pub static called_emsg: GlobalCell<c_int> = GlobalCell::new(0);
pub static ex_exitval: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static emsg_on_display: GlobalCell<bool> = GlobalCell::new(false);
pub static rc_did_emsg: GlobalCell<bool> = GlobalCell::new(false);
pub static no_wait_return: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static need_wait_return: GlobalCell<bool> = GlobalCell::new(false);
pub static did_wait_return: GlobalCell<bool> = GlobalCell::new(false);
pub static need_maketitle: GlobalCell<bool> = GlobalCell::new(true);
pub static quit_more: GlobalCell<bool> = GlobalCell::new(false);
pub static vgetc_busy: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static didset_vim: GlobalCell<bool> = GlobalCell::new(false);
pub static didset_vimruntime: GlobalCell<bool> = GlobalCell::new(false);
pub static lines_left: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static msg_no_more: GlobalCell<bool> = GlobalCell::new(false);
pub static ex_nesting_level: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static debug_break_level: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static debug_did_msg: GlobalCell<bool> = GlobalCell::new(false);
pub static debug_tick: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static debug_backtrace_level: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static do_profiling: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static current_exception: GlobalCell<*mut except_T> =
    GlobalCell::new(::core::ptr::null_mut::<except_T>());
pub static did_throw: GlobalCell<bool> = GlobalCell::new(false);
pub static need_rethrow: GlobalCell<bool> = GlobalCell::new(false);
pub static check_cstack: GlobalCell<bool> = GlobalCell::new(false);
pub static trylevel: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static force_abort: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_list: GlobalCell<*mut *mut msglist_T> =
    GlobalCell::new(::core::ptr::null_mut::<*mut msglist_T>());
pub static suppress_errthrow: GlobalCell<bool> = GlobalCell::new(false);
pub static caught_stack: GlobalCell<*mut except_T> =
    GlobalCell::new(::core::ptr::null_mut::<except_T>());
pub static may_garbage_collect: GlobalCell<bool> = GlobalCell::new(false);
pub static want_garbage_collect: GlobalCell<bool> = GlobalCell::new(false);
pub static garbage_collect_at_exit: GlobalCell<bool> = GlobalCell::new(false);
pub const SID_CMDARG: c_int = -2 as c_int;
pub const SID_CARG: c_int = -3 as c_int;
pub const SID_ENV: c_int = -4 as c_int;
pub static current_sctx: GlobalCell<sctx_T> = GlobalCell::new(sctx_T {
    sc_sid: 0 as scid_T,
    sc_seq: 0 as c_int,
    sc_lnum: 0 as linenr_T,
    sc_chan: 0 as uint64_t,
});
pub static current_ui: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub static did_source_packages: GlobalCell<bool> = GlobalCell::new(false);
pub static provider_caller_scope: GlobalCell<caller_scope> = GlobalCell::new(caller_scope {
    script_ctx: sctx_T {
        sc_sid: 0,
        sc_seq: 0,
        sc_lnum: 0,
        sc_chan: 0,
    },
    es_entry: estack_T {
        es_lnum: 0,
        es_name: ::core::ptr::null_mut::<c_char>(),
        es_type: ETYPE_TOP,
        es_info: estack_T_es_info {
            sctx: ::core::ptr::null_mut::<sctx_T>(),
        },
    },
    autocmd_fname: ::core::ptr::null_mut::<c_char>(),
    autocmd_match: ::core::ptr::null_mut::<c_char>(),
    autocmd_fname_full: false,
    autocmd_bufnr: 0,
    funccalp: ::core::ptr::null_mut::<c_void>(),
});
pub static provider_call_nesting: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static t_colors: GlobalCell<c_int> = GlobalCell::new(256 as c_int);
pub static include_none: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static include_default: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static include_link: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static highlight_match: GlobalCell<bool> = GlobalCell::new(false);
pub static search_match_lines: GlobalCell<linenr_T> = GlobalCell::new(0);
pub static search_match_endcol: GlobalCell<colnr_T> = GlobalCell::new(0);
pub static search_first_line: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
pub static search_last_line: GlobalCell<linenr_T> = GlobalCell::new(MAXLNUM as c_int as linenr_T);
pub static no_smartcase: GlobalCell<bool> = GlobalCell::new(false);
pub static need_check_timestamps: GlobalCell<bool> = GlobalCell::new(false);
pub static did_check_timestamps: GlobalCell<bool> = GlobalCell::new(false);
pub static no_check_timestamps: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static mouse_grid: GlobalCell<c_int> = GlobalCell::new(0);
pub static mouse_row: GlobalCell<c_int> = GlobalCell::new(0);
pub static mouse_col: GlobalCell<c_int> = GlobalCell::new(0);
pub static mouse_past_bottom: GlobalCell<bool> = GlobalCell::new(false);
pub static mouse_past_eol: GlobalCell<bool> = GlobalCell::new(false);
pub static mouse_dragging: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static root_menu: GlobalCell<*mut vimmenu_T> =
    GlobalCell::new(::core::ptr::null_mut::<vimmenu_T>());
pub static sys_menu: GlobalCell<bool> = GlobalCell::new(false);
#[unsafe(no_mangle)]
pub static firstwin: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
#[unsafe(no_mangle)]
pub static lastwin: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
#[unsafe(no_mangle)]
pub static prevwin: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
#[unsafe(no_mangle)]
pub static curwin: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static topframe: GlobalCell<*mut frame_T> = GlobalCell::new(::core::ptr::null_mut::<frame_T>());
#[unsafe(no_mangle)]
pub static first_tabpage: GlobalCell<*mut tabpage_T> =
    GlobalCell::new(::core::ptr::null_mut::<tabpage_T>());
#[unsafe(no_mangle)]
pub static curtab: GlobalCell<*mut tabpage_T> =
    GlobalCell::new(::core::ptr::null_mut::<tabpage_T>());
pub static lastused_tabpage: GlobalCell<*mut tabpage_T> =
    GlobalCell::new(::core::ptr::null_mut::<tabpage_T>());
pub static redraw_tabline: GlobalCell<bool> = GlobalCell::new(false);
pub static firstbuf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
pub static lastbuf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
#[unsafe(no_mangle)]
pub static curbuf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
pub static global_alist: GlobalCell<alist_T> = GlobalCell::new(alist_T {
    al_ga: garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<c_void>(),
    },
    al_refcount: 0,
    id: 0,
});
pub static max_alist_id: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static arg_had_last: GlobalCell<bool> = GlobalCell::new(false);
pub static ru_col: GlobalCell<c_int> = GlobalCell::new(0);
pub static ru_wid: GlobalCell<c_int> = GlobalCell::new(0);
pub static sc_col: GlobalCell<c_int> = GlobalCell::new(0);
#[unsafe(no_mangle)]
pub static starting: GlobalCell<c_int> = GlobalCell::new(2 as c_int);
#[unsafe(no_mangle)]
pub static exiting: GlobalCell<bool> = GlobalCell::new(false);
pub static v_dying: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static stdin_isatty: GlobalCell<bool> = GlobalCell::new(true);
pub static stdout_isatty: GlobalCell<bool> = GlobalCell::new(true);
pub static stderr_isatty: GlobalCell<bool> = GlobalCell::new(true);
pub static stdin_fd: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static full_screen: GlobalCell<bool> = GlobalCell::new(false);
pub static secure: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static textlock: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static allbuf_lock: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
#[unsafe(no_mangle)]
pub static sandbox: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static silent_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static VIsual: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
});
pub static VIsual_active: GlobalCell<bool> = GlobalCell::new(false);
pub static VIsual_select: GlobalCell<bool> = GlobalCell::new(false);
pub static VIsual_select_reg: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static VIsual_select_exclu_adj: GlobalCell<bool> = GlobalCell::new(false);
pub static restart_VIsual_select: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static VIsual_reselect: GlobalCell<c_int> = GlobalCell::new(0);
pub static VIsual_mode: GlobalCell<c_int> = GlobalCell::new('v' as c_int);
pub static redo_VIsual_busy: GlobalCell<bool> = GlobalCell::new(false);
pub static resel_VIsual_mode: GlobalCell<c_int> = GlobalCell::new('\0' as c_int);
pub static resel_VIsual_line_count: GlobalCell<linenr_T> = GlobalCell::new(0);
pub static resel_VIsual_vcol: GlobalCell<colnr_T> = GlobalCell::new(0);
pub static where_paste_started: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
});
pub static did_ai: GlobalCell<bool> = GlobalCell::new(false);
pub static ai_col: GlobalCell<colnr_T> = GlobalCell::new(0 as colnr_T);
pub static end_comment_pending: GlobalCell<c_int> = GlobalCell::new('\0' as c_int);
pub static did_syncbind: GlobalCell<bool> = GlobalCell::new(false);
pub static did_si: GlobalCell<bool> = GlobalCell::new(false);
pub static can_si: GlobalCell<bool> = GlobalCell::new(false);
pub static can_si_back: GlobalCell<bool> = GlobalCell::new(false);
pub static old_indent: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static saved_cursor: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0 as linenr_T,
    col: 0 as colnr_T,
    coladd: 0 as colnr_T,
});
pub static Insstart: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
});
pub static Insstart_orig: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
});
pub static orig_line_count: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
pub static vr_lines_changed: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static inhibit_delete_count: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static fenc_default: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static State: GlobalCell<c_int> = GlobalCell::new(MODE_NORMAL as c_int);
pub static debug_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static finish_op: GlobalCell<bool> = GlobalCell::new(false);
pub static opcount: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static motion_force: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static exmode_active: GlobalCell<bool> = GlobalCell::new(false);
pub static pending_exmode_active: GlobalCell<bool> = GlobalCell::new(false);
pub static ex_no_reprint: GlobalCell<bool> = GlobalCell::new(false);
pub static cmdpreview: GlobalCell<bool> = GlobalCell::new(false);
pub static reg_recording: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static reg_executing: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static pending_end_reg_executing: GlobalCell<bool> = GlobalCell::new(false);
pub static reg_recorded: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static no_mapping: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static no_zero_mapping: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static allow_keys: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static no_u_sync: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static u_sync_once: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static force_restart_edit: GlobalCell<bool> = GlobalCell::new(false);
pub static restart_edit: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static arrow_used: GlobalCell<bool> = GlobalCell::new(false);
pub static ins_at_eol: GlobalCell<bool> = GlobalCell::new(false);
pub static no_abbr: GlobalCell<bool> = GlobalCell::new(true);
pub static mapped_ctrl_c: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static ctrl_c_interrupts: GlobalCell<bool> = GlobalCell::new(true);
#[unsafe(no_mangle)]
pub static cmdmod: GlobalCell<cmdmod_T> = GlobalCell::new(cmdmod_T {
    cmod_flags: 0,
    cmod_split: 0,
    cmod_tab: 0,
    cmod_filter_pat: ::core::ptr::null_mut::<c_char>(),
    cmod_filter_regmatch: regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<c_char>(); 10],
        endp: [::core::ptr::null_mut::<c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    },
    cmod_filter_force: false,
    cmod_verbose: 0,
    cmod_save_ei: ::core::ptr::null_mut::<c_char>(),
    cmod_did_sandbox: 0,
    cmod_verbose_save: 0,
    cmod_save_msg_silent: 0,
    cmod_save_msg_scroll: 0,
    cmod_did_esilent: 0,
});
pub static msg_silent: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
#[unsafe(no_mangle)]
pub static emsg_silent: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static emsg_noredir: GlobalCell<bool> = GlobalCell::new(false);
pub static cmd_silent: GlobalCell<bool> = GlobalCell::new(false);
pub static in_assert_fails: GlobalCell<bool> = GlobalCell::new(false);
pub const SEA_NONE: c_int = 0 as c_int;
pub const SEA_DIALOG: c_int = 1 as c_int;
pub const SEA_QUIT: c_int = 2 as c_int;
pub static swap_exists_action: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static swap_exists_did_quit: GlobalCell<bool> = GlobalCell::new(false);
pub static IObuff: GlobalCell<[c_char; 1025]> = GlobalCell::new([0; 1025]);
#[unsafe(no_mangle)]
pub static NameBuff: GlobalCell<[c_char; 4096]> = GlobalCell::new([0; 4096]);
pub static msg_buf: GlobalCell<[c_char; 480]> = GlobalCell::new([0; 480]);
pub static os_buf: GlobalCell<[c_char; 4096]> = GlobalCell::new([0; 4096]);
pub static RedrawingDisabled: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static readonlymode: GlobalCell<bool> = GlobalCell::new(false);
pub static recoverymode: GlobalCell<bool> = GlobalCell::new(false);
pub static typebuf: GlobalCell<typebuf_T> = GlobalCell::new(typebuf_T {
    tb_buf: ::core::ptr::null_mut::<uint8_t>(),
    tb_noremap: ::core::ptr::null_mut::<uint8_t>(),
    tb_buflen: 0 as c_int,
    tb_off: 0 as c_int,
    tb_len: 0 as c_int,
    tb_maplen: 0 as c_int,
    tb_silent: 0 as c_int,
    tb_no_abbr_cnt: 0 as c_int,
    tb_change_cnt: 0 as c_int,
});
pub static typebuf_was_empty: GlobalCell<bool> = GlobalCell::new(false);
pub static ex_normal_busy: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static expr_map_lock: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static ignore_script: GlobalCell<bool> = GlobalCell::new(false);
pub static stop_insert_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static KeyTyped: GlobalCell<bool> = GlobalCell::new(false);
pub static KeyStuffed: GlobalCell<c_int> = GlobalCell::new(0);
pub static maptick: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static must_redraw: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static skip_redraw: GlobalCell<bool> = GlobalCell::new(false);
pub static do_redraw: GlobalCell<bool> = GlobalCell::new(false);
pub static must_redraw_pum: GlobalCell<bool> = GlobalCell::new(false);
pub static need_highlight_changed: GlobalCell<bool> = GlobalCell::new(true);
pub static scriptout: GlobalCell<*mut FILE> = GlobalCell::new(::core::ptr::null_mut::<FILE>());
pub static got_int: GlobalCell<bool> = GlobalCell::new(false);
pub static bangredo: GlobalCell<bool> = GlobalCell::new(false);
pub static searchcmdlen: GlobalCell<c_int> = GlobalCell::new(0);
pub static reg_do_extmatch: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static re_extmatch_in: GlobalCell<*mut reg_extmatch_T> =
    GlobalCell::new(::core::ptr::null_mut::<reg_extmatch_T>());
pub static re_extmatch_out: GlobalCell<*mut reg_extmatch_T> =
    GlobalCell::new(::core::ptr::null_mut::<reg_extmatch_T>());
pub static did_outofmem_msg: GlobalCell<bool> = GlobalCell::new(false);
pub static did_swapwrite_msg: GlobalCell<bool> = GlobalCell::new(false);
pub static global_busy: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static listcmd_busy: GlobalCell<bool> = GlobalCell::new(false);
pub static need_start_insertmode: GlobalCell<bool> = GlobalCell::new(false);
pub static last_mode: GlobalCell<[c_char; 4]> =
    GlobalCell::new(unsafe { ::core::mem::transmute::<[u8; 4], [c_char; 4]>(*b"n\0\0\0") });
pub static last_cmdline: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static repeat_cmdline: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static new_last_cmdline: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static postponed_split: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static postponed_split_flags: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static postponed_split_tab: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static g_do_tagpreview: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static g_tag_at_cursor: GlobalCell<bool> = GlobalCell::new(false);
pub static replace_offset: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static escape_chars: GlobalCell<*mut c_char> =
    GlobalCell::new(b" \t\\\"|\0".as_ptr() as *const c_char as *mut c_char);
pub static keep_help_flag: GlobalCell<bool> = GlobalCell::new(false);
pub static redir_off: GlobalCell<bool> = GlobalCell::new(false);
pub static redir_fd: GlobalCell<*mut FILE> = GlobalCell::new(::core::ptr::null_mut::<FILE>());
pub static redir_reg: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static redir_vname: GlobalCell<bool> = GlobalCell::new(false);
pub static capture_ga: GlobalCell<*mut garray_T> =
    GlobalCell::new(::core::ptr::null_mut::<garray_T>());
pub static langmap_mapchar: GlobalCell<[uint8_t; 256]> = GlobalCell::new([0; 256]);
pub static save_p_ls: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static save_p_wmh: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static wild_menu_showing: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static globaldir: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static last_chdir_reason: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static km_stopsel: GlobalCell<bool> = GlobalCell::new(false);
pub static km_startsel: GlobalCell<bool> = GlobalCell::new(false);
pub static cmdwin_type: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static cmdwin_result: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static cmdwin_level: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static cmdwin_buf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
pub static cmdwin_win: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static cmdwin_old_curwin: GlobalCell<*mut win_T> =
    GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static cmdline_win: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static no_lines_msg: GlobalCell<[c_char; 23]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 23], [c_char; 23]>(*b"--No lines in buffer--\0")
});
pub static sub_nsubs: GlobalCell<c_int> = GlobalCell::new(0);
pub static sub_nlines: GlobalCell<linenr_T> = GlobalCell::new(0);
pub static wim_flags: GlobalCell<[uint8_t; 4]> = GlobalCell::new([0; 4]);
pub static stl_syntax: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static no_hlsearch: GlobalCell<bool> = GlobalCell::new(false);
pub static typebuf_was_filled: GlobalCell<bool> = GlobalCell::new(false);
pub static virtual_op: GlobalCell<TriState> = GlobalCell::new(kNone);
#[unsafe(no_mangle)]
pub static display_tick: GlobalCell<disptick_T> = GlobalCell::new(0 as disptick_T);
pub static spell_redraw_lnum: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
pub static time_fd: GlobalCell<*mut FILE> = GlobalCell::new(::core::ptr::null_mut::<FILE>());
pub static vim_ignored: GlobalCell<c_int> = GlobalCell::new(0);
pub static embedded_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static headless_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static windowsVersion: GlobalCell<[c_char; 20]> = GlobalCell::new([0 as c_char; 20]);
pub static magic_overruled: GlobalCell<optmagic_T> = GlobalCell::new(OPTION_MAGIC_NOT_SET);
pub static skip_win_fix_cursor: GlobalCell<bool> = GlobalCell::new(false);
pub static skip_win_fix_scroll: GlobalCell<bool> = GlobalCell::new(false);
pub static skip_update_topline: GlobalCell<bool> = GlobalCell::new(false);
pub static default_grid: GlobalCell<ScreenGrid> = GlobalCell::new(ScreenGrid {
    handle: 0 as handle_T,
    chars: ::core::ptr::null_mut::<schar_T>(),
    attrs: ::core::ptr::null_mut::<sattr_T>(),
    vcols: ::core::ptr::null_mut::<colnr_T>(),
    line_offset: ::core::ptr::null_mut::<size_t>(),
    dirty_col: ::core::ptr::null_mut::<c_int>(),
    rows: 0 as c_int,
    cols: 0 as c_int,
    valid: false,
    throttled: false,
    blending: false,
    mouse_enabled: true,
    zindex: 0 as c_int,
    comp_row: 0 as c_int,
    comp_col: 0 as c_int,
    comp_width: 0 as c_int,
    comp_height: 0 as c_int,
    comp_index: 0 as size_t,
    comp_disabled: false,
    pending_comp_index_update: true,
});
pub static default_gridview: GlobalCell<GridView> = GlobalCell::new(GridView {
    target: (default_grid.as_raw() as *const _) as *mut ScreenGrid,
    row_offset: 0,
    col_offset: 0,
});
pub static resizing_screen: GlobalCell<bool> = GlobalCell::new(false);
pub static linebuf_char: GlobalCell<*mut schar_T> =
    GlobalCell::new(::core::ptr::null_mut::<schar_T>());
pub static linebuf_attr: GlobalCell<*mut sattr_T> =
    GlobalCell::new(::core::ptr::null_mut::<sattr_T>());
pub static linebuf_vcol: GlobalCell<*mut colnr_T> =
    GlobalCell::new(::core::ptr::null_mut::<colnr_T>());
pub static linebuf_scratch: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static empty_string_option: GlobalCell<[c_char; 1]> =
    GlobalCell::new(unsafe { ::core::mem::transmute::<[u8; 1], [c_char; 1]>(*b"\0") });
pub static p_ambw: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_acd: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ai: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_bin: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_bomb: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_bl: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_cin: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_channel: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_cink: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cinsd: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cinw: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cfu: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ofu: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_tsrfu: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ci: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ar: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_aw: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_awa: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_bs: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_bg: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_bk: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_bkc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static bkc_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_bdir: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_bex: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_bo: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static breakat_flags: GlobalCell<[c_char; 256]> = GlobalCell::new([0; 256]);
pub static bo_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_bsk: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_breakat: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_bh: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_bt: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_busy: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_cmp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static cmp_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_enc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_deco: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ccv: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cino: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cedit: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cb: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static cb_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_cwh: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ch: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_cms: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cpt: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cto: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_columns: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_confirm: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_cia: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static cia_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_cot: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static cot_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_ac: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_act: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_acl: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_pumborder: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_pb: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ph: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_pw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_pmw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_com: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cpo: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_debug: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_def: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_inc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_dia: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_dip: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_dex: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_dict: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_dg: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_dir: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_dy: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static dy_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_ead: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_emoji: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ea: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ep: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_eb: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ef: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_efm: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_gefm: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_gp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_eof: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_eol: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ei: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_et: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_exrc: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_fenc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_fencs: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ff: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ffs: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
#[unsafe(no_mangle)]
pub static p_fic: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ft: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_fcs: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ffu: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_fixeol: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_fcl: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_fdls: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_fdo: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static fdo_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_fex: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_flp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_fo: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_fp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_fs: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_gd: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_guicursor: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_guifont: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_guifontwide: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_hf: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_hh: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_hlg: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_hid: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_hl: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_hls: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_hi: GlobalCell<OptInt> = GlobalCell::new(0);
#[unsafe(no_mangle)]
pub static p_arshape: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_icon: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_iconstring: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ic: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_iminsert: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_imsearch: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_inf: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_inex: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_is: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_inde: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_indk: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_icm: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_isf: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_isi: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_isk: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_isp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_js: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_jop: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static jop_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_keymap: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_kp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_km: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_langmap: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_lnr: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_lrm: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_lm: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_lines: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_linespace: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_lisp: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_lop: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_lispwords: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ls: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_stal: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_lcs: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_lz: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_lpl: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_magic: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_menc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_mef: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_mp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_mps: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_mat: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mco: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mfd: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mmd: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mmp: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mis: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mopt: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_msc: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_msm: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ml: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_mle: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_mls: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ma: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_mod: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_mouse: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_mousem: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_mousemev: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_mousef: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_mh: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_mousescroll: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_mousescroll_vert: GlobalCell<OptInt> = GlobalCell::new(3 as OptInt);
pub static p_mousescroll_hor: GlobalCell<OptInt> = GlobalCell::new(6 as OptInt);
pub static p_mouset: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_more: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_nf: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_opfunc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_para: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_paste: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_pex: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_pm: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_path: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cdpath: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_pi: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_pyx: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_qe: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ro: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_rdb: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static rdb_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_rdt: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_re: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_report: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_pvh: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_chi: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ari: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ri: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ru: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ruf: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_pp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_qftf: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_rtp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_scbk: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_sj: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_so: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_sbo: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_sections: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_secure: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_sel: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_slm: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ssop: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static ssop_flags: GlobalCell<c_uint> = GlobalCell::new(0);
#[unsafe(no_mangle)]
pub static p_sh: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
#[unsafe(no_mangle)]
pub static p_shcf: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_sp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_shq: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
#[unsafe(no_mangle)]
pub static p_sxq: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
#[unsafe(no_mangle)]
pub static p_sxe: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_srr: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_stmp: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_stl: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_wbr: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_sr: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_sw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_shm: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_sbr: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_sc: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_sloc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_sft: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_sm: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_smd: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ss: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_siso: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_scs: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_si: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_sta: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_sts: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_sb: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_sua: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_swf: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_smc: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_tpm: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_tal: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_tpf: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static tpf_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_tfu: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_spc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_spf: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_spl: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_spo: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static spo_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_sps: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_spr: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_sol: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_su: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_swb: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static swb_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_spk: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_syn: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_tcl: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static tcl_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_ts: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_tbs: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_tc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static tc_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_tl: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_tr: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_tags: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_tgst: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_tbidi: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_tw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_to: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_timeout: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_tm: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_title: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_titlelen: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_titleold: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_titlestring: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_tsr: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_tgc: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ttimeout: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ttm: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_tf: GlobalCell<c_int> = GlobalCell::new(0);
#[unsafe(no_mangle)]
pub static p_udir: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_udf: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ul: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ur: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_uc: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ut: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_shada: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_shadafile: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_termsync: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_vsts: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_vts: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_vdir: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_vop: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static vop_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_vb: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ve: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static ve_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_verbose: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_warn: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_wop: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static wop_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_window: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wak: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_wig: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ww: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_wc: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wcm: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wic: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_wim: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_wmnu: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_winborder: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_wh: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wmh: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wmw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wiw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wm: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ws: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_write: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_wa: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_wb: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_wd: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_cdh: GlobalCell<c_int> = GlobalCell::new(0);
pub static hlf_names: GlobalCell<[*const c_char; 76]> = GlobalCell::new([
    ::core::ptr::null::<c_char>(),
    b"SpecialKey\0".as_ptr() as *const c_char,
    b"EndOfBuffer\0".as_ptr() as *const c_char,
    b"TermCursor\0".as_ptr() as *const c_char,
    b"NonText\0".as_ptr() as *const c_char,
    b"Directory\0".as_ptr() as *const c_char,
    b"ErrorMsg\0".as_ptr() as *const c_char,
    b"IncSearch\0".as_ptr() as *const c_char,
    b"Search\0".as_ptr() as *const c_char,
    b"CurSearch\0".as_ptr() as *const c_char,
    b"MoreMsg\0".as_ptr() as *const c_char,
    b"ModeMsg\0".as_ptr() as *const c_char,
    b"LineNr\0".as_ptr() as *const c_char,
    b"LineNrAbove\0".as_ptr() as *const c_char,
    b"LineNrBelow\0".as_ptr() as *const c_char,
    b"CursorLineNr\0".as_ptr() as *const c_char,
    b"CursorLineSign\0".as_ptr() as *const c_char,
    b"CursorLineFold\0".as_ptr() as *const c_char,
    b"Question\0".as_ptr() as *const c_char,
    b"StatusLine\0".as_ptr() as *const c_char,
    b"StatusLineNC\0".as_ptr() as *const c_char,
    b"WinSeparator\0".as_ptr() as *const c_char,
    b"VertSplit\0".as_ptr() as *const c_char,
    b"Title\0".as_ptr() as *const c_char,
    b"Visual\0".as_ptr() as *const c_char,
    b"VisualNC\0".as_ptr() as *const c_char,
    b"WarningMsg\0".as_ptr() as *const c_char,
    b"WildMenu\0".as_ptr() as *const c_char,
    b"Folded\0".as_ptr() as *const c_char,
    b"FoldColumn\0".as_ptr() as *const c_char,
    b"DiffAdd\0".as_ptr() as *const c_char,
    b"DiffChange\0".as_ptr() as *const c_char,
    b"DiffDelete\0".as_ptr() as *const c_char,
    b"DiffText\0".as_ptr() as *const c_char,
    b"DiffTextAdd\0".as_ptr() as *const c_char,
    b"SignColumn\0".as_ptr() as *const c_char,
    b"Conceal\0".as_ptr() as *const c_char,
    b"SpellBad\0".as_ptr() as *const c_char,
    b"SpellCap\0".as_ptr() as *const c_char,
    b"SpellRare\0".as_ptr() as *const c_char,
    b"SpellLocal\0".as_ptr() as *const c_char,
    b"Pmenu\0".as_ptr() as *const c_char,
    b"PmenuSel\0".as_ptr() as *const c_char,
    b"PmenuMatch\0".as_ptr() as *const c_char,
    b"PmenuMatchSel\0".as_ptr() as *const c_char,
    b"PmenuKind\0".as_ptr() as *const c_char,
    b"PmenuKindSel\0".as_ptr() as *const c_char,
    b"PmenuExtra\0".as_ptr() as *const c_char,
    b"PmenuExtraSel\0".as_ptr() as *const c_char,
    b"PmenuSbar\0".as_ptr() as *const c_char,
    b"PmenuThumb\0".as_ptr() as *const c_char,
    b"PmenuBorder\0".as_ptr() as *const c_char,
    b"TabLine\0".as_ptr() as *const c_char,
    b"TabLineSel\0".as_ptr() as *const c_char,
    b"TabLineFill\0".as_ptr() as *const c_char,
    b"CursorColumn\0".as_ptr() as *const c_char,
    b"CursorLine\0".as_ptr() as *const c_char,
    b"ColorColumn\0".as_ptr() as *const c_char,
    b"QuickFixLine\0".as_ptr() as *const c_char,
    b"Whitespace\0".as_ptr() as *const c_char,
    b"NormalNC\0".as_ptr() as *const c_char,
    b"MsgSeparator\0".as_ptr() as *const c_char,
    b"NormalFloat\0".as_ptr() as *const c_char,
    b"MsgArea\0".as_ptr() as *const c_char,
    b"FloatBorder\0".as_ptr() as *const c_char,
    b"WinBar\0".as_ptr() as *const c_char,
    b"WinBarNC\0".as_ptr() as *const c_char,
    b"Cursor\0".as_ptr() as *const c_char,
    b"FloatTitle\0".as_ptr() as *const c_char,
    b"FloatFooter\0".as_ptr() as *const c_char,
    b"StatusLineTerm\0".as_ptr() as *const c_char,
    b"StatusLineTermNC\0".as_ptr() as *const c_char,
    b"StderrMsg\0".as_ptr() as *const c_char,
    b"StdoutMsg\0".as_ptr() as *const c_char,
    b"OkMsg\0".as_ptr() as *const c_char,
    b"PreInsert\0".as_ptr() as *const c_char,
]);
pub static highlight_attr: GlobalCell<[c_int; 76]> = GlobalCell::new([0; 76]);
pub static highlight_attr_last: GlobalCell<[c_int; 76]> = GlobalCell::new([0; 76]);
pub static highlight_user: GlobalCell<[c_int; 9]> = GlobalCell::new([0; 9]);
pub static highlight_stlnc: GlobalCell<[c_int; 9]> = GlobalCell::new([0; 9]);
pub static cterm_normal_fg_color: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static cterm_normal_bg_color: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static normal_fg: GlobalCell<RgbValue> = GlobalCell::new(-1 as RgbValue);
pub static normal_bg: GlobalCell<RgbValue> = GlobalCell::new(-1 as RgbValue);
pub static normal_sp: GlobalCell<RgbValue> = GlobalCell::new(-1 as RgbValue);
pub static ns_hl_global: GlobalCell<NS> = GlobalCell::new(0 as NS);
pub static ns_hl_win: GlobalCell<NS> = GlobalCell::new(-1 as NS);
pub static ns_hl_fast: GlobalCell<NS> = GlobalCell::new(-1 as NS);
pub static ns_hl_active: GlobalCell<NS> = GlobalCell::new(0 as NS);
pub static hl_attr_active: GlobalCell<*mut c_int> =
    GlobalCell::new((highlight_attr.as_raw() as *const _) as *mut c_int);
pub static curbuf_splice_pending: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub const LUA_GLOBALSINDEX: c_int = -10002 as c_int;
pub static nlua_global_refs: GlobalCell<*mut nlua_ref_state_t> =
    GlobalCell::new(::core::ptr::null_mut::<nlua_ref_state_t>());
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
        wq_mutex: pthread_mutex_t {
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
        },
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
        cloexec_lock: pthread_rwlock_t {
            __data: __pthread_rwlock_arch_t {
                __readers: 0,
                __writers: 0,
                __wrphase_futex: 0,
                __writers_futex: 0,
                __pad3: 0,
                __pad4: 0,
                __cur_writer: 0,
                __shared: 0,
                __rwelision: 0,
                __pad1: [0; 7],
                __pad2: 0,
                __flags: 0,
            },
        },
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
    children: ::core::ptr::null_mut::<c_void>(),
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
    mutex: pthread_mutex_t {
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
    },
    recursive: 0,
    closing: false,
});
static argv0: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
static err_arg_missing: GlobalCell<*const c_char> =
    GlobalCell::new(b"Argument missing after\0".as_ptr() as *const c_char);
static err_opt_garbage: GlobalCell<*const c_char> =
    GlobalCell::new(b"Garbage after option argument\0".as_ptr() as *const c_char);
static err_opt_unknown: GlobalCell<*const c_char> =
    GlobalCell::new(b"Unknown option argument\0".as_ptr() as *const c_char);
static err_too_many_args: GlobalCell<*const c_char> =
    GlobalCell::new(b"Too many edit arguments\0".as_ptr() as *const c_char);
static err_extra_cmd: GlobalCell<*const c_char> = GlobalCell::new(
    b"Too many \"+command\", \"-c command\" or \"--cmd command\" arguments\0".as_ptr()
        as *const c_char,
);
#[unsafe(no_mangle)]
pub unsafe extern "C" fn event_init() {
    loop_init(main_loop.ptr());
    env_init();
    resize_events.set(multiqueue_new_child((*main_loop.ptr()).events));
    autocmd_init();
    signal_init();
    channel_init();
    terminal_init();
    ui_init();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"event init\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
unsafe extern "C" fn event_teardown() -> bool {
    if (*main_loop.ptr()).events.is_null() {
        input_stop();
        return true_0 != 0;
    }
    multiqueue_process_events((*main_loop.ptr()).events);
    loop_poll_events(main_loop.ptr(), 0 as int64_t);
    input_stop();
    server_teardown();
    channel_teardown();
    proc_teardown(main_loop.ptr());
    timer_teardown();
    signal_teardown();
    terminal_teardown();
    return loop_close(main_loop.ptr(), true_0 != 0);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn early_init(mut paramp: *mut mparm_T) {
    os_hint_priority();
    estack_init();
    cmdline_init();
    eval_init();
    set_vim_var_nr(VV_STARTTIME, os_realtime());
    init_path(if !(*argv0.ptr()).is_null() {
        argv0.get() as *const c_char
    } else {
        b"nvim\0".as_ptr() as *const c_char
    });
    init_normal_cmds();
    runtime_init();
    highlight_init();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"early init\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    init_locale();
    set_init_tablocal();
    win_alloc_first();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"init first window\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    alist_init(global_alist.ptr());
    (*global_alist.ptr()).id = 0 as c_int;
    init_homedir();
    set_init_1(
        if !paramp.is_null() {
            (*paramp).clean as c_int
        } else {
            false_0
        } != 0,
    );
    log_init();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"inits 1\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    set_lang_var();
    qf_init_stack();
}
unsafe fn main_0(mut argc: c_int, mut argv: *mut *mut c_char) -> c_int {
    argv0.set(*argv.offset(0 as c_int as isize));
    if !appname_is_valid() {
        fprintf(
            stderr,
            b"$NVIM_APPNAME must be a name or relative path.\n\0".as_ptr() as *const c_char,
        );
        exit(1 as c_int);
    }
    if argc > 1 as c_int
        && strcasecmp(
            *argv.offset(1 as c_int as isize),
            b"-ll\0".as_ptr() as *const c_char as *mut c_char,
        ) == 0 as c_int
    {
        if argc == 2 as c_int {
            print_mainerr(
                err_arg_missing.get(),
                *argv.offset(1 as c_int as isize),
                ::core::ptr::null::<c_char>(),
            );
            exit(1 as c_int);
        }
        nlua_run_script(argv, argc, 3 as c_int);
    }
    let mut fname: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut params: mparm_T = mparm_T {
        argc: 0,
        argv: ::core::ptr::null_mut::<*mut c_char>(),
        use_vimrc: ::core::ptr::null_mut::<c_char>(),
        clean: false,
        n_commands: 0,
        commands: [::core::ptr::null_mut::<c_char>(); 10],
        cmds_tofree: [0; 10],
        n_pre_commands: 0,
        pre_commands: [::core::ptr::null_mut::<c_char>(); 10],
        luaf: ::core::ptr::null_mut::<c_char>(),
        lua_arg0: 0,
        edit_type: 0,
        tagname: ::core::ptr::null_mut::<c_char>(),
        use_ef: ::core::ptr::null_mut::<c_char>(),
        input_istext: false,
        no_swap_file: 0,
        use_debug_break_level: 0,
        window_count: 0,
        window_layout: 0,
        diff_mode: 0,
        listen_addr: ::core::ptr::null_mut::<c_char>(),
        remote: 0,
        server_addr: ::core::ptr::null_mut::<c_char>(),
        scriptin: ::core::ptr::null_mut::<c_char>(),
        scriptout: ::core::ptr::null_mut::<c_char>(),
        scriptout_append: false,
        had_stdin_file: false,
    };
    init_params(&raw mut params, argc, argv);
    init_startuptime(&raw mut params);
    let mut i: c_int = 1 as c_int;
    while i < params.argc {
        if strcasecmp(
            *params.argv.offset(i as isize),
            b"--clean\0".as_ptr() as *const c_char as *mut c_char,
        ) == 0 as c_int
        {
            params.clean = true_0 != 0;
            break;
        } else {
            i += 1;
        }
    }
    event_init();
    early_init(&raw mut params);
    set_argv_var(argv, argc);
    check_and_set_isatty(&raw mut params);
    command_line_scan(&raw mut params);
    set_argf_var();
    nlua_init(argv, argc, params.lua_arg0);
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"init lua interpreter\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if embedded_mode.get() {
        let mut err: *const c_char = ::core::ptr::null::<c_char>();
        if channel_from_stdio(
            true_0 != 0,
            CallbackReader {
                cb: Callback {
                    data: Callback_data {
                        funcref: ::core::ptr::null_mut::<c_char>(),
                    },
                    type_0: kCallbackNone,
                },
                self_0: ::core::ptr::null_mut::<dict_T>(),
                buffer: GA_EMPTY_INIT_VALUE,
                eof: false,
                buffered: false_0 != 0,
                fwd_err: false_0 != 0,
                type_0: ::core::ptr::null::<c_char>(),
            },
            &raw mut err,
        ) == 0
        {
            abort();
        }
    }
    if (*global_alist.ptr()).al_ga.ga_len > 0 as c_int {
        fname = get_fname(&raw mut params);
    }
    if recoverymode.get() as c_int != 0 && fname.is_null() {
        headless_mode.set(true_0 != 0);
    }
    let mut has_term: bool = stdin_isatty.get() as c_int != 0
        || stdout_isatty.get() as c_int != 0
        || stderr_isatty.get() as c_int != 0;
    let mut use_builtin_ui: bool = has_term as c_int != 0
        && !headless_mode.get()
        && !embedded_mode.get()
        && !silent_mode.get();
    if params.remote != 0 {
        remote_request(
            &raw mut params,
            params.remote,
            params.server_addr,
            argc,
            argv,
            use_builtin_ui,
        );
    }
    let mut remote_ui: bool = ui_client_channel_id.get() != 0 as uint64_t;
    if use_builtin_ui as c_int != 0 && !remote_ui {
        ui_client_forward_stdin.set(!stdin_isatty.get());
        let mut rv: uint64_t = ui_client_start_server(
            get_vim_var_str(VV_PROGPATH),
            params.argc as size_t,
            params.argv,
        );
        if rv == 0 {
            fprintf(
                stderr,
                b"Failed to start Nvim server!\n\0".as_ptr() as *const c_char,
            );
            os_exit(1 as c_int);
        }
        ui_client_channel_id.set(rv);
    }
    if ui_client_channel_id.get() != 0 {
        ui_client_run();
    }
    '_c2rust_label: {
        if ui_client_channel_id.get() == 0 && !use_builtin_ui {
        } else {
            __assert_fail(
                b"!ui_client_channel_id && !use_builtin_ui\0".as_ptr() as *const c_char,
                b"src/nvim/main.rs\0".as_ptr() as *const c_char,
                369 as c_uint,
                b"int main(int, char **)\0".as_ptr() as *const c_char,
            );
        }
    };
    if !server_init(params.listen_addr) {
        mainerr(
            IObuff.ptr() as *mut c_char,
            ::core::ptr::null::<c_char>(),
            ::core::ptr::null::<c_char>(),
        );
    }
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"expanding arguments\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if params.diff_mode != 0 && params.window_count == -1 as c_int {
        params.window_count = 0 as c_int;
    }
    (*RedrawingDisabled.ptr()) += 1;
    setbuf(stdout, ::core::ptr::null_mut::<c_char>());
    full_screen.set(!silent_mode.get());
    win_init_size();
    if params.diff_mode != 0 {
        diff_win_options(firstwin.get(), false_0 != 0);
    }
    '_c2rust_label_0: {
        if p_ch.get() >= 0 as OptInt
            && Rows.get() as OptInt >= p_ch.get()
            && Rows.get() as OptInt - p_ch.get() <= 2147483647 as OptInt
        {
        } else {
            __assert_fail(
                b"p_ch >= 0 && Rows >= p_ch && Rows - p_ch <= INT_MAX\0".as_ptr() as *const c_char,
                b"src/nvim/main.rs\0".as_ptr() as *const c_char,
                414 as c_uint,
                b"int main(int, char **)\0".as_ptr() as *const c_char,
            );
        }
    };
    cmdline_row.set(Rows.get() - p_ch.get() as c_int);
    msg_row.set(cmdline_row.get());
    default_grid_alloc();
    set_init_2(headless_mode.get());
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"inits 2\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    msg_scroll.set(true_0);
    no_wait_return.set(true_0);
    init_highlight(true_0 != 0, false_0 != 0);
    ui_comp_syn_init();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"init highlight\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    debug_break_level.set(params.use_debug_break_level);
    if !stdin_isatty.get()
        && !params.input_istext
        && silent_mode.get() as c_int != 0
        && exmode_active.get() as c_int != 0
    {
        input_start();
    }
    let mut use_remote_ui: bool = embedded_mode.get() as c_int != 0 && !headless_mode.get();
    if use_remote_ui {
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"waiting for UI\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
        remote_ui_wait_for_attach();
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"done waiting for UI\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
        (*firstwin.get()).w_prev_height = (*firstwin.get()).w_height;
    }
    starting.set(NO_BUFFERS);
    screenclear();
    win_new_screensize();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"clear screen\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if edit_stdin(&raw mut params) {
        params.edit_type = EDIT_STDIN as c_int;
    }
    if !params.scriptin.is_null() {
        if !open_scriptin(params.scriptin) {
            os_exit(2 as c_int);
        }
    }
    if !params.scriptout.is_null() {
        scriptout.set(os_fopen(
            params.scriptout,
            if params.scriptout_append as c_int != 0 {
                APPENDBIN.as_ptr()
            } else {
                WRITEBIN.as_ptr()
            },
        ));
        if (*scriptout.ptr()).is_null() {
            fprintf(
                stderr,
                gettext(b"Cannot open for script output: \"\0".as_ptr() as *const c_char),
            );
            fprintf(
                stderr,
                b"%s\"\n\0".as_ptr() as *const c_char,
                params.scriptout,
            );
            os_exit(2 as c_int);
        }
    }
    nlua_init_defaults();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"init default mappings & autocommands\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    let mut vimrc_none: bool = strequal(params.use_vimrc, b"NONE\0".as_ptr() as *const c_char);
    p_lpl.set(if vimrc_none as c_int != 0 {
        params.clean as c_int
    } else {
        p_lpl.get()
    });
    exe_pre_commands(&raw mut params);
    if !vimrc_none || params.clean as c_int != 0 {
        filetype_plugin_enable();
    }
    source_startup_scripts(&raw mut params);
    if !vimrc_none || params.clean as c_int != 0 {
        filetype_maybe_enable();
        syn_maybe_enable();
    }
    set_vim_var_nr(VV_VIM_DID_INIT, 1 as varnumber_T);
    load_plugins();
    set_window_layout(&raw mut params);
    if recoverymode.get() as c_int != 0 && fname.is_null() {
        recover_names(
            ::core::ptr::null_mut::<c_char>(),
            true_0 != 0,
            ::core::ptr::null_mut::<list_T>(),
            0 as c_int,
            ::core::ptr::null_mut::<*mut c_char>(),
        );
        os_exit(0 as c_int);
    }
    set_init_3();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"inits 3\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if params.no_swap_file != 0 {
        p_uc.set(0 as OptInt);
    }
    if silent_mode.get() {
        p_ut.set(1 as OptInt);
    }
    if *p_shada.get() as c_int != NUL {
        shada_read_everything(::core::ptr::null::<c_char>(), false_0 != 0, true_0 != 0);
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"reading ShaDa\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
    if get_vim_var_list(VV_OLDFILES).is_null() {
        set_vim_var_list(VV_OLDFILES, tv_list_alloc(0 as ptrdiff_t));
    }
    handle_quickfix(&raw mut params);
    starting.set(NO_BUFFERS);
    no_wait_return.set(false_0);
    if !exmode_active.get() {
        msg_scroll.set(false_0);
    }
    if params.edit_type == EDIT_STDIN as c_int && !recoverymode.get() {
        read_stdin();
    }
    setmouse();
    redraw_later(curwin.get(), UPD_VALID as c_int);
    no_wait_return.set(true_0);
    create_windows(&raw mut params);
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"opening buffers\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    set_vim_var_string(
        VV_SWAPCOMMAND,
        ::core::ptr::null::<c_char>(),
        -1 as ptrdiff_t,
    );
    if exmode_active.get() {
        (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
    }
    apply_autocmds(
        EVENT_BUFENTER,
        ::core::ptr::null_mut::<c_char>(),
        ::core::ptr::null_mut::<c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"BufEnter autocommands\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    setpcmark();
    if params.edit_type == EDIT_QF as c_int {
        qf_jump(
            ::core::ptr::null_mut::<qf_info_T>(),
            0 as c_int,
            0 as c_int,
            false_0,
        );
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"jump to first error\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
    edit_buffers(&raw mut params);
    if params.diff_mode != 0 {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_arg_idx_invalid == 0 {
                diff_win_options(wp, true_0 != 0);
            }
            wp = (*wp).w_next;
        }
    }
    shorten_fnames(false_0);
    handle_tag(params.tagname);
    if params.n_commands > 0 as c_int {
        exe_commands(&raw mut params);
    }
    starting.set(0 as c_int);
    RedrawingDisabled.set(0 as c_int);
    redraw_all_later(UPD_NOT_VALID as c_int);
    no_wait_return.set(false_0);
    do_autochdir();
    set_vim_var_nr(VV_VIM_DID_ENTER, 1 as varnumber_T);
    apply_autocmds(
        EVENT_VIMENTER,
        ::core::ptr::null_mut::<c_char>(),
        ::core::ptr::null_mut::<c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"VimEnter autocommands\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if use_remote_ui {
        do_autocmd_uienter_all();
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"UIEnter autocommands\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
    set_reg_var(get_default_register_name());
    if (*curwin.get()).w_onebuf_opt.wo_diff != 0 && (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
        update_topline(curwin.get());
        check_scrollbind(0 as linenr_T, 0 as c_int);
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"diff scrollbinding\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
    if restart_edit.get() != 0 as c_int {
        stuffcharReadbuff(-(253 as c_int + ((KE_NOP as c_int) << 8 as c_int)));
    }
    if cb_flags.get() & (kOptCbFlagUnnamed as c_int | kOptCbFlagUnnamedplus as c_int) as c_uint != 0
    {
        eval_has_provider(b"clipboard\0".as_ptr() as *const c_char, false_0 != 0);
    }
    if !params.luaf.is_null() {
        msg_scroll.set(true_0);
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<c_char>(),
            b"main\0".as_ptr() as *const c_char,
            678 as c_int,
            true_0 != 0,
            b"executing Lua -l script\0".as_ptr() as *const c_char,
        );
        let mut lua_ok: bool = nlua_exec_file(params.luaf);
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"executing Lua -l script\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
        if msg_didout.get() {
            msg_putchar('\n' as c_int);
            msg_didout.set(false_0 != 0);
        }
        getout(if lua_ok as c_int != 0 {
            0 as c_int
        } else {
            1 as c_int
        });
    }
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"before starting main loop\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    logmsg(
        LOGLVL_INF,
        ::core::ptr::null::<c_char>(),
        b"main\0".as_ptr() as *const c_char,
        689 as c_int,
        true_0 != 0,
        b"starting main loop\0".as_ptr() as *const c_char,
    );
    normal_enter(false_0 != 0, false_0 != 0);
    return 0 as c_int;
}
pub unsafe extern "C" fn os_exit(mut r: c_int) -> ! {
    exiting.set(true_0 != 0);
    if ui_client_channel_id.get() != 0 {
        ui_client_stop();
        if r == 0 as c_int {
            r = ui_client_exit_status.get();
        }
    } else {
        ui_flush();
        ui_call_stop();
    }
    if !event_teardown() && r == 0 as c_int {
        r = 1 as c_int;
    }
    if ui_client_channel_id.get() != 0 {
        if stdout_isatty.get() {
            tcdrain(STDOUT_FILENO);
        }
        if stderr_isatty.get() {
            tcdrain(STDERR_FILENO);
        }
    } else {
        ml_close_all(true_0 != 0);
    }
    if used_stdin.get() {
        stream_set_blocking(STDIN_FILENO, true_0 != 0);
    }
    logmsg(
        LOGLVL_INF,
        ::core::ptr::null::<c_char>(),
        b"os_exit\0".as_ptr() as *const c_char,
        737 as c_int,
        true_0 != 0,
        b"Nvim exit: %d\0".as_ptr() as *const c_char,
        r,
    );
    exit(r);
}
pub unsafe extern "C" fn getout(mut exitval: c_int) -> ! {
    '_c2rust_label: {
        if ui_client_channel_id.get() == 0 {
        } else {
            __assert_fail(
                b"!ui_client_channel_id\0".as_ptr() as *const c_char,
                b"src/nvim/main.rs\0".as_ptr() as *const c_char,
                750 as c_uint,
                b"void getout(int)\0".as_ptr() as *const c_char,
            );
        }
    };
    exiting.set(true_0 != 0);
    time_finish();
    if exmode_active.get() {
        exitval += ex_exitval.get();
    }
    set_vim_var_type(VV_EXITING, VAR_NUMBER);
    set_vim_var_nr(VV_EXITING, exitval as varnumber_T);
    if *get_vim_var_str(VV_EXITREASON) as c_int == NUL {
        set_vim_var_string(
            VV_EXITREASON,
            b"quit\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 5]>().wrapping_sub(1 as usize) as ptrdiff_t,
        );
    }
    invoke_all_defer();
    if v_dying.get() <= 1 as c_int {
        let mut next_tp: *const tabpage_T = ::core::ptr::null::<tabpage_T>();
        let mut tp: *const tabpage_T = first_tabpage.get();
        while !tp.is_null() {
            next_tp = (*tp).tp_next;
            let mut wp: *mut win_T = if tp == curtab.get() as *const tabpage_T {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if !((*wp).w_buffer.is_null() || !buf_valid((*wp).w_buffer)) {
                    let mut buf: *mut buf_T = (*wp).w_buffer;
                    if buf_get_changedtick(buf) != -1 as varnumber_T {
                        let mut bufref: bufref_T = bufref_T {
                            br_buf: ::core::ptr::null_mut::<buf_T>(),
                            br_fnum: 0,
                            br_buf_free_count: 0,
                        };
                        set_bufref(&raw mut bufref, buf);
                        apply_autocmds(
                            EVENT_BUFWINLEAVE,
                            (*buf).b_fname,
                            (*buf).b_fname,
                            false_0 != 0,
                            buf,
                        );
                        if bufref_valid(&raw mut bufref) {
                            buf_set_changedtick(buf, -1 as varnumber_T);
                        }
                        next_tp = first_tabpage.get();
                        break;
                    }
                }
                wp = (*wp).w_next;
            }
            tp = next_tp;
        }
        let mut buf_0: *mut buf_T = firstbuf.get();
        while !buf_0.is_null() {
            if !(*buf_0).b_ml.ml_mfp.is_null() {
                let mut bufref_0: bufref_T = bufref_T {
                    br_buf: ::core::ptr::null_mut::<buf_T>(),
                    br_fnum: 0,
                    br_buf_free_count: 0,
                };
                set_bufref(&raw mut bufref_0, buf_0);
                apply_autocmds(
                    EVENT_BUFUNLOAD,
                    (*buf_0).b_fname,
                    (*buf_0).b_fname,
                    false_0 != 0,
                    buf_0,
                );
                if !bufref_valid(&raw mut bufref_0) {
                    break;
                }
            }
            buf_0 = (*buf_0).b_next;
        }
        let mut unblock: c_int = 0 as c_int;
        if is_autocmd_blocked() {
            unblock_autocmds();
            unblock += 1;
        }
        apply_autocmds(
            EVENT_VIMLEAVEPRE,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        if unblock != 0 {
            block_autocmds();
        }
    }
    if !(*p_shada.ptr()).is_null() && *p_shada.get() as c_int != NUL {
        shada_write_file(::core::ptr::null::<c_char>(), false_0 != 0);
    }
    if v_dying.get() <= 1 as c_int {
        let mut unblock_0: c_int = 0 as c_int;
        if is_autocmd_blocked() {
            unblock_autocmds();
            unblock_0 += 1;
        }
        apply_autocmds(
            EVENT_VIMLEAVE,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        if unblock_0 != 0 {
            block_autocmds();
        }
    }
    profile_dump();
    if did_emsg.get() != 0 {
        no_wait_return.set(false_0);
        wait_return(false_0);
    }
    if p_title.get() != 0 && *p_titleold.get() as c_int != NUL {
        ui_call_set_title(cstr_as_string(p_titleold.get()));
    }
    if garbage_collect_at_exit.get() {
        garbage_collect(false_0 != 0);
    }
    os_exit(exitval);
}
pub unsafe extern "C" fn preserve_exit(mut errmsg: *const c_char) -> ! {
    static really_exiting: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if really_exiting.get() {
        if used_stdin.get() {
            stream_set_blocking(STDIN_FILENO, true_0 != 0);
        }
        exit(2 as c_int);
    }
    really_exiting.set(true_0 != 0);
    signal_reject_deadly();
    if ui_client_channel_id.get() != 0 {
        ui_client_stop();
    }
    if !errmsg.is_null() && *errmsg.offset(0 as c_int as isize) as c_int != NUL {
        let mut has_eol: bool = '\n' as c_int
            == *errmsg.offset(strlen(errmsg).wrapping_sub(1 as size_t) as isize) as c_int;
        fprintf(
            stderr,
            if has_eol as c_int != 0 {
                b"%s\0".as_ptr() as *const c_char
            } else {
                b"%s\n\0".as_ptr() as *const c_char
            },
            errmsg,
        );
    }
    if ui_client_channel_id.get() != 0 {
        os_exit(1 as c_int);
    }
    ml_close_notmod();
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if !(*buf).b_ml.ml_mfp.is_null() && !(*(*buf).b_ml.ml_mfp).mf_fname.is_null() {
            if !errmsg.is_null() {
                fprintf(
                    stderr,
                    b"Nvim: preserving files...\n\0".as_ptr() as *const c_char,
                );
            }
            ml_sync_all(false_0, false_0, true_0 != 0);
            break;
        } else {
            buf = (*buf).b_next;
        }
    }
    ml_close_all(false_0 != 0);
    if !errmsg.is_null() {
        fprintf(stderr, b"Nvim: Finished.\n\0".as_ptr() as *const c_char);
    }
    getout(1 as c_int);
}
unsafe extern "C" fn get_number_arg(
    mut p: *const c_char,
    mut idx: *mut c_int,
    mut def: c_int,
) -> c_int {
    if ascii_isdigit(*p.offset(*idx as isize) as c_int) {
        def = atoi(p.offset(*idx as isize));
        while ascii_isdigit(*p.offset(*idx as isize) as c_int) {
            *idx = *idx + 1 as c_int;
        }
    }
    return def;
}
unsafe extern "C" fn server_connect(
    mut server_addr: *mut c_char,
    mut errmsg: *mut *const c_char,
) -> uint64_t {
    if server_addr.is_null() {
        *errmsg = b"no address specified\0".as_ptr() as *const c_char;
        return 0 as uint64_t;
    }
    let mut on_data: CallbackReader = CallbackReader {
        cb: Callback {
            data: Callback_data {
                funcref: ::core::ptr::null_mut::<c_char>(),
            },
            type_0: kCallbackNone,
        },
        self_0: ::core::ptr::null_mut::<dict_T>(),
        buffer: GA_EMPTY_INIT_VALUE,
        eof: false,
        buffered: false_0 != 0,
        fwd_err: false_0 != 0,
        type_0: ::core::ptr::null::<c_char>(),
    };
    let mut error: *const c_char = ::core::ptr::null::<c_char>();
    let mut is_tcp: bool = socket_address_is_tcp(CStr::from_ptr(server_addr));
    let mut chan: uint64_t = channel_connect(
        is_tcp,
        server_addr,
        true_0 != 0,
        on_data,
        500 as c_int,
        &raw mut error,
    );
    if !error.is_null() {
        *errmsg = error;
        return 0 as uint64_t;
    }
    return chan;
}
unsafe extern "C" fn remote_request(
    mut params: *mut mparm_T,
    mut remote_args: c_int,
    mut server_addr: *mut c_char,
    mut argc: c_int,
    mut argv: *mut *mut c_char,
    mut ui_only: bool,
) {
    let mut is_ui: bool = strequal(
        *argv.offset(remote_args as isize),
        b"--remote-ui\0".as_ptr() as *const c_char,
    );
    if ui_only as c_int != 0 && !is_ui {
        return;
    }
    let mut connect_error: *const c_char = ::core::ptr::null::<c_char>();
    let mut chan: uint64_t = server_connect(server_addr, &raw mut connect_error);
    let mut rvobj: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    if is_ui {
        if chan == 0 {
            fprintf(
                stderr,
                b"Remote ui failed to start: %s\n\0".as_ptr() as *const c_char,
                connect_error,
            );
            os_exit(1 as c_int);
        } else if strequal(
            server_addr,
            os_getenv_noalloc(b"NVIM\0".as_ptr() as *const c_char),
        ) {
            fprintf(
                stderr,
                b"%s\0".as_ptr() as *const c_char,
                b"Cannot attach UI of :terminal child to its parent. \0".as_ptr() as *const c_char,
            );
            fprintf(
                stderr,
                b"%s\n\0".as_ptr() as *const c_char,
                b"(Unset $NVIM to skip this check)\0".as_ptr() as *const c_char,
            );
            os_exit(1 as c_int);
        }
        ui_client_channel_id.set(chan);
        return;
    }
    let mut args: Array = ARRAY_DICT_INIT;
    args.capacity = (argc - remote_args) as size_t;
    args.items = xrealloc(
        args.items as *mut c_void,
        ::core::mem::size_of::<Object>().wrapping_mul(args.capacity),
    ) as *mut Object;
    let mut t_argc: c_int = remote_args;
    while t_argc < argc {
        let c2rust_fresh1 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh1 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(*argv.offset(t_argc as isize)),
            },
        };
        t_argc += 1;
    }
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<c_char>(),
    };
    let mut a: Array = ARRAY_DICT_INIT;
    let mut a__items: [Object; 4] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 4];
    a.capacity = 4 as size_t;
    a.items = &raw mut a__items as *mut Object;
    let c2rust_fresh2 = a.size;
    a.size = a.size.wrapping_add(1);
    *a.items.offset(c2rust_fresh2 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: chan as c_int as Integer,
        },
    };
    let c2rust_fresh3 = a.size;
    a.size = a.size.wrapping_add(1);
    *a.items.offset(c2rust_fresh3 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstr_as_string(server_addr),
        },
    };
    let c2rust_fresh4 = a.size;
    a.size = a.size.wrapping_add(1);
    *a.items.offset(c2rust_fresh4 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstr_as_string(connect_error),
        },
    };
    let c2rust_fresh5 = a.size;
    a.size = a.size.wrapping_add(1);
    *a.items.offset(c2rust_fresh5 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed { array: args },
    };
    let mut s: String_0 = String_0 {
        data: b"return vim._cs_remote(...)\0".as_ptr() as *const c_char as *mut c_char,
        size: ::core::mem::size_of::<[c_char; 27]>().wrapping_sub(1 as size_t),
    };
    let mut o: Object = nlua_exec(
        s,
        ::core::ptr::null::<c_char>(),
        a,
        kRetObject,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    xfree(args.items as *mut c_void);
    args.capacity = 0 as size_t;
    args.size = args.capacity;
    args.items = ::core::ptr::null_mut::<Object>();
    if err.type_0 as c_int != kErrorTypeNone as c_int {
        fprintf(stderr, b"%s\n\0".as_ptr() as *const c_char, err.msg);
        os_exit(2 as c_int);
    }
    if o.type_0 as c_uint == kObjectTypeDict as c_int as c_uint {
        rvobj.data.dict = o.data.dict;
    } else {
        fprintf(
            stderr,
            b"vim._cs_remote returned unexpected value\n\0".as_ptr() as *const c_char,
        );
        os_exit(2 as c_int);
    }
    let mut should_exit: TriState = kNone;
    let mut tabbed: TriState = kNone;
    let mut i: size_t = 0 as size_t;
    while i < rvobj.data.dict.size {
        if strequal(
            (*rvobj.data.dict.items.offset(i as isize)).key.data,
            b"errmsg\0".as_ptr() as *const c_char,
        ) {
            if (*rvobj.data.dict.items.offset(i as isize)).value.type_0 as c_uint
                != kObjectTypeString as c_int as c_uint
            {
                fprintf(
                    stderr,
                    b"vim._cs_remote returned an unexpected type for 'errmsg'\n\0".as_ptr()
                        as *const c_char,
                );
                os_exit(2 as c_int);
            }
            fprintf(
                stderr,
                b"%s\n\0".as_ptr() as *const c_char,
                (*rvobj.data.dict.items.offset(i as isize))
                    .value
                    .data
                    .string
                    .data,
            );
            os_exit(2 as c_int);
        } else if strequal(
            (*rvobj.data.dict.items.offset(i as isize)).key.data,
            b"result\0".as_ptr() as *const c_char,
        ) {
            if (*rvobj.data.dict.items.offset(i as isize)).value.type_0 as c_uint
                != kObjectTypeString as c_int as c_uint
            {
                fprintf(
                    stderr,
                    b"vim._cs_remote returned an unexpected type for 'result'\n\0".as_ptr()
                        as *const c_char,
                );
                os_exit(2 as c_int);
            }
            printf(
                b"%s\0".as_ptr() as *const c_char,
                (*rvobj.data.dict.items.offset(i as isize))
                    .value
                    .data
                    .string
                    .data,
            );
        } else if strequal(
            (*rvobj.data.dict.items.offset(i as isize)).key.data,
            b"tabbed\0".as_ptr() as *const c_char,
        ) {
            if (*rvobj.data.dict.items.offset(i as isize)).value.type_0 as c_uint
                != kObjectTypeBoolean as c_int as c_uint
            {
                fprintf(
                    stderr,
                    b"vim._cs_remote returned an unexpected type for 'tabbed'\n\0".as_ptr()
                        as *const c_char,
                );
                os_exit(2 as c_int);
            }
            tabbed = (if (*rvobj.data.dict.items.offset(i as isize))
                .value
                .data
                .boolean as c_int
                != 0
            {
                kTrue as c_int
            } else {
                kFalse as c_int
            }) as TriState;
        } else if strequal(
            (*rvobj.data.dict.items.offset(i as isize)).key.data,
            b"should_exit\0".as_ptr() as *const c_char,
        ) {
            if (*rvobj.data.dict.items.offset(i as isize)).value.type_0 as c_uint
                != kObjectTypeBoolean as c_int as c_uint
            {
                fprintf(
                    stderr,
                    b"vim._cs_remote returned an unexpected type for 'should_exit'\n\0".as_ptr()
                        as *const c_char,
                );
                os_exit(2 as c_int);
            }
            should_exit = (if (*rvobj.data.dict.items.offset(i as isize))
                .value
                .data
                .boolean as c_int
                != 0
            {
                kTrue as c_int
            } else {
                kFalse as c_int
            }) as TriState;
        }
        i = i.wrapping_add(1);
    }
    if should_exit as c_int == kNone as c_int || tabbed as c_int == kNone as c_int {
        fprintf(
            stderr,
            b"vim._cs_remote didn't return a value for should_exit or tabbed, bailing\n\0".as_ptr()
                as *const c_char,
        );
        os_exit(2 as c_int);
    }
    api_free_object(o);
    if should_exit as c_int == kTrue as c_int {
        os_exit(0 as c_int);
    }
    if tabbed as c_int == kTrue as c_int {
        (*params).window_count = argc - remote_args - 1 as c_int;
        (*params).window_layout = WIN_TABS as c_int;
    }
}
unsafe extern "C" fn edit_stdin(mut parmp: *mut mparm_T) -> bool {
    let mut implicit: bool = !headless_mode.get()
        && !(embedded_mode.get() as c_int != 0 && stdin_fd.get() <= 0 as c_int)
        && (!exmode_active.get() || (*parmp).input_istext as c_int != 0)
        && !stdin_isatty.get()
        && (*parmp).edit_type <= EDIT_STDIN as c_int
        && (*parmp).scriptin.is_null();
    return (*parmp).had_stdin_file as c_int != 0 || implicit as c_int != 0;
}
unsafe extern "C" fn command_line_scan(mut parmp: *mut mparm_T) {
    let mut argc: c_int = (*parmp).argc;
    let mut argv: *mut *mut c_char = (*parmp).argv;
    let mut argv_idx: c_int = 0;
    let mut had_minmin: bool = false_0 != 0;
    let mut want_argument: bool = false;
    let mut n: c_int = 0;
    argc -= 1;
    argv = argv.offset(1);
    argv_idx = 1 as c_int;
    while argc > 0 as c_int {
        if *(*argv.offset(0 as c_int as isize)).offset(0 as c_int as isize) as c_int == '+' as c_int
            && !had_minmin
        {
            if (*parmp).n_commands >= MAX_ARG_CMDS {
                mainerr(
                    err_extra_cmd.get(),
                    ::core::ptr::null::<c_char>(),
                    ::core::ptr::null::<c_char>(),
                );
            }
            argv_idx = -1 as c_int;
            if *(*argv.offset(0 as c_int as isize)).offset(1 as c_int as isize) as c_int == NUL {
                let c2rust_fresh6 = (*parmp).n_commands;
                (*parmp).n_commands = (*parmp).n_commands + 1;
                let c2rust_lvalue_ptr = &raw mut (*parmp).commands[c2rust_fresh6 as usize];
                *c2rust_lvalue_ptr = b"$\0".as_ptr() as *const c_char as *mut c_char;
            } else {
                let c2rust_fresh7 = (*parmp).n_commands;
                (*parmp).n_commands = (*parmp).n_commands + 1;
                let c2rust_lvalue_ptr_0 = &raw mut (*parmp).commands[c2rust_fresh7 as usize];
                *c2rust_lvalue_ptr_0 =
                    (*argv.offset(0 as c_int as isize)).offset(1 as c_int as isize);
            }
        } else if *(*argv.offset(0 as c_int as isize)).offset(0 as c_int as isize) as c_int
            == '-' as c_int
            && !had_minmin
        {
            want_argument = false_0 != 0;
            let c2rust_fresh8 = argv_idx;
            argv_idx = argv_idx + 1;
            let mut c: c_char = *(*argv.offset(0 as c_int as isize)).offset(c2rust_fresh8 as isize);
            's_747: {
                'c_49604: {
                    match c as c_int {
                        NUL => {
                            if exmode_active.get() {
                                silent_mode.set(true_0 != 0);
                                (*parmp).no_swap_file = true_0;
                            } else {
                                if (*parmp).edit_type > EDIT_STDIN as c_int {
                                    mainerr(
                                        err_too_many_args.get(),
                                        *argv.offset(0 as c_int as isize),
                                        ::core::ptr::null::<c_char>(),
                                    );
                                }
                                (*parmp).had_stdin_file = true_0 != 0;
                                (*parmp).edit_type = EDIT_STDIN as c_int;
                            }
                            argv_idx = -1 as c_int;
                            break 's_747;
                        }
                        45 => {
                            if strcasecmp(
                                (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                b"help\0".as_ptr() as *const c_char as *mut c_char,
                            ) == 0 as c_int
                            {
                                usage();
                                os_exit(0 as c_int);
                            } else if strcasecmp(
                                (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                b"version\0".as_ptr() as *const c_char as *mut c_char,
                            ) == 0 as c_int
                            {
                                version();
                                os_exit(0 as c_int);
                            } else if strcasecmp(
                                (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                b"api-info\0".as_ptr() as *const c_char as *mut c_char,
                            ) == 0 as c_int
                            {
                                let mut data: String_0 = api_metadata_raw();
                                let written_bytes: ptrdiff_t =
                                    os_write(STDOUT_FILENO, data.data, data.size, false_0 != 0);
                                if written_bytes < 0 as ptrdiff_t {
                                    semsg(
                                        gettext(b"E5420: Failed to write to file: %s\0".as_ptr()
                                            as *const c_char),
                                        uv_strerror(written_bytes as c_int),
                                    );
                                }
                                os_exit(0 as c_int);
                            } else if strcasecmp(
                                (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                b"headless\0".as_ptr() as *const c_char as *mut c_char,
                            ) == 0 as c_int
                            {
                                headless_mode.set(true_0 != 0);
                            } else if strcasecmp(
                                (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                b"embed\0".as_ptr() as *const c_char as *mut c_char,
                            ) == 0 as c_int
                            {
                                embedded_mode.set(true_0 != 0);
                            } else if strncasecmp(
                                (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                b"listen\0".as_ptr() as *const c_char as *mut c_char,
                                6 as c_int as size_t,
                            ) == 0 as c_int
                            {
                                want_argument = true_0 != 0;
                                argv_idx += 6 as c_int;
                            } else if strncasecmp(
                                (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                b"literal\0".as_ptr() as *const c_char as *mut c_char,
                                7 as c_int as size_t,
                            ) != 0 as c_int
                            {
                                if strncasecmp(
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                    b"remote\0".as_ptr() as *const c_char as *mut c_char,
                                    6 as c_int as size_t,
                                ) == 0 as c_int
                                {
                                    (*parmp).remote = (*parmp).argc - argc;
                                } else if strncasecmp(
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                    b"server\0".as_ptr() as *const c_char as *mut c_char,
                                    6 as c_int as size_t,
                                ) == 0 as c_int
                                {
                                    want_argument = true_0 != 0;
                                    argv_idx += 6 as c_int;
                                } else if strncasecmp(
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                    b"noplugin\0".as_ptr() as *const c_char as *mut c_char,
                                    8 as c_int as size_t,
                                ) == 0 as c_int
                                {
                                    p_lpl.set(false_0);
                                } else if strncasecmp(
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                    b"cmd\0".as_ptr() as *const c_char as *mut c_char,
                                    3 as c_int as size_t,
                                ) == 0 as c_int
                                {
                                    want_argument = true_0 != 0;
                                    argv_idx += 3 as c_int;
                                } else if strncasecmp(
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                    b"startuptime\0".as_ptr() as *const c_char as *mut c_char,
                                    11 as c_int as size_t,
                                ) == 0 as c_int
                                {
                                    want_argument = true_0 != 0;
                                    argv_idx += 11 as c_int;
                                } else if strncasecmp(
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                    b"clean\0".as_ptr() as *const c_char as *mut c_char,
                                    5 as c_int as size_t,
                                ) == 0 as c_int
                                {
                                    (*parmp).use_vimrc =
                                        b"NONE\0".as_ptr() as *const c_char as *mut c_char;
                                    (*parmp).clean = true_0 != 0;
                                    set_option_value_give_err(
                                        kOptShadafile,
                                        OptVal {
                                            type_0: kOptValTypeString,
                                            data: OptValData {
                                                string: String_0 {
                                                    data: b"NONE\0".as_ptr() as *const c_char
                                                        as *mut c_char,
                                                    size: ::core::mem::size_of::<[c_char; 5]>()
                                                        .wrapping_sub(1 as size_t),
                                                },
                                            },
                                        },
                                        0 as c_int,
                                    );
                                } else if strncasecmp(
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize),
                                    b"luamod-dev\0".as_ptr() as *const c_char as *mut c_char,
                                    9 as c_int as size_t,
                                ) == 0 as c_int
                                {
                                    nlua_disable_preload.set(true_0 != 0);
                                } else {
                                    if *(*argv.offset(0 as c_int as isize))
                                        .offset(argv_idx as isize)
                                        != 0
                                    {
                                        mainerr(
                                            err_opt_unknown.get(),
                                            *argv.offset(0 as c_int as isize),
                                            ::core::ptr::null::<c_char>(),
                                        );
                                    }
                                    had_minmin = true_0 != 0;
                                }
                            }
                            if !want_argument {
                                argv_idx = -1 as c_int;
                            }
                            break 's_747;
                        }
                        65 => {
                            set_option_value_give_err(
                                kOptArabic,
                                OptVal {
                                    type_0: kOptValTypeBoolean,
                                    data: OptValData { boolean: kTrue },
                                },
                                0 as c_int,
                            );
                            break 's_747;
                        }
                        98 => {
                            set_options_bin((*curbuf.get()).b_p_bin, 1 as c_int, 0 as c_int);
                            (*curbuf.get()).b_p_bin = 1 as c_int;
                            break 's_747;
                        }
                        68 => {
                            (*parmp).use_debug_break_level = 9999 as c_int;
                            break 's_747;
                        }
                        100 => {
                            (*parmp).diff_mode = true_0;
                            break 's_747;
                        }
                        101 => {
                            exmode_active.set(true_0 != 0);
                            break 's_747;
                        }
                        69 => {
                            exmode_active.set(true_0 != 0);
                            (*parmp).input_istext = true_0 != 0;
                            break 's_747;
                        }
                        63 | 104 => {
                            usage();
                            os_exit(0 as c_int);
                        }
                        72 => {
                            set_option_value_give_err(
                                kOptKeymap,
                                OptVal {
                                    type_0: kOptValTypeString,
                                    data: OptValData {
                                        string: String_0 {
                                            data: b"hebrew\0".as_ptr() as *const c_char
                                                as *mut c_char,
                                            size: ::core::mem::size_of::<[c_char; 7]>()
                                                .wrapping_sub(1 as size_t),
                                        },
                                    },
                                },
                                0 as c_int,
                            );
                            set_option_value_give_err(
                                kOptRightleft,
                                OptVal {
                                    type_0: kOptValTypeBoolean,
                                    data: OptValData { boolean: kTrue },
                                },
                                0 as c_int,
                            );
                            break 's_747;
                        }
                        77 => {
                            reset_modifiable();
                        }
                        109 => {}
                        102 | 78 | 88 => {
                            break 's_747;
                        }
                        110 => {
                            (*parmp).no_swap_file = true_0;
                            break 's_747;
                        }
                        112 => {
                            (*parmp).window_count = get_number_arg(
                                *argv.offset(0 as c_int as isize),
                                &raw mut argv_idx,
                                0 as c_int,
                            );
                            (*parmp).window_layout = WIN_TABS as c_int;
                            break 's_747;
                        }
                        111 => {
                            (*parmp).window_count = get_number_arg(
                                *argv.offset(0 as c_int as isize),
                                &raw mut argv_idx,
                                0 as c_int,
                            );
                            (*parmp).window_layout = WIN_HOR as c_int;
                            break 's_747;
                        }
                        79 => {
                            (*parmp).window_count = get_number_arg(
                                *argv.offset(0 as c_int as isize),
                                &raw mut argv_idx,
                                0 as c_int,
                            );
                            (*parmp).window_layout = WIN_VER as c_int;
                            break 's_747;
                        }
                        113 => {
                            if (*parmp).edit_type != EDIT_NONE as c_int {
                                mainerr(
                                    err_too_many_args.get(),
                                    *argv.offset(0 as c_int as isize),
                                    ::core::ptr::null::<c_char>(),
                                );
                            }
                            (*parmp).edit_type = EDIT_QF as c_int;
                            if *(*argv.offset(0 as c_int as isize)).offset(argv_idx as isize) != 0 {
                                (*parmp).use_ef =
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize);
                                argv_idx = -1 as c_int;
                            } else if argc > 1 as c_int {
                                want_argument = true_0 != 0;
                            }
                            break 's_747;
                        }
                        82 => {
                            readonlymode.set(true_0 != 0);
                            (*curbuf.get()).b_p_ro = true_0;
                            p_uc.set(10000 as OptInt);
                            break 's_747;
                        }
                        114 | 76 => {
                            recoverymode.set(true);
                            break 's_747;
                        }
                        115 => {
                            if exmode_active.get() {
                                silent_mode.set(true_0 != 0);
                                (*parmp).no_swap_file = true_0;
                                if (*p_shadafile.ptr()).is_null()
                                    || *p_shadafile.get() as c_int == NUL
                                {
                                    set_option_value_give_err(
                                        kOptShadafile,
                                        OptVal {
                                            type_0: kOptValTypeString,
                                            data: OptValData {
                                                string: String_0 {
                                                    data: b"NONE\0".as_ptr() as *const c_char
                                                        as *mut c_char,
                                                    size: ::core::mem::size_of::<[c_char; 5]>()
                                                        .wrapping_sub(1 as size_t),
                                                },
                                            },
                                        },
                                        0 as c_int,
                                    );
                                }
                            } else {
                                want_argument = true_0 != 0;
                            }
                            break 's_747;
                        }
                        116 => {
                            if (*parmp).edit_type != EDIT_NONE as c_int {
                                mainerr(
                                    err_too_many_args.get(),
                                    *argv.offset(0 as c_int as isize),
                                    ::core::ptr::null::<c_char>(),
                                );
                            }
                            (*parmp).edit_type = EDIT_TAG as c_int;
                            if *(*argv.offset(0 as c_int as isize)).offset(argv_idx as isize) != 0 {
                                (*parmp).tagname =
                                    (*argv.offset(0 as c_int as isize)).offset(argv_idx as isize);
                                argv_idx = -1 as c_int;
                            } else {
                                want_argument = true_0 != 0;
                            }
                            break 's_747;
                        }
                        118 => {
                            version();
                            os_exit(0 as c_int);
                        }
                        86 => {
                            p_verbose.set(get_number_arg(
                                *argv.offset(0 as c_int as isize),
                                &raw mut argv_idx,
                                10 as c_int,
                            ) as OptInt);
                            if *(*argv.offset(0 as c_int as isize)).offset(argv_idx as isize)
                                as c_int
                                != NUL
                            {
                                set_option_value_give_err(
                                    kOptVerbosefile,
                                    OptVal {
                                        type_0: kOptValTypeString,
                                        data: OptValData {
                                            string: cstr_as_string(
                                                (*argv.offset(0 as c_int as isize))
                                                    .offset(argv_idx as isize),
                                            ),
                                        },
                                    },
                                    0 as c_int,
                                );
                                argv_idx = strlen(*argv.offset(0 as c_int as isize)) as c_int;
                            }
                            break 's_747;
                        }
                        119 => {
                            if ascii_isdigit(
                                *(*argv.offset(0 as c_int as isize)).offset(argv_idx as isize)
                                    as c_int,
                            ) {
                                n = get_number_arg(
                                    *argv.offset(0 as c_int as isize),
                                    &raw mut argv_idx,
                                    10 as c_int,
                                );
                                set_option_value_give_err(
                                    kOptWindow,
                                    OptVal {
                                        type_0: kOptValTypeNumber,
                                        data: OptValData {
                                            number: n as OptInt,
                                        },
                                    },
                                    0 as c_int,
                                );
                                break 's_747;
                            } else {
                                want_argument = true_0 != 0;
                                break 's_747;
                            }
                        }
                        99 => {
                            if *(*argv.offset(0 as c_int as isize)).offset(argv_idx as isize)
                                as c_int
                                != NUL
                            {
                                if (*parmp).n_commands >= MAX_ARG_CMDS {
                                    mainerr(
                                        err_extra_cmd.get(),
                                        ::core::ptr::null::<c_char>(),
                                        ::core::ptr::null::<c_char>(),
                                    );
                                }
                                let c2rust_fresh9 = (*parmp).n_commands;
                                (*parmp).n_commands = (*parmp).n_commands + 1;
                                let c2rust_lvalue_ptr_1 =
                                    &raw mut (*parmp).commands[c2rust_fresh9 as usize];
                                *c2rust_lvalue_ptr_1 = (*argv).offset(argv_idx as isize);
                                argv_idx = -1 as c_int;
                                break 's_747;
                            } else {
                                break 'c_49604;
                            }
                        }
                        83 | 105 | 108 | 117 | 85 | 87 => {
                            break 'c_49604;
                        }
                        _ => {
                            mainerr(
                                err_opt_unknown.get(),
                                *argv.offset(0 as c_int as isize),
                                ::core::ptr::null::<c_char>(),
                            );
                        }
                    }
                    p_write.set(false_0);
                    break 's_747;
                }
                want_argument = true_0 != 0;
            }
            if want_argument {
                if *(*argv.offset(0 as c_int as isize)).offset(argv_idx as isize) as c_int != NUL {
                    mainerr(
                        err_opt_garbage.get(),
                        *argv.offset(0 as c_int as isize),
                        ::core::ptr::null::<c_char>(),
                    );
                }
                argc -= 1;
                if argc < 1 as c_int && c as c_int != 'S' as c_int {
                    mainerr(
                        err_arg_missing.get(),
                        *argv.offset(0 as c_int as isize),
                        ::core::ptr::null::<c_char>(),
                    );
                }
                argv = argv.offset(1);
                argv_idx = -1 as c_int;
                's_1076: {
                    '_scripterror: {
                        's_1075: {
                            match c as c_int {
                                99 | 83 => {
                                    if (*parmp).n_commands >= MAX_ARG_CMDS {
                                        mainerr(
                                            err_extra_cmd.get(),
                                            ::core::ptr::null::<c_char>(),
                                            ::core::ptr::null::<c_char>(),
                                        );
                                    }
                                    if c as c_int == 'S' as c_int {
                                        let mut a: *mut c_char = ::core::ptr::null_mut::<c_char>();
                                        if argc < 1 as c_int {
                                            a = SESSION_FILE.as_ptr() as *mut c_char;
                                        } else if *(*argv.offset(0 as c_int as isize))
                                            .offset(0 as c_int as isize)
                                            as c_int
                                            == '-' as c_int
                                        {
                                            a = SESSION_FILE.as_ptr() as *mut c_char;
                                            argc += 1;
                                            argv = argv.offset(-1);
                                        } else {
                                            a = *argv.offset(0 as c_int as isize);
                                        }
                                        let mut s_size: size_t =
                                            strlen(a).wrapping_add(9 as size_t);
                                        let mut s: *mut c_char = xmalloc(s_size) as *mut c_char;
                                        snprintf(
                                            s,
                                            s_size,
                                            b"so %s\0".as_ptr() as *const c_char,
                                            a,
                                        );
                                        (*parmp).cmds_tofree[(*parmp).n_commands as usize] =
                                            true_0 as c_char;
                                        let c2rust_fresh10 = (*parmp).n_commands;
                                        (*parmp).n_commands = (*parmp).n_commands + 1;
                                        let c2rust_lvalue_ptr_2 =
                                            &raw mut (*parmp).commands[c2rust_fresh10 as usize];
                                        *c2rust_lvalue_ptr_2 = s;
                                    } else {
                                        let c2rust_fresh11 = (*parmp).n_commands;
                                        (*parmp).n_commands = (*parmp).n_commands + 1;
                                        let c2rust_lvalue_ptr_3 =
                                            &raw mut (*parmp).commands[c2rust_fresh11 as usize];
                                        *c2rust_lvalue_ptr_3 = *argv.offset(0 as c_int as isize);
                                    }
                                    break 's_1075;
                                }
                                45 => {
                                    if strequal(
                                        *argv.offset(-1 as c_int as isize),
                                        b"--cmd\0".as_ptr() as *const c_char,
                                    ) {
                                        if (*parmp).n_pre_commands >= MAX_ARG_CMDS {
                                            mainerr(
                                                err_extra_cmd.get(),
                                                ::core::ptr::null::<c_char>(),
                                                ::core::ptr::null::<c_char>(),
                                            );
                                        }
                                        let c2rust_fresh12 = (*parmp).n_pre_commands;
                                        (*parmp).n_pre_commands = (*parmp).n_pre_commands + 1;
                                        let c2rust_lvalue_ptr_4 =
                                            &raw mut (*parmp).pre_commands[c2rust_fresh12 as usize];
                                        *c2rust_lvalue_ptr_4 = *argv.offset(0 as c_int as isize);
                                    } else if strequal(
                                        *argv.offset(-1 as c_int as isize),
                                        b"--listen\0".as_ptr() as *const c_char,
                                    ) {
                                        (*parmp).listen_addr = *argv.offset(0 as c_int as isize);
                                    } else if strequal(
                                        *argv.offset(-1 as c_int as isize),
                                        b"--server\0".as_ptr() as *const c_char,
                                    ) {
                                        (*parmp).server_addr = *argv.offset(0 as c_int as isize);
                                    }
                                    break 's_1075;
                                }
                                113 => {
                                    (*parmp).use_ef = *argv.offset(0 as c_int as isize);
                                    break 's_1075;
                                }
                                105 => {
                                    set_option_value_give_err(
                                        kOptShadafile,
                                        OptVal {
                                            type_0: kOptValTypeString,
                                            data: OptValData {
                                                string: cstr_as_string(
                                                    *argv.offset(0 as c_int as isize),
                                                ),
                                            },
                                        },
                                        0 as c_int,
                                    );
                                    break 's_1075;
                                }
                                108 => {
                                    headless_mode.set(true_0 != 0);
                                    silent_mode.set(true_0 != 0);
                                    p_verbose.set(1 as OptInt);
                                    (*parmp).no_swap_file = true_0;
                                    (*parmp).use_vimrc = (if !(*parmp).use_vimrc.is_null() {
                                        (*parmp).use_vimrc as *const c_char
                                    } else {
                                        b"NONE\0".as_ptr() as *const c_char
                                    })
                                        as *mut c_char;
                                    if (*p_shadafile.ptr()).is_null()
                                        || *p_shadafile.get() as c_int == NUL
                                    {
                                        set_option_value_give_err(
                                            kOptShadafile,
                                            OptVal {
                                                type_0: kOptValTypeString,
                                                data: OptValData {
                                                    string: String_0 {
                                                        data: b"NONE\0".as_ptr() as *const c_char
                                                            as *mut c_char,
                                                        size: ::core::mem::size_of::<[c_char; 5]>()
                                                            .wrapping_sub(1 as size_t),
                                                    },
                                                },
                                            },
                                            0 as c_int,
                                        );
                                    }
                                    (*parmp).luaf = *argv.offset(0 as c_int as isize);
                                    argc -= 1;
                                    if argc >= 0 as c_int {
                                        (*parmp).lua_arg0 = (*parmp).argc - argc;
                                        argc = 0 as c_int;
                                    }
                                    break 's_1075;
                                }
                                115 => {
                                    if !(*parmp).scriptin.is_null() {
                                        break '_scripterror;
                                    } else {
                                        (*parmp).scriptin = *argv.offset(0 as c_int as isize);
                                        break 's_1075;
                                    }
                                }
                                116 => {
                                    (*parmp).tagname = *argv.offset(0 as c_int as isize);
                                    break 's_1075;
                                }
                                117 => {
                                    (*parmp).use_vimrc = *argv.offset(0 as c_int as isize);
                                    break 's_1075;
                                }
                                119 => {
                                    if ascii_isdigit(**argv.offset(0 as c_int as isize) as c_int) {
                                        argv_idx = 0 as c_int;
                                        n = get_number_arg(
                                            *argv.offset(0 as c_int as isize),
                                            &raw mut argv_idx,
                                            10 as c_int,
                                        );
                                        set_option_value_give_err(
                                            kOptWindow,
                                            OptVal {
                                                type_0: kOptValTypeNumber,
                                                data: OptValData {
                                                    number: n as OptInt,
                                                },
                                            },
                                            0 as c_int,
                                        );
                                        argv_idx = -1 as c_int;
                                        break 's_1075;
                                    }
                                }
                                87 => {}
                                85 | _ => {
                                    break 's_1075;
                                }
                            }
                            if !(*parmp).scriptout.is_null() {
                                break '_scripterror;
                            } else {
                                (*parmp).scriptout = *argv.offset(0 as c_int as isize);
                                (*parmp).scriptout_append = c as c_int == 'w' as c_int;
                            }
                        }
                        break 's_1076;
                    }
                    vim_snprintf(
                        IObuff.ptr() as *mut c_char,
                        IOSIZE as size_t,
                        gettext(b"Attempt to open script file again: \"%s %s\"\n\0".as_ptr()
                            as *const c_char),
                        *argv.offset(-1 as c_int as isize),
                        *argv.offset(0 as c_int as isize),
                    );
                    fprintf(
                        stderr,
                        b"%s\0".as_ptr() as *const c_char,
                        IObuff.ptr() as *mut c_char,
                    );
                    os_exit(2 as c_int);
                }
            }
        } else {
            argv_idx = -1 as c_int;
            if (*parmp).edit_type > EDIT_STDIN as c_int {
                mainerr(
                    err_too_many_args.get(),
                    *argv.offset(0 as c_int as isize),
                    ::core::ptr::null::<c_char>(),
                );
            }
            (*parmp).edit_type = EDIT_FILE as c_int;
            ga_grow(&raw mut (*global_alist.ptr()).al_ga, 1 as c_int);
            let mut p: *mut c_char = xstrdup(*argv.offset(0 as c_int as isize));
            if (*parmp).diff_mode != 0
                && os_isdir(p) as c_int != 0
                && (*global_alist.ptr()).al_ga.ga_len > 0 as c_int
                && !os_isdir(alist_name(
                    ((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T)
                        .offset(0 as c_int as isize),
                ))
            {
                let mut r: *mut c_char = concat_fnames(
                    p,
                    path_tail(alist_name(
                        ((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T)
                            .offset(0 as c_int as isize),
                    )),
                    true_0 != 0,
                );
                xfree(p as *mut c_void);
                p = r;
            }
            let mut alist_fnum_flag: c_int = if edit_stdin(parmp) as c_int != 0 {
                1 as c_int
            } else {
                2 as c_int
            };
            alist_add(global_alist.ptr(), p, alist_fnum_flag);
        }
        if argv_idx <= 0 as c_int
            || *(*argv.offset(0 as c_int as isize)).offset(argv_idx as isize) as c_int == NUL
        {
            argc -= 1;
            argv = argv.offset(1);
            argv_idx = 1 as c_int;
        }
    }
    if embedded_mode.get() as c_int != 0
        && (silent_mode.get() as c_int != 0 || !(*parmp).luaf.is_null())
    {
        mainerr(
            gettext(b"--embed conflicts with -es/-Es/-l\0".as_ptr() as *const c_char),
            ::core::ptr::null::<c_char>(),
            ::core::ptr::null::<c_char>(),
        );
    }
    if (*parmp).n_commands > 0 as c_int {
        let swcmd_len: size_t =
            strlen((*parmp).commands[0 as c_int as usize]).wrapping_add(2 as size_t);
        let swcmd: *mut c_char = xmalloc(swcmd_len.wrapping_add(1 as size_t)) as *mut c_char;
        snprintf(
            swcmd,
            swcmd_len.wrapping_add(1 as size_t),
            b":%s\r\0".as_ptr() as *const c_char,
            (*parmp).commands[0 as c_int as usize],
        );
        set_vim_var_string(VV_SWAPCOMMAND, swcmd, swcmd_len as ptrdiff_t);
        xfree(swcmd as *mut c_void);
    }
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"parsing arguments\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
unsafe extern "C" fn set_argf_var() {
    let mut list: *mut list_T = tv_list_alloc(kListLenMayKnow as c_int as ptrdiff_t);
    let mut i: c_int = 0 as c_int;
    while i < (*global_alist.ptr()).al_ga.ga_len {
        let mut fname: *mut c_char =
            alist_name(((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T).offset(i as isize));
        if !fname.is_null() {
            vim_FullName(
                fname,
                NameBuff.ptr() as *mut c_char,
                ::core::mem::size_of::<[c_char; 4096]>(),
                false_0 != 0,
            );
            tv_list_append_string(list, NameBuff.ptr() as *mut c_char, -1 as ssize_t);
        }
        i += 1;
    }
    tv_list_set_lock(list, VAR_FIXED);
    set_vim_var_list(VV_ARGF, list);
}
unsafe extern "C" fn init_params(
    mut paramp: *mut mparm_T,
    mut argc: c_int,
    mut argv: *mut *mut c_char,
) {
    memset(
        paramp as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<mparm_T>(),
    );
    (*paramp).argc = argc;
    (*paramp).argv = argv;
    (*paramp).use_debug_break_level = -1 as c_int;
    (*paramp).window_count = -1 as c_int;
    (*paramp).listen_addr = ::core::ptr::null_mut::<c_char>();
    (*paramp).server_addr = ::core::ptr::null_mut::<c_char>();
    (*paramp).remote = 0 as c_int;
    (*paramp).luaf = ::core::ptr::null_mut::<c_char>();
    (*paramp).lua_arg0 = -1 as c_int;
}
unsafe extern "C" fn init_startuptime(mut paramp: *mut mparm_T) {
    let mut is_embed: bool = false_0 != 0;
    let mut i: c_int = 1 as c_int;
    while i < (*paramp).argc - 1 as c_int {
        if strcasecmp(
            *(*paramp).argv.offset(i as isize),
            b"--embed\0".as_ptr() as *const c_char as *mut c_char,
        ) == 0 as c_int
        {
            is_embed = true_0 != 0;
            break;
        } else {
            i += 1;
        }
    }
    let mut i_0: c_int = 1 as c_int;
    while i_0 < (*paramp).argc - 1 as c_int {
        if strcasecmp(
            *(*paramp).argv.offset(i_0 as isize),
            b"--startuptime\0".as_ptr() as *const c_char as *mut c_char,
        ) == 0 as c_int
        {
            time_init(
                *(*paramp).argv.offset((i_0 + 1 as c_int) as isize),
                if is_embed as c_int != 0 {
                    b"Embedded\0".as_ptr() as *const c_char
                } else {
                    b"Primary (or UI client)\0".as_ptr() as *const c_char
                },
            );
            time_start(b"--- NVIM STARTING ---\0".as_ptr() as *const c_char);
            break;
        } else {
            i_0 += 1;
        }
    }
}
unsafe extern "C" fn check_and_set_isatty(mut _paramp: *mut mparm_T) {
    stdin_isatty.set(os_isatty(STDIN_FILENO));
    stdout_isatty.set(os_isatty(STDOUT_FILENO));
    stderr_isatty.set(os_isatty(STDERR_FILENO));
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"window checked\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
unsafe extern "C" fn init_path(mut exename: *const c_char) {
    let mut exepath: [c_char; 4096] = [0 as c_char; 4096];
    let mut exepathlen: size_t = MAXPATHL as size_t;
    if os_exepath(&raw mut exepath as *mut c_char, &raw mut exepathlen) != 0 as c_int {
        path_guess_exepath(
            exename,
            &raw mut exepath as *mut c_char,
            ::core::mem::size_of::<[c_char; 4096]>(),
        );
    }
    set_vim_var_string(
        VV_PROGPATH,
        &raw mut exepath as *mut c_char,
        -1 as ptrdiff_t,
    );
    set_vim_var_string(VV_PROGNAME, path_tail(exename), -1 as ptrdiff_t);
}
unsafe extern "C" fn get_fname(mut _parmp: *mut mparm_T) -> *mut c_char {
    return alist_name(
        ((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T).offset(0 as c_int as isize),
    );
}
unsafe extern "C" fn set_window_layout(mut paramp: *mut mparm_T) {
    if (*paramp).diff_mode != 0 && (*paramp).window_layout == 0 as c_int {
        if diffopt_horizontal() {
            (*paramp).window_layout = WIN_HOR as c_int;
        } else {
            (*paramp).window_layout = WIN_VER as c_int;
        }
    }
}
unsafe extern "C" fn handle_quickfix(mut paramp: *mut mparm_T) {
    if (*paramp).edit_type == EDIT_QF as c_int {
        if !(*paramp).use_ef.is_null() {
            set_option_direct(
                kOptErrorfile,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string((*paramp).use_ef),
                    },
                },
                0 as c_int,
                SID_CARG,
            );
        }
        vim_snprintf(
            IObuff.ptr() as *mut c_char,
            IOSIZE as size_t,
            b"cfile %s\0".as_ptr() as *const c_char,
            p_ef.get(),
        );
        if qf_init(
            ::core::ptr::null_mut::<win_T>(),
            p_ef.get(),
            p_efm.get(),
            true_0,
            IObuff.ptr() as *mut c_char,
            p_menc.get(),
        ) < 0 as c_int
        {
            msg_putchar('\n' as c_int);
            os_exit(3 as c_int);
        }
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"reading errorfile\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
}
unsafe extern "C" fn handle_tag(mut tagname: *mut c_char) {
    if !tagname.is_null() {
        swap_exists_did_quit.set(false_0 != 0);
        vim_snprintf(
            IObuff.ptr() as *mut c_char,
            IOSIZE as size_t,
            b"ta %s\0".as_ptr() as *const c_char,
            tagname,
        );
        do_cmdline_cmd(IObuff.ptr() as *mut c_char);
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"jumping to tag\0".as_ptr() as *const c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
        if swap_exists_did_quit.get() {
            ui_call_error_exit(1 as Integer);
            getout(1 as c_int);
        }
    }
}
unsafe extern "C" fn read_stdin() {
    swap_exists_action.set(SEA_DIALOG);
    no_wait_return.set(true_0);
    let mut save_msg_didany: bool = msg_didany.get();
    if !(*curbuf.get()).b_ffname.is_null() {
        let mut stdin_buf: *mut buf_T = buflist_new(
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            0 as linenr_T,
            BLN_LISTED as c_int,
        );
        if stdin_buf.is_null() {
            semsg(b"Failed to create buffer for stdin\0".as_ptr() as *const c_char);
            return;
        }
        let mut initial_buf_handle: handle_T = (*curbuf.get()).handle;
        set_curbuf(stdin_buf, 0 as c_int, false_0 != 0);
        readfile(
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            0 as linenr_T,
            0 as linenr_T,
            MAXLNUM as c_int as linenr_T,
            ::core::ptr::null_mut::<exarg_T>(),
            READ_NEW as c_int + READ_STDIN as c_int,
            true_0 != 0,
        );
        let mut stdin_buf_handle: handle_T = (*stdin_buf).handle;
        let mut stdin_buf_empty: bool = buf_is_empty(curbuf.get());
        let mut buf: [c_char; 100] = [0; 100];
        vim_snprintf(
            &raw mut buf as *mut c_char,
            ::core::mem::size_of::<[c_char; 100]>(),
            b"silent! buffer %d\0".as_ptr() as *const c_char,
            initial_buf_handle,
        );
        do_cmdline_cmd(&raw mut buf as *mut c_char);
        if stdin_buf_empty {
            vim_snprintf(
                &raw mut buf as *mut c_char,
                ::core::mem::size_of::<[c_char; 100]>(),
                b"silent! bwipeout! %d\0".as_ptr() as *const c_char,
                stdin_buf_handle,
            );
            do_cmdline_cmd(&raw mut buf as *mut c_char);
        }
    } else {
        set_buflisted(true_0);
        open_buffer(true_0 != 0, ::core::ptr::null_mut::<exarg_T>(), 0 as c_int);
        if buf_is_empty(curbuf.get()) as c_int != 0 && !(*curbuf.get()).b_next.is_null() {
            do_cmdline_cmd(b"silent! bnext\0".as_ptr() as *const c_char);
            do_cmdline_cmd(b"silent! bwipeout 1\0".as_ptr() as *const c_char);
        }
    }
    no_wait_return.set(false_0);
    msg_didany.set(save_msg_didany);
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"reading stdin\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    check_swap_exists_action();
}
unsafe extern "C" fn create_windows(mut parmp: *mut mparm_T) {
    if (*parmp).window_count == -1 as c_int {
        (*parmp).window_count = 1 as c_int;
    }
    if (*parmp).window_count == 0 as c_int {
        (*parmp).window_count = (*global_alist.ptr()).al_ga.ga_len;
    }
    if (*parmp).window_count > 1 as c_int {
        if (*parmp).window_layout == 0 as c_int {
            (*parmp).window_layout = WIN_HOR as c_int;
        }
        if (*parmp).window_layout == WIN_TABS as c_int {
            (*parmp).window_count = make_tabpages((*parmp).window_count);
            if !(*time_fd.ptr()).is_null() {
                time_msg(
                    b"making tab pages\0".as_ptr() as *const c_char,
                    ::core::ptr::null::<proftime_T>(),
                );
            }
        } else if (*firstwin.get()).w_next.is_null()
            || (*(*firstwin.get()).w_next).w_floating as c_int != 0
        {
            (*parmp).window_count = make_windows(
                (*parmp).window_count,
                (*parmp).window_layout == WIN_VER as c_int,
            );
            if !(*time_fd.ptr()).is_null() {
                time_msg(
                    b"making windows\0".as_ptr() as *const c_char,
                    ::core::ptr::null::<proftime_T>(),
                );
            }
        } else {
            (*parmp).window_count = win_count();
        }
    } else {
        (*parmp).window_count = 1 as c_int;
    }
    if recoverymode.get() {
        msg_scroll.set(true_0);
        ml_recover(true_0 != 0);
        if (*curbuf.get()).b_ml.ml_mfp.is_null() {
            getout(1 as c_int);
        }
        do_modelines(0 as c_int);
    } else {
        let mut done: c_int = 0 as c_int;
        (*autocmd_no_enter.ptr()) += 1;
        (*autocmd_no_leave.ptr()) += 1;
        let mut dorewind: bool = true_0 != 0;
        loop {
            let c2rust_fresh0 = done;
            done = done + 1;
            if c2rust_fresh0 >= 1000 as c_int {
                break;
            }
            if dorewind {
                if (*parmp).window_layout == WIN_TABS as c_int {
                    goto_tabpage(1 as c_int);
                } else {
                    curwin.set(firstwin.get());
                }
            } else if (*parmp).window_layout == WIN_TABS as c_int {
                if (*curtab.get()).tp_next.is_null() {
                    break;
                }
                goto_tabpage(0 as c_int);
            } else {
                if (*curwin.get()).w_next.is_null() {
                    break;
                }
                curwin.set((*curwin.get()).w_next);
            }
            dorewind = false_0 != 0;
            curbuf.set((*curwin.get()).w_buffer);
            if (*curbuf.get()).b_ml.ml_mfp.is_null() {
                if p_fdls.get() >= 0 as OptInt {
                    (*curwin.get()).w_onebuf_opt.wo_fdl = p_fdls.get();
                }
                swap_exists_action.set(SEA_DIALOG);
                set_buflisted(true_0);
                open_buffer(false_0 != 0, ::core::ptr::null_mut::<exarg_T>(), 0 as c_int);
                if swap_exists_action.get() == SEA_QUIT {
                    if got_int.get() as c_int != 0 || only_one_window() as c_int != 0 {
                        did_emsg.set(false_0);
                        ui_call_error_exit(1 as Integer);
                        getout(1 as c_int);
                    }
                    setfname(
                        curbuf.get(),
                        ::core::ptr::null_mut::<c_char>(),
                        ::core::ptr::null_mut::<c_char>(),
                        false_0 != 0,
                    );
                    (*curwin.get()).w_arg_idx = -1 as c_int;
                    swap_exists_action.set(SEA_NONE);
                } else {
                    handle_swap_exists(::core::ptr::null_mut::<bufref_T>());
                }
                dorewind = true_0 != 0;
            }
            os_breakcheck();
            if !got_int.get() {
                continue;
            }
            vgetc();
            break;
        }
        if (*parmp).window_layout == WIN_TABS as c_int {
            goto_tabpage(1 as c_int);
        } else {
            curwin.set(firstwin.get());
        }
        curbuf.set((*curwin.get()).w_buffer);
        (*autocmd_no_enter.ptr()) -= 1;
        (*autocmd_no_leave.ptr()) -= 1;
    };
}
unsafe extern "C" fn edit_buffers(mut parmp: *mut mparm_T) {
    let mut arg_idx: c_int = 0;
    let mut advance: bool = true_0 != 0;
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut p_shm_save: *mut c_char = ::core::ptr::null_mut::<c_char>();
    (*autocmd_no_enter.ptr()) += 1;
    (*autocmd_no_leave.ptr()) += 1;
    if (*curwin.get()).w_arg_idx == -1 as c_int {
        win_close(curwin.get(), true_0 != 0, false_0 != 0);
        advance = false_0 != 0;
    }
    arg_idx = 1 as c_int;
    let mut i: c_int = 1 as c_int;
    while i < (*parmp).window_count {
        if (*curwin.get()).w_arg_idx == -1 as c_int {
            arg_idx += 1;
            win_close(curwin.get(), true_0 != 0, false_0 != 0);
            advance = false_0 != 0;
        } else {
            if advance {
                if (*parmp).window_layout == WIN_TABS as c_int {
                    if (*curtab.get()).tp_next.is_null() {
                        break;
                    }
                    goto_tabpage(0 as c_int);
                    if i == 1 as c_int {
                        let mut buf: [c_char; 100] = [0; 100];
                        p_shm_save = xstrdup(p_shm.get());
                        snprintf(
                            &raw mut buf as *mut c_char,
                            ::core::mem::size_of::<[c_char; 100]>(),
                            b"F%s\0".as_ptr() as *const c_char,
                            p_shm.get(),
                        );
                        set_option_value_give_err(
                            kOptShortmess,
                            OptVal {
                                type_0: kOptValTypeString,
                                data: OptValData {
                                    string: cstr_as_string(&raw mut buf as *mut c_char),
                                },
                            },
                            0 as c_int,
                        );
                    }
                } else {
                    if (*curwin.get()).w_next.is_null() {
                        break;
                    }
                    win_enter((*curwin.get()).w_next, false_0 != 0);
                }
            }
            advance = true_0 != 0;
            if curbuf.get() == (*firstwin.get()).w_buffer || (*curbuf.get()).b_ffname.is_null() {
                (*curwin.get()).w_arg_idx = arg_idx;
                swap_exists_did_quit.set(false_0 != 0);
                do_ecmd(
                    0 as c_int,
                    if arg_idx < (*global_alist.ptr()).al_ga.ga_len {
                        alist_name(
                            ((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T)
                                .offset(arg_idx as isize),
                        )
                    } else {
                        ::core::ptr::null_mut::<c_char>()
                    },
                    ::core::ptr::null_mut::<c_char>(),
                    ::core::ptr::null_mut::<exarg_T>(),
                    ECMD_LASTL as c_int as linenr_T,
                    ECMD_HIDE as c_int,
                    curwin.get(),
                );
                if swap_exists_did_quit.get() {
                    if got_int.get() as c_int != 0 || only_one_window() as c_int != 0 {
                        did_emsg.set(false_0);
                        ui_call_error_exit(1 as Integer);
                        getout(1 as c_int);
                    }
                    win_close(curwin.get(), true_0 != 0, false_0 != 0);
                    advance = false_0 != 0;
                }
                if arg_idx == (*global_alist.ptr()).al_ga.ga_len - 1 as c_int {
                    arg_had_last.set(true_0 != 0);
                }
                arg_idx += 1;
            }
            os_breakcheck();
            if got_int.get() {
                vgetc();
                break;
            }
        }
        i += 1;
    }
    if !p_shm_save.is_null() {
        set_option_value_give_err(
            kOptShortmess,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(p_shm_save),
                },
            },
            0 as c_int,
        );
        xfree(p_shm_save as *mut c_void);
    }
    if (*parmp).window_layout == WIN_TABS as c_int {
        goto_tabpage(1 as c_int);
    }
    (*autocmd_no_enter.ptr()) -= 1;
    win = firstwin.get();
    while (*win).w_onebuf_opt.wo_pvw != 0 {
        win = (*win).w_next;
        if !win.is_null() {
            continue;
        }
        win = firstwin.get();
        break;
    }
    win_enter(win, false_0 != 0);
    (*autocmd_no_leave.ptr()) -= 1;
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"editing files in windows\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if (*parmp).window_count > 1 as c_int && (*parmp).window_layout != WIN_TABS as c_int {
        win_equal(curwin.get(), false_0 != 0, 'b' as c_int);
    }
}
unsafe extern "C" fn exe_pre_commands(mut parmp: *mut mparm_T) {
    let mut cmds: *mut *mut c_char = &raw mut (*parmp).pre_commands as *mut *mut c_char;
    let mut cnt: c_int = (*parmp).n_pre_commands;
    if cnt <= 0 as c_int {
        return;
    }
    (*curwin.get()).w_cursor.lnum = 0 as c_int as linenr_T;
    estack_push(
        ETYPE_ARGS,
        gettext(b"pre-vimrc command line\0".as_ptr() as *const c_char),
        0 as linenr_T,
    );
    (*current_sctx.ptr()).sc_sid = SID_CMDARG as scid_T;
    let mut i: c_int = 0 as c_int;
    while i < cnt {
        do_cmdline_cmd(*cmds.offset(i as isize));
        i += 1;
    }
    estack_pop();
    (*current_sctx.ptr()).sc_sid = 0 as c_int as scid_T;
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"--cmd commands\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
unsafe extern "C" fn exe_commands(mut parmp: *mut mparm_T) {
    msg_scroll.set(true_0);
    if (*parmp).tagname.is_null() && (*curwin.get()).w_cursor.lnum <= 1 as linenr_T {
        (*curwin.get()).w_cursor.lnum = 0 as c_int as linenr_T;
    }
    estack_push(
        ETYPE_ARGS,
        b"command line\0".as_ptr() as *const c_char as *mut c_char,
        0 as linenr_T,
    );
    (*current_sctx.ptr()).sc_sid = SID_CARG as scid_T;
    (*current_sctx.ptr()).sc_seq = 0 as c_int;
    let mut i: c_int = 0 as c_int;
    while i < (*parmp).n_commands {
        do_cmdline_cmd((*parmp).commands[i as usize]);
        if (*parmp).cmds_tofree[i as usize] != 0 {
            xfree((*parmp).commands[i as usize] as *mut c_void);
        }
        i += 1;
    }
    estack_pop();
    (*current_sctx.ptr()).sc_sid = 0 as c_int as scid_T;
    if (*curwin.get()).w_cursor.lnum == 0 as linenr_T {
        (*curwin.get()).w_cursor.lnum = 1 as c_int as linenr_T;
    }
    if !exmode_active.get() {
        msg_scroll.set(false_0);
    }
    if (*parmp).edit_type == EDIT_QF as c_int {
        qf_jump(
            ::core::ptr::null_mut::<qf_info_T>(),
            0 as c_int,
            0 as c_int,
            false_0,
        );
    }
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"executing command arguments\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
unsafe extern "C" fn do_system_initialization() {
    let config_dirs: *mut c_char = stdpaths_get_xdg_var(kXDGConfigDirs);
    if !config_dirs.is_null() {
        let mut iter: *const c_void = ::core::ptr::null::<c_void>();
        let mut appname: *const c_char = get_appname(false_0 != 0);
        let mut appname_len: size_t = strlen(appname);
        let sysinit_suffix: [c_char; 13] = [
            PATHSEP as c_char,
            's' as c_char,
            'y' as c_char,
            's' as c_char,
            'i' as c_char,
            'n' as c_char,
            'i' as c_char,
            't' as c_char,
            '.' as c_char,
            'v' as c_char,
            'i' as c_char,
            'm' as c_char,
            NUL as c_char,
        ];
        loop {
            let mut dir: *const c_char = ::core::ptr::null::<c_char>();
            let mut dir_len: size_t = 0;
            iter = vim_env_iter(
                ':' as c_char,
                config_dirs,
                iter,
                &raw mut dir,
                &raw mut dir_len,
            );
            if dir.is_null() || dir_len == 0 as size_t {
                break;
            }
            let mut path_len: size_t = dir_len
                .wrapping_add(1 as size_t)
                .wrapping_add(appname_len)
                .wrapping_add(::core::mem::size_of::<[c_char; 13]>());
            let mut vimrc: *mut c_char = xmalloc(path_len) as *mut c_char;
            memcpy(vimrc as *mut c_void, dir as *const c_void, dir_len);
            if *vimrc.offset(dir_len.wrapping_sub(1 as size_t) as isize) as c_int != PATHSEP {
                *vimrc.offset(dir_len as isize) = PATHSEP as c_char;
                dir_len = dir_len.wrapping_add(1 as size_t);
            }
            memcpy(
                vimrc.offset(dir_len as isize) as *mut c_void,
                appname as *const c_void,
                appname_len,
            );
            memcpy(
                vimrc.offset(dir_len as isize).offset(appname_len as isize) as *mut c_void,
                &raw const sysinit_suffix as *const c_char as *const c_void,
                ::core::mem::size_of::<[c_char; 13]>(),
            );
            if do_source(
                vimrc,
                false_0 != 0,
                DOSO_NONE as c_int,
                ::core::ptr::null_mut::<c_int>(),
            ) != FAIL
            {
                xfree(vimrc as *mut c_void);
                xfree(config_dirs as *mut c_void);
                return;
            }
            xfree(vimrc as *mut c_void);
            if iter.is_null() {
                break;
            }
        }
        xfree(config_dirs as *mut c_void);
    }
    do_source(
        SYS_VIMRC_FILE.as_ptr() as *mut c_char,
        false_0 != 0,
        DOSO_NONE as c_int,
        ::core::ptr::null_mut::<c_int>(),
    );
}
unsafe extern "C" fn do_user_initialization() -> bool {
    let mut do_exrc: bool = p_exrc.get() != 0;
    if execute_env(b"VIMINIT\0".as_ptr() as *const c_char as *mut c_char) == OK {
        do_exrc = p_exrc.get() != 0;
        return do_exrc;
    }
    let mut init_lua_path: *mut c_char =
        stdpaths_user_conf_subpath(b"init.lua\0".as_ptr() as *const c_char);
    let mut user_vimrc: *mut c_char =
        stdpaths_user_conf_subpath(b"init.vim\0".as_ptr() as *const c_char);
    if os_path_exists(init_lua_path) as c_int != 0
        && do_source(
            init_lua_path,
            true_0 != 0,
            DOSO_VIMRC as c_int,
            ::core::ptr::null_mut::<c_int>(),
        ) != 0
    {
        if os_path_exists(user_vimrc) {
            semsg(
                (e_conflicting_configs.ptr() as *const _) as *const c_char,
                init_lua_path,
                user_vimrc,
            );
        }
        xfree(user_vimrc as *mut c_void);
        xfree(init_lua_path as *mut c_void);
        do_exrc = p_exrc.get() != 0;
        return do_exrc;
    }
    xfree(init_lua_path as *mut c_void);
    if do_source(
        user_vimrc,
        true_0 != 0,
        DOSO_VIMRC as c_int,
        ::core::ptr::null_mut::<c_int>(),
    ) != FAIL
    {
        do_exrc = p_exrc.get() != 0;
        if do_exrc {
            do_exrc = path_full_compare(
                VIMRC_FILE.as_ptr() as *mut c_char,
                user_vimrc,
                false_0 != 0,
                true_0 != 0,
            ) as c_uint
                != kEqualFiles as c_int as c_uint;
        }
        xfree(user_vimrc as *mut c_void);
        return do_exrc;
    }
    xfree(user_vimrc as *mut c_void);
    let config_dirs: *mut c_char = stdpaths_get_xdg_var(kXDGConfigDirs);
    if !config_dirs.is_null() {
        let mut appname: *const c_char = get_appname(false_0 != 0);
        let mut appname_len: size_t = strlen(appname);
        let mut iter: *const c_void = ::core::ptr::null::<c_void>();
        loop {
            let mut dir: *const c_char = ::core::ptr::null::<c_char>();
            let mut dir_len: size_t = 0;
            iter = vim_env_iter(
                ':' as c_char,
                config_dirs,
                iter,
                &raw mut dir,
                &raw mut dir_len,
            );
            if dir.is_null() || dir_len == 0 as size_t {
                break;
            }
            let init_lua_suffix: [c_char; 10] = [
                PATHSEP as c_char,
                'i' as c_char,
                'n' as c_char,
                'i' as c_char,
                't' as c_char,
                '.' as c_char,
                'l' as c_char,
                'u' as c_char,
                'a' as c_char,
                NUL as c_char,
            ];
            let mut init_lua_len: size_t = dir_len
                .wrapping_add(1 as size_t)
                .wrapping_add(appname_len)
                .wrapping_add(::core::mem::size_of::<[c_char; 10]>());
            let mut init_lua: *mut c_char = xmalloc(init_lua_len) as *mut c_char;
            memcpy(init_lua as *mut c_void, dir as *const c_void, dir_len);
            *init_lua.offset(dir_len as isize) = PATHSEP as c_char;
            memcpy(
                init_lua
                    .offset(dir_len as isize)
                    .offset(1 as c_int as isize) as *mut c_void,
                appname as *const c_void,
                appname_len,
            );
            memcpy(
                init_lua
                    .offset(dir_len as isize)
                    .offset(1 as c_int as isize)
                    .offset(appname_len as isize) as *mut c_void,
                &raw const init_lua_suffix as *const c_char as *const c_void,
                ::core::mem::size_of::<[c_char; 10]>(),
            );
            let init_vim_suffix: [c_char; 10] = [
                PATHSEP as c_char,
                'i' as c_char,
                'n' as c_char,
                'i' as c_char,
                't' as c_char,
                '.' as c_char,
                'v' as c_char,
                'i' as c_char,
                'm' as c_char,
                NUL as c_char,
            ];
            let mut init_vim_len: size_t = dir_len
                .wrapping_add(1 as size_t)
                .wrapping_add(appname_len)
                .wrapping_add(::core::mem::size_of::<[c_char; 10]>());
            let mut init_vim: *mut c_char = xmalloc(init_vim_len) as *mut c_char;
            memcpy(init_vim as *mut c_void, dir as *const c_void, dir_len);
            *init_vim.offset(dir_len as isize) = PATHSEP as c_char;
            memcpy(
                init_vim
                    .offset(dir_len as isize)
                    .offset(1 as c_int as isize) as *mut c_void,
                appname as *const c_void,
                appname_len,
            );
            memcpy(
                init_vim
                    .offset(dir_len as isize)
                    .offset(1 as c_int as isize)
                    .offset(appname_len as isize) as *mut c_void,
                &raw const init_vim_suffix as *const c_char as *const c_void,
                ::core::mem::size_of::<[c_char; 10]>(),
            );
            if os_path_exists(init_lua) as c_int != 0
                && do_source(
                    init_lua,
                    true_0 != 0,
                    DOSO_VIMRC as c_int,
                    ::core::ptr::null_mut::<c_int>(),
                ) != 0
            {
                if os_path_exists(init_vim) {
                    semsg(
                        (e_conflicting_configs.ptr() as *const _) as *const c_char,
                        init_lua,
                        init_vim,
                    );
                }
                xfree(init_vim as *mut c_void);
                xfree(init_lua as *mut c_void);
                xfree(config_dirs as *mut c_void);
                do_exrc = p_exrc.get() != 0;
                return do_exrc;
            }
            xfree(init_lua as *mut c_void);
            if do_source(
                init_vim,
                true_0 != 0,
                DOSO_VIMRC as c_int,
                ::core::ptr::null_mut::<c_int>(),
            ) != FAIL
            {
                do_exrc = p_exrc.get() != 0;
                if do_exrc {
                    do_exrc = path_full_compare(
                        VIMRC_FILE.as_ptr() as *mut c_char,
                        init_vim,
                        false_0 != 0,
                        true_0 != 0,
                    ) as c_uint
                        != kEqualFiles as c_int as c_uint;
                }
                xfree(init_vim as *mut c_void);
                xfree(config_dirs as *mut c_void);
                return do_exrc;
            }
            xfree(init_vim as *mut c_void);
            if iter.is_null() {
                break;
            }
        }
        xfree(config_dirs as *mut c_void);
    }
    if execute_env(b"EXINIT\0".as_ptr() as *const c_char as *mut c_char) == OK {
        do_exrc = p_exrc.get() != 0;
        return do_exrc;
    }
    return do_exrc;
}
unsafe extern "C" fn do_exrc_initialization() {
    let L: *mut lua_State = get_global_lstate();
    '_c2rust_label: {
        if !L.is_null() {
        } else {
            __assert_fail(
                b"L\0".as_ptr() as *const c_char,
                b"src/nvim/main.rs\0".as_ptr() as *const c_char,
                2207 as c_uint,
                b"void do_exrc_initialization(void)\0".as_ptr() as *const c_char,
            );
        }
    };
    lua_getfield(L, LUA_GLOBALSINDEX, b"require\0".as_ptr() as *const c_char);
    lua_pushstring(L, b"vim._core.exrc\0".as_ptr() as *const c_char);
    if nlua_pcall(L, 1 as c_int, 0 as c_int) != 0 {
        fprintf(
            stderr,
            b"%s\n\0".as_ptr() as *const c_char,
            lua_tolstring(L, -1 as c_int, ::core::ptr::null_mut::<size_t>()),
        );
    }
}
unsafe extern "C" fn source_startup_scripts(parmp: *const mparm_T) {
    if !(*parmp).use_vimrc.is_null() {
        if !(strequal((*parmp).use_vimrc, b"NONE\0".as_ptr() as *const c_char) as c_int != 0
            || strequal((*parmp).use_vimrc, b"NORC\0".as_ptr() as *const c_char) as c_int != 0)
        {
            if do_source(
                (*parmp).use_vimrc,
                false_0 != 0,
                DOSO_NONE as c_int,
                ::core::ptr::null_mut::<c_int>(),
            ) != OK
            {
                semsg(
                    gettext((e_cannot_read_from_str_2.ptr() as *const _) as *const c_char),
                    (*parmp).use_vimrc,
                );
            }
        }
    } else if !silent_mode.get() {
        do_system_initialization();
        if do_user_initialization() {
            do_exrc_initialization();
        }
    }
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"sourcing vimrc file(s)\0".as_ptr() as *const c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
unsafe extern "C" fn execute_env(mut env: *mut c_char) -> c_int {
    let mut initstr: *mut c_char = os_getenv(env);
    if initstr.is_null() {
        return FAIL;
    }
    estack_push(ETYPE_ENV, env, 0 as linenr_T);
    let save_current_sctx: sctx_T = current_sctx.get();
    (*current_sctx.ptr()).sc_sid = SID_ENV as scid_T;
    (*current_sctx.ptr()).sc_seq = 0 as c_int;
    (*current_sctx.ptr()).sc_lnum = 0 as c_int as linenr_T;
    do_cmdline_cmd(initstr);
    estack_pop();
    current_sctx.set(save_current_sctx);
    xfree(initstr as *mut c_void);
    return OK;
}
unsafe extern "C" fn mainerr(
    mut msg1: *const c_char,
    mut msg2: *const c_char,
    mut msg3: *const c_char,
) -> ! {
    print_mainerr(msg1, msg2, msg3);
    os_exit(1 as c_int);
}
unsafe extern "C" fn print_mainerr(
    mut msg1: *const c_char,
    mut msg2: *const c_char,
    mut msg3: *const c_char,
) {
    let mut prgname: *mut c_char = path_tail(argv0.get());
    signal_stop();
    fprintf(
        stderr,
        b"%s: %s\0".as_ptr() as *const c_char,
        prgname,
        gettext(msg1),
    );
    if !msg2.is_null() {
        fprintf(stderr, b": \"%s\"\0".as_ptr() as *const c_char, msg2);
    }
    if !msg3.is_null() {
        fprintf(stderr, b": \"%s\"\0".as_ptr() as *const c_char, msg3);
    }
    fprintf(
        stderr,
        gettext(b"\nMore info with \"\0".as_ptr() as *const c_char),
    );
    fprintf(stderr, b"%s -h\"\n\0".as_ptr() as *const c_char, prgname);
}
unsafe extern "C" fn version() {
    nlua_init(
        ::core::ptr::null_mut::<*mut c_char>(),
        0 as c_int,
        -1 as c_int,
    );
    info_message.set(true_0 != 0);
    list_version();
    msg_putchar('\n' as c_int);
    msg_didout.set(false_0 != 0);
}
unsafe extern "C" fn usage() {
    signal_stop();
    printf(gettext(b"Usage:\n\0".as_ptr() as *const c_char));
    printf(gettext(
        b"  nvim [options] [file ...]\n\0".as_ptr() as *const c_char
    ));
    printf(gettext(b"\nOptions:\n\0".as_ptr() as *const c_char));
    printf(gettext(
        b"  --cmd <cmd>           Execute <cmd> before any config\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  +<cmd>, -c <cmd>      Execute <cmd> after config and first file\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  -l <script> [args...] Execute Lua <script> (with optional args)\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  -S <session>          Source <session> after loading the first file\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  -s <scriptin>         Read Normal mode commands from <scriptin>\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  -u <config>           Use this config file\n\0".as_ptr() as *const c_char,
    ));
    printf(b"\n\0".as_ptr() as *const c_char);
    printf(gettext(
        b"  -d                    Diff mode\n\0".as_ptr() as *const c_char
    ));
    printf(gettext(
        b"  -es, -Es              Silent (batch) mode\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  -h, --help            Print this help message\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  -i <shada>            Use this shada file\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  -n                    No swap file, use memory only\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  -o[N]                 Open N windows (default: one per file)\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  -O[N]                 Open N vertical windows (default: one per file)\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  -p[N]                 Open N tab pages (default: one per file)\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  -R                    Read-only (view) mode\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  -v, --version         Print version information\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  -V[N][file]           Verbose [level][file]\n\0".as_ptr() as *const c_char,
    ));
    printf(b"\n\0".as_ptr() as *const c_char);
    printf(gettext(
        b"  --                    Only file names after this\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  --api-info            Write msgpack-encoded API metadata to stdout\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  --clean               \"Factory defaults\" (skip user config and plugins, shada)\n\0"
            .as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  --embed               Use stdin/stdout as a msgpack-rpc channel\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  --headless            Don't start a user interface\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  --listen <address>    Serve RPC API from this address\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  --remote[-subcommand] Execute commands remotely on a server\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"  --server <address>    Connect to this Nvim server\n\0".as_ptr() as *const c_char,
    ));
    printf(gettext(
        b"  --startuptime <file>  Write startup timing messages to <file>\n\0".as_ptr()
            as *const c_char,
    ));
    printf(gettext(
        b"\nSee \":help startup-options\" for all options.\n\0".as_ptr() as *const c_char,
    ));
}
unsafe extern "C" fn check_swap_exists_action() {
    if swap_exists_action.get() == SEA_QUIT {
        ui_call_error_exit(1 as Integer);
        getout(1 as c_int);
    }
    handle_swap_exists(::core::ptr::null_mut::<bufref_T>());
}
pub static tslua_query_parse_count: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub const MAX_ARG_CMDS: c_int = 10 as c_int;
pub static namedfm: GlobalCell<[xfmark_T; 36]> = GlobalCell::new(
    [xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0 as linenr_T,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<c_char>(),
    }; 36],
);
pub static ch_before_blocking_events: GlobalCell<*mut MultiQueue> =
    GlobalCell::new(::core::ptr::null_mut::<MultiQueue>());
pub static showcmd_buf: GlobalCell<[c_char; 41]> = GlobalCell::new([0; 41]);
pub static repeat_luaref: GlobalCell<LuaRef> = GlobalCell::new(-2 as LuaRef);
pub static used_stdin: GlobalCell<bool> = GlobalCell::new(false);
pub static nvim_testing: GlobalCell<bool> = GlobalCell::new(false);
pub static pum_grid: GlobalCell<ScreenGrid> = GlobalCell::new(ScreenGrid {
    handle: 0 as handle_T,
    chars: ::core::ptr::null_mut::<schar_T>(),
    attrs: ::core::ptr::null_mut::<sattr_T>(),
    vcols: ::core::ptr::null_mut::<colnr_T>(),
    line_offset: ::core::ptr::null_mut::<size_t>(),
    dirty_col: ::core::ptr::null_mut::<c_int>(),
    rows: 0 as c_int,
    cols: 0 as c_int,
    valid: false,
    throttled: false,
    blending: false,
    mouse_enabled: true,
    zindex: 0 as c_int,
    comp_row: 0 as c_int,
    comp_col: 0 as c_int,
    comp_width: 0 as c_int,
    comp_height: 0 as c_int,
    comp_index: 0 as size_t,
    comp_disabled: false,
    pending_comp_index_update: true,
});
#[unsafe(no_mangle)]
pub static pum_want: GlobalCell<PumWant> = GlobalCell::new(PumWant {
    active: false,
    item: 0,
    insert: false,
    finish: false,
});
pub static tab_page_click_defs: GlobalCell<*mut StlClickDefinition> =
    GlobalCell::new(::core::ptr::null_mut::<StlClickDefinition>());
pub static tab_page_click_defs_size: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
pub static noargs: GlobalCell<Array> = GlobalCell::new(Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
});
pub static ui_event_ns_id: GlobalCell<uint32_t> = GlobalCell::new(0 as uint32_t);
pub static resize_events: GlobalCell<*mut MultiQueue> =
    GlobalCell::new(::core::ptr::null_mut::<MultiQueue>());
pub static ui_refresh_cmdheight: GlobalCell<bool> = GlobalCell::new(true);
pub static grid_line_buf_size: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
pub static grid_line_buf_char: GlobalCell<*mut schar_T> =
    GlobalCell::new(::core::ptr::null_mut::<schar_T>());
pub static grid_line_buf_attr: GlobalCell<*mut sattr_T> =
    GlobalCell::new(::core::ptr::null_mut::<sattr_T>());
pub static ui_client_channel_id: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub static ui_client_error_exit: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static ui_client_exit_status: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static ui_client_attached: GlobalCell<bool> = GlobalCell::new(false);
pub static ui_client_forward_stdin: GlobalCell<bool> = GlobalCell::new(false);
pub static tabpage_move_disallowed: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static float_anchor_str: GlobalCell<[*const c_char; 4]> = GlobalCell::new([
    b"NW\0".as_ptr() as *const c_char,
    b"NE\0".as_ptr() as *const c_char,
    b"SW\0".as_ptr() as *const c_char,
    b"SE\0".as_ptr() as *const c_char,
]);
pub const true_0: c_int = 1 as c_int;
pub const false_0: c_int = 0 as c_int;
pub const WRITEBIN: [c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [c_char; 3]>(*b"wb\0") };
pub const APPENDBIN: [c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [c_char; 3]>(*b"ab\0") };
pub fn main() {
    let mut args_strings: Vec<Vec<u8>> = ::std::env::args()
        .map(|arg| {
            ::std::ffi::CString::new(arg)
                .expect("Failed to convert argument into CString.")
                .into_bytes_with_nul()
        })
        .collect();
    let mut args_ptrs: Vec<*mut c_char> = args_strings
        .iter_mut()
        .map(|arg| arg.as_mut_ptr() as *mut c_char)
        .chain(::core::iter::once(::core::ptr::null_mut()))
        .collect();
    unsafe {
        ::std::process::exit(main_0(
            (args_ptrs.len() - 1) as c_int,
            args_ptrs.as_mut_ptr() as *mut *mut c_char,
        ) as i32)
    }
}
unsafe extern "C" fn c2rust_run_static_initializers() {
    kTVCstring.set((18446744073709551615 as size_t).wrapping_sub(1 as size_t));
}
#[used]
#[cfg_attr(target_os = "linux", unsafe(link_section = ".init_array"))]
#[cfg_attr(target_os = "windows", unsafe(link_section = ".CRT$XIB"))]
#[cfg_attr(target_os = "macos", unsafe(link_section = "__DATA,__mod_init_func"))]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [c2rust_run_static_initializers];
