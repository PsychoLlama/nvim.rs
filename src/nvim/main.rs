use crate::src::nvim::api::private::helpers::{api_free_object, api_metadata_raw, cstr_as_string};
use crate::src::nvim::api::ui::remote_ui_wait_for_attach;
use crate::src::nvim::arglist::{alist_add, alist_init, alist_name};
use crate::src::nvim::autocmd::{
    apply_autocmds, autocmd_init, block_autocmds, is_autocmd_blocked, unblock_autocmds,
};
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
use crate::src::nvim::eval::typval::{tv_list_alloc, tv_list_append_string};
use crate::src::nvim::eval::userfunc::invoke_all_defer;
use crate::src::nvim::eval::vars::{
    get_vim_var_list, get_vim_var_str, set_reg_var, set_vim_var_list, set_vim_var_nr,
    set_vim_var_string, set_vim_var_type,
};
use crate::src::nvim::eval_1::{
    eval_has_provider, eval_init, garbage_collect, set_argv_var, timer_teardown,
};
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::event::multiqueue::{multiqueue_new_child, multiqueue_process_events};
use crate::src::nvim::event::proc::proc_teardown;
use crate::src::nvim::event::r#loop::{loop_close, loop_init, loop_poll_events};
use crate::src::nvim::event::socket::socket_address_tcp_host_end;
use crate::src::nvim::event::stream::stream_set_blocking;
use crate::src::nvim::ex_cmds::do_ecmd;
use crate::src::nvim::ex_docmd::{do_cmdline_cmd, filetype_maybe_enable, filetype_plugin_enable};
use crate::src::nvim::ex_getln::cmdline_init;
use crate::src::nvim::fileio::{readfile, shorten_fnames};
use crate::src::nvim::getchar::{open_scriptin, stuffcharReadbuff, vgetc};
use crate::src::nvim::global_cell::{GlobalCell, SharedCell};
use crate::src::nvim::hashtab::hash_debug_results;
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
use crate::src::nvim::msgpack_rpc::server::{server_init, server_teardown};
use crate::src::nvim::normal::{check_scrollbind, init_normal_cmds, normal_enter};
use crate::src::nvim::option::{
    reset_modifiable, set_init_1, set_init_2, set_init_3, set_init_tablocal, set_option_direct,
    set_option_value_give_err, set_options_bin,
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
use crate::src::nvim::r#move::update_topline;
use crate::src::nvim::register::get_default_register_name;
use crate::src::nvim::runtime::{
    do_source, estack_init, estack_pop, estack_push, load_plugins, runtime_init,
};
use crate::src::nvim::shada::{shada_read_everything, shada_write_file};
use crate::src::nvim::strings::vim_snprintf;
use crate::src::nvim::syntax::syn_maybe_enable;
use crate::src::nvim::terminal::{terminal_init, terminal_teardown};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, Arena, Array, AutoPat, AutoPatCmd, AutoPatCmd_S, BoolVarValue,
    Boolean, BufUpdateCallbacks, CMD_index, Callback, CallbackReader, CallbackType,
    Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorPriorityInternal, DecorRange, DecorRangeKind,
    DecorRangeSlot, DecorRange_data as C2Rust_Unnamed_32, DecorRange_data_ui as C2Rust_Unnamed_33,
    DecorSignHighlight, DecorState, DecorState_ranges_i as C2Rust_Unnamed_34,
    DecorState_slots as C2Rust_Unnamed_35, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2,
    Dict, Error, ErrorType, ExtmarkMove, ExtmarkSavePos, ExtmarkSplice, ExtmarkUndoObject,
    FileComparison, FileID, Float, FloatAnchor, FloatRelative, GridView, Integer, Intersection,
    KeyValuePair, LineGetter, ListLenSpecials, Loop, LuaRef, LuaRetMode, MTKey, MTNode, MTPos,
    MapHash, Map_String_int, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_int_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MarkTreeIter,
    MarkTreeIter_s as C2Rust_Unnamed_29, MultiQueue, Object, ObjectType, OptIndex, OptInt, OptVal,
    OptValData, OptValType, Proc, ProcType, RStream, RgbValue, ScopeDictDictItem, ScopeType,
    ScreenGrid, Set_String, Set_int, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue,
    StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_13, Stream, String_0, Terminal,
    Timestamp, TriState, UndoObjectType, VarLockStatus, VarType, VimMenu, VimVarIndex, VirtLines,
    VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinExtmark, WinInfo, WinSplit, WinStyle,
    Window, XDGVarType, _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, __off64_t, __off_t,
    __pthread_internal_list, __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t,
    __time_t, aentry_T, alist_T, aucmdwin_T, auto_event, bcount_t, bhdr_T, bln_values, blob_T,
    blobvar_S, blocknr_T, buf_T, bufref_T, bufstate_T, caller_scope, chunksize_T, cmd_addr_T,
    cmdidx_T, cmdmod_T, colnr_T, cstack_T, cstack_T_cs_pend as C2Rust_Unnamed_30, dict_T,
    dictvar_S, diff_T, diffblock_S, disptick_T, eslist_T, eslist_elem, estack_T,
    estack_T_es_info as C2Rust_Unnamed_43, etype_T, evalarg_T, event_T, exarg, exarg_T, except_T,
    except_type_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_3, file_buffer_b_wininfo as C2Rust_Unnamed_12,
    file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, file_comparison, float_T, fmark_T, fmarkv_T,
    frame_S, frame_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T,
    handle_T, hash_T, hashitem_T, hashtab_T, hlf_T, infoptr_T, int16_t, int32_t, int64_t,
    internal_proc_cb, key_extra, key_value_pair, lcs_chars_T, linenr_T, list_T, listitem_S,
    listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, loop_0,
    loop_0_children as C2Rust_Unnamed_22, lpos_T, lua_State, mapblock, mapblock_T, match_T,
    matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, msglist, msglist_T, mtnode_inner_s,
    mtnode_s, multiqueue, nlua_ref_state_t, nvim_stats_s, object, object_data as C2Rust_Unnamed,
    optmagic_T, partial_S, partial_T, pos_T, pos_save_T, proc, proc_exit_cb, proc_state_cb,
    proftime_T, pthread_mutex_t, pthread_rwlock_t, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue,
    reg_extmatch_T, regmatch_T, regmmatch_T, regprog, regprog_T, rstream, sattr_T, schar_T, scid_T,
    sctx_T, size_t, ssize_t, stream, stream_close_cb, stream_read_cb,
    stream_uv as C2Rust_Unnamed_24, stream_write_cb, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, tabpage_S,
    tabpage_T, taggy_T, terminal, time_t, typebuf_T, typval_T, typval_vval_union, u_entry,
    u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_9,
    u_header_uh_alt_prev as C2Rust_Unnamed_8, u_header_uh_next as C2Rust_Unnamed_11,
    u_header_uh_prev as C2Rust_Unnamed_10, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, undo_object_data as C2Rust_Unnamed_7, uv__io_cb, uv__io_s, uv__io_t, uv__queue,
    uv_alloc_cb, uv_async_cb, uv_async_s, uv_async_s_u as C2Rust_Unnamed_19, uv_async_t, uv_buf_t,
    uv_close_cb, uv_connect_cb, uv_connect_s, uv_connect_t, uv_connection_cb, uv_file, uv_handle_s,
    uv_handle_s_u as C2Rust_Unnamed_14, uv_handle_t, uv_handle_type, uv_idle_cb, uv_idle_s,
    uv_idle_s_u as C2Rust_Unnamed_25, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_18, uv_loop_s_timer_heap as C2Rust_Unnamed_17,
    uv_loop_t, uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_27, uv_pipe_t, uv_read_cb,
    uv_req_type, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t, uv_signal_cb,
    uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed_15, uv_signal_s_u as C2Rust_Unnamed_16,
    uv_signal_t, uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_23, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_26, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_20, uv_timer_s_u as C2Rust_Unnamed_21, uv_timer_t,
    varnumber_T, vim_exception, vimmenu_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S,
    winopt_T, wline_T, xfmark_T, FILE, NS, QUEUE, _IO_FILE,
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
extern "C" {
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn qf_init(
        wp: *mut win_T,
        efile: *const ::core::ffi::c_char,
        errorformat: *mut ::core::ffi::c_char,
        newlist: ::core::ffi::c_int,
        qf_title: *const ::core::ffi::c_char,
        enc: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn qf_jump(
        qi: *mut qf_info_T,
        dir: ::core::ffi::c_int,
        errornr: ::core::ffi::c_int,
        forceit: ::core::ffi::c_int,
    );
}
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
pub const kObjectTypeTabpage: ObjectType = 10;
pub const kObjectTypeWindow: ObjectType = 9;
pub const kObjectTypeBuffer: ObjectType = 8;
pub const kObjectTypeLuaRef: ObjectType = 7;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeArray: ObjectType = 5;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeFloat: ObjectType = 3;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub const kVPosWinCol: VirtTextPos = 5;
pub const kVPosRightAlign: VirtTextPos = 4;
pub const kVPosOverlay: VirtTextPos = 3;
pub const kVPosInline: VirtTextPos = 2;
pub const kVPosEndOfLineRightAlign: VirtTextPos = 1;
pub const kVPosEndOfLine: VirtTextPos = 0;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
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
pub const kExtmarkClear: UndoObjectType = 4;
pub const kExtmarkSavePos: UndoObjectType = 3;
pub const kExtmarkUpdate: UndoObjectType = 2;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub const kStlClickFuncRun: C2Rust_Unnamed_13 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_13 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_13 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_13 = 0;
pub const kAlignRight: AlignTextPos = 2;
pub const kAlignCenter: AlignTextPos = 1;
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleMinimal: WinStyle = 1;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitBelow: WinSplit = 3;
pub const kWinSplitAbove: WinSplit = 2;
pub const kWinSplitRight: WinSplit = 1;
pub const kWinSplitLeft: WinSplit = 0;
pub const kFloatRelativeLaststatus: FloatRelative = 5;
pub const kFloatRelativeTabline: FloatRelative = 4;
pub const kFloatRelativeMouse: FloatRelative = 3;
pub const kFloatRelativeCursor: FloatRelative = 2;
pub const kFloatRelativeWindow: FloatRelative = 1;
pub const kFloatRelativeEditor: FloatRelative = 0;
pub const MF_DIRTY_YES_NOSYNC: mfdirty_T = 2;
pub const MF_DIRTY_YES: mfdirty_T = 1;
pub const MF_DIRTY_NO: mfdirty_T = 0;
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
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_28 = 2147483647;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
pub const HLF_COUNT: hlf_T = 76;
pub const HLF_PRE: hlf_T = 75;
pub const HLF_OK: hlf_T = 74;
pub const HLF_SO: hlf_T = 73;
pub const HLF_SE: hlf_T = 72;
pub const HLF_TSNC: hlf_T = 71;
pub const HLF_TS: hlf_T = 70;
pub const HLF_BFOOTER: hlf_T = 69;
pub const HLF_BTITLE: hlf_T = 68;
pub const HLF_CU: hlf_T = 67;
pub const HLF_WBRNC: hlf_T = 66;
pub const HLF_WBR: hlf_T = 65;
pub const HLF_BORDER: hlf_T = 64;
pub const HLF_MSG: hlf_T = 63;
pub const HLF_NFLOAT: hlf_T = 62;
pub const HLF_MSGSEP: hlf_T = 61;
pub const HLF_INACTIVE: hlf_T = 60;
pub const HLF_0: hlf_T = 59;
pub const HLF_QFL: hlf_T = 58;
pub const HLF_MC: hlf_T = 57;
pub const HLF_CUL: hlf_T = 56;
pub const HLF_CUC: hlf_T = 55;
pub const HLF_TPF: hlf_T = 54;
pub const HLF_TPS: hlf_T = 53;
pub const HLF_TP: hlf_T = 52;
pub const HLF_PBR: hlf_T = 51;
pub const HLF_PST: hlf_T = 50;
pub const HLF_PSB: hlf_T = 49;
pub const HLF_PSX: hlf_T = 48;
pub const HLF_PNX: hlf_T = 47;
pub const HLF_PSK: hlf_T = 46;
pub const HLF_PNK: hlf_T = 45;
pub const HLF_PMSI: hlf_T = 44;
pub const HLF_PMNI: hlf_T = 43;
pub const HLF_PSI: hlf_T = 42;
pub const HLF_PNI: hlf_T = 41;
pub const HLF_SPL: hlf_T = 40;
pub const HLF_SPR: hlf_T = 39;
pub const HLF_SPC: hlf_T = 38;
pub const HLF_SPB: hlf_T = 37;
pub const HLF_CONCEAL: hlf_T = 36;
pub const HLF_SC: hlf_T = 35;
pub const HLF_TXA: hlf_T = 34;
pub const HLF_TXD: hlf_T = 33;
pub const HLF_DED: hlf_T = 32;
pub const HLF_CHD: hlf_T = 31;
pub const HLF_ADD: hlf_T = 30;
pub const HLF_FC: hlf_T = 29;
pub const HLF_FL: hlf_T = 28;
pub const HLF_WM: hlf_T = 27;
pub const HLF_W: hlf_T = 26;
pub const HLF_VNC: hlf_T = 25;
pub const HLF_V: hlf_T = 24;
pub const HLF_T: hlf_T = 23;
pub const HLF_VSP: hlf_T = 22;
pub const HLF_C: hlf_T = 21;
pub const HLF_SNC: hlf_T = 20;
pub const HLF_S: hlf_T = 19;
pub const HLF_R: hlf_T = 18;
pub const HLF_CLF: hlf_T = 17;
pub const HLF_CLS: hlf_T = 16;
pub const HLF_CLN: hlf_T = 15;
pub const HLF_LNB: hlf_T = 14;
pub const HLF_LNA: hlf_T = 13;
pub const HLF_N: hlf_T = 12;
pub const HLF_CM: hlf_T = 11;
pub const HLF_M: hlf_T = 10;
pub const HLF_LC: hlf_T = 9;
pub const HLF_L: hlf_T = 8;
pub const HLF_I: hlf_T = 7;
pub const HLF_E: hlf_T = 6;
pub const HLF_D: hlf_T = 5;
pub const HLF_AT: hlf_T = 4;
pub const HLF_TERM: hlf_T = 3;
pub const HLF_EOB: hlf_T = 2;
pub const HLF_8: hlf_T = 1;
pub const HLF_NONE: hlf_T = 0;
pub const OPTION_MAGIC_OFF: optmagic_T = 2;
pub const OPTION_MAGIC_ON: optmagic_T = 1;
pub const OPTION_MAGIC_NOT_SET: optmagic_T = 0;
pub const kOptWritedelay: OptIndex = 373;
pub const kOptWritebackup: OptIndex = 372;
pub const kOptWriteany: OptIndex = 371;
pub const kOptWrite: OptIndex = 370;
pub const kOptWrapscan: OptIndex = 369;
pub const kOptWrapmargin: OptIndex = 368;
pub const kOptWrap: OptIndex = 367;
pub const kOptWinwidth: OptIndex = 366;
pub const kOptWinminwidth: OptIndex = 365;
pub const kOptWinminheight: OptIndex = 364;
pub const kOptWinhighlight: OptIndex = 363;
pub const kOptWinheight: OptIndex = 362;
pub const kOptWinfixwidth: OptIndex = 361;
pub const kOptWinfixheight: OptIndex = 360;
pub const kOptWinfixbuf: OptIndex = 359;
pub const kOptWindow: OptIndex = 358;
pub const kOptWinborder: OptIndex = 357;
pub const kOptWinblend: OptIndex = 356;
pub const kOptWinbar: OptIndex = 355;
pub const kOptWinaltkeys: OptIndex = 354;
pub const kOptWildoptions: OptIndex = 353;
pub const kOptWildmode: OptIndex = 352;
pub const kOptWildmenu: OptIndex = 351;
pub const kOptWildignorecase: OptIndex = 350;
pub const kOptWildignore: OptIndex = 349;
pub const kOptWildcharm: OptIndex = 348;
pub const kOptWildchar: OptIndex = 347;
pub const kOptWhichwrap: OptIndex = 346;
pub const kOptWarn: OptIndex = 345;
pub const kOptVisualbell: OptIndex = 344;
pub const kOptVirtualedit: OptIndex = 343;
pub const kOptViewoptions: OptIndex = 342;
pub const kOptViewdir: OptIndex = 341;
pub const kOptVerbosefile: OptIndex = 340;
pub const kOptVerbose: OptIndex = 339;
pub const kOptVartabstop: OptIndex = 338;
pub const kOptVarsofttabstop: OptIndex = 337;
pub const kOptUpdatetime: OptIndex = 336;
pub const kOptUpdatecount: OptIndex = 335;
pub const kOptUndoreload: OptIndex = 334;
pub const kOptUndolevels: OptIndex = 333;
pub const kOptUndofile: OptIndex = 332;
pub const kOptUndodir: OptIndex = 331;
pub const kOptTtyfast: OptIndex = 330;
pub const kOptTtimeoutlen: OptIndex = 329;
pub const kOptTtimeout: OptIndex = 328;
pub const kOptTitlestring: OptIndex = 327;
pub const kOptTitleold: OptIndex = 326;
pub const kOptTitlelen: OptIndex = 325;
pub const kOptTitle: OptIndex = 324;
pub const kOptTimeoutlen: OptIndex = 323;
pub const kOptTimeout: OptIndex = 322;
pub const kOptTildeop: OptIndex = 321;
pub const kOptThesaurusfunc: OptIndex = 320;
pub const kOptThesaurus: OptIndex = 319;
pub const kOptTextwidth: OptIndex = 318;
pub const kOptTerse: OptIndex = 317;
pub const kOptTermsync: OptIndex = 316;
pub const kOptTermpastefilter: OptIndex = 315;
pub const kOptTermguicolors: OptIndex = 314;
pub const kOptTermencoding: OptIndex = 313;
pub const kOptTermbidi: OptIndex = 312;
pub const kOptTagstack: OptIndex = 311;
pub const kOptTags: OptIndex = 310;
pub const kOptTagrelative: OptIndex = 309;
pub const kOptTaglength: OptIndex = 308;
pub const kOptTagfunc: OptIndex = 307;
pub const kOptTagcase: OptIndex = 306;
pub const kOptTagbsearch: OptIndex = 305;
pub const kOptTabstop: OptIndex = 304;
pub const kOptTabpagemax: OptIndex = 303;
pub const kOptTabline: OptIndex = 302;
pub const kOptTabclose: OptIndex = 301;
pub const kOptSyntax: OptIndex = 300;
pub const kOptSynmaxcol: OptIndex = 299;
pub const kOptSwitchbuf: OptIndex = 298;
pub const kOptSwapfile: OptIndex = 297;
pub const kOptSuffixesadd: OptIndex = 296;
pub const kOptSuffixes: OptIndex = 295;
pub const kOptStatusline: OptIndex = 294;
pub const kOptStatuscolumn: OptIndex = 293;
pub const kOptStartofline: OptIndex = 292;
pub const kOptSplitright: OptIndex = 291;
pub const kOptSplitkeep: OptIndex = 290;
pub const kOptSplitbelow: OptIndex = 289;
pub const kOptSpellsuggest: OptIndex = 288;
pub const kOptSpelloptions: OptIndex = 287;
pub const kOptSpelllang: OptIndex = 286;
pub const kOptSpellfile: OptIndex = 285;
pub const kOptSpellcapcheck: OptIndex = 284;
pub const kOptSpell: OptIndex = 283;
pub const kOptSofttabstop: OptIndex = 282;
pub const kOptSmoothscroll: OptIndex = 281;
pub const kOptSmarttab: OptIndex = 280;
pub const kOptSmartindent: OptIndex = 279;
pub const kOptSmartcase: OptIndex = 278;
pub const kOptSigncolumn: OptIndex = 277;
pub const kOptSidescrolloff: OptIndex = 276;
pub const kOptSidescroll: OptIndex = 275;
pub const kOptShowtabline: OptIndex = 274;
pub const kOptShowmode: OptIndex = 273;
pub const kOptShowmatch: OptIndex = 272;
pub const kOptShowfulltag: OptIndex = 271;
pub const kOptShowcmdloc: OptIndex = 270;
pub const kOptShowcmd: OptIndex = 269;
pub const kOptShowbreak: OptIndex = 268;
pub const kOptShortmess: OptIndex = 267;
pub const kOptShiftwidth: OptIndex = 266;
pub const kOptShiftround: OptIndex = 265;
pub const kOptShellxquote: OptIndex = 264;
pub const kOptShellxescape: OptIndex = 263;
pub const kOptShelltemp: OptIndex = 262;
pub const kOptShellslash: OptIndex = 261;
pub const kOptShellredir: OptIndex = 260;
pub const kOptShellquote: OptIndex = 259;
pub const kOptShellpipe: OptIndex = 258;
pub const kOptShellcmdflag: OptIndex = 257;
pub const kOptShell: OptIndex = 256;
pub const kOptShadafile: OptIndex = 255;
pub const kOptShada: OptIndex = 254;
pub const kOptSessionoptions: OptIndex = 253;
pub const kOptSelectmode: OptIndex = 252;
pub const kOptSelection: OptIndex = 251;
pub const kOptSecure: OptIndex = 250;
pub const kOptSections: OptIndex = 249;
pub const kOptScrollopt: OptIndex = 248;
pub const kOptScrolloff: OptIndex = 247;
pub const kOptScrolljump: OptIndex = 246;
pub const kOptScrollbind: OptIndex = 245;
pub const kOptScrollback: OptIndex = 244;
pub const kOptScroll: OptIndex = 243;
pub const kOptRuntimepath: OptIndex = 242;
pub const kOptRulerformat: OptIndex = 241;
pub const kOptRuler: OptIndex = 240;
pub const kOptRightleftcmd: OptIndex = 239;
pub const kOptRightleft: OptIndex = 238;
pub const kOptRevins: OptIndex = 237;
pub const kOptReport: OptIndex = 236;
pub const kOptRemap: OptIndex = 235;
pub const kOptRelativenumber: OptIndex = 234;
pub const kOptRegexpengine: OptIndex = 233;
pub const kOptRedrawtime: OptIndex = 232;
pub const kOptRedrawdebug: OptIndex = 231;
pub const kOptReadonly: OptIndex = 230;
pub const kOptQuoteescape: OptIndex = 229;
pub const kOptQuickfixtextfunc: OptIndex = 228;
pub const kOptPyxversion: OptIndex = 227;
pub const kOptPumwidth: OptIndex = 226;
pub const kOptPummaxwidth: OptIndex = 225;
pub const kOptPumheight: OptIndex = 224;
pub const kOptPumborder: OptIndex = 223;
pub const kOptPumblend: OptIndex = 222;
pub const kOptPrompt: OptIndex = 221;
pub const kOptPreviewwindow: OptIndex = 220;
pub const kOptPreviewheight: OptIndex = 219;
pub const kOptPreserveindent: OptIndex = 218;
pub const kOptPath: OptIndex = 217;
pub const kOptPatchmode: OptIndex = 216;
pub const kOptPatchexpr: OptIndex = 215;
pub const kOptPastetoggle: OptIndex = 214;
pub const kOptPaste: OptIndex = 213;
pub const kOptParagraphs: OptIndex = 212;
pub const kOptPackpath: OptIndex = 211;
pub const kOptOperatorfunc: OptIndex = 210;
pub const kOptOpendevice: OptIndex = 209;
pub const kOptOmnifunc: OptIndex = 208;
pub const kOptNumberwidth: OptIndex = 207;
pub const kOptNumber: OptIndex = 206;
pub const kOptNrformats: OptIndex = 205;
pub const kOptMousetime: OptIndex = 204;
pub const kOptMouseshape: OptIndex = 203;
pub const kOptMousescroll: OptIndex = 202;
pub const kOptMousemoveevent: OptIndex = 201;
pub const kOptMousemodel: OptIndex = 200;
pub const kOptMousehide: OptIndex = 199;
pub const kOptMousefocus: OptIndex = 198;
pub const kOptMouse: OptIndex = 197;
pub const kOptMore: OptIndex = 196;
pub const kOptModified: OptIndex = 195;
pub const kOptModifiable: OptIndex = 194;
pub const kOptModelines: OptIndex = 193;
pub const kOptModelineexpr: OptIndex = 192;
pub const kOptModeline: OptIndex = 191;
pub const kOptMkspellmem: OptIndex = 190;
pub const kOptMessagesopt: OptIndex = 189;
pub const kOptMenuitems: OptIndex = 188;
pub const kOptMaxsearchcount: OptIndex = 187;
pub const kOptMaxmempattern: OptIndex = 186;
pub const kOptMaxmapdepth: OptIndex = 185;
pub const kOptMaxfuncdepth: OptIndex = 184;
pub const kOptMaxcombine: OptIndex = 183;
pub const kOptMatchtime: OptIndex = 182;
pub const kOptMatchpairs: OptIndex = 181;
pub const kOptMakeprg: OptIndex = 180;
pub const kOptMakeencoding: OptIndex = 179;
pub const kOptMakeef: OptIndex = 178;
pub const kOptMagic: OptIndex = 177;
pub const kOptLoadplugins: OptIndex = 176;
pub const kOptListchars: OptIndex = 175;
pub const kOptList: OptIndex = 174;
pub const kOptLispwords: OptIndex = 173;
pub const kOptLispoptions: OptIndex = 172;
pub const kOptLisp: OptIndex = 171;
pub const kOptLinespace: OptIndex = 170;
pub const kOptLines: OptIndex = 169;
pub const kOptLinebreak: OptIndex = 168;
pub const kOptLhistory: OptIndex = 167;
pub const kOptLazyredraw: OptIndex = 166;
pub const kOptLaststatus: OptIndex = 165;
pub const kOptLangremap: OptIndex = 164;
pub const kOptLangnoremap: OptIndex = 163;
pub const kOptLangmenu: OptIndex = 162;
pub const kOptLangmap: OptIndex = 161;
pub const kOptKeywordprg: OptIndex = 160;
pub const kOptKeymodel: OptIndex = 159;
pub const kOptKeymap: OptIndex = 158;
pub const kOptJumpoptions: OptIndex = 157;
pub const kOptJoinspaces: OptIndex = 156;
pub const kOptIsprint: OptIndex = 155;
pub const kOptIskeyword: OptIndex = 154;
pub const kOptIsident: OptIndex = 153;
pub const kOptIsfname: OptIndex = 152;
pub const kOptInsertmode: OptIndex = 151;
pub const kOptInfercase: OptIndex = 150;
pub const kOptIndentkeys: OptIndex = 149;
pub const kOptIndentexpr: OptIndex = 148;
pub const kOptIncsearch: OptIndex = 147;
pub const kOptIncludeexpr: OptIndex = 146;
pub const kOptInclude: OptIndex = 145;
pub const kOptInccommand: OptIndex = 144;
pub const kOptImsearch: OptIndex = 143;
pub const kOptIminsert: OptIndex = 142;
pub const kOptImdisable: OptIndex = 141;
pub const kOptImcmdline: OptIndex = 140;
pub const kOptIgnorecase: OptIndex = 139;
pub const kOptIconstring: OptIndex = 138;
pub const kOptIcon: OptIndex = 137;
pub const kOptHlsearch: OptIndex = 136;
pub const kOptHkmapp: OptIndex = 135;
pub const kOptHkmap: OptIndex = 134;
pub const kOptHistory: OptIndex = 133;
pub const kOptHighlight: OptIndex = 132;
pub const kOptHidden: OptIndex = 131;
pub const kOptHelplang: OptIndex = 130;
pub const kOptHelpheight: OptIndex = 129;
pub const kOptHelpfile: OptIndex = 128;
pub const kOptGuitabtooltip: OptIndex = 127;
pub const kOptGuitablabel: OptIndex = 126;
pub const kOptGuioptions: OptIndex = 125;
pub const kOptGuifontwide: OptIndex = 124;
pub const kOptGuifont: OptIndex = 123;
pub const kOptGuicursor: OptIndex = 122;
pub const kOptGrepprg: OptIndex = 121;
pub const kOptGrepformat: OptIndex = 120;
pub const kOptGdefault: OptIndex = 119;
pub const kOptFsync: OptIndex = 118;
pub const kOptFormatprg: OptIndex = 117;
pub const kOptFormatoptions: OptIndex = 116;
pub const kOptFormatlistpat: OptIndex = 115;
pub const kOptFormatexpr: OptIndex = 114;
pub const kOptFoldtext: OptIndex = 113;
pub const kOptFoldopen: OptIndex = 112;
pub const kOptFoldnestmax: OptIndex = 111;
pub const kOptFoldminlines: OptIndex = 110;
pub const kOptFoldmethod: OptIndex = 109;
pub const kOptFoldmarker: OptIndex = 108;
pub const kOptFoldlevelstart: OptIndex = 107;
pub const kOptFoldlevel: OptIndex = 106;
pub const kOptFoldignore: OptIndex = 105;
pub const kOptFoldexpr: OptIndex = 104;
pub const kOptFoldenable: OptIndex = 103;
pub const kOptFoldcolumn: OptIndex = 102;
pub const kOptFoldclose: OptIndex = 101;
pub const kOptFixendofline: OptIndex = 100;
pub const kOptFindfunc: OptIndex = 99;
pub const kOptFillchars: OptIndex = 98;
pub const kOptFiletype: OptIndex = 97;
pub const kOptFileignorecase: OptIndex = 96;
pub const kOptFileformats: OptIndex = 95;
pub const kOptFileformat: OptIndex = 94;
pub const kOptFileencodings: OptIndex = 93;
pub const kOptFileencoding: OptIndex = 92;
pub const kOptExrc: OptIndex = 91;
pub const kOptExpandtab: OptIndex = 90;
pub const kOptEventignorewin: OptIndex = 89;
pub const kOptEventignore: OptIndex = 88;
pub const kOptErrorformat: OptIndex = 87;
pub const kOptErrorfile: OptIndex = 86;
pub const kOptErrorbells: OptIndex = 85;
pub const kOptEqualprg: OptIndex = 84;
pub const kOptEqualalways: OptIndex = 83;
pub const kOptEndofline: OptIndex = 82;
pub const kOptEndoffile: OptIndex = 81;
pub const kOptEncoding: OptIndex = 80;
pub const kOptEmoji: OptIndex = 79;
pub const kOptEdcompatible: OptIndex = 78;
pub const kOptEadirection: OptIndex = 77;
pub const kOptDisplay: OptIndex = 76;
pub const kOptDirectory: OptIndex = 75;
pub const kOptDigraph: OptIndex = 74;
pub const kOptDiffopt: OptIndex = 73;
pub const kOptDiffexpr: OptIndex = 72;
pub const kOptDiffanchors: OptIndex = 71;
pub const kOptDiff: OptIndex = 70;
pub const kOptDictionary: OptIndex = 69;
pub const kOptDelcombine: OptIndex = 68;
pub const kOptDefine: OptIndex = 67;
pub const kOptDebug: OptIndex = 66;
pub const kOptCursorlineopt: OptIndex = 65;
pub const kOptCursorline: OptIndex = 64;
pub const kOptCursorcolumn: OptIndex = 63;
pub const kOptCursorbind: OptIndex = 62;
pub const kOptCpoptions: OptIndex = 61;
pub const kOptCopyindent: OptIndex = 60;
pub const kOptConfirm: OptIndex = 59;
pub const kOptConceallevel: OptIndex = 58;
pub const kOptConcealcursor: OptIndex = 57;
pub const kOptCompletetimeout: OptIndex = 56;
pub const kOptCompleteslash: OptIndex = 55;
pub const kOptCompleteopt: OptIndex = 54;
pub const kOptCompleteitemalign: OptIndex = 53;
pub const kOptCompletefunc: OptIndex = 52;
pub const kOptComplete: OptIndex = 51;
pub const kOptCompatible: OptIndex = 50;
pub const kOptCommentstring: OptIndex = 49;
pub const kOptComments: OptIndex = 48;
pub const kOptColumns: OptIndex = 47;
pub const kOptColorcolumn: OptIndex = 46;
pub const kOptCmdwinheight: OptIndex = 45;
pub const kOptCmdheight: OptIndex = 44;
pub const kOptClipboard: OptIndex = 43;
pub const kOptCinwords: OptIndex = 42;
pub const kOptCinscopedecls: OptIndex = 41;
pub const kOptCinoptions: OptIndex = 40;
pub const kOptCinkeys: OptIndex = 39;
pub const kOptCindent: OptIndex = 38;
pub const kOptChistory: OptIndex = 37;
pub const kOptCharconvert: OptIndex = 36;
pub const kOptChannel: OptIndex = 35;
pub const kOptCedit: OptIndex = 34;
pub const kOptCdpath: OptIndex = 33;
pub const kOptCdhome: OptIndex = 32;
pub const kOptCasemap: OptIndex = 31;
pub const kOptBusy: OptIndex = 30;
pub const kOptBuftype: OptIndex = 29;
pub const kOptBuflisted: OptIndex = 28;
pub const kOptBufhidden: OptIndex = 27;
pub const kOptBrowsedir: OptIndex = 26;
pub const kOptBreakindentopt: OptIndex = 25;
pub const kOptBreakindent: OptIndex = 24;
pub const kOptBreakat: OptIndex = 23;
pub const kOptBomb: OptIndex = 22;
pub const kOptBinary: OptIndex = 21;
pub const kOptBelloff: OptIndex = 20;
pub const kOptBackupskip: OptIndex = 19;
pub const kOptBackupext: OptIndex = 18;
pub const kOptBackupdir: OptIndex = 17;
pub const kOptBackupcopy: OptIndex = 16;
pub const kOptBackup: OptIndex = 15;
pub const kOptBackspace: OptIndex = 14;
pub const kOptBackground: OptIndex = 13;
pub const kOptAutowriteall: OptIndex = 12;
pub const kOptAutowrite: OptIndex = 11;
pub const kOptAutoread: OptIndex = 10;
pub const kOptAutoindent: OptIndex = 9;
pub const kOptAutocompletetimeout: OptIndex = 8;
pub const kOptAutocompletedelay: OptIndex = 7;
pub const kOptAutocomplete: OptIndex = 6;
pub const kOptAutochdir: OptIndex = 5;
pub const kOptArabicshape: OptIndex = 4;
pub const kOptArabic: OptIndex = 3;
pub const kOptAmbiwidth: OptIndex = 2;
pub const kOptAllowrevins: OptIndex = 1;
pub const kOptAleph: OptIndex = 0;
pub const kOptInvalid: OptIndex = -1;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
pub const CMD_USER_BUF: CMD_index = -2;
pub const CMD_USER: CMD_index = -1;
pub const CMD_SIZE: CMD_index = 557;
pub const CMD_Next: CMD_index = 556;
pub const CMD_tilde: CMD_index = 555;
pub const CMD_at: CMD_index = 554;
pub const CMD_rshift: CMD_index = 553;
pub const CMD_equal: CMD_index = 552;
pub const CMD_lshift: CMD_index = 551;
pub const CMD_and: CMD_index = 550;
pub const CMD_pound: CMD_index = 549;
pub const CMD_bang: CMD_index = 548;
pub const CMD_z: CMD_index = 547;
pub const CMD_yank: CMD_index = 546;
pub const CMD_xunmenu: CMD_index = 545;
pub const CMD_xunmap: CMD_index = 544;
pub const CMD_xnoremenu: CMD_index = 543;
pub const CMD_xnoremap: CMD_index = 542;
pub const CMD_xmenu: CMD_index = 541;
pub const CMD_xmapclear: CMD_index = 540;
pub const CMD_xmap: CMD_index = 539;
pub const CMD_xall: CMD_index = 538;
pub const CMD_xit: CMD_index = 537;
pub const CMD_wviminfo: CMD_index = 536;
pub const CMD_wundo: CMD_index = 535;
pub const CMD_wshada: CMD_index = 534;
pub const CMD_wqall: CMD_index = 533;
pub const CMD_wq: CMD_index = 532;
pub const CMD_wprevious: CMD_index = 531;
pub const CMD_wnext: CMD_index = 530;
pub const CMD_winpos: CMD_index = 529;
pub const CMD_windo: CMD_index = 528;
pub const CMD_wincmd: CMD_index = 527;
pub const CMD_winsize: CMD_index = 526;
pub const CMD_while: CMD_index = 525;
pub const CMD_wall: CMD_index = 524;
pub const CMD_wNext: CMD_index = 523;
pub const CMD_write: CMD_index = 522;
pub const CMD_vunmenu: CMD_index = 521;
pub const CMD_vunmap: CMD_index = 520;
pub const CMD_vsplit: CMD_index = 519;
pub const CMD_vnoremenu: CMD_index = 518;
pub const CMD_vnew: CMD_index = 517;
pub const CMD_vnoremap: CMD_index = 516;
pub const CMD_vmenu: CMD_index = 515;
pub const CMD_vmapclear: CMD_index = 514;
pub const CMD_vmap: CMD_index = 513;
pub const CMD_viusage: CMD_index = 512;
pub const CMD_vimgrepadd: CMD_index = 511;
pub const CMD_vimgrep: CMD_index = 510;
pub const CMD_view: CMD_index = 509;
pub const CMD_visual: CMD_index = 508;
pub const CMD_vertical: CMD_index = 507;
pub const CMD_verbose: CMD_index = 506;
pub const CMD_version: CMD_index = 505;
pub const CMD_vglobal: CMD_index = 504;
pub const CMD_update: CMD_index = 503;
pub const CMD_unsilent: CMD_index = 502;
pub const CMD_unmenu: CMD_index = 501;
pub const CMD_unmap: CMD_index = 500;
pub const CMD_unlockvar: CMD_index = 499;
pub const CMD_unlet: CMD_index = 498;
pub const CMD_uniq: CMD_index = 497;
pub const CMD_unhide: CMD_index = 496;
pub const CMD_unabbreviate: CMD_index = 495;
pub const CMD_undolist: CMD_index = 494;
pub const CMD_undojoin: CMD_index = 493;
pub const CMD_undo: CMD_index = 492;
pub const CMD_tunmap: CMD_index = 491;
pub const CMD_tunmenu: CMD_index = 490;
pub const CMD_tselect: CMD_index = 489;
pub const CMD_try: CMD_index = 488;
pub const CMD_trust: CMD_index = 487;
pub const CMD_trewind: CMD_index = 486;
pub const CMD_tprevious: CMD_index = 485;
pub const CMD_topleft: CMD_index = 484;
pub const CMD_tnoremap: CMD_index = 483;
pub const CMD_tnext: CMD_index = 482;
pub const CMD_tmapclear: CMD_index = 481;
pub const CMD_tmap: CMD_index = 480;
pub const CMD_tmenu: CMD_index = 479;
pub const CMD_tlunmenu: CMD_index = 478;
pub const CMD_tlnoremenu: CMD_index = 477;
pub const CMD_tlmenu: CMD_index = 476;
pub const CMD_tlast: CMD_index = 475;
pub const CMD_tjump: CMD_index = 474;
pub const CMD_throw: CMD_index = 473;
pub const CMD_tfirst: CMD_index = 472;
pub const CMD_terminal: CMD_index = 471;
pub const CMD_tclfile: CMD_index = 470;
pub const CMD_tcldo: CMD_index = 469;
pub const CMD_tcl: CMD_index = 468;
pub const CMD_tabs: CMD_index = 467;
pub const CMD_tabrewind: CMD_index = 466;
pub const CMD_tabNext: CMD_index = 465;
pub const CMD_tabprevious: CMD_index = 464;
pub const CMD_tabonly: CMD_index = 463;
pub const CMD_tabnew: CMD_index = 462;
pub const CMD_tabnext: CMD_index = 461;
pub const CMD_tablast: CMD_index = 460;
pub const CMD_tabmove: CMD_index = 459;
pub const CMD_tabfirst: CMD_index = 458;
pub const CMD_tabfind: CMD_index = 457;
pub const CMD_tabedit: CMD_index = 456;
pub const CMD_tabdo: CMD_index = 455;
pub const CMD_tabclose: CMD_index = 454;
pub const CMD_tab: CMD_index = 453;
pub const CMD_tags: CMD_index = 452;
pub const CMD_tag: CMD_index = 451;
pub const CMD_tNext: CMD_index = 450;
pub const CMD_tchdir: CMD_index = 449;
pub const CMD_tcd: CMD_index = 448;
pub const CMD_t: CMD_index = 447;
pub const CMD_syncbind: CMD_index = 446;
pub const CMD_syntime: CMD_index = 445;
pub const CMD_syntax: CMD_index = 444;
pub const CMD_swapname: CMD_index = 443;
pub const CMD_sview: CMD_index = 442;
pub const CMD_suspend: CMD_index = 441;
pub const CMD_sunmenu: CMD_index = 440;
pub const CMD_sunmap: CMD_index = 439;
pub const CMD_sunhide: CMD_index = 438;
pub const CMD_stselect: CMD_index = 437;
pub const CMD_stjump: CMD_index = 436;
pub const CMD_stopinsert: CMD_index = 435;
pub const CMD_startreplace: CMD_index = 434;
pub const CMD_startgreplace: CMD_index = 433;
pub const CMD_startinsert: CMD_index = 432;
pub const CMD_stag: CMD_index = 431;
pub const CMD_stop: CMD_index = 430;
pub const CMD_srewind: CMD_index = 429;
pub const CMD_sprevious: CMD_index = 428;
pub const CMD_spellwrong: CMD_index = 427;
pub const CMD_spellundo: CMD_index = 426;
pub const CMD_spellrare: CMD_index = 425;
pub const CMD_spellrepall: CMD_index = 424;
pub const CMD_spellinfo: CMD_index = 423;
pub const CMD_spelldump: CMD_index = 422;
pub const CMD_spellgood: CMD_index = 421;
pub const CMD_split: CMD_index = 420;
pub const CMD_sort: CMD_index = 419;
pub const CMD_source: CMD_index = 418;
pub const CMD_snoremenu: CMD_index = 417;
pub const CMD_snoremap: CMD_index = 416;
pub const CMD_snomagic: CMD_index = 415;
pub const CMD_snext: CMD_index = 414;
pub const CMD_smenu: CMD_index = 413;
pub const CMD_smapclear: CMD_index = 412;
pub const CMD_smap: CMD_index = 411;
pub const CMD_smagic: CMD_index = 410;
pub const CMD_slast: CMD_index = 409;
pub const CMD_sleep: CMD_index = 408;
pub const CMD_silent: CMD_index = 407;
pub const CMD_sign: CMD_index = 406;
pub const CMD_simalt: CMD_index = 405;
pub const CMD_sfirst: CMD_index = 404;
pub const CMD_sfind: CMD_index = 403;
pub const CMD_setlocal: CMD_index = 402;
pub const CMD_setglobal: CMD_index = 401;
pub const CMD_setfiletype: CMD_index = 400;
pub const CMD_set: CMD_index = 399;
pub const CMD_scriptencoding: CMD_index = 398;
pub const CMD_scriptnames: CMD_index = 397;
pub const CMD_sbrewind: CMD_index = 396;
pub const CMD_sbprevious: CMD_index = 395;
pub const CMD_sbnext: CMD_index = 394;
pub const CMD_sbmodified: CMD_index = 393;
pub const CMD_sblast: CMD_index = 392;
pub const CMD_sbfirst: CMD_index = 391;
pub const CMD_sball: CMD_index = 390;
pub const CMD_sbNext: CMD_index = 389;
pub const CMD_sbuffer: CMD_index = 388;
pub const CMD_saveas: CMD_index = 387;
pub const CMD_sandbox: CMD_index = 386;
pub const CMD_sall: CMD_index = 385;
pub const CMD_sargument: CMD_index = 384;
pub const CMD_sNext: CMD_index = 383;
pub const CMD_substitute: CMD_index = 382;
pub const CMD_rviminfo: CMD_index = 381;
pub const CMD_rubyfile: CMD_index = 380;
pub const CMD_rubydo: CMD_index = 379;
pub const CMD_ruby: CMD_index = 378;
pub const CMD_rundo: CMD_index = 377;
pub const CMD_runtime: CMD_index = 376;
pub const CMD_rshada: CMD_index = 375;
pub const CMD_rightbelow: CMD_index = 374;
pub const CMD_right: CMD_index = 373;
pub const CMD_rewind: CMD_index = 372;
pub const CMD_return: CMD_index = 371;
pub const CMD_retab: CMD_index = 370;
pub const CMD_restart: CMD_index = 369;
pub const CMD_resize: CMD_index = 368;
pub const CMD_registers: CMD_index = 367;
pub const CMD_redrawtabline: CMD_index = 366;
pub const CMD_redrawstatus: CMD_index = 365;
pub const CMD_redraw: CMD_index = 364;
pub const CMD_redir: CMD_index = 363;
pub const CMD_redo: CMD_index = 362;
pub const CMD_recover: CMD_index = 361;
pub const CMD_read: CMD_index = 360;
pub const CMD_qall: CMD_index = 359;
pub const CMD_quitall: CMD_index = 358;
pub const CMD_quit: CMD_index = 357;
pub const CMD_pyxfile: CMD_index = 356;
pub const CMD_pythonx: CMD_index = 355;
pub const CMD_pyxdo: CMD_index = 354;
pub const CMD_pyx: CMD_index = 353;
pub const CMD_py3file: CMD_index = 352;
pub const CMD_python3: CMD_index = 351;
pub const CMD_py3do: CMD_index = 350;
pub const CMD_py3: CMD_index = 349;
pub const CMD_pyfile: CMD_index = 348;
pub const CMD_pydo: CMD_index = 347;
pub const CMD_python: CMD_index = 346;
pub const CMD_pwd: CMD_index = 345;
pub const CMD_put: CMD_index = 344;
pub const CMD_ptselect: CMD_index = 343;
pub const CMD_ptrewind: CMD_index = 342;
pub const CMD_ptprevious: CMD_index = 341;
pub const CMD_ptnext: CMD_index = 340;
pub const CMD_ptlast: CMD_index = 339;
pub const CMD_ptjump: CMD_index = 338;
pub const CMD_ptfirst: CMD_index = 337;
pub const CMD_ptNext: CMD_index = 336;
pub const CMD_ptag: CMD_index = 335;
pub const CMD_psearch: CMD_index = 334;
pub const CMD_profdel: CMD_index = 333;
pub const CMD_profile: CMD_index = 332;
pub const CMD_previous: CMD_index = 331;
pub const CMD_preserve: CMD_index = 330;
pub const CMD_ppop: CMD_index = 329;
pub const CMD_popup: CMD_index = 328;
pub const CMD_pop: CMD_index = 327;
pub const CMD_pedit: CMD_index = 326;
pub const CMD_perlfile: CMD_index = 325;
pub const CMD_perldo: CMD_index = 324;
pub const CMD_perl: CMD_index = 323;
pub const CMD_pclose: CMD_index = 322;
pub const CMD_pbuffer: CMD_index = 321;
pub const CMD_packloadall: CMD_index = 320;
pub const CMD_packadd: CMD_index = 319;
pub const CMD_print: CMD_index = 318;
pub const CMD_ownsyntax: CMD_index = 317;
pub const CMD_ounmenu: CMD_index = 316;
pub const CMD_ounmap: CMD_index = 315;
pub const CMD_options: CMD_index = 314;
pub const CMD_onoremenu: CMD_index = 313;
pub const CMD_onoremap: CMD_index = 312;
pub const CMD_only: CMD_index = 311;
pub const CMD_omenu: CMD_index = 310;
pub const CMD_omapclear: CMD_index = 309;
pub const CMD_omap: CMD_index = 308;
pub const CMD_oldfiles: CMD_index = 307;
pub const CMD_nunmenu: CMD_index = 306;
pub const CMD_nunmap: CMD_index = 305;
pub const CMD_number: CMD_index = 304;
pub const CMD_normal: CMD_index = 303;
pub const CMD_noswapfile: CMD_index = 302;
pub const CMD_noremenu: CMD_index = 301;
pub const CMD_noreabbrev: CMD_index = 300;
pub const CMD_nohlsearch: CMD_index = 299;
pub const CMD_noautocmd: CMD_index = 298;
pub const CMD_noremap: CMD_index = 297;
pub const CMD_nnoremenu: CMD_index = 296;
pub const CMD_nnoremap: CMD_index = 295;
pub const CMD_nmenu: CMD_index = 294;
pub const CMD_nmapclear: CMD_index = 293;
pub const CMD_nmap: CMD_index = 292;
pub const CMD_new: CMD_index = 291;
pub const CMD_next: CMD_index = 290;
pub const CMD_mzfile: CMD_index = 289;
pub const CMD_mzscheme: CMD_index = 288;
pub const CMD_mode: CMD_index = 287;
pub const CMD_mkview: CMD_index = 286;
pub const CMD_mkvimrc: CMD_index = 285;
pub const CMD_mkspell: CMD_index = 284;
pub const CMD_mksession: CMD_index = 283;
pub const CMD_mkexrc: CMD_index = 282;
pub const CMD_messages: CMD_index = 281;
pub const CMD_menutranslate: CMD_index = 280;
pub const CMD_menu: CMD_index = 279;
pub const CMD_match: CMD_index = 278;
pub const CMD_marks: CMD_index = 277;
pub const CMD_mapclear: CMD_index = 276;
pub const CMD_map: CMD_index = 275;
pub const CMD_make: CMD_index = 274;
pub const CMD_mark: CMD_index = 273;
pub const CMD_move: CMD_index = 272;
pub const CMD_lsp: CMD_index = 271;
pub const CMD_ls: CMD_index = 270;
pub const CMD_lwindow: CMD_index = 269;
pub const CMD_lvimgrepadd: CMD_index = 268;
pub const CMD_lvimgrep: CMD_index = 267;
pub const CMD_luafile: CMD_index = 266;
pub const CMD_luado: CMD_index = 265;
pub const CMD_lua: CMD_index = 264;
pub const CMD_lunmap: CMD_index = 263;
pub const CMD_ltag: CMD_index = 262;
pub const CMD_lrewind: CMD_index = 261;
pub const CMD_lpfile: CMD_index = 260;
pub const CMD_lprevious: CMD_index = 259;
pub const CMD_lopen: CMD_index = 258;
pub const CMD_lolder: CMD_index = 257;
pub const CMD_lockvar: CMD_index = 256;
pub const CMD_lockmarks: CMD_index = 255;
pub const CMD_loadkeymap: CMD_index = 254;
pub const CMD_loadview: CMD_index = 253;
pub const CMD_lnfile: CMD_index = 252;
pub const CMD_lnewer: CMD_index = 251;
pub const CMD_lnext: CMD_index = 250;
pub const CMD_lnoremap: CMD_index = 249;
pub const CMD_lmake: CMD_index = 248;
pub const CMD_lmapclear: CMD_index = 247;
pub const CMD_lmap: CMD_index = 246;
pub const CMD_llist: CMD_index = 245;
pub const CMD_llast: CMD_index = 244;
pub const CMD_ll: CMD_index = 243;
pub const CMD_lhistory: CMD_index = 242;
pub const CMD_lhelpgrep: CMD_index = 241;
pub const CMD_lgrepadd: CMD_index = 240;
pub const CMD_lgrep: CMD_index = 239;
pub const CMD_lgetexpr: CMD_index = 238;
pub const CMD_lgetbuffer: CMD_index = 237;
pub const CMD_lgetfile: CMD_index = 236;
pub const CMD_lfirst: CMD_index = 235;
pub const CMD_lfdo: CMD_index = 234;
pub const CMD_lfile: CMD_index = 233;
pub const CMD_lexpr: CMD_index = 232;
pub const CMD_let: CMD_index = 231;
pub const CMD_leftabove: CMD_index = 230;
pub const CMD_left: CMD_index = 229;
pub const CMD_ldo: CMD_index = 228;
pub const CMD_lclose: CMD_index = 227;
pub const CMD_lchdir: CMD_index = 226;
pub const CMD_lcd: CMD_index = 225;
pub const CMD_lbottom: CMD_index = 224;
pub const CMD_lbelow: CMD_index = 223;
pub const CMD_lbefore: CMD_index = 222;
pub const CMD_lbuffer: CMD_index = 221;
pub const CMD_later: CMD_index = 220;
pub const CMD_lafter: CMD_index = 219;
pub const CMD_laddfile: CMD_index = 218;
pub const CMD_laddbuffer: CMD_index = 217;
pub const CMD_laddexpr: CMD_index = 216;
pub const CMD_language: CMD_index = 215;
pub const CMD_labove: CMD_index = 214;
pub const CMD_last: CMD_index = 213;
pub const CMD_lNfile: CMD_index = 212;
pub const CMD_lNext: CMD_index = 211;
pub const CMD_list: CMD_index = 210;
pub const CMD_keepalt: CMD_index = 209;
pub const CMD_keeppatterns: CMD_index = 208;
pub const CMD_keepjumps: CMD_index = 207;
pub const CMD_keepmarks: CMD_index = 206;
pub const CMD_k: CMD_index = 205;
pub const CMD_jumps: CMD_index = 204;
pub const CMD_join: CMD_index = 203;
pub const CMD_iunmenu: CMD_index = 202;
pub const CMD_iunabbrev: CMD_index = 201;
pub const CMD_iunmap: CMD_index = 200;
pub const CMD_isplit: CMD_index = 199;
pub const CMD_isearch: CMD_index = 198;
pub const CMD_iput: CMD_index = 197;
pub const CMD_intro: CMD_index = 196;
pub const CMD_inoremenu: CMD_index = 195;
pub const CMD_inoreabbrev: CMD_index = 194;
pub const CMD_inoremap: CMD_index = 193;
pub const CMD_imenu: CMD_index = 192;
pub const CMD_imapclear: CMD_index = 191;
pub const CMD_imap: CMD_index = 190;
pub const CMD_ilist: CMD_index = 189;
pub const CMD_ijump: CMD_index = 188;
pub const CMD_if: CMD_index = 187;
pub const CMD_iabclear: CMD_index = 186;
pub const CMD_iabbrev: CMD_index = 185;
pub const CMD_insert: CMD_index = 184;
pub const CMD_horizontal: CMD_index = 183;
pub const CMD_history: CMD_index = 182;
pub const CMD_hide: CMD_index = 181;
pub const CMD_highlight: CMD_index = 180;
pub const CMD_helptags: CMD_index = 179;
pub const CMD_helpgrep: CMD_index = 178;
pub const CMD_helpclose: CMD_index = 177;
pub const CMD_help: CMD_index = 176;
pub const CMD_gvim: CMD_index = 175;
pub const CMD_gui: CMD_index = 174;
pub const CMD_grepadd: CMD_index = 173;
pub const CMD_grep: CMD_index = 172;
pub const CMD_goto: CMD_index = 171;
pub const CMD_global: CMD_index = 170;
pub const CMD_fclose: CMD_index = 169;
pub const CMD_function: CMD_index = 168;
pub const CMD_for: CMD_index = 167;
pub const CMD_foldopen: CMD_index = 166;
pub const CMD_folddoclosed: CMD_index = 165;
pub const CMD_folddoopen: CMD_index = 164;
pub const CMD_foldclose: CMD_index = 163;
pub const CMD_fold: CMD_index = 162;
pub const CMD_first: CMD_index = 161;
pub const CMD_finish: CMD_index = 160;
pub const CMD_finally: CMD_index = 159;
pub const CMD_find: CMD_index = 158;
pub const CMD_filter: CMD_index = 157;
pub const CMD_filetype: CMD_index = 156;
pub const CMD_files: CMD_index = 155;
pub const CMD_file: CMD_index = 154;
pub const CMD_exusage: CMD_index = 153;
pub const CMD_exit: CMD_index = 152;
pub const CMD_execute: CMD_index = 151;
pub const CMD_ex: CMD_index = 150;
pub const CMD_eval: CMD_index = 149;
pub const CMD_enew: CMD_index = 148;
pub const CMD_endwhile: CMD_index = 147;
pub const CMD_endtry: CMD_index = 146;
pub const CMD_endfor: CMD_index = 145;
pub const CMD_endfunction: CMD_index = 144;
pub const CMD_endif: CMD_index = 143;
pub const CMD_emenu: CMD_index = 142;
pub const CMD_elseif: CMD_index = 141;
pub const CMD_else: CMD_index = 140;
pub const CMD_echon: CMD_index = 139;
pub const CMD_echomsg: CMD_index = 138;
pub const CMD_echohl: CMD_index = 137;
pub const CMD_echoerr: CMD_index = 136;
pub const CMD_echo: CMD_index = 135;
pub const CMD_earlier: CMD_index = 134;
pub const CMD_edit: CMD_index = 133;
pub const CMD_dsplit: CMD_index = 132;
pub const CMD_dsearch: CMD_index = 131;
pub const CMD_drop: CMD_index = 130;
pub const CMD_doautoall: CMD_index = 129;
pub const CMD_doautocmd: CMD_index = 128;
pub const CMD_dlist: CMD_index = 127;
pub const CMD_djump: CMD_index = 126;
pub const CMD_digraphs: CMD_index = 125;
pub const CMD_diffthis: CMD_index = 124;
pub const CMD_diffsplit: CMD_index = 123;
pub const CMD_diffput: CMD_index = 122;
pub const CMD_diffpatch: CMD_index = 121;
pub const CMD_diffoff: CMD_index = 120;
pub const CMD_diffget: CMD_index = 119;
pub const CMD_diffupdate: CMD_index = 118;
pub const CMD_display: CMD_index = 117;
pub const CMD_detach: CMD_index = 116;
pub const CMD_delfunction: CMD_index = 115;
pub const CMD_delcommand: CMD_index = 114;
pub const CMD_defer: CMD_index = 113;
pub const CMD_debuggreedy: CMD_index = 112;
pub const CMD_debug: CMD_index = 111;
pub const CMD_delmarks: CMD_index = 110;
pub const CMD_delete: CMD_index = 109;
pub const CMD_cwindow: CMD_index = 108;
pub const CMD_cunmenu: CMD_index = 107;
pub const CMD_cunabbrev: CMD_index = 106;
pub const CMD_cunmap: CMD_index = 105;
pub const CMD_crewind: CMD_index = 104;
pub const CMD_cquit: CMD_index = 103;
pub const CMD_cpfile: CMD_index = 102;
pub const CMD_cprevious: CMD_index = 101;
pub const CMD_copen: CMD_index = 100;
pub const CMD_const: CMD_index = 99;
pub const CMD_connect: CMD_index = 98;
pub const CMD_confirm: CMD_index = 97;
pub const CMD_continue: CMD_index = 96;
pub const CMD_compiler: CMD_index = 95;
pub const CMD_comclear: CMD_index = 94;
pub const CMD_command: CMD_index = 93;
pub const CMD_colorscheme: CMD_index = 92;
pub const CMD_colder: CMD_index = 91;
pub const CMD_copy: CMD_index = 90;
pub const CMD_cnoremenu: CMD_index = 89;
pub const CMD_cnoreabbrev: CMD_index = 88;
pub const CMD_cnoremap: CMD_index = 87;
pub const CMD_cnfile: CMD_index = 86;
pub const CMD_cnewer: CMD_index = 85;
pub const CMD_cnext: CMD_index = 84;
pub const CMD_cmenu: CMD_index = 83;
pub const CMD_cmapclear: CMD_index = 82;
pub const CMD_cmap: CMD_index = 81;
pub const CMD_clearjumps: CMD_index = 80;
pub const CMD_close: CMD_index = 79;
pub const CMD_clast: CMD_index = 78;
pub const CMD_clist: CMD_index = 77;
pub const CMD_chistory: CMD_index = 76;
pub const CMD_checktime: CMD_index = 75;
pub const CMD_checkpath: CMD_index = 74;
pub const CMD_checkhealth: CMD_index = 73;
pub const CMD_changes: CMD_index = 72;
pub const CMD_chdir: CMD_index = 71;
pub const CMD_cgetexpr: CMD_index = 70;
pub const CMD_cgetbuffer: CMD_index = 69;
pub const CMD_cgetfile: CMD_index = 68;
pub const CMD_cfirst: CMD_index = 67;
pub const CMD_cfdo: CMD_index = 66;
pub const CMD_cfile: CMD_index = 65;
pub const CMD_cexpr: CMD_index = 64;
pub const CMD_center: CMD_index = 63;
pub const CMD_cdo: CMD_index = 62;
pub const CMD_cd: CMD_index = 61;
pub const CMD_cclose: CMD_index = 60;
pub const CMD_cc: CMD_index = 59;
pub const CMD_cbottom: CMD_index = 58;
pub const CMD_cbelow: CMD_index = 57;
pub const CMD_cbefore: CMD_index = 56;
pub const CMD_cbuffer: CMD_index = 55;
pub const CMD_catch: CMD_index = 54;
pub const CMD_call: CMD_index = 53;
pub const CMD_cafter: CMD_index = 52;
pub const CMD_caddfile: CMD_index = 51;
pub const CMD_caddexpr: CMD_index = 50;
pub const CMD_caddbuffer: CMD_index = 49;
pub const CMD_cabove: CMD_index = 48;
pub const CMD_cabclear: CMD_index = 47;
pub const CMD_cabbrev: CMD_index = 46;
pub const CMD_cNfile: CMD_index = 45;
pub const CMD_cNext: CMD_index = 44;
pub const CMD_change: CMD_index = 43;
pub const CMD_bwipeout: CMD_index = 42;
pub const CMD_bunload: CMD_index = 41;
pub const CMD_bufdo: CMD_index = 40;
pub const CMD_buffers: CMD_index = 39;
pub const CMD_browse: CMD_index = 38;
pub const CMD_breaklist: CMD_index = 37;
pub const CMD_breakdel: CMD_index = 36;
pub const CMD_breakadd: CMD_index = 35;
pub const CMD_break: CMD_index = 34;
pub const CMD_brewind: CMD_index = 33;
pub const CMD_bprevious: CMD_index = 32;
pub const CMD_botright: CMD_index = 31;
pub const CMD_bnext: CMD_index = 30;
pub const CMD_bmodified: CMD_index = 29;
pub const CMD_blast: CMD_index = 28;
pub const CMD_bfirst: CMD_index = 27;
pub const CMD_belowright: CMD_index = 26;
pub const CMD_bdelete: CMD_index = 25;
pub const CMD_balt: CMD_index = 24;
pub const CMD_badd: CMD_index = 23;
pub const CMD_ball: CMD_index = 22;
pub const CMD_bNext: CMD_index = 21;
pub const CMD_buffer: CMD_index = 20;
pub const CMD_aunmenu: CMD_index = 19;
pub const CMD_augroup: CMD_index = 18;
pub const CMD_autocmd: CMD_index = 17;
pub const CMD_ascii: CMD_index = 16;
pub const CMD_argument: CMD_index = 15;
pub const CMD_arglocal: CMD_index = 14;
pub const CMD_argglobal: CMD_index = 13;
pub const CMD_argedit: CMD_index = 12;
pub const CMD_argdedupe: CMD_index = 11;
pub const CMD_argdo: CMD_index = 10;
pub const CMD_argdelete: CMD_index = 9;
pub const CMD_argadd: CMD_index = 8;
pub const CMD_args: CMD_index = 7;
pub const CMD_anoremenu: CMD_index = 6;
pub const CMD_amenu: CMD_index = 5;
pub const CMD_all: CMD_index = 4;
pub const CMD_aboveleft: CMD_index = 3;
pub const CMD_abclear: CMD_index = 2;
pub const CMD_abbreviate: CMD_index = 1;
pub const CMD_append: CMD_index = 0;
pub const ADDR_NONE: cmd_addr_T = 11;
pub const ADDR_OTHER: cmd_addr_T = 10;
pub const ADDR_UNSIGNED: cmd_addr_T = 9;
pub const ADDR_QUICKFIX: cmd_addr_T = 8;
pub const ADDR_QUICKFIX_VALID: cmd_addr_T = 7;
pub const ADDR_TABS_RELATIVE: cmd_addr_T = 6;
pub const ADDR_TABS: cmd_addr_T = 5;
pub const ADDR_BUFFERS: cmd_addr_T = 4;
pub const ADDR_LOADED_BUFFERS: cmd_addr_T = 3;
pub const ADDR_ARGUMENTS: cmd_addr_T = 2;
pub const ADDR_WINDOWS: cmd_addr_T = 1;
pub const ADDR_LINES: cmd_addr_T = 0;
pub const NUM_EVENTS: auto_event = 145;
pub const EVENT_WINSCROLLED: auto_event = 144;
pub const EVENT_WINRESIZED: auto_event = 143;
pub const EVENT_WINNEWPRE: auto_event = 142;
pub const EVENT_WINNEW: auto_event = 141;
pub const EVENT_WINLEAVE: auto_event = 140;
pub const EVENT_WINENTER: auto_event = 139;
pub const EVENT_WINCLOSED: auto_event = 138;
pub const EVENT_VIMSUSPEND: auto_event = 137;
pub const EVENT_VIMRESUME: auto_event = 136;
pub const EVENT_VIMRESIZED: auto_event = 135;
pub const EVENT_VIMLEAVEPRE: auto_event = 134;
pub const EVENT_VIMLEAVE: auto_event = 133;
pub const EVENT_VIMENTER: auto_event = 132;
pub const EVENT_USER: auto_event = 131;
pub const EVENT_UILEAVE: auto_event = 130;
pub const EVENT_UIENTER: auto_event = 129;
pub const EVENT_TEXTYANKPOST: auto_event = 128;
pub const EVENT_TEXTCHANGEDT: auto_event = 127;
pub const EVENT_TEXTCHANGEDP: auto_event = 126;
pub const EVENT_TEXTCHANGEDI: auto_event = 125;
pub const EVENT_TEXTCHANGED: auto_event = 124;
pub const EVENT_TERMRESPONSE: auto_event = 123;
pub const EVENT_TERMREQUEST: auto_event = 122;
pub const EVENT_TERMOPEN: auto_event = 121;
pub const EVENT_TERMLEAVE: auto_event = 120;
pub const EVENT_TERMENTER: auto_event = 119;
pub const EVENT_TERMCLOSE: auto_event = 118;
pub const EVENT_TERMCHANGED: auto_event = 117;
pub const EVENT_TABNEWENTERED: auto_event = 116;
pub const EVENT_TABNEW: auto_event = 115;
pub const EVENT_TABLEAVE: auto_event = 114;
pub const EVENT_TABENTER: auto_event = 113;
pub const EVENT_TABCLOSEDPRE: auto_event = 112;
pub const EVENT_TABCLOSED: auto_event = 111;
pub const EVENT_SYNTAX: auto_event = 110;
pub const EVENT_SWAPEXISTS: auto_event = 109;
pub const EVENT_STDINREADPRE: auto_event = 108;
pub const EVENT_STDINREADPOST: auto_event = 107;
pub const EVENT_SPELLFILEMISSING: auto_event = 106;
pub const EVENT_SOURCEPRE: auto_event = 105;
pub const EVENT_SOURCEPOST: auto_event = 104;
pub const EVENT_SOURCECMD: auto_event = 103;
pub const EVENT_SIGNAL: auto_event = 102;
pub const EVENT_SHELLFILTERPOST: auto_event = 101;
pub const EVENT_SHELLCMDPOST: auto_event = 100;
pub const EVENT_SESSIONWRITEPOST: auto_event = 99;
pub const EVENT_SESSIONLOADPRE: auto_event = 98;
pub const EVENT_SESSIONLOADPOST: auto_event = 97;
pub const EVENT_SEARCHWRAPPED: auto_event = 96;
pub const EVENT_SAFESTATE: auto_event = 95;
pub const EVENT_REMOTEREPLY: auto_event = 94;
pub const EVENT_RECORDINGLEAVE: auto_event = 93;
pub const EVENT_RECORDINGENTER: auto_event = 92;
pub const EVENT_QUITPRE: auto_event = 91;
pub const EVENT_QUICKFIXCMDPRE: auto_event = 90;
pub const EVENT_QUICKFIXCMDPOST: auto_event = 89;
pub const EVENT_PROGRESS: auto_event = 88;
pub const EVENT_PACKCHANGEDPRE: auto_event = 87;
pub const EVENT_PACKCHANGED: auto_event = 86;
pub const EVENT_OPTIONSET: auto_event = 85;
pub const EVENT_MODECHANGED: auto_event = 84;
pub const EVENT_MENUPOPUP: auto_event = 83;
pub const EVENT_MARKSET: auto_event = 82;
pub const EVENT_LSPTOKENUPDATE: auto_event = 81;
pub const EVENT_LSPREQUEST: auto_event = 80;
pub const EVENT_LSPPROGRESS: auto_event = 79;
pub const EVENT_LSPNOTIFY: auto_event = 78;
pub const EVENT_LSPDETACH: auto_event = 77;
pub const EVENT_LSPATTACH: auto_event = 76;
pub const EVENT_INSERTLEAVEPRE: auto_event = 75;
pub const EVENT_INSERTLEAVE: auto_event = 74;
pub const EVENT_INSERTENTER: auto_event = 73;
pub const EVENT_INSERTCHARPRE: auto_event = 72;
pub const EVENT_INSERTCHANGE: auto_event = 71;
pub const EVENT_GUIFAILED: auto_event = 70;
pub const EVENT_GUIENTER: auto_event = 69;
pub const EVENT_FUNCUNDEFINED: auto_event = 68;
pub const EVENT_FOCUSLOST: auto_event = 67;
pub const EVENT_FOCUSGAINED: auto_event = 66;
pub const EVENT_FILTERWRITEPRE: auto_event = 65;
pub const EVENT_FILTERWRITEPOST: auto_event = 64;
pub const EVENT_FILTERREADPRE: auto_event = 63;
pub const EVENT_FILTERREADPOST: auto_event = 62;
pub const EVENT_FILEWRITEPRE: auto_event = 61;
pub const EVENT_FILEWRITEPOST: auto_event = 60;
pub const EVENT_FILEWRITECMD: auto_event = 59;
pub const EVENT_FILETYPE: auto_event = 58;
pub const EVENT_FILEREADPRE: auto_event = 57;
pub const EVENT_FILEREADPOST: auto_event = 56;
pub const EVENT_FILEREADCMD: auto_event = 55;
pub const EVENT_FILEENCODING: auto_event = 54;
pub const EVENT_FILECHANGEDSHELLPOST: auto_event = 53;
pub const EVENT_FILECHANGEDSHELL: auto_event = 52;
pub const EVENT_FILECHANGEDRO: auto_event = 51;
pub const EVENT_FILEAPPENDPRE: auto_event = 50;
pub const EVENT_FILEAPPENDPOST: auto_event = 49;
pub const EVENT_FILEAPPENDCMD: auto_event = 48;
pub const EVENT_EXITPRE: auto_event = 47;
pub const EVENT_ENCODINGCHANGED: auto_event = 46;
pub const EVENT_DIRCHANGEDPRE: auto_event = 45;
pub const EVENT_DIRCHANGED: auto_event = 44;
pub const EVENT_DIFFUPDATED: auto_event = 43;
pub const EVENT_DIAGNOSTICCHANGED: auto_event = 42;
pub const EVENT_CURSORMOVEDI: auto_event = 41;
pub const EVENT_CURSORMOVEDC: auto_event = 40;
pub const EVENT_CURSORMOVED: auto_event = 39;
pub const EVENT_CURSORHOLDI: auto_event = 38;
pub const EVENT_CURSORHOLD: auto_event = 37;
pub const EVENT_COMPLETEDONEPRE: auto_event = 36;
pub const EVENT_COMPLETEDONE: auto_event = 35;
pub const EVENT_COMPLETECHANGED: auto_event = 34;
pub const EVENT_COLORSCHEMEPRE: auto_event = 33;
pub const EVENT_COLORSCHEME: auto_event = 32;
pub const EVENT_CMDWINLEAVE: auto_event = 31;
pub const EVENT_CMDWINENTER: auto_event = 30;
pub const EVENT_CMDUNDEFINED: auto_event = 29;
pub const EVENT_CMDLINELEAVEPRE: auto_event = 28;
pub const EVENT_CMDLINELEAVE: auto_event = 27;
pub const EVENT_CMDLINEENTER: auto_event = 26;
pub const EVENT_CMDLINECHANGED: auto_event = 25;
pub const EVENT_CHANOPEN: auto_event = 24;
pub const EVENT_CHANINFO: auto_event = 23;
pub const EVENT_BUFWRITEPRE: auto_event = 22;
pub const EVENT_BUFWRITEPOST: auto_event = 21;
pub const EVENT_BUFWRITECMD: auto_event = 20;
pub const EVENT_BUFWRITE: auto_event = 19;
pub const EVENT_BUFWIPEOUT: auto_event = 18;
pub const EVENT_BUFWINLEAVE: auto_event = 17;
pub const EVENT_BUFWINENTER: auto_event = 16;
pub const EVENT_BUFUNLOAD: auto_event = 15;
pub const EVENT_BUFREADPRE: auto_event = 14;
pub const EVENT_BUFREADPOST: auto_event = 13;
pub const EVENT_BUFREADCMD: auto_event = 12;
pub const EVENT_BUFREAD: auto_event = 11;
pub const EVENT_BUFNEWFILE: auto_event = 10;
pub const EVENT_BUFNEW: auto_event = 9;
pub const EVENT_BUFMODIFIEDSET: auto_event = 8;
pub const EVENT_BUFLEAVE: auto_event = 7;
pub const EVENT_BUFHIDDEN: auto_event = 6;
pub const EVENT_BUFFILEPRE: auto_event = 5;
pub const EVENT_BUFFILEPOST: auto_event = 4;
pub const EVENT_BUFENTER: auto_event = 3;
pub const EVENT_BUFDELETE: auto_event = 2;
pub const EVENT_BUFCREATE: auto_event = 1;
pub const EVENT_BUFADD: auto_event = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_31 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut aucmdwin_T,
}
pub const BLN_NOCURWIN: bln_values = 128;
pub const BLN_NOOPT: bln_values = 16;
pub const BLN_NEW: bln_values = 8;
pub const BLN_DUMMY: bln_values = 4;
pub const BLN_LISTED: bln_values = 2;
pub const BLN_CURBUF: bln_values = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_36 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut DecorSignHighlight,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_37 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut WinExtmark,
}
pub type C2Rust_Unnamed_38 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_38 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_38 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_38 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_38 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_38 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_38 = 20;
pub const UPD_VALID: C2Rust_Unnamed_38 = 10;
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
pub type C2Rust_Unnamed_39 = ::core::ffi::c_uint;
pub const EVAL_EVALUATE: C2Rust_Unnamed_39 = 1;
pub type C2Rust_Unnamed_40 = ::core::ffi::c_uint;
pub const ECMD_NOWINENTER: C2Rust_Unnamed_40 = 64;
pub const ECMD_ALTBUF: C2Rust_Unnamed_40 = 32;
pub const ECMD_ADDBUF: C2Rust_Unnamed_40 = 16;
pub const ECMD_FORCEIT: C2Rust_Unnamed_40 = 8;
pub const ECMD_OLDBUF: C2Rust_Unnamed_40 = 4;
pub const ECMD_SET_HELP: C2Rust_Unnamed_40 = 2;
pub const ECMD_HIDE: C2Rust_Unnamed_40 = 1;
pub type C2Rust_Unnamed_41 = ::core::ffi::c_int;
pub const ECMD_ONE: C2Rust_Unnamed_41 = 1;
pub const ECMD_LAST: C2Rust_Unnamed_41 = -1;
pub const ECMD_LASTL: C2Rust_Unnamed_41 = 0;
pub type C2Rust_Unnamed_42 = ::core::ffi::c_uint;
pub const READ_NOFILE: C2Rust_Unnamed_42 = 256;
pub const READ_NOWINENTER: C2Rust_Unnamed_42 = 128;
pub const READ_FIFO: C2Rust_Unnamed_42 = 64;
pub const READ_KEEP_UNDO: C2Rust_Unnamed_42 = 32;
pub const READ_DUMMY: C2Rust_Unnamed_42 = 16;
pub const READ_BUFFER: C2Rust_Unnamed_42 = 8;
pub const READ_STDIN: C2Rust_Unnamed_42 = 4;
pub const READ_FILTER: C2Rust_Unnamed_42 = 2;
pub const READ_NEW: C2Rust_Unnamed_42 = 1;
pub const ETYPE_SPELL: etype_T = 9;
pub const ETYPE_INTERNAL: etype_T = 8;
pub const ETYPE_ENV: etype_T = 7;
pub const ETYPE_ARGS: etype_T = 6;
pub const ETYPE_EXCEPT: etype_T = 5;
pub const ETYPE_MODELINE: etype_T = 4;
pub const ETYPE_AUCMD: etype_T = 3;
pub const ETYPE_UFUNC: etype_T = 2;
pub const ETYPE_SCRIPT: etype_T = 1;
pub const ETYPE_TOP: etype_T = 0;
pub type C2Rust_Unnamed_44 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_44 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_44 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_44 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_44 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_44 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_44 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_44 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_44 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_44 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_44 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_44 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_44 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_44 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_44 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_44 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_44 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_44 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_44 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_44 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_44 = 1;
pub type C2Rust_Unnamed_45 = ::core::ffi::c_uint;
pub const kOptCbFlagUnnamedplus: C2Rust_Unnamed_45 = 2;
pub const kOptCbFlagUnnamed: C2Rust_Unnamed_45 = 1;
pub const KE_WILD: key_extra = 108;
pub const KE_COMMAND: key_extra = 104;
pub const KE_LUA: key_extra = 103;
pub const KE_EVENT: key_extra = 102;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_NOP: key_extra = 97;
pub const KE_DROP: key_extra = 95;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_C_END: key_extra = 88;
pub const KE_C_HOME: key_extra = 87;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_CMDWIN: key_extra = 84;
pub const KE_PLUG: key_extra = 83;
pub const KE_SNR: key_extra = 82;
pub const KE_KDEL: key_extra = 80;
pub const KE_KINS: key_extra = 79;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_S_XF4: key_extra = 74;
pub const KE_S_XF3: key_extra = 73;
pub const KE_S_XF2: key_extra = 72;
pub const KE_S_XF1: key_extra = 71;
pub const KE_LEFTRELEASE_NM: key_extra = 70;
pub const KE_LEFTMOUSE_NM: key_extra = 69;
pub const KE_XRIGHT: key_extra = 68;
pub const KE_XLEFT: key_extra = 67;
pub const KE_XDOWN: key_extra = 66;
pub const KE_XUP: key_extra = 65;
pub const KE_ZHOME: key_extra = 64;
pub const KE_XHOME: key_extra = 63;
pub const KE_ZEND: key_extra = 62;
pub const KE_XEND: key_extra = 61;
pub const KE_XF4: key_extra = 60;
pub const KE_XF3: key_extra = 59;
pub const KE_XF2: key_extra = 58;
pub const KE_XF1: key_extra = 57;
pub const KE_S_TAB_OLD: key_extra = 55;
pub const KE_TAB: key_extra = 54;
pub const KE_IGNORE: key_extra = 53;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_RIGHTDRAG: key_extra = 51;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_MIDDLEDRAG: key_extra = 48;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_LEFTMOUSE: key_extra = 44;
pub const KE_MOUSE: key_extra = 43;
pub const KE_S_F37: key_extra = 42;
pub const KE_S_F36: key_extra = 41;
pub const KE_S_F35: key_extra = 40;
pub const KE_S_F34: key_extra = 39;
pub const KE_S_F33: key_extra = 38;
pub const KE_S_F32: key_extra = 37;
pub const KE_S_F31: key_extra = 36;
pub const KE_S_F30: key_extra = 35;
pub const KE_S_F29: key_extra = 34;
pub const KE_S_F28: key_extra = 33;
pub const KE_S_F27: key_extra = 32;
pub const KE_S_F26: key_extra = 31;
pub const KE_S_F25: key_extra = 30;
pub const KE_S_F24: key_extra = 29;
pub const KE_S_F23: key_extra = 28;
pub const KE_S_F22: key_extra = 27;
pub const KE_S_F21: key_extra = 26;
pub const KE_S_F20: key_extra = 25;
pub const KE_S_F19: key_extra = 24;
pub const KE_S_F18: key_extra = 23;
pub const KE_S_F17: key_extra = 22;
pub const KE_S_F16: key_extra = 21;
pub const KE_S_F15: key_extra = 20;
pub const KE_S_F14: key_extra = 19;
pub const KE_S_F13: key_extra = 18;
pub const KE_S_F12: key_extra = 17;
pub const KE_S_F11: key_extra = 16;
pub const KE_S_F10: key_extra = 15;
pub const KE_S_F9: key_extra = 14;
pub const KE_S_F8: key_extra = 13;
pub const KE_S_F7: key_extra = 12;
pub const KE_S_F6: key_extra = 11;
pub const KE_S_F5: key_extra = 10;
pub const KE_S_F4: key_extra = 9;
pub const KE_S_F3: key_extra = 8;
pub const KE_S_F2: key_extra = 7;
pub const KE_S_F1: key_extra = 6;
pub const KE_S_DOWN: key_extra = 5;
pub const KE_S_UP: key_extra = 4;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mparm_T {
    pub argc: ::core::ffi::c_int,
    pub argv: *mut *mut ::core::ffi::c_char,
    pub use_vimrc: *mut ::core::ffi::c_char,
    pub clean: bool,
    pub n_commands: ::core::ffi::c_int,
    pub commands: [*mut ::core::ffi::c_char; 10],
    pub cmds_tofree: [::core::ffi::c_char; 10],
    pub n_pre_commands: ::core::ffi::c_int,
    pub pre_commands: [*mut ::core::ffi::c_char; 10],
    pub luaf: *mut ::core::ffi::c_char,
    pub lua_arg0: ::core::ffi::c_int,
    pub edit_type: ::core::ffi::c_int,
    pub tagname: *mut ::core::ffi::c_char,
    pub use_ef: *mut ::core::ffi::c_char,
    pub input_istext: bool,
    pub no_swap_file: ::core::ffi::c_int,
    pub use_debug_break_level: ::core::ffi::c_int,
    pub window_count: ::core::ffi::c_int,
    pub window_layout: ::core::ffi::c_int,
    pub diff_mode: ::core::ffi::c_int,
    pub listen_addr: *mut ::core::ffi::c_char,
    pub remote: ::core::ffi::c_int,
    pub server_addr: *mut ::core::ffi::c_char,
    pub scriptin: *mut ::core::ffi::c_char,
    pub scriptout: *mut ::core::ffi::c_char,
    pub scriptout_append: bool,
    pub had_stdin_file: bool,
}
pub const EDIT_QF: C2Rust_Unnamed_49 = 4;
pub const WIN_TABS: C2Rust_Unnamed_48 = 3;
pub const WIN_VER: C2Rust_Unnamed_48 = 2;
pub const WIN_HOR: C2Rust_Unnamed_48 = 1;
pub const EDIT_STDIN: C2Rust_Unnamed_49 = 2;
pub const kEqualFiles: file_comparison = 1;
pub const kEqualFileNames: file_comparison = 7;
pub const kOneFileMissing: file_comparison = 6;
pub const kBothFilesMissing: file_comparison = 4;
pub const kDifferentFiles: file_comparison = 2;
pub const DOSO_VIMRC: C2Rust_Unnamed_47 = 1;
pub const DOSO_NONE: C2Rust_Unnamed_47 = 0;
pub const EDIT_FILE: C2Rust_Unnamed_49 = 1;
pub const EDIT_TAG: C2Rust_Unnamed_49 = 3;
pub const EDIT_NONE: C2Rust_Unnamed_49 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_46 {
    pub active: bool,
    pub item: ::core::ffi::c_int,
    pub insert: bool,
    pub finish: bool,
}
pub type C2Rust_Unnamed_47 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_48 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_49 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub static arena_alloc_count: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL_0,
};
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LOGLVL_INF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub static g_min_log_level: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub const SESSION_FILE: [::core::ffi::c_char; 12] =
    unsafe { ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"Session.vim\0") };
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub static global_opt_idx: GlobalCell<[OptIndex; 264]> = GlobalCell::new([
    kOptAleph,
    kOptAllowrevins,
    kOptAmbiwidth,
    kOptArabicshape,
    kOptAutochdir,
    kOptAutocomplete,
    kOptAutocompletedelay,
    kOptAutocompletetimeout,
    kOptAutoread,
    kOptAutowrite,
    kOptAutowriteall,
    kOptBackground,
    kOptBackspace,
    kOptBackup,
    kOptBackupcopy,
    kOptBackupdir,
    kOptBackupext,
    kOptBackupskip,
    kOptBelloff,
    kOptBreakat,
    kOptBrowsedir,
    kOptCasemap,
    kOptCdhome,
    kOptCdpath,
    kOptCedit,
    kOptCharconvert,
    kOptChistory,
    kOptClipboard,
    kOptCmdheight,
    kOptCmdwinheight,
    kOptColumns,
    kOptCompatible,
    kOptCompleteitemalign,
    kOptCompleteopt,
    kOptCompletetimeout,
    kOptConfirm,
    kOptCpoptions,
    kOptDebug,
    kOptDefine,
    kOptDelcombine,
    kOptDictionary,
    kOptDiffanchors,
    kOptDiffexpr,
    kOptDiffopt,
    kOptDigraph,
    kOptDirectory,
    kOptDisplay,
    kOptEadirection,
    kOptEdcompatible,
    kOptEmoji,
    kOptEncoding,
    kOptEqualalways,
    kOptEqualprg,
    kOptErrorbells,
    kOptErrorfile,
    kOptErrorformat,
    kOptEventignore,
    kOptExrc,
    kOptFileencodings,
    kOptFileformats,
    kOptFileignorecase,
    kOptFillchars,
    kOptFindfunc,
    kOptFoldclose,
    kOptFoldlevelstart,
    kOptFoldopen,
    kOptFormatprg,
    kOptFsync,
    kOptGdefault,
    kOptGrepformat,
    kOptGrepprg,
    kOptGuicursor,
    kOptGuifont,
    kOptGuifontwide,
    kOptGuioptions,
    kOptGuitablabel,
    kOptGuitabtooltip,
    kOptHelpfile,
    kOptHelpheight,
    kOptHelplang,
    kOptHidden,
    kOptHighlight,
    kOptHistory,
    kOptHkmap,
    kOptHkmapp,
    kOptHlsearch,
    kOptIcon,
    kOptIconstring,
    kOptIgnorecase,
    kOptImcmdline,
    kOptImdisable,
    kOptInccommand,
    kOptInclude,
    kOptIncsearch,
    kOptInsertmode,
    kOptIsfname,
    kOptIsident,
    kOptIsprint,
    kOptJoinspaces,
    kOptJumpoptions,
    kOptKeymodel,
    kOptKeywordprg,
    kOptLangmap,
    kOptLangmenu,
    kOptLangnoremap,
    kOptLangremap,
    kOptLaststatus,
    kOptLazyredraw,
    kOptLines,
    kOptLinespace,
    kOptLispwords,
    kOptListchars,
    kOptLoadplugins,
    kOptMagic,
    kOptMakeef,
    kOptMakeencoding,
    kOptMakeprg,
    kOptMatchtime,
    kOptMaxcombine,
    kOptMaxfuncdepth,
    kOptMaxmapdepth,
    kOptMaxmempattern,
    kOptMaxsearchcount,
    kOptMenuitems,
    kOptMessagesopt,
    kOptMkspellmem,
    kOptModelineexpr,
    kOptModelines,
    kOptMore,
    kOptMouse,
    kOptMousefocus,
    kOptMousehide,
    kOptMousemodel,
    kOptMousemoveevent,
    kOptMousescroll,
    kOptMouseshape,
    kOptMousetime,
    kOptOpendevice,
    kOptOperatorfunc,
    kOptPackpath,
    kOptParagraphs,
    kOptPaste,
    kOptPastetoggle,
    kOptPatchexpr,
    kOptPatchmode,
    kOptPath,
    kOptPreviewheight,
    kOptPrompt,
    kOptPumblend,
    kOptPumborder,
    kOptPumheight,
    kOptPummaxwidth,
    kOptPumwidth,
    kOptPyxversion,
    kOptQuickfixtextfunc,
    kOptRedrawdebug,
    kOptRedrawtime,
    kOptRegexpengine,
    kOptRemap,
    kOptReport,
    kOptRevins,
    kOptRuler,
    kOptRulerformat,
    kOptRuntimepath,
    kOptScrolljump,
    kOptScrolloff,
    kOptScrollopt,
    kOptSections,
    kOptSecure,
    kOptSelection,
    kOptSelectmode,
    kOptSessionoptions,
    kOptShada,
    kOptShadafile,
    kOptShell,
    kOptShellcmdflag,
    kOptShellpipe,
    kOptShellquote,
    kOptShellredir,
    kOptShellslash,
    kOptShelltemp,
    kOptShellxescape,
    kOptShellxquote,
    kOptShiftround,
    kOptShortmess,
    kOptShowbreak,
    kOptShowcmd,
    kOptShowcmdloc,
    kOptShowfulltag,
    kOptShowmatch,
    kOptShowmode,
    kOptShowtabline,
    kOptSidescroll,
    kOptSidescrolloff,
    kOptSmartcase,
    kOptSmarttab,
    kOptSpellsuggest,
    kOptSplitbelow,
    kOptSplitkeep,
    kOptSplitright,
    kOptStartofline,
    kOptStatusline,
    kOptSuffixes,
    kOptSwitchbuf,
    kOptTabclose,
    kOptTabline,
    kOptTabpagemax,
    kOptTagbsearch,
    kOptTagcase,
    kOptTaglength,
    kOptTagrelative,
    kOptTags,
    kOptTagstack,
    kOptTermbidi,
    kOptTermencoding,
    kOptTermguicolors,
    kOptTermpastefilter,
    kOptTermsync,
    kOptTerse,
    kOptThesaurus,
    kOptThesaurusfunc,
    kOptTildeop,
    kOptTimeout,
    kOptTimeoutlen,
    kOptTitle,
    kOptTitlelen,
    kOptTitleold,
    kOptTitlestring,
    kOptTtimeout,
    kOptTtimeoutlen,
    kOptTtyfast,
    kOptUndodir,
    kOptUndolevels,
    kOptUndoreload,
    kOptUpdatecount,
    kOptUpdatetime,
    kOptVerbose,
    kOptVerbosefile,
    kOptViewdir,
    kOptViewoptions,
    kOptVirtualedit,
    kOptVisualbell,
    kOptWarn,
    kOptWhichwrap,
    kOptWildchar,
    kOptWildcharm,
    kOptWildignore,
    kOptWildignorecase,
    kOptWildmenu,
    kOptWildmode,
    kOptWildoptions,
    kOptWinaltkeys,
    kOptWinbar,
    kOptWinborder,
    kOptWindow,
    kOptWinheight,
    kOptWinminheight,
    kOptWinminwidth,
    kOptWinwidth,
    kOptWrapscan,
    kOptWrite,
    kOptWriteany,
    kOptWritebackup,
    kOptWritedelay,
]);
pub static buf_opt_idx: GlobalCell<[OptIndex; 92]> = GlobalCell::new([
    kOptAutocomplete,
    kOptAutoindent,
    kOptAutoread,
    kOptBackupcopy,
    kOptBinary,
    kOptBomb,
    kOptBufhidden,
    kOptBuflisted,
    kOptBuftype,
    kOptBusy,
    kOptChannel,
    kOptCindent,
    kOptCinkeys,
    kOptCinoptions,
    kOptCinscopedecls,
    kOptCinwords,
    kOptComments,
    kOptCommentstring,
    kOptComplete,
    kOptCompletefunc,
    kOptCompleteopt,
    kOptCompleteslash,
    kOptCopyindent,
    kOptDefine,
    kOptDictionary,
    kOptDiffanchors,
    kOptEndoffile,
    kOptEndofline,
    kOptEqualprg,
    kOptErrorformat,
    kOptExpandtab,
    kOptFileencoding,
    kOptFileformat,
    kOptFiletype,
    kOptFindfunc,
    kOptFixendofline,
    kOptFormatexpr,
    kOptFormatlistpat,
    kOptFormatoptions,
    kOptFormatprg,
    kOptFsync,
    kOptGrepformat,
    kOptGrepprg,
    kOptIminsert,
    kOptImsearch,
    kOptInclude,
    kOptIncludeexpr,
    kOptIndentexpr,
    kOptIndentkeys,
    kOptInfercase,
    kOptIskeyword,
    kOptKeymap,
    kOptKeywordprg,
    kOptLisp,
    kOptLispoptions,
    kOptLispwords,
    kOptMakeencoding,
    kOptMakeprg,
    kOptMatchpairs,
    kOptModeline,
    kOptModifiable,
    kOptModified,
    kOptNrformats,
    kOptOmnifunc,
    kOptPath,
    kOptPreserveindent,
    kOptQuoteescape,
    kOptReadonly,
    kOptScrollback,
    kOptShiftwidth,
    kOptSmartindent,
    kOptSofttabstop,
    kOptSpellcapcheck,
    kOptSpellfile,
    kOptSpelllang,
    kOptSpelloptions,
    kOptSuffixesadd,
    kOptSwapfile,
    kOptSynmaxcol,
    kOptSyntax,
    kOptTabstop,
    kOptTagcase,
    kOptTagfunc,
    kOptTags,
    kOptTextwidth,
    kOptThesaurus,
    kOptThesaurusfunc,
    kOptUndofile,
    kOptUndolevels,
    kOptVarsofttabstop,
    kOptVartabstop,
    kOptWrapmargin,
]);
pub static win_opt_idx: GlobalCell<[OptIndex; 51]> = GlobalCell::new([
    kOptArabic,
    kOptBreakindent,
    kOptBreakindentopt,
    kOptColorcolumn,
    kOptConcealcursor,
    kOptConceallevel,
    kOptCursorbind,
    kOptCursorcolumn,
    kOptCursorline,
    kOptCursorlineopt,
    kOptDiff,
    kOptEventignorewin,
    kOptFillchars,
    kOptFoldcolumn,
    kOptFoldenable,
    kOptFoldexpr,
    kOptFoldignore,
    kOptFoldlevel,
    kOptFoldmarker,
    kOptFoldmethod,
    kOptFoldminlines,
    kOptFoldnestmax,
    kOptFoldtext,
    kOptLhistory,
    kOptLinebreak,
    kOptList,
    kOptListchars,
    kOptNumber,
    kOptNumberwidth,
    kOptPreviewwindow,
    kOptRelativenumber,
    kOptRightleft,
    kOptRightleftcmd,
    kOptScroll,
    kOptScrollbind,
    kOptScrolloff,
    kOptShowbreak,
    kOptSidescrolloff,
    kOptSigncolumn,
    kOptSmoothscroll,
    kOptSpell,
    kOptStatuscolumn,
    kOptStatusline,
    kOptVirtualedit,
    kOptWinbar,
    kOptWinblend,
    kOptWinfixbuf,
    kOptWinfixheight,
    kOptWinfixwidth,
    kOptWinhighlight,
    kOptWrap,
]);
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
    values: ::core::ptr::null_mut::<::core::ffi::c_int>(),
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
        keys: ::core::ptr::null_mut::<::core::ffi::c_int>(),
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
        keys: ::core::ptr::null_mut::<::core::ffi::c_int>(),
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
        keys: ::core::ptr::null_mut::<::core::ffi::c_int>(),
    },
    values: ::core::ptr::null_mut::<ptr_t>(),
});
pub static ui_ext_names: GlobalCell<[*const ::core::ffi::c_char; 10]> = GlobalCell::new([
    b"ext_cmdline\0".as_ptr() as *const ::core::ffi::c_char,
    b"ext_popupmenu\0".as_ptr() as *const ::core::ffi::c_char,
    b"ext_tabline\0".as_ptr() as *const ::core::ffi::c_char,
    b"ext_wildmenu\0".as_ptr() as *const ::core::ffi::c_char,
    b"ext_messages\0".as_ptr() as *const ::core::ffi::c_char,
    b"ext_linegrid\0".as_ptr() as *const ::core::ffi::c_char,
    b"ext_multigrid\0".as_ptr() as *const ::core::ffi::c_char,
    b"ext_hlstate\0".as_ptr() as *const ::core::ffi::c_char,
    b"ext_termcolors\0".as_ptr() as *const ::core::ffi::c_char,
    b"_debug_float\0".as_ptr() as *const ::core::ffi::c_char,
]);
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub static last_cursormoved_win: GlobalCell<*mut win_T> =
    GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static last_cursormoved: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0 as linenr_T,
    col: 0 as colnr_T,
    coladd: 0 as colnr_T,
});
pub static autocmd_busy: GlobalCell<bool> = GlobalCell::new(false);
pub static autocmd_no_enter: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static autocmd_no_leave: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static au_new_curbuf: GlobalCell<bufref_T> = GlobalCell::new(bufref_T {
    br_buf: ::core::ptr::null_mut::<buf_T>(),
    br_fnum: 0 as ::core::ffi::c_int,
    br_buf_free_count: 0 as ::core::ffi::c_int,
});
pub static au_pending_free_buf: GlobalCell<*mut buf_T> =
    GlobalCell::new(::core::ptr::null_mut::<buf_T>());
pub static au_pending_free_win: GlobalCell<*mut win_T> =
    GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static autocmd_fname: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static autocmd_fname_full: GlobalCell<bool> = GlobalCell::new(false);
pub static autocmd_bufnr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static autocmd_match: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static did_cursorhold: GlobalCell<bool> = GlobalCell::new(true);
#[no_mangle]
pub static aucmd_win_vec: GlobalCell<C2Rust_Unnamed_31> = GlobalCell::new(C2Rust_Unnamed_31 {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<aucmdwin_T>(),
});
pub static deferred_events: GlobalCell<*mut MultiQueue> =
    GlobalCell::new(::core::ptr::null_mut::<MultiQueue>());
pub static msg_loclist: GlobalCell<*mut ::core::ffi::c_char> = GlobalCell::new(
    b"[Location List]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
);
pub static msg_qflist: GlobalCell<*mut ::core::ffi::c_char> = GlobalCell::new(
    b"[Quickfix List]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
);
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
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
    data: C2Rust_Unnamed_5 {
        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    type_0: kCallbackNone,
});
pub static virt_text_pos_str: GlobalCell<[*const ::core::ffi::c_char; 6]> = GlobalCell::new([
    b"eol\0".as_ptr() as *const ::core::ffi::c_char,
    b"eol_right_align\0".as_ptr() as *const ::core::ffi::c_char,
    b"inline\0".as_ptr() as *const ::core::ffi::c_char,
    b"overlay\0".as_ptr() as *const ::core::ffi::c_char,
    b"right_align\0".as_ptr() as *const ::core::ffi::c_char,
    b"win_col\0".as_ptr() as *const ::core::ffi::c_char,
]);
pub static hl_mode_str: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"\0".as_ptr() as *const ::core::ffi::c_char,
    b"replace\0".as_ptr() as *const ::core::ffi::c_char,
    b"combine\0".as_ptr() as *const ::core::ffi::c_char,
    b"blend\0".as_ptr() as *const ::core::ffi::c_char,
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
        s: [C2Rust_Unnamed_29 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }],
    slots: C2Rust_Unnamed_35 {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<DecorRangeSlot>(),
    },
    ranges_i: C2Rust_Unnamed_34 {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<::core::ffi::c_int>(),
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
#[no_mangle]
pub static decor_items: GlobalCell<C2Rust_Unnamed_36> = GlobalCell::new(C2Rust_Unnamed_36 {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<DecorSignHighlight>(),
});
pub static diff_context: GlobalCell<::core::ffi::c_int> = GlobalCell::new(6 as ::core::ffi::c_int);
pub static diff_foldcolumn: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(2 as ::core::ffi::c_int);
pub static diff_need_scrollbind: GlobalCell<bool> = GlobalCell::new(false);
pub static need_diff_redraw: GlobalCell<bool> = GlobalCell::new(false);
#[no_mangle]
pub static win_extmark_arr: GlobalCell<C2Rust_Unnamed_37> = GlobalCell::new(C2Rust_Unnamed_37 {
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
pub static e_abort: GlobalCell<[::core::ffi::c_char; 22]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"E470: Command aborted\0")
});
pub static e_afterinit: GlobalCell<[::core::ffi::c_char; 43]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 43], [::core::ffi::c_char; 43]>(
        *b"E905: Cannot set this option after startup\0",
    )
});
pub static e_api_spawn_failed: GlobalCell<[::core::ffi::c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E903: Could not spawn API job\0",
    )
});
pub static e_argreq: GlobalCell<[::core::ffi::c_char; 24]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E471: Argument required\0")
});
pub static e_backslash: GlobalCell<[::core::ffi::c_char; 39]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
        *b"E10: \\ should be followed by /, ? or &\0",
    )
});
pub static e_cmdwin: GlobalCell<[::core::ffi::c_char; 65]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 65], [::core::ffi::c_char; 65]>(
        *b"E11: Invalid in command-line window; <CR> executes, CTRL-C quits\0",
    )
});
pub static e_curdir: GlobalCell<[::core::ffi::c_char; 69]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 69], [::core::ffi::c_char; 69]>(
        *b"E12: Command not allowed in secure mode in current dir or tag search\0",
    )
});
pub static e_invalid_buffer_name_str: GlobalCell<[::core::ffi::c_char; 30]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
            *b"E158: Invalid buffer name: %s\0",
        )
    });
pub static e_command_too_recursive: GlobalCell<[::core::ffi::c_char; 28]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(
            *b"E169: Command too recursive\0",
        )
    });
pub static e_buffer_is_not_loaded: GlobalCell<[::core::ffi::c_char; 27]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(
            *b"E681: Buffer is not loaded\0",
        )
    });
pub static e_endif: GlobalCell<[::core::ffi::c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"E171: Missing :endif\0")
});
pub static e_endtry: GlobalCell<[::core::ffi::c_char; 22]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"E600: Missing :endtry\0")
});
pub static e_endwhile: GlobalCell<[::core::ffi::c_char; 24]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E170: Missing :endwhile\0")
});
pub static e_endfor: GlobalCell<[::core::ffi::c_char; 22]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"E170: Missing :endfor\0")
});
pub static e_while: GlobalCell<[::core::ffi::c_char; 31]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"E588: :endwhile without :while\0",
    )
});
pub static e_for: GlobalCell<[::core::ffi::c_char; 27]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E588: :endfor without :for\0")
});
pub static e_exists: GlobalCell<[::core::ffi::c_char; 37]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"E13: File exists (add ! to override)\0",
    )
});
pub static e_failed: GlobalCell<[::core::ffi::c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"E472: Command failed\0")
});
pub static e_intern2: GlobalCell<[::core::ffi::c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E685: Internal error: %s\0")
});
pub static e_interr: GlobalCell<[::core::ffi::c_char; 12]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"Interrupted\0")
});
pub static e_invarg: GlobalCell<[::core::ffi::c_char; 23]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"E474: Invalid argument\0")
});
pub static e_invarg2: GlobalCell<[::core::ffi::c_char; 27]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E475: Invalid argument: %s\0")
});
pub static e_invargval: GlobalCell<[::core::ffi::c_char; 36]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"E475: Invalid value for argument %s\0",
    )
});
pub static e_invargNval: GlobalCell<[::core::ffi::c_char; 40]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"E475: Invalid value for argument %s: %s\0",
    )
});
pub static e_duparg2: GlobalCell<[::core::ffi::c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E983: Duplicate argument: %s\0",
    )
});
pub static e_invexpr2: GlobalCell<[::core::ffi::c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E15: Invalid expression: \"%s\"\0",
    )
});
pub static e_invrange: GlobalCell<[::core::ffi::c_char; 19]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"E16: Invalid range\0")
});
pub static e_internal_error_in_regexp: GlobalCell<[::core::ffi::c_char; 31]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
            *b"E473: Internal error in regexp\0",
        )
    });
pub static e_invcmd: GlobalCell<[::core::ffi::c_char; 22]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"E476: Invalid command\0")
});
pub static e_isadir2: GlobalCell<[::core::ffi::c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E17: \"%s\" is a directory\0")
});
pub static e_no_spell: GlobalCell<[::core::ffi::c_char; 37]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"E756: Spell checking is not possible\0",
    )
});
pub static e_invchan: GlobalCell<[::core::ffi::c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E900: Invalid channel id\0")
});
pub static e_invchanjob: GlobalCell<[::core::ffi::c_char; 36]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"E900: Invalid channel id: not a job\0",
    )
});
pub static e_jobtblfull: GlobalCell<[::core::ffi::c_char; 24]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E901: Job table is full\0")
});
pub static e_jobspawn: GlobalCell<[::core::ffi::c_char; 40]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"E903: Process failed to start: %s: \"%s\"\0",
    )
});
pub static e_channotpty: GlobalCell<[::core::ffi::c_char; 27]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E904: channel is not a pty\0")
});
pub static e_stdiochan2: GlobalCell<[::core::ffi::c_char; 38]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"E905: Couldn't open stdio channel: %s\0",
    )
});
pub static e_invstream: GlobalCell<[::core::ffi::c_char; 33]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
        *b"E906: invalid stream for channel\0",
    )
});
pub static e_invstreamrpc: GlobalCell<[::core::ffi::c_char; 48]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 48], [::core::ffi::c_char; 48]>(
        *b"E906: invalid stream for rpc channel, use 'rpc'\0",
    )
});
pub static e_streamkey: GlobalCell<[::core::ffi::c_char; 68]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 68], [::core::ffi::c_char; 68]>(
        *b"E5210: dict key '%s' already set for buffered stream in channel %lu\0",
    )
});
pub static e_libcall: GlobalCell<[::core::ffi::c_char; 37]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"E364: Library call failed for \"%s()\"\0",
    )
});
pub static e_fsync: GlobalCell<[::core::ffi::c_char; 23]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"E667: Fsync failed: %s\0")
});
pub static e_mkdir: GlobalCell<[::core::ffi::c_char; 37]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"E739: Cannot create directory %s: %s\0",
    )
});
pub static e_markinval: GlobalCell<[::core::ffi::c_char; 34]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"E19: Mark has invalid line number\0",
    )
});
pub static e_marknotset: GlobalCell<[::core::ffi::c_char; 18]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"E20: Mark not set\0")
});
pub static e_modifiable: GlobalCell<[::core::ffi::c_char; 46]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 46], [::core::ffi::c_char; 46]>(
        *b"E21: Cannot make changes, 'modifiable' is off\0",
    )
});
pub static e_nesting: GlobalCell<[::core::ffi::c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E22: Scripts nested too deep\0",
    )
});
pub static e_noalt: GlobalCell<[::core::ffi::c_char; 23]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"E23: No alternate file\0")
});
pub static e_noabbr: GlobalCell<[::core::ffi::c_char; 26]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E24: No such abbreviation\0")
});
pub static e_nobang: GlobalCell<[::core::ffi::c_char; 19]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"E477: No ! allowed\0")
});
pub static e_nogroup: GlobalCell<[::core::ffi::c_char; 38]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"E28: No such highlight group name: %s\0",
    )
});
pub static e_noinstext: GlobalCell<[::core::ffi::c_char; 26]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E29: No inserted text yet\0")
});
pub static e_nolastcmd: GlobalCell<[::core::ffi::c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E30: No previous command line\0",
    )
});
pub static e_nomap: GlobalCell<[::core::ffi::c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"E31: No such mapping\0")
});
pub static e_noident: GlobalCell<[::core::ffi::c_char; 33]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
        *b"E349: No identifier under cursor\0",
    )
});
pub static e_nomatch: GlobalCell<[::core::ffi::c_char; 15]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"E479: No match\0")
});
pub static e_nomatch2: GlobalCell<[::core::ffi::c_char; 19]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"E480: No match: %s\0")
});
pub static e_noname: GlobalCell<[::core::ffi::c_char; 18]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"E32: No file name\0")
});
pub static e_nopresub: GlobalCell<[::core::ffi::c_char; 47]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 47], [::core::ffi::c_char; 47]>(
        *b"E33: No previous substitute regular expression\0",
    )
});
pub static e_noprev: GlobalCell<[::core::ffi::c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E34: No previous command\0")
});
pub static e_noprevre: GlobalCell<[::core::ffi::c_char; 36]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"E35: No previous regular expression\0",
    )
});
pub static e_norange: GlobalCell<[::core::ffi::c_char; 23]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"E481: No range allowed\0")
});
pub static e_noroom: GlobalCell<[::core::ffi::c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"E36: Not enough room\0")
});
pub static e_notmp: GlobalCell<[::core::ffi::c_char; 31]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"E483: Can't get temp file name\0",
    )
});
pub static e_notopen: GlobalCell<[::core::ffi::c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E484: Can't open file %s\0")
});
pub static e_notopen_2: GlobalCell<[::core::ffi::c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E484: Can't open file %s: %s\0",
    )
});
pub static e_cant_read_file_str: GlobalCell<[::core::ffi::c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E485: Can't read file %s\0")
});
pub static e_null: GlobalCell<[::core::ffi::c_char; 19]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"E38: Null argument\0")
});
pub static e_number_exp: GlobalCell<[::core::ffi::c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"E39: Number expected\0")
});
pub static e_openerrf: GlobalCell<[::core::ffi::c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E40: Can't open errorfile %s\0",
    )
});
pub static e_outofmem: GlobalCell<[::core::ffi::c_char; 20]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"E41: Out of memory!\0")
});
pub static e_patnotf: GlobalCell<[::core::ffi::c_char; 18]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"Pattern not found\0")
});
pub static e_patnotf2: GlobalCell<[::core::ffi::c_char; 28]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"E486: Pattern not found: %s\0")
});
pub static e_positive: GlobalCell<[::core::ffi::c_char; 32]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
        *b"E487: Argument must be positive\0",
    )
});
pub static e_prev_dir: GlobalCell<[::core::ffi::c_char; 43]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 43], [::core::ffi::c_char; 43]>(
        *b"E459: Cannot go back to previous directory\0",
    )
});
pub static e_no_errors: GlobalCell<[::core::ffi::c_char; 15]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"E42: No Errors\0")
});
pub static e_loclist: GlobalCell<[::core::ffi::c_char; 23]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"E776: No location list\0")
});
pub static e_re_damg: GlobalCell<[::core::ffi::c_char; 26]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E43: Damaged match string\0")
});
pub static e_re_corr: GlobalCell<[::core::ffi::c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E44: Corrupted regexp program\0",
    )
});
pub static e_readonly: GlobalCell<[::core::ffi::c_char; 50]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 50], [::core::ffi::c_char; 50]>(
        *b"E45: 'readonly' option is set (add ! to override)\0",
    )
});
pub static e_letwrong: GlobalCell<[::core::ffi::c_char; 34]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"E734: Wrong variable type for %s=\0",
    )
});
pub static e_illvar: GlobalCell<[::core::ffi::c_char; 32]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
        *b"E461: Illegal variable name: %s\0",
    )
});
pub static e_cannot_mod: GlobalCell<[::core::ffi::c_char; 38]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"E995: Cannot modify existing variable\0",
    )
});
pub static e_cannot_change_readonly_variable_str: GlobalCell<[::core::ffi::c_char; 45]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
            *b"E46: Cannot change read-only variable \"%.*s\"\0",
        )
    });
pub static e_dictreq: GlobalCell<[::core::ffi::c_char; 26]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E715: Dictionary required\0")
});
pub static e_blobidx: GlobalCell<[::core::ffi::c_char; 35]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
        *b"E979: Blob index out of range: %ld\0",
    )
});
pub static e_invalblob: GlobalCell<[::core::ffi::c_char; 33]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
        *b"E978: Invalid operation for Blob\0",
    )
});
pub static e_toomanyarg: GlobalCell<[::core::ffi::c_char; 42]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 42], [::core::ffi::c_char; 42]>(
        *b"E118: Too many arguments for function: %s\0",
    )
});
pub static e_toofewarg: GlobalCell<[::core::ffi::c_char; 44]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
        *b"E119: Not enough arguments for function: %s\0",
    )
});
pub static e_dictkey: GlobalCell<[::core::ffi::c_char; 42]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 42], [::core::ffi::c_char; 42]>(
        *b"E716: Key not present in Dictionary: \"%s\"\0",
    )
});
pub static e_dictkey_len: GlobalCell<[::core::ffi::c_char; 44]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
        *b"E716: Key not present in Dictionary: \"%.*s\"\0",
    )
});
pub static e_listreq: GlobalCell<[::core::ffi::c_char; 20]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"E714: List required\0")
});
pub static e_listblobreq: GlobalCell<[::core::ffi::c_char; 28]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"E897: List or Blob required\0")
});
pub static e_listblobarg: GlobalCell<[::core::ffi::c_char; 44]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
        *b"E899: Argument of %s must be a List or Blob\0",
    )
});
pub static e_listdictarg: GlobalCell<[::core::ffi::c_char; 50]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 50], [::core::ffi::c_char; 50]>(
        *b"E712: Argument of %s must be a List or Dictionary\0",
    )
});
pub static e_listdictblobarg: GlobalCell<[::core::ffi::c_char; 56]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 56], [::core::ffi::c_char; 56]>(
        *b"E896: Argument of %s must be a List, Dictionary or Blob\0",
    )
});
pub static e_readerrf: GlobalCell<[::core::ffi::c_char; 35]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
        *b"E47: Error while reading errorfile\0",
    )
});
pub static e_sandbox: GlobalCell<[::core::ffi::c_char; 28]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"E48: Not allowed in sandbox\0")
});
pub static e_secure: GlobalCell<[::core::ffi::c_char; 23]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"E523: Not allowed here\0")
});
pub static e_textlock: GlobalCell<[::core::ffi::c_char; 50]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 50], [::core::ffi::c_char; 50]>(
        *b"E565: Not allowed to change text or change window\0",
    )
});
pub static e_screenmode: GlobalCell<[::core::ffi::c_char; 40]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"E359: Screen mode setting not supported\0",
    )
});
pub static e_scroll: GlobalCell<[::core::ffi::c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E49: Invalid scroll size\0")
});
pub static e_shellempty: GlobalCell<[::core::ffi::c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E91: 'shell' option is empty\0",
    )
});
pub static e_signdata: GlobalCell<[::core::ffi::c_char; 34]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"E255: Couldn't read in sign data!\0",
    )
});
pub static e_swapclose: GlobalCell<[::core::ffi::c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E72: Close error on swap file\0",
    )
});
pub static e_toocompl: GlobalCell<[::core::ffi::c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E74: Command too complex\0")
});
pub static e_longname: GlobalCell<[::core::ffi::c_char; 19]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"E75: Name too long\0")
});
pub static e_toomsbra: GlobalCell<[::core::ffi::c_char; 16]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 16], [::core::ffi::c_char; 16]>(*b"E76: Too many [\0")
});
pub static e_toomany: GlobalCell<[::core::ffi::c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E77: Too many file names\0")
});
pub static e_trailing: GlobalCell<[::core::ffi::c_char; 26]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E488: Trailing characters\0")
});
pub static e_trailing_arg: GlobalCell<[::core::ffi::c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E488: Trailing characters: %s\0",
    )
});
pub static e_umark: GlobalCell<[::core::ffi::c_char; 18]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"E78: Unknown mark\0")
});
pub static e_wildexpand: GlobalCell<[::core::ffi::c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E79: Cannot expand wildcards\0",
    )
});
pub static e_winheight: GlobalCell<[::core::ffi::c_char; 56]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 56], [::core::ffi::c_char; 56]>(
        *b"E591: 'winheight' cannot be smaller than 'winminheight'\0",
    )
});
pub static e_winwidth: GlobalCell<[::core::ffi::c_char; 54]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 54], [::core::ffi::c_char; 54]>(
        *b"E592: 'winwidth' cannot be smaller than 'winminwidth'\0",
    )
});
pub static e_write: GlobalCell<[::core::ffi::c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E80: Error while writing\0")
});
pub static e_zerocount: GlobalCell<[::core::ffi::c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E939: Positive count required\0",
    )
});
pub static e_usingsid: GlobalCell<[::core::ffi::c_char; 41]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
        *b"E81: Using <SID> not in a script context\0",
    )
});
pub static e_missingparen: GlobalCell<[::core::ffi::c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E107: Missing parentheses: %s\0",
    )
});
pub static e_empty_buffer: GlobalCell<[::core::ffi::c_char; 19]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"E749: Empty buffer\0")
});
pub static e_nobufnr: GlobalCell<[::core::ffi::c_char; 31]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"E86: Buffer %ld does not exist\0",
    )
});
pub static e_no_write_since_last_change: GlobalCell<[::core::ffi::c_char; 32]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
            *b"E37: No write since last change\0",
        )
    });
pub static e_no_write_since_last_change_add_bang_to_override: GlobalCell<
    [::core::ffi::c_char; 52],
> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 52], [::core::ffi::c_char; 52]>(
        *b"E37: No write since last change (add ! to override)\0",
    )
});
pub static e_no_write_since_last_change_for_buffer_nr_add_bang_to_override: GlobalCell<
    [::core::ffi::c_char; 66],
> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 66], [::core::ffi::c_char; 66]>(
        *b"E89: No write since last change for buffer %d (add ! to override)\0",
    )
});
pub static e_buffer_nr_not_found: GlobalCell<[::core::ffi::c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E92: Buffer %d not found\0")
});
pub static e_unknown_function_str: GlobalCell<[::core::ffi::c_char; 27]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(
            *b"E117: Unknown function: %s\0",
        )
    });
pub static e_str_not_inside_function: GlobalCell<[::core::ffi::c_char; 31]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
            *b"E193: %s not inside a function\0",
        )
    });
pub static e_job_still_running: GlobalCell<[::core::ffi::c_char; 24]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E948: Job still running\0")
});
pub static e_job_still_running_add_bang_to_end_the_job: GlobalCell<[::core::ffi::c_char; 47]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 47], [::core::ffi::c_char; 47]>(
            *b"E948: Job still running (add ! to end the job)\0",
        )
    });
pub static e_invalpat: GlobalCell<[::core::ffi::c_char; 42]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 42], [::core::ffi::c_char; 42]>(
        *b"E682: Invalid search pattern or delimiter\0",
    )
});
pub static e_bufloaded: GlobalCell<[::core::ffi::c_char; 39]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
        *b"E139: File is loaded in another buffer\0",
    )
});
pub static e_notset: GlobalCell<[::core::ffi::c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E764: Option '%s' is not set\0",
    )
});
pub static e_invalidreg: GlobalCell<[::core::ffi::c_char; 28]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"E850: Invalid register name\0")
});
pub static e_dirnotf: GlobalCell<[::core::ffi::c_char; 40]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"E919: Directory not found in '%s': \"%s\"\0",
    )
});
pub static e_au_recursive: GlobalCell<[::core::ffi::c_char; 44]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
        *b"E952: Autocommand caused recursive behavior\0",
    )
});
pub static e_menu_only_exists_in_another_mode: GlobalCell<[::core::ffi::c_char; 39]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
            *b"E328: Menu only exists in another mode\0",
        )
    });
pub static e_autocmd_close: GlobalCell<[::core::ffi::c_char; 34]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"E813: Cannot close autocmd window\0",
    )
});
pub static e_list_index_out_of_range_nr: GlobalCell<[::core::ffi::c_char; 35]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
            *b"E684: List index out of range: %ld\0",
        )
    });
pub static e_listarg: GlobalCell<[::core::ffi::c_char; 36]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"E686: Argument of %s must be a List\0",
    )
});
pub static e_unsupportedoption: GlobalCell<[::core::ffi::c_char; 27]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E519: Option not supported\0")
});
pub static e_fnametoolong: GlobalCell<[::core::ffi::c_char; 24]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E856: Filename too long\0")
});
pub static e_using_float_as_string: GlobalCell<[::core::ffi::c_char; 32]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
            *b"E806: Using a Float as a String\0",
        )
    });
pub static e_cannot_edit_other_buf: GlobalCell<[::core::ffi::c_char; 45]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
            *b"E788: Not allowed to edit another buffer now\0",
        )
    });
pub static e_using_number_as_bool_nr: GlobalCell<[::core::ffi::c_char; 36]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
            *b"E1023: Using a Number as a Bool: %d\0",
        )
    });
pub static e_not_callable_type_str: GlobalCell<[::core::ffi::c_char; 31]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
            *b"E1085: Not a callable type: %s\0",
        )
    });
pub static e_auabort: GlobalCell<[::core::ffi::c_char; 43]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 43], [::core::ffi::c_char; 43]>(
        *b"E855: Autocommands caused command to abort\0",
    )
});
pub static e_api_error: GlobalCell<[::core::ffi::c_char; 20]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"E5555: API call: %s\0")
});
pub static e_fast_api_disabled: GlobalCell<[::core::ffi::c_char; 53]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 53], [::core::ffi::c_char; 53]>(
        *b"E5560: %s must not be called in a fast event context\0",
    )
});
pub static e_floatonly: GlobalCell<[::core::ffi::c_char; 62]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 62], [::core::ffi::c_char; 62]>(
        *b"E5601: Cannot close window, only floating window would remain\0",
    )
});
pub static e_floatexchange: GlobalCell<[::core::ffi::c_char; 39]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
        *b"E5602: Cannot exchange or rotate float\0",
    )
});
pub static e_cant_find_directory_str_in_cdpath: GlobalCell<[::core::ffi::c_char; 42]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 42], [::core::ffi::c_char; 42]>(
            *b"E344: Can't find directory \"%s\" in cdpath\0",
        )
    });
pub static e_cant_find_file_str_in_path: GlobalCell<[::core::ffi::c_char; 35]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
            *b"E345: Can't find file \"%s\" in path\0",
        )
    });
pub static e_no_more_directory_str_found_in_cdpath: GlobalCell<[::core::ffi::c_char; 45]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
            *b"E346: No more directory \"%s\" found in cdpath\0",
        )
    });
pub static e_no_more_file_str_found_in_path: GlobalCell<[::core::ffi::c_char; 38]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
            *b"E347: No more file \"%s\" found in path\0",
        )
    });
pub static e_value_is_locked: GlobalCell<[::core::ffi::c_char; 22]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"E741: Value is locked\0")
});
pub static e_value_is_locked_str: GlobalCell<[::core::ffi::c_char; 28]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"E741: Value is locked: %.*s\0")
});
pub static e_cannot_change_value: GlobalCell<[::core::ffi::c_char; 26]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E742: Cannot change value\0")
});
pub static e_cannot_change_value_of_str: GlobalCell<[::core::ffi::c_char; 34]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
            *b"E742: Cannot change value of %.*s\0",
        )
    });
pub static e_cannot_set_variable_in_sandbox_str: GlobalCell<[::core::ffi::c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
            *b"E794: Cannot set variable in the sandbox: \"%.*s\"\0",
        )
    });
pub static e_cannot_delete_variable_str: GlobalCell<[::core::ffi::c_char; 34]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
            *b"E795: Cannot delete variable %.*s\0",
        )
    });
pub static e_invalwindow: GlobalCell<[::core::ffi::c_char; 28]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"E957: Invalid window number\0")
});
pub static e_problem_creating_internal_diff: GlobalCell<[::core::ffi::c_char; 41]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
            *b"E960: Problem creating the internal diff\0",
        )
    });
pub static e_cannot_define_autocommands_for_all_events: GlobalCell<[::core::ffi::c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
            *b"E1155: Cannot define autocommands for ALL events\0",
        )
    });
pub static e_cannot_change_arglist_recursively: GlobalCell<[::core::ffi::c_char; 51]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 51], [::core::ffi::c_char; 51]>(
            *b"E1156: Cannot change the argument list recursively\0",
        )
    });
pub static e_resulting_text_too_long: GlobalCell<[::core::ffi::c_char; 31]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
            *b"E1240: Resulting text too long\0",
        )
    });
pub static e_line_number_out_of_range: GlobalCell<[::core::ffi::c_char; 32]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
            *b"E1247: Line number out of range\0",
        )
    });
pub static e_highlight_group_name_invalid_char: GlobalCell<[::core::ffi::c_char; 39]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
            *b"E5248: Invalid character in group name\0",
        )
    });
pub static e_highlight_group_name_too_long: GlobalCell<[::core::ffi::c_char; 37]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
            *b"E1249: Highlight group name too long\0",
        )
    });
pub static e_string_required: GlobalCell<[::core::ffi::c_char; 22]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"E928: String required\0")
});
pub static e_invalid_column_number_nr: GlobalCell<[::core::ffi::c_char; 33]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
            *b"E964: Invalid column number: %ld\0",
        )
    });
pub static e_invalid_line_number_nr: GlobalCell<[::core::ffi::c_char; 31]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
            *b"E966: Invalid line number: %ld\0",
        )
    });
pub static e_reduce_of_an_empty_str_with_no_initial_value: GlobalCell<[::core::ffi::c_char; 50]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 50], [::core::ffi::c_char; 50]>(
            *b"E998: Reduce of an empty %s with no initial value\0",
        )
    });
pub static e_invalid_value_for_blob_nr: GlobalCell<[::core::ffi::c_char; 36]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
            *b"E1239: Invalid value for blob: 0xlX\0",
        )
    });
pub static e_stray_closing_curly_str: GlobalCell<[::core::ffi::c_char; 44]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
            *b"E1278: Stray '}' without a matching '{': %s\0",
        )
    });
pub static e_missing_close_curly_str: GlobalCell<[::core::ffi::c_char; 23]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"E1279: Missing '}': %s\0")
    });
pub static e_cannot_change_menus_while_listing: GlobalCell<[::core::ffi::c_char; 41]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
            *b"E1310: Cannot change menus while listing\0",
        )
    });
pub static e_not_allowed_to_change_window_layout_in_this_autocmd: GlobalCell<
    [::core::ffi::c_char; 63],
> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 63], [::core::ffi::c_char; 63]>(
        *b"E1312: Not allowed to change the window layout in this autocmd\0",
    )
});
pub static e_val_too_large: GlobalCell<[::core::ffi::c_char; 27]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E1510: Value too large: %s\0")
});
pub static e_val_too_large_len: GlobalCell<[::core::ffi::c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E1510: Value too large: %.*s\0",
    )
});
pub static e_undobang_cannot_redo_or_move_branch: GlobalCell<[::core::ffi::c_char; 68]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 68], [::core::ffi::c_char; 68]>(
            *b"E5767: Cannot use :undo! to redo or move to a different undo branch\0",
        )
    });
pub static e_winfixbuf_cannot_go_to_buffer: GlobalCell<[::core::ffi::c_char; 52]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 52], [::core::ffi::c_char; 52]>(
            *b"E1513: Cannot switch buffer. 'winfixbuf' is enabled\0",
        )
    });
pub static e_invalid_return_type_from_findfunc: GlobalCell<[::core::ffi::c_char; 45]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
            *b"E1514: 'findfunc' did not return a List type\0",
        )
    });
pub static e_cannot_switch_to_a_closing_buffer: GlobalCell<[::core::ffi::c_char; 41]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
            *b"E1546: Cannot switch to a closing buffer\0",
        )
    });
pub static e_cannot_have_more_than_nr_diff_anchors: GlobalCell<[::core::ffi::c_char; 45]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
            *b"E1549: Cannot have more than %d diff anchors\0",
        )
    });
pub static e_failed_to_find_all_diff_anchors: GlobalCell<[::core::ffi::c_char; 39]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
            *b"E1550: Failed to find all diff anchors\0",
        )
    });
pub static e_diff_anchors_with_hidden_windows: GlobalCell<[::core::ffi::c_char; 60]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 60], [::core::ffi::c_char; 60]>(
            *b"E1562: Diff anchors cannot be used with hidden diff windows\0",
        )
    });
pub static e_leadtab_requires_tab: GlobalCell<[::core::ffi::c_char; 66]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 66], [::core::ffi::c_char; 66]>(
            *b"E1572: 'listchars' field \"leadtab\" requires \"tab\" to be specified\0",
        )
    });
pub static e_invalid_format_string_single_percent_s: GlobalCell<[::core::ffi::c_char; 55]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 55], [::core::ffi::c_char; 55]>(
            *b"E1577: Invalid format string, only one \"%s\" is allowed\0",
        )
    });
pub static e_trustfile: GlobalCell<[::core::ffi::c_char; 36]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"E5570: Cannot update trust file: %s\0",
    )
});
pub static e_cannot_read_from_str_2: GlobalCell<[::core::ffi::c_char; 28]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(
            *b"E282: Cannot read from \"%s\"\0",
        )
    });
pub static e_conflicting_configs: GlobalCell<[::core::ffi::c_char; 38]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"E5422: Conflicting configs: \"%s\" \"%s\"\0",
    )
});
pub static e_unknown_option2: GlobalCell<[::core::ffi::c_char; 25]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E355: Unknown option: %s\0")
});
pub static top_bot_msg: GlobalCell<[::core::ffi::c_char; 37]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"search hit TOP, continuing at BOTTOM\0",
    )
});
pub static bot_top_msg: GlobalCell<[::core::ffi::c_char; 37]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"search hit BOTTOM, continuing at TOP\0",
    )
});
pub static line_msg: GlobalCell<[::core::ffi::c_char; 7]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b" line \0")
});
pub static EVALARG_EVALUATE: GlobalCell<evalarg_T> = GlobalCell::new(evalarg_T {
    eval_flags: EVAL_EVALUATE as ::core::ffi::c_int,
    eval_getline: None,
    eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
});
pub static msg_ext_need_clear: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_ext_skip_flush: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_ext_overwrite: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_ext_skip_verbose: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_grid: GlobalCell<ScreenGrid> = GlobalCell::new(ScreenGrid {
    handle: 0 as handle_T,
    chars: ::core::ptr::null_mut::<schar_T>(),
    attrs: ::core::ptr::null_mut::<sattr_T>(),
    vcols: ::core::ptr::null_mut::<colnr_T>(),
    line_offset: ::core::ptr::null_mut::<size_t>(),
    dirty_col: ::core::ptr::null_mut::<::core::ffi::c_int>(),
    rows: 0 as ::core::ffi::c_int,
    cols: 0 as ::core::ffi::c_int,
    valid: false,
    throttled: false,
    blending: false,
    mouse_enabled: true,
    zindex: 0 as ::core::ffi::c_int,
    comp_row: 0 as ::core::ffi::c_int,
    comp_col: 0 as ::core::ffi::c_int,
    comp_width: 0 as ::core::ffi::c_int,
    comp_height: 0 as ::core::ffi::c_int,
    comp_index: 0 as size_t,
    comp_disabled: false,
    pending_comp_index_update: true,
});
pub static msg_grid_pos: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static msg_grid_adj: GlobalCell<GridView> = GlobalCell::new(GridView {
    target: ::core::ptr::null_mut::<ScreenGrid>(),
    row_offset: 0,
    col_offset: 0,
});
pub static msg_scrolled_at_flush: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static msg_grid_scroll_discount: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static msg_listdo_overwrite: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
#[inline]
unsafe extern "C" fn tv_list_set_lock(l: *mut list_T, lock: VarLockStatus) {
    if l.is_null() {
        '_c2rust_label: {
            if lock as ::core::ffi::c_uint == VAR_FIXED as ::core::ffi::c_int as ::core::ffi::c_uint
            {
            } else {
                __assert_fail(
                    b"lock == VAR_FIXED\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/main.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    76 as ::core::ffi::c_uint,
                    b"void tv_list_set_lock(list_T *const, const VarLockStatus)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return;
    }
    (*l).lv_lock = lock;
}
// TV_CSTRING (SIZE_MAX - 1): c2rust dropped the initializer expression and
// left 0, which is a valid pointer-sentinel value and would corrupt any
// caller comparing against it (the unit tests do, via FFI).
#[no_mangle]
pub static kTVCstring: GlobalCell<size_t> = GlobalCell::new(18446744073709551614);
pub static kTVTranslate: GlobalCell<size_t> = GlobalCell::new(18446744073709551615 as size_t);
pub static disable_fold_update: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static test_disable_char_avail: GlobalCell<bool> = GlobalCell::new(false);
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const SYS_VIMRC_FILE: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"$VIM/sysinit.vim\0")
};
pub const VIMRC_FILE: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b".nvimrc\0") };
pub static g_stats: GlobalCell<nvim_stats_s> = GlobalCell::new(nvim_stats_s {
    fsync: 0 as int64_t,
    redraw: 0 as int64_t,
    log_skip: 0 as int16_t,
});
pub const NO_BUFFERS: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub static Rows: GlobalCell<::core::ffi::c_int> = GlobalCell::new(24 as ::core::ffi::c_int);
pub static Columns: GlobalCell<::core::ffi::c_int> = GlobalCell::new(80 as ::core::ffi::c_int);
pub static mod_mask: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static vgetc_mod_mask: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static vgetc_char: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static cmdline_row: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static redraw_cmdline: GlobalCell<bool> = GlobalCell::new(false);
pub static redraw_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static clear_cmdline: GlobalCell<bool> = GlobalCell::new(false);
pub static mode_displayed: GlobalCell<bool> = GlobalCell::new(false);
pub static cmdline_star: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static redrawing_cmdline: GlobalCell<bool> = GlobalCell::new(false);
pub static cmdline_was_last_drawn: GlobalCell<bool> = GlobalCell::new(false);
pub static exec_from_reg: GlobalCell<bool> = GlobalCell::new(false);
pub static dollar_vcol: GlobalCell<colnr_T> = GlobalCell::new(-1 as colnr_T);
pub static edit_submode: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static edit_submode_pre: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static edit_submode_extra: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static edit_submode_highl: GlobalCell<hlf_T> = GlobalCell::new(HLF_NONE);
pub static cmdmsg_rl: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static msg_row: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static msg_scrolled: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static msg_scrolled_ign: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_did_scroll: GlobalCell<bool> = GlobalCell::new(false);
pub static keep_msg: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static keep_msg_hl_id: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static need_fileinfo: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_scroll: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static msg_didout: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_didany: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_nowait: GlobalCell<bool> = GlobalCell::new(false);
pub static emsg_off: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static info_message: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_hist_off: GlobalCell<bool> = GlobalCell::new(false);
pub static need_clr_eos: GlobalCell<bool> = GlobalCell::new(false);
#[no_mangle]
pub static emsg_skip: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static emsg_severe: GlobalCell<bool> = GlobalCell::new(false);
pub static emsg_assert_fails_msg: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static emsg_assert_fails_lnum: GlobalCell<::core::ffi::c_long> =
    GlobalCell::new(0 as ::core::ffi::c_long);
pub static emsg_assert_fails_context: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static did_endif: GlobalCell<bool> = GlobalCell::new(false);
pub static did_emsg: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static called_vim_beep: GlobalCell<bool> = GlobalCell::new(false);
pub static did_emsg_syntax: GlobalCell<bool> = GlobalCell::new(false);
pub static called_emsg: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static ex_exitval: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static emsg_on_display: GlobalCell<bool> = GlobalCell::new(false);
pub static rc_did_emsg: GlobalCell<bool> = GlobalCell::new(false);
pub static no_wait_return: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static need_wait_return: GlobalCell<bool> = GlobalCell::new(false);
pub static did_wait_return: GlobalCell<bool> = GlobalCell::new(false);
pub static need_maketitle: GlobalCell<bool> = GlobalCell::new(true);
pub static quit_more: GlobalCell<bool> = GlobalCell::new(false);
pub static vgetc_busy: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static didset_vim: GlobalCell<bool> = GlobalCell::new(false);
pub static didset_vimruntime: GlobalCell<bool> = GlobalCell::new(false);
pub static lines_left: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
pub static msg_no_more: GlobalCell<bool> = GlobalCell::new(false);
pub static ex_nesting_level: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static debug_break_level: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(-1 as ::core::ffi::c_int);
pub static debug_did_msg: GlobalCell<bool> = GlobalCell::new(false);
pub static debug_tick: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static debug_backtrace_level: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static do_profiling: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static current_exception: GlobalCell<*mut except_T> =
    GlobalCell::new(::core::ptr::null_mut::<except_T>());
pub static did_throw: GlobalCell<bool> = GlobalCell::new(false);
pub static need_rethrow: GlobalCell<bool> = GlobalCell::new(false);
pub static check_cstack: GlobalCell<bool> = GlobalCell::new(false);
pub static trylevel: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static force_abort: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_list: GlobalCell<*mut *mut msglist_T> =
    GlobalCell::new(::core::ptr::null_mut::<*mut msglist_T>());
pub static suppress_errthrow: GlobalCell<bool> = GlobalCell::new(false);
pub static caught_stack: GlobalCell<*mut except_T> =
    GlobalCell::new(::core::ptr::null_mut::<except_T>());
pub static may_garbage_collect: GlobalCell<bool> = GlobalCell::new(false);
pub static want_garbage_collect: GlobalCell<bool> = GlobalCell::new(false);
pub static garbage_collect_at_exit: GlobalCell<bool> = GlobalCell::new(false);
pub const SID_CMDARG: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const SID_CARG: ::core::ffi::c_int = -3 as ::core::ffi::c_int;
pub const SID_ENV: ::core::ffi::c_int = -4 as ::core::ffi::c_int;
pub static current_sctx: GlobalCell<sctx_T> = GlobalCell::new(sctx_T {
    sc_sid: 0 as scid_T,
    sc_seq: 0 as ::core::ffi::c_int,
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
        es_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        es_type: ETYPE_TOP,
        es_info: C2Rust_Unnamed_43 {
            sctx: ::core::ptr::null_mut::<sctx_T>(),
        },
    },
    autocmd_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    autocmd_match: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    autocmd_fname_full: false,
    autocmd_bufnr: 0,
    funccalp: ::core::ptr::null_mut::<::core::ffi::c_void>(),
});
pub static provider_call_nesting: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static t_colors: GlobalCell<::core::ffi::c_int> = GlobalCell::new(256 as ::core::ffi::c_int);
pub static include_none: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static include_default: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static include_link: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static highlight_match: GlobalCell<bool> = GlobalCell::new(false);
pub static search_match_lines: GlobalCell<linenr_T> = GlobalCell::new(0);
pub static search_match_endcol: GlobalCell<colnr_T> = GlobalCell::new(0);
pub static search_first_line: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
pub static search_last_line: GlobalCell<linenr_T> =
    GlobalCell::new(MAXLNUM as ::core::ffi::c_int as linenr_T);
pub static no_smartcase: GlobalCell<bool> = GlobalCell::new(false);
pub static need_check_timestamps: GlobalCell<bool> = GlobalCell::new(false);
pub static did_check_timestamps: GlobalCell<bool> = GlobalCell::new(false);
pub static no_check_timestamps: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static mouse_grid: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static mouse_row: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static mouse_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static mouse_past_bottom: GlobalCell<bool> = GlobalCell::new(false);
pub static mouse_past_eol: GlobalCell<bool> = GlobalCell::new(false);
pub static mouse_dragging: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static root_menu: GlobalCell<*mut vimmenu_T> =
    GlobalCell::new(::core::ptr::null_mut::<vimmenu_T>());
pub static sys_menu: GlobalCell<bool> = GlobalCell::new(false);
#[no_mangle]
pub static firstwin: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
#[no_mangle]
pub static lastwin: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
#[no_mangle]
pub static prevwin: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
#[no_mangle]
pub static curwin: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static topframe: GlobalCell<*mut frame_T> = GlobalCell::new(::core::ptr::null_mut::<frame_T>());
#[no_mangle]
pub static first_tabpage: GlobalCell<*mut tabpage_T> =
    GlobalCell::new(::core::ptr::null_mut::<tabpage_T>());
#[no_mangle]
pub static curtab: GlobalCell<*mut tabpage_T> =
    GlobalCell::new(::core::ptr::null_mut::<tabpage_T>());
pub static lastused_tabpage: GlobalCell<*mut tabpage_T> =
    GlobalCell::new(::core::ptr::null_mut::<tabpage_T>());
pub static redraw_tabline: GlobalCell<bool> = GlobalCell::new(false);
pub static firstbuf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
pub static lastbuf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
#[no_mangle]
pub static curbuf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
pub static global_alist: GlobalCell<alist_T> = GlobalCell::new(alist_T {
    al_ga: garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    },
    al_refcount: 0,
    id: 0,
});
pub static max_alist_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static arg_had_last: GlobalCell<bool> = GlobalCell::new(false);
pub static ru_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static ru_wid: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static sc_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
#[no_mangle]
pub static starting: GlobalCell<::core::ffi::c_int> = GlobalCell::new(2 as ::core::ffi::c_int);
#[no_mangle]
pub static exiting: GlobalCell<bool> = GlobalCell::new(false);
pub static v_dying: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static stdin_isatty: GlobalCell<bool> = GlobalCell::new(true);
pub static stdout_isatty: GlobalCell<bool> = GlobalCell::new(true);
pub static stderr_isatty: GlobalCell<bool> = GlobalCell::new(true);
pub static stdin_fd: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
pub static full_screen: GlobalCell<bool> = GlobalCell::new(false);
pub static secure: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static textlock: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static allbuf_lock: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
#[no_mangle]
pub static sandbox: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static silent_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static VIsual: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
});
pub static VIsual_active: GlobalCell<bool> = GlobalCell::new(false);
pub static VIsual_select: GlobalCell<bool> = GlobalCell::new(false);
pub static VIsual_select_reg: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static VIsual_select_exclu_adj: GlobalCell<bool> = GlobalCell::new(false);
pub static restart_VIsual_select: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static VIsual_reselect: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static VIsual_mode: GlobalCell<::core::ffi::c_int> = GlobalCell::new('v' as ::core::ffi::c_int);
pub static redo_VIsual_busy: GlobalCell<bool> = GlobalCell::new(false);
pub static resel_VIsual_mode: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new('\0' as ::core::ffi::c_int);
pub static resel_VIsual_line_count: GlobalCell<linenr_T> = GlobalCell::new(0);
pub static resel_VIsual_vcol: GlobalCell<colnr_T> = GlobalCell::new(0);
pub static where_paste_started: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
});
pub static did_ai: GlobalCell<bool> = GlobalCell::new(false);
pub static ai_col: GlobalCell<colnr_T> = GlobalCell::new(0 as colnr_T);
pub static end_comment_pending: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new('\0' as ::core::ffi::c_int);
pub static did_syncbind: GlobalCell<bool> = GlobalCell::new(false);
pub static did_si: GlobalCell<bool> = GlobalCell::new(false);
pub static can_si: GlobalCell<bool> = GlobalCell::new(false);
pub static can_si_back: GlobalCell<bool> = GlobalCell::new(false);
pub static old_indent: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
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
pub static vr_lines_changed: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static inhibit_delete_count: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static fenc_default: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static State: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(MODE_NORMAL as ::core::ffi::c_int);
pub static debug_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static finish_op: GlobalCell<bool> = GlobalCell::new(false);
pub static opcount: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static motion_force: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static exmode_active: GlobalCell<bool> = GlobalCell::new(false);
pub static pending_exmode_active: GlobalCell<bool> = GlobalCell::new(false);
pub static ex_no_reprint: GlobalCell<bool> = GlobalCell::new(false);
pub static cmdpreview: GlobalCell<bool> = GlobalCell::new(false);
pub static reg_recording: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static reg_executing: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static pending_end_reg_executing: GlobalCell<bool> = GlobalCell::new(false);
pub static reg_recorded: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static no_mapping: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static no_zero_mapping: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static allow_keys: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static no_u_sync: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static u_sync_once: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static force_restart_edit: GlobalCell<bool> = GlobalCell::new(false);
pub static restart_edit: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static arrow_used: GlobalCell<bool> = GlobalCell::new(false);
pub static ins_at_eol: GlobalCell<bool> = GlobalCell::new(false);
pub static no_abbr: GlobalCell<bool> = GlobalCell::new(true);
pub static mapped_ctrl_c: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static ctrl_c_interrupts: GlobalCell<bool> = GlobalCell::new(true);
#[no_mangle]
pub static cmdmod: GlobalCell<cmdmod_T> = GlobalCell::new(cmdmod_T {
    cmod_flags: 0,
    cmod_split: 0,
    cmod_tab: 0,
    cmod_filter_pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    cmod_filter_regmatch: regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    },
    cmod_filter_force: false,
    cmod_verbose: 0,
    cmod_save_ei: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    cmod_did_sandbox: 0,
    cmod_verbose_save: 0,
    cmod_save_msg_silent: 0,
    cmod_save_msg_scroll: 0,
    cmod_did_esilent: 0,
});
pub static msg_silent: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
#[no_mangle]
pub static emsg_silent: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static emsg_noredir: GlobalCell<bool> = GlobalCell::new(false);
pub static cmd_silent: GlobalCell<bool> = GlobalCell::new(false);
pub static in_assert_fails: GlobalCell<bool> = GlobalCell::new(false);
pub const SEA_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEA_DIALOG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SEA_QUIT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub static swap_exists_action: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static swap_exists_did_quit: GlobalCell<bool> = GlobalCell::new(false);
pub static IObuff: GlobalCell<[::core::ffi::c_char; 1025]> = GlobalCell::new([0; 1025]);
#[no_mangle]
pub static NameBuff: GlobalCell<[::core::ffi::c_char; 4096]> = GlobalCell::new([0; 4096]);
pub static msg_buf: GlobalCell<[::core::ffi::c_char; 480]> = GlobalCell::new([0; 480]);
pub static os_buf: GlobalCell<[::core::ffi::c_char; 4096]> = GlobalCell::new([0; 4096]);
pub static RedrawingDisabled: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static readonlymode: GlobalCell<bool> = GlobalCell::new(false);
pub static recoverymode: GlobalCell<bool> = GlobalCell::new(false);
pub static typebuf: GlobalCell<typebuf_T> = GlobalCell::new(typebuf_T {
    tb_buf: ::core::ptr::null_mut::<uint8_t>(),
    tb_noremap: ::core::ptr::null_mut::<uint8_t>(),
    tb_buflen: 0 as ::core::ffi::c_int,
    tb_off: 0 as ::core::ffi::c_int,
    tb_len: 0 as ::core::ffi::c_int,
    tb_maplen: 0 as ::core::ffi::c_int,
    tb_silent: 0 as ::core::ffi::c_int,
    tb_no_abbr_cnt: 0 as ::core::ffi::c_int,
    tb_change_cnt: 0 as ::core::ffi::c_int,
});
pub static typebuf_was_empty: GlobalCell<bool> = GlobalCell::new(false);
pub static ex_normal_busy: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static expr_map_lock: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static ignore_script: GlobalCell<bool> = GlobalCell::new(false);
pub static stop_insert_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static KeyTyped: GlobalCell<bool> = GlobalCell::new(false);
pub static KeyStuffed: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static maptick: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static must_redraw: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static skip_redraw: GlobalCell<bool> = GlobalCell::new(false);
pub static do_redraw: GlobalCell<bool> = GlobalCell::new(false);
pub static must_redraw_pum: GlobalCell<bool> = GlobalCell::new(false);
pub static need_highlight_changed: GlobalCell<bool> = GlobalCell::new(true);
pub static scriptout: GlobalCell<*mut FILE> = GlobalCell::new(::core::ptr::null_mut::<FILE>());
pub static got_int: GlobalCell<bool> = GlobalCell::new(false);
pub static bangredo: GlobalCell<bool> = GlobalCell::new(false);
pub static searchcmdlen: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static reg_do_extmatch: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static re_extmatch_in: GlobalCell<*mut reg_extmatch_T> =
    GlobalCell::new(::core::ptr::null_mut::<reg_extmatch_T>());
pub static re_extmatch_out: GlobalCell<*mut reg_extmatch_T> =
    GlobalCell::new(::core::ptr::null_mut::<reg_extmatch_T>());
pub static did_outofmem_msg: GlobalCell<bool> = GlobalCell::new(false);
pub static did_swapwrite_msg: GlobalCell<bool> = GlobalCell::new(false);
pub static global_busy: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static listcmd_busy: GlobalCell<bool> = GlobalCell::new(false);
pub static need_start_insertmode: GlobalCell<bool> = GlobalCell::new(false);
pub static last_mode: GlobalCell<[::core::ffi::c_char; 4]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"n\0\0\0")
});
pub static last_cmdline: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static repeat_cmdline: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static new_last_cmdline: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static postponed_split: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static postponed_split_flags: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static postponed_split_tab: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static g_do_tagpreview: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static g_tag_at_cursor: GlobalCell<bool> = GlobalCell::new(false);
pub static replace_offset: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static escape_chars: GlobalCell<*mut ::core::ffi::c_char> = GlobalCell::new(
    b" \t\\\"|\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
);
pub static keep_help_flag: GlobalCell<bool> = GlobalCell::new(false);
pub static redir_off: GlobalCell<bool> = GlobalCell::new(false);
pub static redir_fd: GlobalCell<*mut FILE> = GlobalCell::new(::core::ptr::null_mut::<FILE>());
pub static redir_reg: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static redir_vname: GlobalCell<bool> = GlobalCell::new(false);
pub static capture_ga: GlobalCell<*mut garray_T> =
    GlobalCell::new(::core::ptr::null_mut::<garray_T>());
pub static langmap_mapchar: GlobalCell<[uint8_t; 256]> = GlobalCell::new([0; 256]);
pub static save_p_ls: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
pub static save_p_wmh: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
pub static wild_menu_showing: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static globaldir: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static last_chdir_reason: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static km_stopsel: GlobalCell<bool> = GlobalCell::new(false);
pub static km_startsel: GlobalCell<bool> = GlobalCell::new(false);
pub static cmdwin_type: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static cmdwin_result: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static cmdwin_level: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static cmdwin_buf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
pub static cmdwin_win: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static cmdwin_old_curwin: GlobalCell<*mut win_T> =
    GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static cmdline_win: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static no_lines_msg: GlobalCell<[::core::ffi::c_char; 23]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"--No lines in buffer--\0")
});
pub static sub_nsubs: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static sub_nlines: GlobalCell<linenr_T> = GlobalCell::new(0);
pub static wim_flags: GlobalCell<[uint8_t; 4]> = GlobalCell::new([0; 4]);
pub static stl_syntax: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub static no_hlsearch: GlobalCell<bool> = GlobalCell::new(false);
pub static typebuf_was_filled: GlobalCell<bool> = GlobalCell::new(false);
pub static virtual_op: GlobalCell<TriState> = GlobalCell::new(kNone);
pub static display_tick: GlobalCell<disptick_T> = GlobalCell::new(0 as disptick_T);
pub static spell_redraw_lnum: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
pub static time_fd: GlobalCell<*mut FILE> = GlobalCell::new(::core::ptr::null_mut::<FILE>());
pub static vim_ignored: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static embedded_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static headless_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static windowsVersion: GlobalCell<[::core::ffi::c_char; 20]> = GlobalCell::new([
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
]);
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
    dirty_col: ::core::ptr::null_mut::<::core::ffi::c_int>(),
    rows: 0 as ::core::ffi::c_int,
    cols: 0 as ::core::ffi::c_int,
    valid: false,
    throttled: false,
    blending: false,
    mouse_enabled: true,
    zindex: 0 as ::core::ffi::c_int,
    comp_row: 0 as ::core::ffi::c_int,
    comp_col: 0 as ::core::ffi::c_int,
    comp_width: 0 as ::core::ffi::c_int,
    comp_height: 0 as ::core::ffi::c_int,
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
pub static linebuf_scratch: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static opt_ambw_values: GlobalCell<[*const ::core::ffi::c_char; 3]> = GlobalCell::new([
    b"single\0".as_ptr() as *const ::core::ffi::c_char,
    b"double\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_bg_values: GlobalCell<[*const ::core::ffi::c_char; 3]> = GlobalCell::new([
    b"light\0".as_ptr() as *const ::core::ffi::c_char,
    b"dark\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_bs_values: GlobalCell<[*const ::core::ffi::c_char; 5]> = GlobalCell::new([
    b"indent\0".as_ptr() as *const ::core::ffi::c_char,
    b"eol\0".as_ptr() as *const ::core::ffi::c_char,
    b"start\0".as_ptr() as *const ::core::ffi::c_char,
    b"nostop\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_bkc_values: GlobalCell<[*const ::core::ffi::c_char; 6]> = GlobalCell::new([
    b"yes\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto\0".as_ptr() as *const ::core::ffi::c_char,
    b"no\0".as_ptr() as *const ::core::ffi::c_char,
    b"breaksymlink\0".as_ptr() as *const ::core::ffi::c_char,
    b"breakhardlink\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_bo_values: GlobalCell<[*const ::core::ffi::c_char; 21]> = GlobalCell::new([
    b"all\0".as_ptr() as *const ::core::ffi::c_char,
    b"backspace\0".as_ptr() as *const ::core::ffi::c_char,
    b"cursor\0".as_ptr() as *const ::core::ffi::c_char,
    b"complete\0".as_ptr() as *const ::core::ffi::c_char,
    b"copy\0".as_ptr() as *const ::core::ffi::c_char,
    b"ctrlg\0".as_ptr() as *const ::core::ffi::c_char,
    b"error\0".as_ptr() as *const ::core::ffi::c_char,
    b"esc\0".as_ptr() as *const ::core::ffi::c_char,
    b"ex\0".as_ptr() as *const ::core::ffi::c_char,
    b"hangul\0".as_ptr() as *const ::core::ffi::c_char,
    b"insertmode\0".as_ptr() as *const ::core::ffi::c_char,
    b"lang\0".as_ptr() as *const ::core::ffi::c_char,
    b"mess\0".as_ptr() as *const ::core::ffi::c_char,
    b"showmatch\0".as_ptr() as *const ::core::ffi::c_char,
    b"operator\0".as_ptr() as *const ::core::ffi::c_char,
    b"register\0".as_ptr() as *const ::core::ffi::c_char,
    b"shell\0".as_ptr() as *const ::core::ffi::c_char,
    b"spell\0".as_ptr() as *const ::core::ffi::c_char,
    b"term\0".as_ptr() as *const ::core::ffi::c_char,
    b"wildmode\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_briopt_values: GlobalCell<[*const ::core::ffi::c_char; 6]> = GlobalCell::new([
    b"shift:\0".as_ptr() as *const ::core::ffi::c_char,
    b"min:\0".as_ptr() as *const ::core::ffi::c_char,
    b"sbr\0".as_ptr() as *const ::core::ffi::c_char,
    b"list:\0".as_ptr() as *const ::core::ffi::c_char,
    b"column:\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_bh_values: GlobalCell<[*const ::core::ffi::c_char; 6]> = GlobalCell::new([
    b"\0".as_ptr() as *const ::core::ffi::c_char,
    b"hide\0".as_ptr() as *const ::core::ffi::c_char,
    b"unload\0".as_ptr() as *const ::core::ffi::c_char,
    b"delete\0".as_ptr() as *const ::core::ffi::c_char,
    b"wipe\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_bt_values: GlobalCell<[*const ::core::ffi::c_char; 9]> = GlobalCell::new([
    b"\0".as_ptr() as *const ::core::ffi::c_char,
    b"acwrite\0".as_ptr() as *const ::core::ffi::c_char,
    b"help\0".as_ptr() as *const ::core::ffi::c_char,
    b"nofile\0".as_ptr() as *const ::core::ffi::c_char,
    b"nowrite\0".as_ptr() as *const ::core::ffi::c_char,
    b"quickfix\0".as_ptr() as *const ::core::ffi::c_char,
    b"terminal\0".as_ptr() as *const ::core::ffi::c_char,
    b"prompt\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_cmp_values: GlobalCell<[*const ::core::ffi::c_char; 3]> = GlobalCell::new([
    b"internal\0".as_ptr() as *const ::core::ffi::c_char,
    b"keepascii\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_cb_values: GlobalCell<[*const ::core::ffi::c_char; 3]> = GlobalCell::new([
    b"unnamed\0".as_ptr() as *const ::core::ffi::c_char,
    b"unnamedplus\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_cpt_values: GlobalCell<[*const ::core::ffi::c_char; 16]> = GlobalCell::new([
    b".\0".as_ptr() as *const ::core::ffi::c_char,
    b"w\0".as_ptr() as *const ::core::ffi::c_char,
    b"b\0".as_ptr() as *const ::core::ffi::c_char,
    b"u\0".as_ptr() as *const ::core::ffi::c_char,
    b"k\0".as_ptr() as *const ::core::ffi::c_char,
    b"kspell\0".as_ptr() as *const ::core::ffi::c_char,
    b"s\0".as_ptr() as *const ::core::ffi::c_char,
    b"i\0".as_ptr() as *const ::core::ffi::c_char,
    b"d\0".as_ptr() as *const ::core::ffi::c_char,
    b"]\0".as_ptr() as *const ::core::ffi::c_char,
    b"t\0".as_ptr() as *const ::core::ffi::c_char,
    b"U\0".as_ptr() as *const ::core::ffi::c_char,
    b"f\0".as_ptr() as *const ::core::ffi::c_char,
    b"F\0".as_ptr() as *const ::core::ffi::c_char,
    b"o\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_cot_values: GlobalCell<[*const ::core::ffi::c_char; 12]> = GlobalCell::new([
    b"menu\0".as_ptr() as *const ::core::ffi::c_char,
    b"menuone\0".as_ptr() as *const ::core::ffi::c_char,
    b"longest\0".as_ptr() as *const ::core::ffi::c_char,
    b"preview\0".as_ptr() as *const ::core::ffi::c_char,
    b"popup\0".as_ptr() as *const ::core::ffi::c_char,
    b"noinsert\0".as_ptr() as *const ::core::ffi::c_char,
    b"noselect\0".as_ptr() as *const ::core::ffi::c_char,
    b"fuzzy\0".as_ptr() as *const ::core::ffi::c_char,
    b"nosort\0".as_ptr() as *const ::core::ffi::c_char,
    b"preinsert\0".as_ptr() as *const ::core::ffi::c_char,
    b"nearest\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_csl_values: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"\0".as_ptr() as *const ::core::ffi::c_char,
    b"slash\0".as_ptr() as *const ::core::ffi::c_char,
    b"backslash\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_culopt_values: GlobalCell<[*const ::core::ffi::c_char; 5]> = GlobalCell::new([
    b"line\0".as_ptr() as *const ::core::ffi::c_char,
    b"screenline\0".as_ptr() as *const ::core::ffi::c_char,
    b"number\0".as_ptr() as *const ::core::ffi::c_char,
    b"both\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_debug_values: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"msg\0".as_ptr() as *const ::core::ffi::c_char,
    b"throw\0".as_ptr() as *const ::core::ffi::c_char,
    b"beep\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_dip_values: GlobalCell<[*const ::core::ffi::c_char; 20]> = GlobalCell::new([
    b"filler\0".as_ptr() as *const ::core::ffi::c_char,
    b"anchor\0".as_ptr() as *const ::core::ffi::c_char,
    b"context:\0".as_ptr() as *const ::core::ffi::c_char,
    b"iblank\0".as_ptr() as *const ::core::ffi::c_char,
    b"icase\0".as_ptr() as *const ::core::ffi::c_char,
    b"iwhite\0".as_ptr() as *const ::core::ffi::c_char,
    b"iwhiteall\0".as_ptr() as *const ::core::ffi::c_char,
    b"iwhiteeol\0".as_ptr() as *const ::core::ffi::c_char,
    b"horizontal\0".as_ptr() as *const ::core::ffi::c_char,
    b"vertical\0".as_ptr() as *const ::core::ffi::c_char,
    b"closeoff\0".as_ptr() as *const ::core::ffi::c_char,
    b"hiddenoff\0".as_ptr() as *const ::core::ffi::c_char,
    b"foldcolumn:\0".as_ptr() as *const ::core::ffi::c_char,
    b"followwrap\0".as_ptr() as *const ::core::ffi::c_char,
    b"internal\0".as_ptr() as *const ::core::ffi::c_char,
    b"indent-heuristic\0".as_ptr() as *const ::core::ffi::c_char,
    b"algorithm:\0".as_ptr() as *const ::core::ffi::c_char,
    b"inline:\0".as_ptr() as *const ::core::ffi::c_char,
    b"linematch:\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_dip_algorithm_values: GlobalCell<[*const ::core::ffi::c_char; 5]> =
    GlobalCell::new([
        b"myers\0".as_ptr() as *const ::core::ffi::c_char,
        b"minimal\0".as_ptr() as *const ::core::ffi::c_char,
        b"patience\0".as_ptr() as *const ::core::ffi::c_char,
        b"histogram\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
    ]);
pub static opt_dip_inline_values: GlobalCell<[*const ::core::ffi::c_char; 5]> = GlobalCell::new([
    b"none\0".as_ptr() as *const ::core::ffi::c_char,
    b"simple\0".as_ptr() as *const ::core::ffi::c_char,
    b"char\0".as_ptr() as *const ::core::ffi::c_char,
    b"word\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_dy_values: GlobalCell<[*const ::core::ffi::c_char; 5]> = GlobalCell::new([
    b"lastline\0".as_ptr() as *const ::core::ffi::c_char,
    b"truncate\0".as_ptr() as *const ::core::ffi::c_char,
    b"uhex\0".as_ptr() as *const ::core::ffi::c_char,
    b"msgsep\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_ead_values: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"both\0".as_ptr() as *const ::core::ffi::c_char,
    b"ver\0".as_ptr() as *const ::core::ffi::c_char,
    b"hor\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_ff_values: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"unix\0".as_ptr() as *const ::core::ffi::c_char,
    b"dos\0".as_ptr() as *const ::core::ffi::c_char,
    b"mac\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_fcl_values: GlobalCell<[*const ::core::ffi::c_char; 2]> = GlobalCell::new([
    b"all\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_fdc_values: GlobalCell<[*const ::core::ffi::c_char; 21]> = GlobalCell::new([
    b"auto\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:1\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:2\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:3\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:4\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:5\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:6\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:7\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:8\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:9\0".as_ptr() as *const ::core::ffi::c_char,
    b"0\0".as_ptr() as *const ::core::ffi::c_char,
    b"1\0".as_ptr() as *const ::core::ffi::c_char,
    b"2\0".as_ptr() as *const ::core::ffi::c_char,
    b"3\0".as_ptr() as *const ::core::ffi::c_char,
    b"4\0".as_ptr() as *const ::core::ffi::c_char,
    b"5\0".as_ptr() as *const ::core::ffi::c_char,
    b"6\0".as_ptr() as *const ::core::ffi::c_char,
    b"7\0".as_ptr() as *const ::core::ffi::c_char,
    b"8\0".as_ptr() as *const ::core::ffi::c_char,
    b"9\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_fdm_values: GlobalCell<[*const ::core::ffi::c_char; 7]> = GlobalCell::new([
    b"manual\0".as_ptr() as *const ::core::ffi::c_char,
    b"expr\0".as_ptr() as *const ::core::ffi::c_char,
    b"marker\0".as_ptr() as *const ::core::ffi::c_char,
    b"indent\0".as_ptr() as *const ::core::ffi::c_char,
    b"syntax\0".as_ptr() as *const ::core::ffi::c_char,
    b"diff\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_fdo_values: GlobalCell<[*const ::core::ffi::c_char; 12]> = GlobalCell::new([
    b"all\0".as_ptr() as *const ::core::ffi::c_char,
    b"block\0".as_ptr() as *const ::core::ffi::c_char,
    b"hor\0".as_ptr() as *const ::core::ffi::c_char,
    b"mark\0".as_ptr() as *const ::core::ffi::c_char,
    b"percent\0".as_ptr() as *const ::core::ffi::c_char,
    b"quickfix\0".as_ptr() as *const ::core::ffi::c_char,
    b"search\0".as_ptr() as *const ::core::ffi::c_char,
    b"tag\0".as_ptr() as *const ::core::ffi::c_char,
    b"insert\0".as_ptr() as *const ::core::ffi::c_char,
    b"undo\0".as_ptr() as *const ::core::ffi::c_char,
    b"jump\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_icm_values: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"nosplit\0".as_ptr() as *const ::core::ffi::c_char,
    b"split\0".as_ptr() as *const ::core::ffi::c_char,
    b"\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_jop_values: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"stack\0".as_ptr() as *const ::core::ffi::c_char,
    b"view\0".as_ptr() as *const ::core::ffi::c_char,
    b"clean\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_km_values: GlobalCell<[*const ::core::ffi::c_char; 3]> = GlobalCell::new([
    b"startsel\0".as_ptr() as *const ::core::ffi::c_char,
    b"stopsel\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_lop_values: GlobalCell<[*const ::core::ffi::c_char; 3]> = GlobalCell::new([
    b"expr:0\0".as_ptr() as *const ::core::ffi::c_char,
    b"expr:1\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_mopt_values: GlobalCell<[*const ::core::ffi::c_char; 5]> = GlobalCell::new([
    b"hit-enter\0".as_ptr() as *const ::core::ffi::c_char,
    b"wait:\0".as_ptr() as *const ::core::ffi::c_char,
    b"history:\0".as_ptr() as *const ::core::ffi::c_char,
    b"progress:\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_mousem_values: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"extend\0".as_ptr() as *const ::core::ffi::c_char,
    b"popup\0".as_ptr() as *const ::core::ffi::c_char,
    b"popup_setpos\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_mousescroll_values: GlobalCell<[*const ::core::ffi::c_char; 3]> = GlobalCell::new([
    b"hor:\0".as_ptr() as *const ::core::ffi::c_char,
    b"ver:\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_nf_values: GlobalCell<[*const ::core::ffi::c_char; 7]> = GlobalCell::new([
    b"bin\0".as_ptr() as *const ::core::ffi::c_char,
    b"octal\0".as_ptr() as *const ::core::ffi::c_char,
    b"hex\0".as_ptr() as *const ::core::ffi::c_char,
    b"alpha\0".as_ptr() as *const ::core::ffi::c_char,
    b"unsigned\0".as_ptr() as *const ::core::ffi::c_char,
    b"blank\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_pumborder_values: GlobalCell<[*const ::core::ffi::c_char; 9]> = GlobalCell::new([
    b"\0".as_ptr() as *const ::core::ffi::c_char,
    b"double\0".as_ptr() as *const ::core::ffi::c_char,
    b"single\0".as_ptr() as *const ::core::ffi::c_char,
    b"shadow\0".as_ptr() as *const ::core::ffi::c_char,
    b"rounded\0".as_ptr() as *const ::core::ffi::c_char,
    b"solid\0".as_ptr() as *const ::core::ffi::c_char,
    b"bold\0".as_ptr() as *const ::core::ffi::c_char,
    b"none\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_rdb_values: GlobalCell<[*const ::core::ffi::c_char; 7]> = GlobalCell::new([
    b"compositor\0".as_ptr() as *const ::core::ffi::c_char,
    b"nothrottle\0".as_ptr() as *const ::core::ffi::c_char,
    b"invalid\0".as_ptr() as *const ::core::ffi::c_char,
    b"nodelta\0".as_ptr() as *const ::core::ffi::c_char,
    b"line\0".as_ptr() as *const ::core::ffi::c_char,
    b"flush\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_rlc_values: GlobalCell<[*const ::core::ffi::c_char; 2]> = GlobalCell::new([
    b"search\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_sbo_values: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"ver\0".as_ptr() as *const ::core::ffi::c_char,
    b"hor\0".as_ptr() as *const ::core::ffi::c_char,
    b"jump\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_sel_values: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"inclusive\0".as_ptr() as *const ::core::ffi::c_char,
    b"exclusive\0".as_ptr() as *const ::core::ffi::c_char,
    b"old\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_slm_values: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"mouse\0".as_ptr() as *const ::core::ffi::c_char,
    b"key\0".as_ptr() as *const ::core::ffi::c_char,
    b"cmd\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_ssop_values: GlobalCell<[*const ::core::ffi::c_char; 19]> = GlobalCell::new([
    b"buffers\0".as_ptr() as *const ::core::ffi::c_char,
    b"winpos\0".as_ptr() as *const ::core::ffi::c_char,
    b"resize\0".as_ptr() as *const ::core::ffi::c_char,
    b"winsize\0".as_ptr() as *const ::core::ffi::c_char,
    b"localoptions\0".as_ptr() as *const ::core::ffi::c_char,
    b"options\0".as_ptr() as *const ::core::ffi::c_char,
    b"help\0".as_ptr() as *const ::core::ffi::c_char,
    b"blank\0".as_ptr() as *const ::core::ffi::c_char,
    b"globals\0".as_ptr() as *const ::core::ffi::c_char,
    b"slash\0".as_ptr() as *const ::core::ffi::c_char,
    b"unix\0".as_ptr() as *const ::core::ffi::c_char,
    b"sesdir\0".as_ptr() as *const ::core::ffi::c_char,
    b"curdir\0".as_ptr() as *const ::core::ffi::c_char,
    b"folds\0".as_ptr() as *const ::core::ffi::c_char,
    b"cursor\0".as_ptr() as *const ::core::ffi::c_char,
    b"tabpages\0".as_ptr() as *const ::core::ffi::c_char,
    b"terminal\0".as_ptr() as *const ::core::ffi::c_char,
    b"skiprtp\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_sloc_values: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"last\0".as_ptr() as *const ::core::ffi::c_char,
    b"statusline\0".as_ptr() as *const ::core::ffi::c_char,
    b"tabline\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_scl_values: GlobalCell<[*const ::core::ffi::c_char; 23]> = GlobalCell::new([
    b"yes\0".as_ptr() as *const ::core::ffi::c_char,
    b"no\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:1\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:2\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:3\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:4\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:5\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:6\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:7\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:8\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:9\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:1\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:2\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:3\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:4\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:5\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:6\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:7\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:8\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:9\0".as_ptr() as *const ::core::ffi::c_char,
    b"number\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_spo_values: GlobalCell<[*const ::core::ffi::c_char; 3]> = GlobalCell::new([
    b"camel\0".as_ptr() as *const ::core::ffi::c_char,
    b"noplainbuffer\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_sps_values: GlobalCell<[*const ::core::ffi::c_char; 7]> = GlobalCell::new([
    b"best\0".as_ptr() as *const ::core::ffi::c_char,
    b"fast\0".as_ptr() as *const ::core::ffi::c_char,
    b"double\0".as_ptr() as *const ::core::ffi::c_char,
    b"expr:\0".as_ptr() as *const ::core::ffi::c_char,
    b"file:\0".as_ptr() as *const ::core::ffi::c_char,
    b"timeout:\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_spk_values: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"cursor\0".as_ptr() as *const ::core::ffi::c_char,
    b"screen\0".as_ptr() as *const ::core::ffi::c_char,
    b"topline\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_swb_values: GlobalCell<[*const ::core::ffi::c_char; 7]> = GlobalCell::new([
    b"useopen\0".as_ptr() as *const ::core::ffi::c_char,
    b"usetab\0".as_ptr() as *const ::core::ffi::c_char,
    b"split\0".as_ptr() as *const ::core::ffi::c_char,
    b"newtab\0".as_ptr() as *const ::core::ffi::c_char,
    b"vsplit\0".as_ptr() as *const ::core::ffi::c_char,
    b"uselast\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_tcl_values: GlobalCell<[*const ::core::ffi::c_char; 3]> = GlobalCell::new([
    b"left\0".as_ptr() as *const ::core::ffi::c_char,
    b"uselast\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_tc_values: GlobalCell<[*const ::core::ffi::c_char; 6]> = GlobalCell::new([
    b"followic\0".as_ptr() as *const ::core::ffi::c_char,
    b"ignore\0".as_ptr() as *const ::core::ffi::c_char,
    b"match\0".as_ptr() as *const ::core::ffi::c_char,
    b"followscs\0".as_ptr() as *const ::core::ffi::c_char,
    b"smart\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_tpf_values: GlobalCell<[*const ::core::ffi::c_char; 8]> = GlobalCell::new([
    b"BS\0".as_ptr() as *const ::core::ffi::c_char,
    b"HT\0".as_ptr() as *const ::core::ffi::c_char,
    b"FF\0".as_ptr() as *const ::core::ffi::c_char,
    b"ESC\0".as_ptr() as *const ::core::ffi::c_char,
    b"DEL\0".as_ptr() as *const ::core::ffi::c_char,
    b"C0\0".as_ptr() as *const ::core::ffi::c_char,
    b"C1\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_ve_values: GlobalCell<[*const ::core::ffi::c_char; 7]> = GlobalCell::new([
    b"block\0".as_ptr() as *const ::core::ffi::c_char,
    b"insert\0".as_ptr() as *const ::core::ffi::c_char,
    b"all\0".as_ptr() as *const ::core::ffi::c_char,
    b"onemore\0".as_ptr() as *const ::core::ffi::c_char,
    b"none\0".as_ptr() as *const ::core::ffi::c_char,
    b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_wim_values: GlobalCell<[*const ::core::ffi::c_char; 6]> = GlobalCell::new([
    b"full\0".as_ptr() as *const ::core::ffi::c_char,
    b"longest\0".as_ptr() as *const ::core::ffi::c_char,
    b"list\0".as_ptr() as *const ::core::ffi::c_char,
    b"lastused\0".as_ptr() as *const ::core::ffi::c_char,
    b"noselect\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_wop_values: GlobalCell<[*const ::core::ffi::c_char; 5]> = GlobalCell::new([
    b"fuzzy\0".as_ptr() as *const ::core::ffi::c_char,
    b"tagfile\0".as_ptr() as *const ::core::ffi::c_char,
    b"pum\0".as_ptr() as *const ::core::ffi::c_char,
    b"exacttext\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_wak_values: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"yes\0".as_ptr() as *const ::core::ffi::c_char,
    b"menu\0".as_ptr() as *const ::core::ffi::c_char,
    b"no\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static opt_winborder_values: GlobalCell<[*const ::core::ffi::c_char; 9]> = GlobalCell::new([
    b"\0".as_ptr() as *const ::core::ffi::c_char,
    b"double\0".as_ptr() as *const ::core::ffi::c_char,
    b"single\0".as_ptr() as *const ::core::ffi::c_char,
    b"shadow\0".as_ptr() as *const ::core::ffi::c_char,
    b"rounded\0".as_ptr() as *const ::core::ffi::c_char,
    b"solid\0".as_ptr() as *const ::core::ffi::c_char,
    b"bold\0".as_ptr() as *const ::core::ffi::c_char,
    b"none\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
]);
pub static empty_string_option: GlobalCell<[::core::ffi::c_char; 1]> =
    GlobalCell::new(unsafe { ::core::mem::transmute::<[u8; 1], [::core::ffi::c_char; 1]>(*b"\0") });
pub static p_ambw: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_acd: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_ai: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_bin: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_bomb: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_bl: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_cin: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_channel: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_cink: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_cinsd: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_cinw: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_cfu: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_ofu: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_tsrfu: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_ci: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_ar: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_aw: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_awa: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_bs: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_bg: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_bk: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_bkc: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static bkc_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_bdir: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_bex: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_bo: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static breakat_flags: GlobalCell<[::core::ffi::c_char; 256]> = GlobalCell::new([0; 256]);
pub static bo_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_bsk: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_breakat: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_bh: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_bt: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_busy: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_cmp: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static cmp_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_enc: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_deco: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_ccv: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_cino: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_cedit: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_cb: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static cb_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_cwh: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ch: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_cms: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_cpt: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_cto: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_columns: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_confirm: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_cia: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static cia_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_cot: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static cot_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_ac: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_act: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_acl: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_pumborder: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_pb: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ph: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_pw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_pmw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_com: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_cpo: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_debug: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_def: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_inc: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_dia: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_dip: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_dex: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_dict: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_dg: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_dir: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_dy: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static dy_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_ead: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_emoji: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_ea: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_ep: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_eb: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_ef: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_efm: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_gefm: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_gp: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_eof: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_eol: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_ei: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_et: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_exrc: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_fenc: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_fencs: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_ff: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_ffs: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
#[no_mangle]
pub static p_fic: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_ft: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_fcs: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_ffu: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_fixeol: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_fcl: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_fdls: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_fdo: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static fdo_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_fex: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_flp: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_fo: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_fp: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_fs: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_gd: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_guicursor: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_guifont: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_guifontwide: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_hf: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_hh: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_hlg: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_hid: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_hl: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_hls: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_hi: GlobalCell<OptInt> = GlobalCell::new(0);
#[no_mangle]
pub static p_arshape: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_icon: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_iconstring: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_ic: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_iminsert: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_imsearch: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_inf: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_inex: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_is: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_inde: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_indk: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_icm: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_isf: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_isi: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_isk: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_isp: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_js: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_jop: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static jop_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_keymap: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_kp: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_km: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_langmap: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_lnr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_lrm: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_lm: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_lines: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_linespace: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_lisp: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_lop: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_lispwords: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_ls: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_stal: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_lcs: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_lz: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_lpl: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_magic: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_menc: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_mef: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_mp: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_mps: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_mat: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mco: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mfd: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mmd: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mmp: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mis: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mopt: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_msc: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_msm: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_ml: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_mle: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_mls: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ma: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_mod: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_mouse: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_mousem: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_mousemev: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_mousef: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_mh: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_mousescroll: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_mousescroll_vert: GlobalCell<OptInt> = GlobalCell::new(3 as OptInt);
pub static p_mousescroll_hor: GlobalCell<OptInt> = GlobalCell::new(6 as OptInt);
pub static p_mouset: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_more: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_nf: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_opfunc: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_para: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_paste: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_pex: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_pm: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_path: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_cdpath: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_pi: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_pyx: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_qe: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_ro: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_rdb: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static rdb_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_rdt: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_re: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_report: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_pvh: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_chi: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ari: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_ri: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_ru: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_ruf: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_pp: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_qftf: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_rtp: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_scbk: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_sj: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_so: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_sbo: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_sections: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_secure: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_sel: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_slm: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_ssop: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static ssop_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
#[no_mangle]
pub static p_sh: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
#[no_mangle]
pub static p_shcf: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_sp: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_shq: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
#[no_mangle]
pub static p_sxq: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
#[no_mangle]
pub static p_sxe: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_srr: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_stmp: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_stl: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_wbr: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_sr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_sw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_shm: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_sbr: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_sc: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_sloc: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_sft: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_sm: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_smd: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_ss: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_siso: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_scs: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_si: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_sta: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_sts: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_sb: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_sua: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_swf: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_smc: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_tpm: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_tal: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_tpf: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static tpf_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_tfu: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_spc: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_spf: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_spl: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_spo: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static spo_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_sps: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_spr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_sol: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_su: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_swb: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static swb_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_spk: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_syn: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_tcl: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static tcl_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_ts: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_tbs: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_tc: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static tc_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_tl: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_tr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_tags: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_tgst: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_tbidi: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_tw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_to: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_timeout: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_tm: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_title: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_titlelen: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_titleold: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_titlestring: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_tsr: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_tgc: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_ttimeout: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_ttm: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_tf: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
#[no_mangle]
pub static p_udir: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_udf: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_ul: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ur: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_uc: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ut: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_shada: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_shadafile: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_termsync: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_vsts: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_vts: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_vdir: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_vop: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static vop_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_vb: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_ve: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static ve_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_verbose: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_warn: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_wop: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static wop_flags: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub static p_window: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wak: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_wig: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_ww: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_wc: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wcm: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wic: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_wim: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_wmnu: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_winborder: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub static p_wh: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wmh: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wmw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wiw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wm: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ws: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_write: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_wa: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_wb: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static p_wd: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_cdh: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub static hlf_names: GlobalCell<[*const ::core::ffi::c_char; 76]> = GlobalCell::new([
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"SpecialKey\0".as_ptr() as *const ::core::ffi::c_char,
    b"EndOfBuffer\0".as_ptr() as *const ::core::ffi::c_char,
    b"TermCursor\0".as_ptr() as *const ::core::ffi::c_char,
    b"NonText\0".as_ptr() as *const ::core::ffi::c_char,
    b"Directory\0".as_ptr() as *const ::core::ffi::c_char,
    b"ErrorMsg\0".as_ptr() as *const ::core::ffi::c_char,
    b"IncSearch\0".as_ptr() as *const ::core::ffi::c_char,
    b"Search\0".as_ptr() as *const ::core::ffi::c_char,
    b"CurSearch\0".as_ptr() as *const ::core::ffi::c_char,
    b"MoreMsg\0".as_ptr() as *const ::core::ffi::c_char,
    b"ModeMsg\0".as_ptr() as *const ::core::ffi::c_char,
    b"LineNr\0".as_ptr() as *const ::core::ffi::c_char,
    b"LineNrAbove\0".as_ptr() as *const ::core::ffi::c_char,
    b"LineNrBelow\0".as_ptr() as *const ::core::ffi::c_char,
    b"CursorLineNr\0".as_ptr() as *const ::core::ffi::c_char,
    b"CursorLineSign\0".as_ptr() as *const ::core::ffi::c_char,
    b"CursorLineFold\0".as_ptr() as *const ::core::ffi::c_char,
    b"Question\0".as_ptr() as *const ::core::ffi::c_char,
    b"StatusLine\0".as_ptr() as *const ::core::ffi::c_char,
    b"StatusLineNC\0".as_ptr() as *const ::core::ffi::c_char,
    b"WinSeparator\0".as_ptr() as *const ::core::ffi::c_char,
    b"VertSplit\0".as_ptr() as *const ::core::ffi::c_char,
    b"Title\0".as_ptr() as *const ::core::ffi::c_char,
    b"Visual\0".as_ptr() as *const ::core::ffi::c_char,
    b"VisualNC\0".as_ptr() as *const ::core::ffi::c_char,
    b"WarningMsg\0".as_ptr() as *const ::core::ffi::c_char,
    b"WildMenu\0".as_ptr() as *const ::core::ffi::c_char,
    b"Folded\0".as_ptr() as *const ::core::ffi::c_char,
    b"FoldColumn\0".as_ptr() as *const ::core::ffi::c_char,
    b"DiffAdd\0".as_ptr() as *const ::core::ffi::c_char,
    b"DiffChange\0".as_ptr() as *const ::core::ffi::c_char,
    b"DiffDelete\0".as_ptr() as *const ::core::ffi::c_char,
    b"DiffText\0".as_ptr() as *const ::core::ffi::c_char,
    b"DiffTextAdd\0".as_ptr() as *const ::core::ffi::c_char,
    b"SignColumn\0".as_ptr() as *const ::core::ffi::c_char,
    b"Conceal\0".as_ptr() as *const ::core::ffi::c_char,
    b"SpellBad\0".as_ptr() as *const ::core::ffi::c_char,
    b"SpellCap\0".as_ptr() as *const ::core::ffi::c_char,
    b"SpellRare\0".as_ptr() as *const ::core::ffi::c_char,
    b"SpellLocal\0".as_ptr() as *const ::core::ffi::c_char,
    b"Pmenu\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuSel\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuMatch\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuMatchSel\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuKind\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuKindSel\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuExtra\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuExtraSel\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuSbar\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuThumb\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuBorder\0".as_ptr() as *const ::core::ffi::c_char,
    b"TabLine\0".as_ptr() as *const ::core::ffi::c_char,
    b"TabLineSel\0".as_ptr() as *const ::core::ffi::c_char,
    b"TabLineFill\0".as_ptr() as *const ::core::ffi::c_char,
    b"CursorColumn\0".as_ptr() as *const ::core::ffi::c_char,
    b"CursorLine\0".as_ptr() as *const ::core::ffi::c_char,
    b"ColorColumn\0".as_ptr() as *const ::core::ffi::c_char,
    b"QuickFixLine\0".as_ptr() as *const ::core::ffi::c_char,
    b"Whitespace\0".as_ptr() as *const ::core::ffi::c_char,
    b"NormalNC\0".as_ptr() as *const ::core::ffi::c_char,
    b"MsgSeparator\0".as_ptr() as *const ::core::ffi::c_char,
    b"NormalFloat\0".as_ptr() as *const ::core::ffi::c_char,
    b"MsgArea\0".as_ptr() as *const ::core::ffi::c_char,
    b"FloatBorder\0".as_ptr() as *const ::core::ffi::c_char,
    b"WinBar\0".as_ptr() as *const ::core::ffi::c_char,
    b"WinBarNC\0".as_ptr() as *const ::core::ffi::c_char,
    b"Cursor\0".as_ptr() as *const ::core::ffi::c_char,
    b"FloatTitle\0".as_ptr() as *const ::core::ffi::c_char,
    b"FloatFooter\0".as_ptr() as *const ::core::ffi::c_char,
    b"StatusLineTerm\0".as_ptr() as *const ::core::ffi::c_char,
    b"StatusLineTermNC\0".as_ptr() as *const ::core::ffi::c_char,
    b"StderrMsg\0".as_ptr() as *const ::core::ffi::c_char,
    b"StdoutMsg\0".as_ptr() as *const ::core::ffi::c_char,
    b"OkMsg\0".as_ptr() as *const ::core::ffi::c_char,
    b"PreInsert\0".as_ptr() as *const ::core::ffi::c_char,
]);
pub static highlight_attr: GlobalCell<[::core::ffi::c_int; 76]> = GlobalCell::new([0; 76]);
pub static highlight_attr_last: GlobalCell<[::core::ffi::c_int; 76]> = GlobalCell::new([0; 76]);
pub static highlight_user: GlobalCell<[::core::ffi::c_int; 9]> = GlobalCell::new([0; 9]);
pub static highlight_stlnc: GlobalCell<[::core::ffi::c_int; 9]> = GlobalCell::new([0; 9]);
pub static cterm_normal_fg_color: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static cterm_normal_bg_color: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static normal_fg: GlobalCell<RgbValue> = GlobalCell::new(-1 as RgbValue);
pub static normal_bg: GlobalCell<RgbValue> = GlobalCell::new(-1 as RgbValue);
pub static normal_sp: GlobalCell<RgbValue> = GlobalCell::new(-1 as RgbValue);
pub static ns_hl_global: GlobalCell<NS> = GlobalCell::new(0 as NS);
pub static ns_hl_win: GlobalCell<NS> = GlobalCell::new(-1 as NS);
pub static ns_hl_fast: GlobalCell<NS> = GlobalCell::new(-1 as NS);
pub static ns_hl_active: GlobalCell<NS> = GlobalCell::new(0 as NS);
pub static hl_attr_active: GlobalCell<*mut ::core::ffi::c_int> =
    GlobalCell::new((highlight_attr.as_raw() as *const _) as *mut ::core::ffi::c_int);
pub static curbuf_splice_pending: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub const LUA_GLOBALSINDEX: ::core::ffi::c_int = -10002 as ::core::ffi::c_int;
pub static nlua_global_refs: GlobalCell<*mut nlua_ref_state_t> =
    GlobalCell::new(::core::ptr::null_mut::<nlua_ref_state_t>());
pub static nlua_disable_preload: SharedCell<bool> = SharedCell::new(false);
pub static main_loop: SharedCell<Loop> = SharedCell::new(Loop {
    uv: uv_loop_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        active_handles: 0,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        active_reqs: C2Rust_Unnamed_18 {
            unused: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        internal_fields: ::core::ptr::null_mut::<::core::ffi::c_void>(),
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
            data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            type_0: UV_UNKNOWN_HANDLE,
            close_cb: None,
            handle_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            u: C2Rust_Unnamed_19 { fd: 0 },
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
        timer_heap: C2Rust_Unnamed_17 {
            min: ::core::ptr::null_mut::<::core::ffi::c_void>(),
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
            data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            type_0: UV_UNKNOWN_HANDLE,
            close_cb: None,
            handle_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            u: C2Rust_Unnamed_16 { fd: 0 },
            next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
            flags: 0,
            signal_cb: None,
            signum: 0,
            tree_entry: C2Rust_Unnamed_15 {
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
        inotify_watchers: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        inotify_fd: 0,
    },
    events: ::core::ptr::null_mut::<MultiQueue>(),
    thread_events: ::core::ptr::null_mut::<MultiQueue>(),
    fast_events: ::core::ptr::null_mut::<MultiQueue>(),
    children: C2Rust_Unnamed_22 {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<*mut Proc>(),
    },
    children_watcher: uv_signal_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_16 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        signal_cb: None,
        signum: 0,
        tree_entry: C2Rust_Unnamed_15 {
            rbe_left: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_right: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_parent: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_color: 0,
        },
        caught_signals: 0,
        dispatched_signals: 0,
    },
    children_kill_timer: uv_timer_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_21 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        timer_cb: None,
        node: C2Rust_Unnamed_20 {
            heap: [::core::ptr::null_mut::<::core::ffi::c_void>(); 3],
        },
        timeout: 0,
        repeat: 0,
        start_id: 0,
    },
    poll_timer: uv_timer_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_21 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        timer_cb: None,
        node: C2Rust_Unnamed_20 {
            heap: [::core::ptr::null_mut::<::core::ffi::c_void>(); 3],
        },
        timeout: 0,
        repeat: 0,
        start_id: 0,
    },
    exit_delay_timer: uv_timer_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_21 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        timer_cb: None,
        node: C2Rust_Unnamed_20 {
            heap: [::core::ptr::null_mut::<::core::ffi::c_void>(); 3],
        },
        timeout: 0,
        repeat: 0,
        start_id: 0,
    },
    async_0: uv_async_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_19 { fd: 0 },
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
static argv0: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static err_arg_missing: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"Argument missing after\0".as_ptr() as *const ::core::ffi::c_char);
static err_opt_garbage: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"Garbage after option argument\0".as_ptr() as *const ::core::ffi::c_char);
static err_opt_unknown: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"Unknown option argument\0".as_ptr() as *const ::core::ffi::c_char);
static err_too_many_args: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"Too many edit arguments\0".as_ptr() as *const ::core::ffi::c_char);
static err_extra_cmd: GlobalCell<*const ::core::ffi::c_char> = GlobalCell::new(
    b"Too many \"+command\", \"-c command\" or \"--cmd command\" arguments\0".as_ptr()
        as *const ::core::ffi::c_char,
);
#[no_mangle]
pub unsafe extern "C" fn event_init() {
    loop_init(main_loop.ptr(), NULL_0);
    env_init();
    resize_events.set(multiqueue_new_child((*main_loop.ptr()).events));
    autocmd_init();
    signal_init();
    channel_init();
    terminal_init();
    ui_init();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"event init\0".as_ptr() as *const ::core::ffi::c_char,
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
#[no_mangle]
pub unsafe extern "C" fn early_init(mut paramp: *mut mparm_T) {
    os_hint_priority();
    estack_init();
    cmdline_init();
    eval_init();
    set_vim_var_nr(VV_STARTTIME, os_realtime());
    init_path(if !(*argv0.ptr()).is_null() {
        argv0.get() as *const ::core::ffi::c_char
    } else {
        b"nvim\0".as_ptr() as *const ::core::ffi::c_char
    });
    init_normal_cmds();
    runtime_init();
    highlight_init();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"early init\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    init_locale();
    set_init_tablocal();
    win_alloc_first();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"init first window\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    alist_init(global_alist.ptr());
    (*global_alist.ptr()).id = 0 as ::core::ffi::c_int;
    init_homedir();
    set_init_1(
        if !paramp.is_null() {
            (*paramp).clean as ::core::ffi::c_int
        } else {
            false_0
        } != 0,
    );
    log_init();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"inits 1\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    set_lang_var();
    qf_init_stack();
}
unsafe fn main_0(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    argv0.set(*argv.offset(0 as ::core::ffi::c_int as isize));
    if !appname_is_valid() {
        fprintf(
            stderr,
            b"$NVIM_APPNAME must be a name or relative path.\n\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        exit(1 as ::core::ffi::c_int);
    }
    if argc > 1 as ::core::ffi::c_int
        && strcasecmp(
            *argv.offset(1 as ::core::ffi::c_int as isize),
            b"-ll\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        if argc == 2 as ::core::ffi::c_int {
            print_mainerr(
                err_arg_missing.get(),
                *argv.offset(1 as ::core::ffi::c_int as isize),
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
            exit(1 as ::core::ffi::c_int);
        }
        nlua_run_script(argv, argc, 3 as ::core::ffi::c_int);
    }
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut params: mparm_T = mparm_T {
        argc: 0,
        argv: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        use_vimrc: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        clean: false,
        n_commands: 0,
        commands: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        cmds_tofree: [0; 10],
        n_pre_commands: 0,
        pre_commands: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        luaf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        lua_arg0: 0,
        edit_type: 0,
        tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        use_ef: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        input_istext: false,
        no_swap_file: 0,
        use_debug_break_level: 0,
        window_count: 0,
        window_layout: 0,
        diff_mode: 0,
        listen_addr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        remote: 0,
        server_addr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        scriptin: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        scriptout: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        scriptout_append: false,
        had_stdin_file: false,
    };
    init_params(&raw mut params, argc, argv);
    init_startuptime(&raw mut params);
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i < params.argc {
        if strcasecmp(
            *params.argv.offset(i as isize),
            b"--clean\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
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
            b"init lua interpreter\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if embedded_mode.get() {
        let mut err: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if channel_from_stdio(
            true_0 != 0,
            CallbackReader {
                cb: Callback {
                    data: C2Rust_Unnamed_5 {
                        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    },
                    type_0: kCallbackNone,
                },
                self_0: ::core::ptr::null_mut::<dict_T>(),
                buffer: GA_EMPTY_INIT_VALUE,
                eof: false,
                buffered: false_0 != 0,
                fwd_err: false_0 != 0,
                type_0: ::core::ptr::null::<::core::ffi::c_char>(),
            },
            &raw mut err,
        ) == 0
        {
            abort();
        }
    }
    if (*global_alist.ptr()).al_ga.ga_len > 0 as ::core::ffi::c_int {
        fname = get_fname(&raw mut params);
    }
    if recoverymode.get() as ::core::ffi::c_int != 0 && fname.is_null() {
        headless_mode.set(true_0 != 0);
    }
    let mut has_term: bool = stdin_isatty.get() as ::core::ffi::c_int != 0
        || stdout_isatty.get() as ::core::ffi::c_int != 0
        || stderr_isatty.get() as ::core::ffi::c_int != 0;
    let mut use_builtin_ui: bool = has_term as ::core::ffi::c_int != 0
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
    if use_builtin_ui as ::core::ffi::c_int != 0 && !remote_ui {
        ui_client_forward_stdin.set(!stdin_isatty.get());
        let mut rv: uint64_t = ui_client_start_server(
            get_vim_var_str(VV_PROGPATH),
            params.argc as size_t,
            params.argv,
        );
        if rv == 0 {
            fprintf(
                stderr,
                b"Failed to start Nvim server!\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            os_exit(1 as ::core::ffi::c_int);
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
                b"!ui_client_channel_id && !use_builtin_ui\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/main.rs\0".as_ptr() as *const ::core::ffi::c_char,
                369 as ::core::ffi::c_uint,
                b"int main(int, char **)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if !server_init(params.listen_addr) {
        mainerr(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
    }
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"expanding arguments\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if params.diff_mode != 0 && params.window_count == -1 as ::core::ffi::c_int {
        params.window_count = 0 as ::core::ffi::c_int;
    }
    (*RedrawingDisabled.ptr()) += 1;
    setbuf(stdout, ::core::ptr::null_mut::<::core::ffi::c_char>());
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
                b"p_ch >= 0 && Rows >= p_ch && Rows - p_ch <= INT_MAX\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/main.rs\0".as_ptr() as *const ::core::ffi::c_char,
                414 as ::core::ffi::c_uint,
                b"int main(int, char **)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    cmdline_row.set(Rows.get() - p_ch.get() as ::core::ffi::c_int);
    msg_row.set(cmdline_row.get());
    default_grid_alloc();
    set_init_2(headless_mode.get());
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"inits 2\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    msg_scroll.set(true_0);
    no_wait_return.set(true_0);
    init_highlight(true_0 != 0, false_0 != 0);
    ui_comp_syn_init();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"init highlight\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    debug_break_level.set(params.use_debug_break_level);
    if !stdin_isatty.get()
        && !params.input_istext
        && silent_mode.get() as ::core::ffi::c_int != 0
        && exmode_active.get() as ::core::ffi::c_int != 0
    {
        input_start();
    }
    let mut use_remote_ui: bool =
        embedded_mode.get() as ::core::ffi::c_int != 0 && !headless_mode.get();
    if use_remote_ui {
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"waiting for UI\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
        remote_ui_wait_for_attach();
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"done waiting for UI\0".as_ptr() as *const ::core::ffi::c_char,
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
            b"clear screen\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if edit_stdin(&raw mut params) {
        params.edit_type = EDIT_STDIN as ::core::ffi::c_int;
    }
    if !params.scriptin.is_null() {
        if !open_scriptin(params.scriptin) {
            os_exit(2 as ::core::ffi::c_int);
        }
    }
    if !params.scriptout.is_null() {
        scriptout.set(os_fopen(
            params.scriptout,
            if params.scriptout_append as ::core::ffi::c_int != 0 {
                APPENDBIN.as_ptr()
            } else {
                WRITEBIN.as_ptr()
            },
        ));
        if (*scriptout.ptr()).is_null() {
            fprintf(
                stderr,
                gettext(
                    b"Cannot open for script output: \"\0".as_ptr() as *const ::core::ffi::c_char
                ),
            );
            fprintf(
                stderr,
                b"%s\"\n\0".as_ptr() as *const ::core::ffi::c_char,
                params.scriptout,
            );
            os_exit(2 as ::core::ffi::c_int);
        }
    }
    nlua_init_defaults();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"init default mappings & autocommands\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    let mut vimrc_none: bool = strequal(
        params.use_vimrc,
        b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
    );
    p_lpl.set(if vimrc_none as ::core::ffi::c_int != 0 {
        params.clean as ::core::ffi::c_int
    } else {
        p_lpl.get()
    });
    exe_pre_commands(&raw mut params);
    if !vimrc_none || params.clean as ::core::ffi::c_int != 0 {
        filetype_plugin_enable();
    }
    source_startup_scripts(&raw mut params);
    if !vimrc_none || params.clean as ::core::ffi::c_int != 0 {
        filetype_maybe_enable();
        syn_maybe_enable();
    }
    set_vim_var_nr(VV_VIM_DID_INIT, 1 as varnumber_T);
    load_plugins();
    set_window_layout(&raw mut params);
    if recoverymode.get() as ::core::ffi::c_int != 0 && fname.is_null() {
        recover_names(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            true_0 != 0,
            ::core::ptr::null_mut::<list_T>(),
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        );
        os_exit(0 as ::core::ffi::c_int);
    }
    set_init_3();
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"inits 3\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if params.no_swap_file != 0 {
        p_uc.set(0 as OptInt);
    }
    if silent_mode.get() {
        p_ut.set(1 as OptInt);
    }
    if *p_shada.get() as ::core::ffi::c_int != NUL {
        shada_read_everything(
            ::core::ptr::null::<::core::ffi::c_char>(),
            false_0 != 0,
            true_0 != 0,
        );
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"reading ShaDa\0".as_ptr() as *const ::core::ffi::c_char,
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
    if params.edit_type == EDIT_STDIN as ::core::ffi::c_int && !recoverymode.get() {
        read_stdin();
    }
    setmouse();
    redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
    no_wait_return.set(true_0);
    create_windows(&raw mut params);
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"opening buffers\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    set_vim_var_string(
        VV_SWAPCOMMAND,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    if exmode_active.get() {
        (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
    }
    apply_autocmds(
        EVENT_BUFENTER,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"BufEnter autocommands\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    setpcmark();
    if params.edit_type == EDIT_QF as ::core::ffi::c_int {
        qf_jump(
            ::core::ptr::null_mut::<qf_info_T>(),
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            false_0,
        );
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"jump to first error\0".as_ptr() as *const ::core::ffi::c_char,
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
    if params.n_commands > 0 as ::core::ffi::c_int {
        exe_commands(&raw mut params);
    }
    starting.set(0 as ::core::ffi::c_int);
    RedrawingDisabled.set(0 as ::core::ffi::c_int);
    redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
    no_wait_return.set(false_0);
    do_autochdir();
    set_vim_var_nr(VV_VIM_DID_ENTER, 1 as varnumber_T);
    apply_autocmds(
        EVENT_VIMENTER,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"VimEnter autocommands\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if use_remote_ui {
        do_autocmd_uienter_all();
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"UIEnter autocommands\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
    set_reg_var(get_default_register_name());
    if (*curwin.get()).w_onebuf_opt.wo_diff != 0 && (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
        update_topline(curwin.get());
        check_scrollbind(0 as linenr_T, 0 as ::core::ffi::c_int);
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"diff scrollbinding\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
    if restart_edit.get() != 0 as ::core::ffi::c_int {
        stuffcharReadbuff(
            -(253 as ::core::ffi::c_int
                + ((KE_NOP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        );
    }
    if cb_flags.get()
        & (kOptCbFlagUnnamed as ::core::ffi::c_int | kOptCbFlagUnnamedplus as ::core::ffi::c_int)
            as ::core::ffi::c_uint
        != 0
    {
        eval_has_provider(
            b"clipboard\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
    }
    if !params.luaf.is_null() {
        msg_scroll.set(true_0);
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"main\0".as_ptr() as *const ::core::ffi::c_char,
            678 as ::core::ffi::c_int,
            true_0 != 0,
            b"executing Lua -l script\0".as_ptr() as *const ::core::ffi::c_char,
        );
        let mut lua_ok: bool = nlua_exec_file(params.luaf);
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"executing Lua -l script\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
        if msg_didout.get() {
            msg_putchar('\n' as ::core::ffi::c_int);
            msg_didout.set(false_0 != 0);
        }
        getout(if lua_ok as ::core::ffi::c_int != 0 {
            0 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        });
    }
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"before starting main loop\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    logmsg(
        LOGLVL_INF,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"main\0".as_ptr() as *const ::core::ffi::c_char,
        689 as ::core::ffi::c_int,
        true_0 != 0,
        b"starting main loop\0".as_ptr() as *const ::core::ffi::c_char,
    );
    normal_enter(false_0 != 0, false_0 != 0);
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn os_exit(mut r: ::core::ffi::c_int) -> ! {
    exiting.set(true_0 != 0);
    if ui_client_channel_id.get() != 0 {
        ui_client_stop();
        if r == 0 as ::core::ffi::c_int {
            r = ui_client_exit_status.get();
        }
    } else {
        ui_flush();
        ui_call_stop();
    }
    if !event_teardown() && r == 0 as ::core::ffi::c_int {
        r = 1 as ::core::ffi::c_int;
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
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"os_exit\0".as_ptr() as *const ::core::ffi::c_char,
        737 as ::core::ffi::c_int,
        true_0 != 0,
        b"Nvim exit: %d\0".as_ptr() as *const ::core::ffi::c_char,
        r,
    );
    exit(r);
}
pub unsafe extern "C" fn getout(mut exitval: ::core::ffi::c_int) -> ! {
    '_c2rust_label: {
        if ui_client_channel_id.get() == 0 {
        } else {
            __assert_fail(
                b"!ui_client_channel_id\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/main.rs\0".as_ptr() as *const ::core::ffi::c_char,
                750 as ::core::ffi::c_uint,
                b"void getout(int)\0".as_ptr() as *const ::core::ffi::c_char,
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
    if *get_vim_var_str(VV_EXITREASON) as ::core::ffi::c_int == NUL {
        set_vim_var_string(
            VV_EXITREASON,
            b"quit\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
    }
    invoke_all_defer();
    hash_debug_results();
    if v_dying.get() <= 1 as ::core::ffi::c_int {
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
        let mut unblock: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if is_autocmd_blocked() {
            unblock_autocmds();
            unblock += 1;
        }
        apply_autocmds(
            EVENT_VIMLEAVEPRE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        if unblock != 0 {
            block_autocmds();
        }
    }
    if !(*p_shada.ptr()).is_null() && *p_shada.get() as ::core::ffi::c_int != NUL {
        shada_write_file(::core::ptr::null::<::core::ffi::c_char>(), false_0 != 0);
    }
    if v_dying.get() <= 1 as ::core::ffi::c_int {
        let mut unblock_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if is_autocmd_blocked() {
            unblock_autocmds();
            unblock_0 += 1;
        }
        apply_autocmds(
            EVENT_VIMLEAVE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
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
    if p_title.get() != 0 && *p_titleold.get() as ::core::ffi::c_int != NUL {
        ui_call_set_title(cstr_as_string(p_titleold.get()));
    }
    if garbage_collect_at_exit.get() {
        garbage_collect(false_0 != 0);
    }
    os_exit(exitval);
}
pub unsafe extern "C" fn preserve_exit(mut errmsg: *const ::core::ffi::c_char) -> ! {
    static really_exiting: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if really_exiting.get() {
        if used_stdin.get() {
            stream_set_blocking(STDIN_FILENO, true_0 != 0);
        }
        exit(2 as ::core::ffi::c_int);
    }
    really_exiting.set(true_0 != 0);
    signal_reject_deadly();
    if ui_client_channel_id.get() != 0 {
        ui_client_stop();
    }
    if !errmsg.is_null()
        && *errmsg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
    {
        let mut has_eol: bool = '\n' as ::core::ffi::c_int
            == *errmsg.offset(strlen(errmsg).wrapping_sub(1 as size_t) as isize)
                as ::core::ffi::c_int;
        fprintf(
            stderr,
            if has_eol as ::core::ffi::c_int != 0 {
                b"%s\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"%s\n\0".as_ptr() as *const ::core::ffi::c_char
            },
            errmsg,
        );
    }
    if ui_client_channel_id.get() != 0 {
        os_exit(1 as ::core::ffi::c_int);
    }
    ml_close_notmod();
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if !(*buf).b_ml.ml_mfp.is_null() && !(*(*buf).b_ml.ml_mfp).mf_fname.is_null() {
            if !errmsg.is_null() {
                fprintf(
                    stderr,
                    b"Nvim: preserving files...\n\0".as_ptr() as *const ::core::ffi::c_char,
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
        fprintf(
            stderr,
            b"Nvim: Finished.\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    getout(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn get_number_arg(
    mut p: *const ::core::ffi::c_char,
    mut idx: *mut ::core::ffi::c_int,
    mut def: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if ascii_isdigit(*p.offset(*idx as isize) as ::core::ffi::c_int) {
        def = atoi(p.offset(*idx as isize));
        while ascii_isdigit(*p.offset(*idx as isize) as ::core::ffi::c_int) {
            *idx = *idx + 1 as ::core::ffi::c_int;
        }
    }
    return def;
}
unsafe extern "C" fn server_connect(
    mut server_addr: *mut ::core::ffi::c_char,
    mut errmsg: *mut *const ::core::ffi::c_char,
) -> uint64_t {
    if server_addr.is_null() {
        *errmsg = b"no address specified\0".as_ptr() as *const ::core::ffi::c_char;
        return 0 as uint64_t;
    }
    let mut on_data: CallbackReader = CallbackReader {
        cb: Callback {
            data: C2Rust_Unnamed_5 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        self_0: ::core::ptr::null_mut::<dict_T>(),
        buffer: GA_EMPTY_INIT_VALUE,
        eof: false,
        buffered: false_0 != 0,
        fwd_err: false_0 != 0,
        type_0: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut is_tcp: bool = !socket_address_tcp_host_end(server_addr).is_null();
    let mut chan: uint64_t = channel_connect(
        is_tcp,
        server_addr,
        true_0 != 0,
        on_data,
        500 as ::core::ffi::c_int,
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
    mut remote_args: ::core::ffi::c_int,
    mut server_addr: *mut ::core::ffi::c_char,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
    mut ui_only: bool,
) {
    let mut is_ui: bool = strequal(
        *argv.offset(remote_args as isize),
        b"--remote-ui\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if ui_only as ::core::ffi::c_int != 0 && !is_ui {
        return;
    }
    let mut connect_error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut chan: uint64_t = server_connect(server_addr, &raw mut connect_error);
    let mut rvobj: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    if is_ui {
        if chan == 0 {
            fprintf(
                stderr,
                b"Remote ui failed to start: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                connect_error,
            );
            os_exit(1 as ::core::ffi::c_int);
        } else if strequal(
            server_addr,
            os_getenv_noalloc(b"NVIM\0".as_ptr() as *const ::core::ffi::c_char),
        ) {
            fprintf(
                stderr,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"Cannot attach UI of :terminal child to its parent. \0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            fprintf(
                stderr,
                b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
                b"(Unset $NVIM to skip this check)\0".as_ptr() as *const ::core::ffi::c_char,
            );
            os_exit(1 as ::core::ffi::c_int);
        }
        ui_client_channel_id.set(chan);
        return;
    }
    let mut args: Array = ARRAY_DICT_INIT;
    args.capacity = (argc - remote_args) as size_t;
    args.items = xrealloc(
        args.items as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<Object>().wrapping_mul(args.capacity),
    ) as *mut Object;
    let mut t_argc: ::core::ffi::c_int = remote_args;
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
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
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
            integer: chan as ::core::ffi::c_int as Integer,
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
        data: b"return vim._cs_remote(...)\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        size: ::core::mem::size_of::<[::core::ffi::c_char; 27]>().wrapping_sub(1 as size_t),
    };
    let mut o: Object = nlua_exec(
        s,
        ::core::ptr::null::<::core::ffi::c_char>(),
        a,
        kRetObject,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    xfree(args.items as *mut ::core::ffi::c_void);
    args.capacity = 0 as size_t;
    args.size = args.capacity;
    args.items = ::core::ptr::null_mut::<Object>();
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        fprintf(
            stderr,
            b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
            err.msg,
        );
        os_exit(2 as ::core::ffi::c_int);
    }
    if o.type_0 as ::core::ffi::c_uint
        == kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        rvobj.data.dict = o.data.dict;
    } else {
        fprintf(
            stderr,
            b"vim._cs_remote returned unexpected value\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        os_exit(2 as ::core::ffi::c_int);
    }
    let mut should_exit: TriState = kNone;
    let mut tabbed: TriState = kNone;
    let mut i: size_t = 0 as size_t;
    while i < rvobj.data.dict.size {
        if strequal(
            (*rvobj.data.dict.items.offset(i as isize)).key.data,
            b"errmsg\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            if (*rvobj.data.dict.items.offset(i as isize)).value.type_0 as ::core::ffi::c_uint
                != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                fprintf(
                    stderr,
                    b"vim._cs_remote returned an unexpected type for 'errmsg'\n\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
                os_exit(2 as ::core::ffi::c_int);
            }
            fprintf(
                stderr,
                b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
                (*rvobj.data.dict.items.offset(i as isize))
                    .value
                    .data
                    .string
                    .data,
            );
            os_exit(2 as ::core::ffi::c_int);
        } else if strequal(
            (*rvobj.data.dict.items.offset(i as isize)).key.data,
            b"result\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            if (*rvobj.data.dict.items.offset(i as isize)).value.type_0 as ::core::ffi::c_uint
                != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                fprintf(
                    stderr,
                    b"vim._cs_remote returned an unexpected type for 'result'\n\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
                os_exit(2 as ::core::ffi::c_int);
            }
            printf(
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                (*rvobj.data.dict.items.offset(i as isize))
                    .value
                    .data
                    .string
                    .data,
            );
        } else if strequal(
            (*rvobj.data.dict.items.offset(i as isize)).key.data,
            b"tabbed\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            if (*rvobj.data.dict.items.offset(i as isize)).value.type_0 as ::core::ffi::c_uint
                != kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                fprintf(
                    stderr,
                    b"vim._cs_remote returned an unexpected type for 'tabbed'\n\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
                os_exit(2 as ::core::ffi::c_int);
            }
            tabbed = (if (*rvobj.data.dict.items.offset(i as isize))
                .value
                .data
                .boolean as ::core::ffi::c_int
                != 0
            {
                kTrue as ::core::ffi::c_int
            } else {
                kFalse as ::core::ffi::c_int
            }) as TriState;
        } else if strequal(
            (*rvobj.data.dict.items.offset(i as isize)).key.data,
            b"should_exit\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            if (*rvobj.data.dict.items.offset(i as isize)).value.type_0 as ::core::ffi::c_uint
                != kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                fprintf(
                    stderr,
                    b"vim._cs_remote returned an unexpected type for 'should_exit'\n\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
                os_exit(2 as ::core::ffi::c_int);
            }
            should_exit = (if (*rvobj.data.dict.items.offset(i as isize))
                .value
                .data
                .boolean as ::core::ffi::c_int
                != 0
            {
                kTrue as ::core::ffi::c_int
            } else {
                kFalse as ::core::ffi::c_int
            }) as TriState;
        }
        i = i.wrapping_add(1);
    }
    if should_exit as ::core::ffi::c_int == kNone as ::core::ffi::c_int
        || tabbed as ::core::ffi::c_int == kNone as ::core::ffi::c_int
    {
        fprintf(
            stderr,
            b"vim._cs_remote didn't return a value for should_exit or tabbed, bailing\n\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        os_exit(2 as ::core::ffi::c_int);
    }
    api_free_object(o);
    if should_exit as ::core::ffi::c_int == kTrue as ::core::ffi::c_int {
        os_exit(0 as ::core::ffi::c_int);
    }
    if tabbed as ::core::ffi::c_int == kTrue as ::core::ffi::c_int {
        (*params).window_count = argc - remote_args - 1 as ::core::ffi::c_int;
        (*params).window_layout = WIN_TABS as ::core::ffi::c_int;
    }
}
unsafe extern "C" fn edit_stdin(mut parmp: *mut mparm_T) -> bool {
    let mut implicit: bool = !headless_mode.get()
        && !(embedded_mode.get() as ::core::ffi::c_int != 0
            && stdin_fd.get() <= 0 as ::core::ffi::c_int)
        && (!exmode_active.get() || (*parmp).input_istext as ::core::ffi::c_int != 0)
        && !stdin_isatty.get()
        && (*parmp).edit_type <= EDIT_STDIN as ::core::ffi::c_int
        && (*parmp).scriptin.is_null();
    return (*parmp).had_stdin_file as ::core::ffi::c_int != 0
        || implicit as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn command_line_scan(mut parmp: *mut mparm_T) {
    let mut argc: ::core::ffi::c_int = (*parmp).argc;
    let mut argv: *mut *mut ::core::ffi::c_char = (*parmp).argv;
    let mut argv_idx: ::core::ffi::c_int = 0;
    let mut had_minmin: bool = false_0 != 0;
    let mut want_argument: bool = false;
    let mut n: ::core::ffi::c_int = 0;
    argc -= 1;
    argv = argv.offset(1);
    argv_idx = 1 as ::core::ffi::c_int;
    while argc > 0 as ::core::ffi::c_int {
        if *(*argv.offset(0 as ::core::ffi::c_int as isize))
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '+' as ::core::ffi::c_int
            && !had_minmin
        {
            if (*parmp).n_commands >= MAX_ARG_CMDS {
                mainerr(
                    err_extra_cmd.get(),
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    ::core::ptr::null::<::core::ffi::c_char>(),
                );
            }
            argv_idx = -1 as ::core::ffi::c_int;
            if *(*argv.offset(0 as ::core::ffi::c_int as isize))
                .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == NUL
            {
                let c2rust_fresh6 = (*parmp).n_commands;
                (*parmp).n_commands = (*parmp).n_commands + 1;
                let c2rust_lvalue_ptr = &raw mut (*parmp).commands[c2rust_fresh6 as usize];
                *c2rust_lvalue_ptr =
                    b"$\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                let c2rust_fresh7 = (*parmp).n_commands;
                (*parmp).n_commands = (*parmp).n_commands + 1;
                let c2rust_lvalue_ptr_0 = &raw mut (*parmp).commands[c2rust_fresh7 as usize];
                *c2rust_lvalue_ptr_0 = (*argv.offset(0 as ::core::ffi::c_int as isize))
                    .offset(1 as ::core::ffi::c_int as isize);
            }
        } else if *(*argv.offset(0 as ::core::ffi::c_int as isize))
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '-' as ::core::ffi::c_int
            && !had_minmin
        {
            want_argument = false_0 != 0;
            let c2rust_fresh8 = argv_idx;
            argv_idx = argv_idx + 1;
            let mut c: ::core::ffi::c_char =
                *(*argv.offset(0 as ::core::ffi::c_int as isize)).offset(c2rust_fresh8 as isize);
            's_747: {
                'c_49604: {
                    match c as ::core::ffi::c_int {
                        NUL => {
                            if exmode_active.get() {
                                silent_mode.set(true_0 != 0);
                                (*parmp).no_swap_file = true_0;
                            } else {
                                if (*parmp).edit_type > EDIT_STDIN as ::core::ffi::c_int {
                                    mainerr(
                                        err_too_many_args.get(),
                                        *argv.offset(0 as ::core::ffi::c_int as isize),
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                    );
                                }
                                (*parmp).had_stdin_file = true_0 != 0;
                                (*parmp).edit_type = EDIT_STDIN as ::core::ffi::c_int;
                            }
                            argv_idx = -1 as ::core::ffi::c_int;
                            break 's_747;
                        }
                        45 => {
                            if strcasecmp(
                                (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize),
                                b"help\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                            ) == 0 as ::core::ffi::c_int
                            {
                                usage();
                                os_exit(0 as ::core::ffi::c_int);
                            } else if strcasecmp(
                                (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize),
                                b"version\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                            ) == 0 as ::core::ffi::c_int
                            {
                                version();
                                os_exit(0 as ::core::ffi::c_int);
                            } else if strcasecmp(
                                (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize),
                                b"api-info\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                            ) == 0 as ::core::ffi::c_int
                            {
                                let mut data: String_0 = api_metadata_raw();
                                let written_bytes: ptrdiff_t =
                                    os_write(STDOUT_FILENO, data.data, data.size, false_0 != 0);
                                if written_bytes < 0 as ptrdiff_t {
                                    semsg(
                                        gettext(b"E5420: Failed to write to file: %s\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                        uv_strerror(written_bytes as ::core::ffi::c_int),
                                    );
                                }
                                os_exit(0 as ::core::ffi::c_int);
                            } else if strcasecmp(
                                (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize),
                                b"headless\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                            ) == 0 as ::core::ffi::c_int
                            {
                                headless_mode.set(true_0 != 0);
                            } else if strcasecmp(
                                (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize),
                                b"embed\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                            ) == 0 as ::core::ffi::c_int
                            {
                                embedded_mode.set(true_0 != 0);
                            } else if strncasecmp(
                                (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize),
                                b"listen\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                6 as ::core::ffi::c_int as size_t,
                            ) == 0 as ::core::ffi::c_int
                            {
                                want_argument = true_0 != 0;
                                argv_idx += 6 as ::core::ffi::c_int;
                            } else if strncasecmp(
                                (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize),
                                b"literal\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                7 as ::core::ffi::c_int as size_t,
                            ) != 0 as ::core::ffi::c_int
                            {
                                if strncasecmp(
                                    (*argv.offset(0 as ::core::ffi::c_int as isize))
                                        .offset(argv_idx as isize),
                                    b"remote\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    6 as ::core::ffi::c_int as size_t,
                                ) == 0 as ::core::ffi::c_int
                                {
                                    (*parmp).remote = (*parmp).argc - argc;
                                } else if strncasecmp(
                                    (*argv.offset(0 as ::core::ffi::c_int as isize))
                                        .offset(argv_idx as isize),
                                    b"server\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    6 as ::core::ffi::c_int as size_t,
                                ) == 0 as ::core::ffi::c_int
                                {
                                    want_argument = true_0 != 0;
                                    argv_idx += 6 as ::core::ffi::c_int;
                                } else if strncasecmp(
                                    (*argv.offset(0 as ::core::ffi::c_int as isize))
                                        .offset(argv_idx as isize),
                                    b"noplugin\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    8 as ::core::ffi::c_int as size_t,
                                ) == 0 as ::core::ffi::c_int
                                {
                                    p_lpl.set(false_0);
                                } else if strncasecmp(
                                    (*argv.offset(0 as ::core::ffi::c_int as isize))
                                        .offset(argv_idx as isize),
                                    b"cmd\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    3 as ::core::ffi::c_int as size_t,
                                ) == 0 as ::core::ffi::c_int
                                {
                                    want_argument = true_0 != 0;
                                    argv_idx += 3 as ::core::ffi::c_int;
                                } else if strncasecmp(
                                    (*argv.offset(0 as ::core::ffi::c_int as isize))
                                        .offset(argv_idx as isize),
                                    b"startuptime\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    11 as ::core::ffi::c_int as size_t,
                                ) == 0 as ::core::ffi::c_int
                                {
                                    want_argument = true_0 != 0;
                                    argv_idx += 11 as ::core::ffi::c_int;
                                } else if strncasecmp(
                                    (*argv.offset(0 as ::core::ffi::c_int as isize))
                                        .offset(argv_idx as isize),
                                    b"clean\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    5 as ::core::ffi::c_int as size_t,
                                ) == 0 as ::core::ffi::c_int
                                {
                                    (*parmp).use_vimrc = b"NONE\0".as_ptr()
                                        as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char;
                                    (*parmp).clean = true_0 != 0;
                                    set_option_value_give_err(
                                        kOptShadafile,
                                        OptVal {
                                            type_0: kOptValTypeString,
                                            data: OptValData {
                                                string: String_0 {
                                                    data: b"NONE\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                        as *mut ::core::ffi::c_char,
                                                    size: ::core::mem::size_of::<
                                                        [::core::ffi::c_char; 5],
                                                    >(
                                                    )
                                                    .wrapping_sub(1 as size_t),
                                                },
                                            },
                                        },
                                        0 as ::core::ffi::c_int,
                                    );
                                } else if strncasecmp(
                                    (*argv.offset(0 as ::core::ffi::c_int as isize))
                                        .offset(argv_idx as isize),
                                    b"luamod-dev\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    9 as ::core::ffi::c_int as size_t,
                                ) == 0 as ::core::ffi::c_int
                                {
                                    nlua_disable_preload.set(true_0 != 0);
                                } else {
                                    if *(*argv.offset(0 as ::core::ffi::c_int as isize))
                                        .offset(argv_idx as isize)
                                        != 0
                                    {
                                        mainerr(
                                            err_opt_unknown.get(),
                                            *argv.offset(0 as ::core::ffi::c_int as isize),
                                            ::core::ptr::null::<::core::ffi::c_char>(),
                                        );
                                    }
                                    had_minmin = true_0 != 0;
                                }
                            }
                            if !want_argument {
                                argv_idx = -1 as ::core::ffi::c_int;
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
                                0 as ::core::ffi::c_int,
                            );
                            break 's_747;
                        }
                        98 => {
                            set_options_bin(
                                (*curbuf.get()).b_p_bin,
                                1 as ::core::ffi::c_int,
                                0 as ::core::ffi::c_int,
                            );
                            (*curbuf.get()).b_p_bin = 1 as ::core::ffi::c_int;
                            break 's_747;
                        }
                        68 => {
                            (*parmp).use_debug_break_level = 9999 as ::core::ffi::c_int;
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
                            os_exit(0 as ::core::ffi::c_int);
                        }
                        72 => {
                            set_option_value_give_err(
                                kOptKeymap,
                                OptVal {
                                    type_0: kOptValTypeString,
                                    data: OptValData {
                                        string: String_0 {
                                            data: b"hebrew\0".as_ptr() as *const ::core::ffi::c_char
                                                as *mut ::core::ffi::c_char,
                                            size: ::core::mem::size_of::<[::core::ffi::c_char; 7]>(
                                            )
                                            .wrapping_sub(1 as size_t),
                                        },
                                    },
                                },
                                0 as ::core::ffi::c_int,
                            );
                            set_option_value_give_err(
                                kOptRightleft,
                                OptVal {
                                    type_0: kOptValTypeBoolean,
                                    data: OptValData { boolean: kTrue },
                                },
                                0 as ::core::ffi::c_int,
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
                                *argv.offset(0 as ::core::ffi::c_int as isize),
                                &raw mut argv_idx,
                                0 as ::core::ffi::c_int,
                            );
                            (*parmp).window_layout = WIN_TABS as ::core::ffi::c_int;
                            break 's_747;
                        }
                        111 => {
                            (*parmp).window_count = get_number_arg(
                                *argv.offset(0 as ::core::ffi::c_int as isize),
                                &raw mut argv_idx,
                                0 as ::core::ffi::c_int,
                            );
                            (*parmp).window_layout = WIN_HOR as ::core::ffi::c_int;
                            break 's_747;
                        }
                        79 => {
                            (*parmp).window_count = get_number_arg(
                                *argv.offset(0 as ::core::ffi::c_int as isize),
                                &raw mut argv_idx,
                                0 as ::core::ffi::c_int,
                            );
                            (*parmp).window_layout = WIN_VER as ::core::ffi::c_int;
                            break 's_747;
                        }
                        113 => {
                            if (*parmp).edit_type != EDIT_NONE as ::core::ffi::c_int {
                                mainerr(
                                    err_too_many_args.get(),
                                    *argv.offset(0 as ::core::ffi::c_int as isize),
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                );
                            }
                            (*parmp).edit_type = EDIT_QF as ::core::ffi::c_int;
                            if *(*argv.offset(0 as ::core::ffi::c_int as isize))
                                .offset(argv_idx as isize)
                                != 0
                            {
                                (*parmp).use_ef = (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize);
                                argv_idx = -1 as ::core::ffi::c_int;
                            } else if argc > 1 as ::core::ffi::c_int {
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
                                    || *p_shadafile.get() as ::core::ffi::c_int == NUL
                                {
                                    set_option_value_give_err(
                                        kOptShadafile,
                                        OptVal {
                                            type_0: kOptValTypeString,
                                            data: OptValData {
                                                string: String_0 {
                                                    data: b"NONE\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                        as *mut ::core::ffi::c_char,
                                                    size: ::core::mem::size_of::<
                                                        [::core::ffi::c_char; 5],
                                                    >(
                                                    )
                                                    .wrapping_sub(1 as size_t),
                                                },
                                            },
                                        },
                                        0 as ::core::ffi::c_int,
                                    );
                                }
                            } else {
                                want_argument = true_0 != 0;
                            }
                            break 's_747;
                        }
                        116 => {
                            if (*parmp).edit_type != EDIT_NONE as ::core::ffi::c_int {
                                mainerr(
                                    err_too_many_args.get(),
                                    *argv.offset(0 as ::core::ffi::c_int as isize),
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                );
                            }
                            (*parmp).edit_type = EDIT_TAG as ::core::ffi::c_int;
                            if *(*argv.offset(0 as ::core::ffi::c_int as isize))
                                .offset(argv_idx as isize)
                                != 0
                            {
                                (*parmp).tagname = (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize);
                                argv_idx = -1 as ::core::ffi::c_int;
                            } else {
                                want_argument = true_0 != 0;
                            }
                            break 's_747;
                        }
                        118 => {
                            version();
                            os_exit(0 as ::core::ffi::c_int);
                        }
                        86 => {
                            p_verbose.set(get_number_arg(
                                *argv.offset(0 as ::core::ffi::c_int as isize),
                                &raw mut argv_idx,
                                10 as ::core::ffi::c_int,
                            ) as OptInt);
                            if *(*argv.offset(0 as ::core::ffi::c_int as isize))
                                .offset(argv_idx as isize)
                                as ::core::ffi::c_int
                                != NUL
                            {
                                set_option_value_give_err(
                                    kOptVerbosefile,
                                    OptVal {
                                        type_0: kOptValTypeString,
                                        data: OptValData {
                                            string: cstr_as_string(
                                                (*argv.offset(0 as ::core::ffi::c_int as isize))
                                                    .offset(argv_idx as isize),
                                            ),
                                        },
                                    },
                                    0 as ::core::ffi::c_int,
                                );
                                argv_idx = strlen(*argv.offset(0 as ::core::ffi::c_int as isize))
                                    as ::core::ffi::c_int;
                            }
                            break 's_747;
                        }
                        119 => {
                            if ascii_isdigit(
                                *(*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize)
                                    as ::core::ffi::c_int,
                            ) {
                                n = get_number_arg(
                                    *argv.offset(0 as ::core::ffi::c_int as isize),
                                    &raw mut argv_idx,
                                    10 as ::core::ffi::c_int,
                                );
                                set_option_value_give_err(
                                    kOptWindow,
                                    OptVal {
                                        type_0: kOptValTypeNumber,
                                        data: OptValData {
                                            number: n as OptInt,
                                        },
                                    },
                                    0 as ::core::ffi::c_int,
                                );
                                break 's_747;
                            } else {
                                want_argument = true_0 != 0;
                                break 's_747;
                            }
                        }
                        99 => {
                            if *(*argv.offset(0 as ::core::ffi::c_int as isize))
                                .offset(argv_idx as isize)
                                as ::core::ffi::c_int
                                != NUL
                            {
                                if (*parmp).n_commands >= MAX_ARG_CMDS {
                                    mainerr(
                                        err_extra_cmd.get(),
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                    );
                                }
                                let c2rust_fresh9 = (*parmp).n_commands;
                                (*parmp).n_commands = (*parmp).n_commands + 1;
                                let c2rust_lvalue_ptr_1 =
                                    &raw mut (*parmp).commands[c2rust_fresh9 as usize];
                                *c2rust_lvalue_ptr_1 = (*argv
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .offset(argv_idx as isize);
                                argv_idx = -1 as ::core::ffi::c_int;
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
                                *argv.offset(0 as ::core::ffi::c_int as isize),
                                ::core::ptr::null::<::core::ffi::c_char>(),
                            );
                        }
                    }
                    p_write.set(false_0);
                    break 's_747;
                }
                want_argument = true_0 != 0;
            }
            if want_argument {
                if *(*argv.offset(0 as ::core::ffi::c_int as isize)).offset(argv_idx as isize)
                    as ::core::ffi::c_int
                    != NUL
                {
                    mainerr(
                        err_opt_garbage.get(),
                        *argv.offset(0 as ::core::ffi::c_int as isize),
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                }
                argc -= 1;
                if argc < 1 as ::core::ffi::c_int
                    && c as ::core::ffi::c_int != 'S' as ::core::ffi::c_int
                {
                    mainerr(
                        err_arg_missing.get(),
                        *argv.offset(0 as ::core::ffi::c_int as isize),
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                }
                argv = argv.offset(1);
                argv_idx = -1 as ::core::ffi::c_int;
                's_1076: {
                    '_scripterror: {
                        's_1075: {
                            match c as ::core::ffi::c_int {
                                99 | 83 => {
                                    if (*parmp).n_commands >= MAX_ARG_CMDS {
                                        mainerr(
                                            err_extra_cmd.get(),
                                            ::core::ptr::null::<::core::ffi::c_char>(),
                                            ::core::ptr::null::<::core::ffi::c_char>(),
                                        );
                                    }
                                    if c as ::core::ffi::c_int == 'S' as ::core::ffi::c_int {
                                        let mut a: *mut ::core::ffi::c_char =
                                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                                        if argc < 1 as ::core::ffi::c_int {
                                            a = SESSION_FILE.as_ptr() as *mut ::core::ffi::c_char;
                                        } else if *(*argv.offset(0 as ::core::ffi::c_int as isize))
                                            .offset(0 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '-' as ::core::ffi::c_int
                                        {
                                            a = SESSION_FILE.as_ptr() as *mut ::core::ffi::c_char;
                                            argc += 1;
                                            argv = argv.offset(-1);
                                        } else {
                                            a = *argv.offset(0 as ::core::ffi::c_int as isize);
                                        }
                                        let mut s_size: size_t =
                                            strlen(a).wrapping_add(9 as size_t);
                                        let mut s: *mut ::core::ffi::c_char =
                                            xmalloc(s_size) as *mut ::core::ffi::c_char;
                                        snprintf(
                                            s,
                                            s_size,
                                            b"so %s\0".as_ptr() as *const ::core::ffi::c_char,
                                            a,
                                        );
                                        (*parmp).cmds_tofree[(*parmp).n_commands as usize] =
                                            true_0 as ::core::ffi::c_char;
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
                                        *c2rust_lvalue_ptr_3 =
                                            *argv.offset(0 as ::core::ffi::c_int as isize);
                                    }
                                    break 's_1075;
                                }
                                45 => {
                                    if strequal(
                                        *argv.offset(-1 as ::core::ffi::c_int as isize),
                                        b"--cmd\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) {
                                        if (*parmp).n_pre_commands >= MAX_ARG_CMDS {
                                            mainerr(
                                                err_extra_cmd.get(),
                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                            );
                                        }
                                        let c2rust_fresh12 = (*parmp).n_pre_commands;
                                        (*parmp).n_pre_commands = (*parmp).n_pre_commands + 1;
                                        let c2rust_lvalue_ptr_4 =
                                            &raw mut (*parmp).pre_commands[c2rust_fresh12 as usize];
                                        *c2rust_lvalue_ptr_4 =
                                            *argv.offset(0 as ::core::ffi::c_int as isize);
                                    } else if strequal(
                                        *argv.offset(-1 as ::core::ffi::c_int as isize),
                                        b"--listen\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) {
                                        (*parmp).listen_addr =
                                            *argv.offset(0 as ::core::ffi::c_int as isize);
                                    } else if strequal(
                                        *argv.offset(-1 as ::core::ffi::c_int as isize),
                                        b"--server\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) {
                                        (*parmp).server_addr =
                                            *argv.offset(0 as ::core::ffi::c_int as isize);
                                    }
                                    break 's_1075;
                                }
                                113 => {
                                    (*parmp).use_ef =
                                        *argv.offset(0 as ::core::ffi::c_int as isize);
                                    break 's_1075;
                                }
                                105 => {
                                    set_option_value_give_err(
                                        kOptShadafile,
                                        OptVal {
                                            type_0: kOptValTypeString,
                                            data: OptValData {
                                                string: cstr_as_string(
                                                    *argv.offset(0 as ::core::ffi::c_int as isize),
                                                ),
                                            },
                                        },
                                        0 as ::core::ffi::c_int,
                                    );
                                    break 's_1075;
                                }
                                108 => {
                                    headless_mode.set(true_0 != 0);
                                    silent_mode.set(true_0 != 0);
                                    p_verbose.set(1 as OptInt);
                                    (*parmp).no_swap_file = true_0;
                                    (*parmp).use_vimrc = (if !(*parmp).use_vimrc.is_null() {
                                        (*parmp).use_vimrc as *const ::core::ffi::c_char
                                    } else {
                                        b"NONE\0".as_ptr() as *const ::core::ffi::c_char
                                    })
                                        as *mut ::core::ffi::c_char;
                                    if (*p_shadafile.ptr()).is_null()
                                        || *p_shadafile.get() as ::core::ffi::c_int == NUL
                                    {
                                        set_option_value_give_err(
                                            kOptShadafile,
                                            OptVal {
                                                type_0: kOptValTypeString,
                                                data: OptValData {
                                                    string: String_0 {
                                                        data: b"NONE\0".as_ptr()
                                                            as *const ::core::ffi::c_char
                                                            as *mut ::core::ffi::c_char,
                                                        size: ::core::mem::size_of::<
                                                            [::core::ffi::c_char; 5],
                                                        >(
                                                        )
                                                        .wrapping_sub(1 as size_t),
                                                    },
                                                },
                                            },
                                            0 as ::core::ffi::c_int,
                                        );
                                    }
                                    (*parmp).luaf = *argv.offset(0 as ::core::ffi::c_int as isize);
                                    argc -= 1;
                                    if argc >= 0 as ::core::ffi::c_int {
                                        (*parmp).lua_arg0 = (*parmp).argc - argc;
                                        argc = 0 as ::core::ffi::c_int;
                                    }
                                    break 's_1075;
                                }
                                115 => {
                                    if !(*parmp).scriptin.is_null() {
                                        break '_scripterror;
                                    } else {
                                        (*parmp).scriptin =
                                            *argv.offset(0 as ::core::ffi::c_int as isize);
                                        break 's_1075;
                                    }
                                }
                                116 => {
                                    (*parmp).tagname =
                                        *argv.offset(0 as ::core::ffi::c_int as isize);
                                    break 's_1075;
                                }
                                117 => {
                                    (*parmp).use_vimrc =
                                        *argv.offset(0 as ::core::ffi::c_int as isize);
                                    break 's_1075;
                                }
                                119 => {
                                    if ascii_isdigit(
                                        **argv.offset(0 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int,
                                    ) {
                                        argv_idx = 0 as ::core::ffi::c_int;
                                        n = get_number_arg(
                                            *argv.offset(0 as ::core::ffi::c_int as isize),
                                            &raw mut argv_idx,
                                            10 as ::core::ffi::c_int,
                                        );
                                        set_option_value_give_err(
                                            kOptWindow,
                                            OptVal {
                                                type_0: kOptValTypeNumber,
                                                data: OptValData {
                                                    number: n as OptInt,
                                                },
                                            },
                                            0 as ::core::ffi::c_int,
                                        );
                                        argv_idx = -1 as ::core::ffi::c_int;
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
                                (*parmp).scriptout = *argv.offset(0 as ::core::ffi::c_int as isize);
                                (*parmp).scriptout_append =
                                    c as ::core::ffi::c_int == 'w' as ::core::ffi::c_int;
                            }
                        }
                        break 's_1076;
                    }
                    vim_snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        gettext(b"Attempt to open script file again: \"%s %s\"\n\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        *argv.offset(-1 as ::core::ffi::c_int as isize),
                        *argv.offset(0 as ::core::ffi::c_int as isize),
                    );
                    fprintf(
                        stderr,
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                    );
                    os_exit(2 as ::core::ffi::c_int);
                }
            }
        } else {
            argv_idx = -1 as ::core::ffi::c_int;
            if (*parmp).edit_type > EDIT_STDIN as ::core::ffi::c_int {
                mainerr(
                    err_too_many_args.get(),
                    *argv.offset(0 as ::core::ffi::c_int as isize),
                    ::core::ptr::null::<::core::ffi::c_char>(),
                );
            }
            (*parmp).edit_type = EDIT_FILE as ::core::ffi::c_int;
            ga_grow(
                &raw mut (*global_alist.ptr()).al_ga,
                1 as ::core::ffi::c_int,
            );
            let mut p: *mut ::core::ffi::c_char =
                xstrdup(*argv.offset(0 as ::core::ffi::c_int as isize));
            if (*parmp).diff_mode != 0
                && os_isdir(p) as ::core::ffi::c_int != 0
                && (*global_alist.ptr()).al_ga.ga_len > 0 as ::core::ffi::c_int
                && !os_isdir(alist_name(
                    ((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T)
                        .offset(0 as ::core::ffi::c_int as isize),
                ))
            {
                let mut r: *mut ::core::ffi::c_char = concat_fnames(
                    p,
                    path_tail(alist_name(
                        ((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T)
                            .offset(0 as ::core::ffi::c_int as isize),
                    )),
                    true_0 != 0,
                );
                xfree(p as *mut ::core::ffi::c_void);
                p = r;
            }
            let mut alist_fnum_flag: ::core::ffi::c_int =
                if edit_stdin(parmp) as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    2 as ::core::ffi::c_int
                };
            alist_add(global_alist.ptr(), p, alist_fnum_flag);
        }
        if argv_idx <= 0 as ::core::ffi::c_int
            || *(*argv.offset(0 as ::core::ffi::c_int as isize)).offset(argv_idx as isize)
                as ::core::ffi::c_int
                == NUL
        {
            argc -= 1;
            argv = argv.offset(1);
            argv_idx = 1 as ::core::ffi::c_int;
        }
    }
    if embedded_mode.get() as ::core::ffi::c_int != 0
        && (silent_mode.get() as ::core::ffi::c_int != 0 || !(*parmp).luaf.is_null())
    {
        mainerr(
            gettext(b"--embed conflicts with -es/-Es/-l\0".as_ptr() as *const ::core::ffi::c_char),
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
    }
    if (*parmp).n_commands > 0 as ::core::ffi::c_int {
        let swcmd_len: size_t =
            strlen((*parmp).commands[0 as ::core::ffi::c_int as usize]).wrapping_add(2 as size_t);
        let swcmd: *mut ::core::ffi::c_char =
            xmalloc(swcmd_len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        snprintf(
            swcmd,
            swcmd_len.wrapping_add(1 as size_t),
            b":%s\r\0".as_ptr() as *const ::core::ffi::c_char,
            (*parmp).commands[0 as ::core::ffi::c_int as usize],
        );
        set_vim_var_string(VV_SWAPCOMMAND, swcmd, swcmd_len as ptrdiff_t);
        xfree(swcmd as *mut ::core::ffi::c_void);
    }
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"parsing arguments\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
unsafe extern "C" fn set_argf_var() {
    let mut list: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*global_alist.ptr()).al_ga.ga_len {
        let mut fname: *mut ::core::ffi::c_char =
            alist_name(((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T).offset(i as isize));
        if !fname.is_null() {
            vim_FullName(
                fname,
                NameBuff.ptr() as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                false_0 != 0,
            );
            tv_list_append_string(
                list,
                NameBuff.ptr() as *mut ::core::ffi::c_char,
                -1 as ssize_t,
            );
        }
        i += 1;
    }
    tv_list_set_lock(list, VAR_FIXED);
    set_vim_var_list(VV_ARGF, list);
}
unsafe extern "C" fn init_params(
    mut paramp: *mut mparm_T,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) {
    memset(
        paramp as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<mparm_T>(),
    );
    (*paramp).argc = argc;
    (*paramp).argv = argv;
    (*paramp).use_debug_break_level = -1 as ::core::ffi::c_int;
    (*paramp).window_count = -1 as ::core::ffi::c_int;
    (*paramp).listen_addr = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*paramp).server_addr = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*paramp).remote = 0 as ::core::ffi::c_int;
    (*paramp).luaf = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*paramp).lua_arg0 = -1 as ::core::ffi::c_int;
}
unsafe extern "C" fn init_startuptime(mut paramp: *mut mparm_T) {
    let mut is_embed: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i < (*paramp).argc - 1 as ::core::ffi::c_int {
        if strcasecmp(
            *(*paramp).argv.offset(i as isize),
            b"--embed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            is_embed = true_0 != 0;
            break;
        } else {
            i += 1;
        }
    }
    let mut i_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i_0 < (*paramp).argc - 1 as ::core::ffi::c_int {
        if strcasecmp(
            *(*paramp).argv.offset(i_0 as isize),
            b"--startuptime\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            time_init(
                *(*paramp)
                    .argv
                    .offset((i_0 + 1 as ::core::ffi::c_int) as isize),
                if is_embed as ::core::ffi::c_int != 0 {
                    b"Embedded\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"Primary (or UI client)\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
            time_start(b"--- NVIM STARTING ---\0".as_ptr() as *const ::core::ffi::c_char);
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
            b"window checked\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
unsafe extern "C" fn init_path(mut exename: *const ::core::ffi::c_char) {
    let mut exepath: [::core::ffi::c_char; 4096] = [
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
    let mut exepathlen: size_t = MAXPATHL as size_t;
    if os_exepath(
        &raw mut exepath as *mut ::core::ffi::c_char,
        &raw mut exepathlen,
    ) != 0 as ::core::ffi::c_int
    {
        path_guess_exepath(
            exename,
            &raw mut exepath as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        );
    }
    set_vim_var_string(
        VV_PROGPATH,
        &raw mut exepath as *mut ::core::ffi::c_char,
        -1 as ptrdiff_t,
    );
    set_vim_var_string(VV_PROGNAME, path_tail(exename), -1 as ptrdiff_t);
}
unsafe extern "C" fn get_fname(mut _parmp: *mut mparm_T) -> *mut ::core::ffi::c_char {
    return alist_name(
        ((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T)
            .offset(0 as ::core::ffi::c_int as isize),
    );
}
unsafe extern "C" fn set_window_layout(mut paramp: *mut mparm_T) {
    if (*paramp).diff_mode != 0 && (*paramp).window_layout == 0 as ::core::ffi::c_int {
        if diffopt_horizontal() {
            (*paramp).window_layout = WIN_HOR as ::core::ffi::c_int;
        } else {
            (*paramp).window_layout = WIN_VER as ::core::ffi::c_int;
        }
    }
}
unsafe extern "C" fn handle_quickfix(mut paramp: *mut mparm_T) {
    if (*paramp).edit_type == EDIT_QF as ::core::ffi::c_int {
        if !(*paramp).use_ef.is_null() {
            set_option_direct(
                kOptErrorfile,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string((*paramp).use_ef),
                    },
                },
                0 as ::core::ffi::c_int,
                SID_CARG,
            );
        }
        vim_snprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            b"cfile %s\0".as_ptr() as *const ::core::ffi::c_char,
            p_ef.get(),
        );
        if qf_init(
            ::core::ptr::null_mut::<win_T>(),
            p_ef.get(),
            p_efm.get(),
            true_0,
            IObuff.ptr() as *mut ::core::ffi::c_char,
            p_menc.get(),
        ) < 0 as ::core::ffi::c_int
        {
            msg_putchar('\n' as ::core::ffi::c_int);
            os_exit(3 as ::core::ffi::c_int);
        }
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"reading errorfile\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
}
unsafe extern "C" fn handle_tag(mut tagname: *mut ::core::ffi::c_char) {
    if !tagname.is_null() {
        swap_exists_did_quit.set(false_0 != 0);
        vim_snprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            b"ta %s\0".as_ptr() as *const ::core::ffi::c_char,
            tagname,
        );
        do_cmdline_cmd(IObuff.ptr() as *mut ::core::ffi::c_char);
        if !(*time_fd.ptr()).is_null() {
            time_msg(
                b"jumping to tag\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
        if swap_exists_did_quit.get() {
            ui_call_error_exit(1 as Integer);
            getout(1 as ::core::ffi::c_int);
        }
    }
}
unsafe extern "C" fn read_stdin() {
    swap_exists_action.set(SEA_DIALOG);
    no_wait_return.set(true_0);
    let mut save_msg_didany: bool = msg_didany.get();
    if !(*curbuf.get()).b_ffname.is_null() {
        let mut stdin_buf: *mut buf_T = buflist_new(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as linenr_T,
            BLN_LISTED as ::core::ffi::c_int,
        );
        if stdin_buf.is_null() {
            semsg(b"Failed to create buffer for stdin\0".as_ptr() as *const ::core::ffi::c_char);
            return;
        }
        let mut initial_buf_handle: handle_T = (*curbuf.get()).handle;
        set_curbuf(stdin_buf, 0 as ::core::ffi::c_int, false_0 != 0);
        readfile(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as linenr_T,
            0 as linenr_T,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            ::core::ptr::null_mut::<exarg_T>(),
            READ_NEW as ::core::ffi::c_int + READ_STDIN as ::core::ffi::c_int,
            true_0 != 0,
        );
        let mut stdin_buf_handle: handle_T = (*stdin_buf).handle;
        let mut stdin_buf_empty: bool = buf_is_empty(curbuf.get());
        let mut buf: [::core::ffi::c_char; 100] = [0; 100];
        vim_snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 100]>(),
            b"silent! buffer %d\0".as_ptr() as *const ::core::ffi::c_char,
            initial_buf_handle,
        );
        do_cmdline_cmd(&raw mut buf as *mut ::core::ffi::c_char);
        if stdin_buf_empty {
            vim_snprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 100]>(),
                b"silent! bwipeout! %d\0".as_ptr() as *const ::core::ffi::c_char,
                stdin_buf_handle,
            );
            do_cmdline_cmd(&raw mut buf as *mut ::core::ffi::c_char);
        }
    } else {
        set_buflisted(true_0);
        open_buffer(
            true_0 != 0,
            ::core::ptr::null_mut::<exarg_T>(),
            0 as ::core::ffi::c_int,
        );
        if buf_is_empty(curbuf.get()) as ::core::ffi::c_int != 0
            && !(*curbuf.get()).b_next.is_null()
        {
            do_cmdline_cmd(b"silent! bnext\0".as_ptr() as *const ::core::ffi::c_char);
            do_cmdline_cmd(b"silent! bwipeout 1\0".as_ptr() as *const ::core::ffi::c_char);
        }
    }
    no_wait_return.set(false_0);
    msg_didany.set(save_msg_didany);
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"reading stdin\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    check_swap_exists_action();
}
unsafe extern "C" fn create_windows(mut parmp: *mut mparm_T) {
    if (*parmp).window_count == -1 as ::core::ffi::c_int {
        (*parmp).window_count = 1 as ::core::ffi::c_int;
    }
    if (*parmp).window_count == 0 as ::core::ffi::c_int {
        (*parmp).window_count = (*global_alist.ptr()).al_ga.ga_len;
    }
    if (*parmp).window_count > 1 as ::core::ffi::c_int {
        if (*parmp).window_layout == 0 as ::core::ffi::c_int {
            (*parmp).window_layout = WIN_HOR as ::core::ffi::c_int;
        }
        if (*parmp).window_layout == WIN_TABS as ::core::ffi::c_int {
            (*parmp).window_count = make_tabpages((*parmp).window_count);
            if !(*time_fd.ptr()).is_null() {
                time_msg(
                    b"making tab pages\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::ptr::null::<proftime_T>(),
                );
            }
        } else if (*firstwin.get()).w_next.is_null()
            || (*(*firstwin.get()).w_next).w_floating as ::core::ffi::c_int != 0
        {
            (*parmp).window_count = make_windows(
                (*parmp).window_count,
                (*parmp).window_layout == WIN_VER as ::core::ffi::c_int,
            );
            if !(*time_fd.ptr()).is_null() {
                time_msg(
                    b"making windows\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::ptr::null::<proftime_T>(),
                );
            }
        } else {
            (*parmp).window_count = win_count();
        }
    } else {
        (*parmp).window_count = 1 as ::core::ffi::c_int;
    }
    if recoverymode.get() {
        msg_scroll.set(true_0);
        ml_recover(true_0 != 0);
        if (*curbuf.get()).b_ml.ml_mfp.is_null() {
            getout(1 as ::core::ffi::c_int);
        }
        do_modelines(0 as ::core::ffi::c_int);
    } else {
        let mut done: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        (*autocmd_no_enter.ptr()) += 1;
        (*autocmd_no_leave.ptr()) += 1;
        let mut dorewind: bool = true_0 != 0;
        loop {
            let c2rust_fresh0 = done;
            done = done + 1;
            if c2rust_fresh0 >= 1000 as ::core::ffi::c_int {
                break;
            }
            if dorewind {
                if (*parmp).window_layout == WIN_TABS as ::core::ffi::c_int {
                    goto_tabpage(1 as ::core::ffi::c_int);
                } else {
                    curwin.set(firstwin.get());
                }
            } else if (*parmp).window_layout == WIN_TABS as ::core::ffi::c_int {
                if (*curtab.get()).tp_next.is_null() {
                    break;
                }
                goto_tabpage(0 as ::core::ffi::c_int);
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
                open_buffer(
                    false_0 != 0,
                    ::core::ptr::null_mut::<exarg_T>(),
                    0 as ::core::ffi::c_int,
                );
                if swap_exists_action.get() == SEA_QUIT {
                    if got_int.get() as ::core::ffi::c_int != 0
                        || only_one_window() as ::core::ffi::c_int != 0
                    {
                        did_emsg.set(false_0);
                        ui_call_error_exit(1 as Integer);
                        getout(1 as ::core::ffi::c_int);
                    }
                    setfname(
                        curbuf.get(),
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        false_0 != 0,
                    );
                    (*curwin.get()).w_arg_idx = -1 as ::core::ffi::c_int;
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
        if (*parmp).window_layout == WIN_TABS as ::core::ffi::c_int {
            goto_tabpage(1 as ::core::ffi::c_int);
        } else {
            curwin.set(firstwin.get());
        }
        curbuf.set((*curwin.get()).w_buffer);
        (*autocmd_no_enter.ptr()) -= 1;
        (*autocmd_no_leave.ptr()) -= 1;
    };
}
unsafe extern "C" fn edit_buffers(mut parmp: *mut mparm_T) {
    let mut arg_idx: ::core::ffi::c_int = 0;
    let mut advance: bool = true_0 != 0;
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut p_shm_save: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*autocmd_no_enter.ptr()) += 1;
    (*autocmd_no_leave.ptr()) += 1;
    if (*curwin.get()).w_arg_idx == -1 as ::core::ffi::c_int {
        win_close(curwin.get(), true_0 != 0, false_0 != 0);
        advance = false_0 != 0;
    }
    arg_idx = 1 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i < (*parmp).window_count {
        if (*curwin.get()).w_arg_idx == -1 as ::core::ffi::c_int {
            arg_idx += 1;
            win_close(curwin.get(), true_0 != 0, false_0 != 0);
            advance = false_0 != 0;
        } else {
            if advance {
                if (*parmp).window_layout == WIN_TABS as ::core::ffi::c_int {
                    if (*curtab.get()).tp_next.is_null() {
                        break;
                    }
                    goto_tabpage(0 as ::core::ffi::c_int);
                    if i == 1 as ::core::ffi::c_int {
                        let mut buf: [::core::ffi::c_char; 100] = [0; 100];
                        p_shm_save = xstrdup(p_shm.get());
                        snprintf(
                            &raw mut buf as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 100]>(),
                            b"F%s\0".as_ptr() as *const ::core::ffi::c_char,
                            p_shm.get(),
                        );
                        set_option_value_give_err(
                            kOptShortmess,
                            OptVal {
                                type_0: kOptValTypeString,
                                data: OptValData {
                                    string: cstr_as_string(
                                        &raw mut buf as *mut ::core::ffi::c_char,
                                    ),
                                },
                            },
                            0 as ::core::ffi::c_int,
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
                    0 as ::core::ffi::c_int,
                    if arg_idx < (*global_alist.ptr()).al_ga.ga_len {
                        alist_name(
                            ((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T)
                                .offset(arg_idx as isize),
                        )
                    } else {
                        ::core::ptr::null_mut::<::core::ffi::c_char>()
                    },
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<exarg_T>(),
                    ECMD_LASTL as ::core::ffi::c_int as linenr_T,
                    ECMD_HIDE as ::core::ffi::c_int,
                    curwin.get(),
                );
                if swap_exists_did_quit.get() {
                    if got_int.get() as ::core::ffi::c_int != 0
                        || only_one_window() as ::core::ffi::c_int != 0
                    {
                        did_emsg.set(false_0);
                        ui_call_error_exit(1 as Integer);
                        getout(1 as ::core::ffi::c_int);
                    }
                    win_close(curwin.get(), true_0 != 0, false_0 != 0);
                    advance = false_0 != 0;
                }
                if arg_idx == (*global_alist.ptr()).al_ga.ga_len - 1 as ::core::ffi::c_int {
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
            0 as ::core::ffi::c_int,
        );
        xfree(p_shm_save as *mut ::core::ffi::c_void);
    }
    if (*parmp).window_layout == WIN_TABS as ::core::ffi::c_int {
        goto_tabpage(1 as ::core::ffi::c_int);
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
            b"editing files in windows\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if (*parmp).window_count > 1 as ::core::ffi::c_int
        && (*parmp).window_layout != WIN_TABS as ::core::ffi::c_int
    {
        win_equal(curwin.get(), false_0 != 0, 'b' as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn exe_pre_commands(mut parmp: *mut mparm_T) {
    let mut cmds: *mut *mut ::core::ffi::c_char =
        &raw mut (*parmp).pre_commands as *mut *mut ::core::ffi::c_char;
    let mut cnt: ::core::ffi::c_int = (*parmp).n_pre_commands;
    if cnt <= 0 as ::core::ffi::c_int {
        return;
    }
    (*curwin.get()).w_cursor.lnum = 0 as ::core::ffi::c_int as linenr_T;
    estack_push(
        ETYPE_ARGS,
        gettext(b"pre-vimrc command line\0".as_ptr() as *const ::core::ffi::c_char),
        0 as linenr_T,
    );
    (*current_sctx.ptr()).sc_sid = SID_CMDARG as scid_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < cnt {
        do_cmdline_cmd(*cmds.offset(i as isize));
        i += 1;
    }
    estack_pop();
    (*current_sctx.ptr()).sc_sid = 0 as ::core::ffi::c_int as scid_T;
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"--cmd commands\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
unsafe extern "C" fn exe_commands(mut parmp: *mut mparm_T) {
    msg_scroll.set(true_0);
    if (*parmp).tagname.is_null() && (*curwin.get()).w_cursor.lnum <= 1 as linenr_T {
        (*curwin.get()).w_cursor.lnum = 0 as ::core::ffi::c_int as linenr_T;
    }
    estack_push(
        ETYPE_ARGS,
        b"command line\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        0 as linenr_T,
    );
    (*current_sctx.ptr()).sc_sid = SID_CARG as scid_T;
    (*current_sctx.ptr()).sc_seq = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*parmp).n_commands {
        do_cmdline_cmd((*parmp).commands[i as usize]);
        if (*parmp).cmds_tofree[i as usize] != 0 {
            xfree((*parmp).commands[i as usize] as *mut ::core::ffi::c_void);
        }
        i += 1;
    }
    estack_pop();
    (*current_sctx.ptr()).sc_sid = 0 as ::core::ffi::c_int as scid_T;
    if (*curwin.get()).w_cursor.lnum == 0 as linenr_T {
        (*curwin.get()).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
    }
    if !exmode_active.get() {
        msg_scroll.set(false_0);
    }
    if (*parmp).edit_type == EDIT_QF as ::core::ffi::c_int {
        qf_jump(
            ::core::ptr::null_mut::<qf_info_T>(),
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            false_0,
        );
    }
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"executing command arguments\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
unsafe extern "C" fn do_system_initialization() {
    let config_dirs: *mut ::core::ffi::c_char = stdpaths_get_xdg_var(kXDGConfigDirs);
    if !config_dirs.is_null() {
        let mut iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
        let mut appname: *const ::core::ffi::c_char = get_appname(false_0 != 0);
        let mut appname_len: size_t = strlen(appname);
        let sysinit_suffix: [::core::ffi::c_char; 13] = [
            PATHSEP as ::core::ffi::c_char,
            's' as ::core::ffi::c_char,
            'y' as ::core::ffi::c_char,
            's' as ::core::ffi::c_char,
            'i' as ::core::ffi::c_char,
            'n' as ::core::ffi::c_char,
            'i' as ::core::ffi::c_char,
            't' as ::core::ffi::c_char,
            '.' as ::core::ffi::c_char,
            'v' as ::core::ffi::c_char,
            'i' as ::core::ffi::c_char,
            'm' as ::core::ffi::c_char,
            NUL as ::core::ffi::c_char,
        ];
        loop {
            let mut dir: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            let mut dir_len: size_t = 0;
            iter = vim_env_iter(
                ':' as ::core::ffi::c_char,
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
                .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 13]>());
            let mut vimrc: *mut ::core::ffi::c_char = xmalloc(path_len) as *mut ::core::ffi::c_char;
            memcpy(
                vimrc as *mut ::core::ffi::c_void,
                dir as *const ::core::ffi::c_void,
                dir_len,
            );
            if *vimrc.offset(dir_len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                != PATHSEP
            {
                *vimrc.offset(dir_len as isize) = PATHSEP as ::core::ffi::c_char;
                dir_len = dir_len.wrapping_add(1 as size_t);
            }
            memcpy(
                vimrc.offset(dir_len as isize) as *mut ::core::ffi::c_void,
                appname as *const ::core::ffi::c_void,
                appname_len,
            );
            memcpy(
                vimrc.offset(dir_len as isize).offset(appname_len as isize)
                    as *mut ::core::ffi::c_void,
                &raw const sysinit_suffix as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<[::core::ffi::c_char; 13]>(),
            );
            if do_source(
                vimrc,
                false_0 != 0,
                DOSO_NONE as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ) != FAIL
            {
                xfree(vimrc as *mut ::core::ffi::c_void);
                xfree(config_dirs as *mut ::core::ffi::c_void);
                return;
            }
            xfree(vimrc as *mut ::core::ffi::c_void);
            if iter.is_null() {
                break;
            }
        }
        xfree(config_dirs as *mut ::core::ffi::c_void);
    }
    do_source(
        SYS_VIMRC_FILE.as_ptr() as *mut ::core::ffi::c_char,
        false_0 != 0,
        DOSO_NONE as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    );
}
unsafe extern "C" fn do_user_initialization() -> bool {
    let mut do_exrc: bool = p_exrc.get() != 0;
    if execute_env(b"VIMINIT\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char)
        == OK
    {
        do_exrc = p_exrc.get() != 0;
        return do_exrc;
    }
    let mut init_lua_path: *mut ::core::ffi::c_char =
        stdpaths_user_conf_subpath(b"init.lua\0".as_ptr() as *const ::core::ffi::c_char);
    let mut user_vimrc: *mut ::core::ffi::c_char =
        stdpaths_user_conf_subpath(b"init.vim\0".as_ptr() as *const ::core::ffi::c_char);
    if os_path_exists(init_lua_path) as ::core::ffi::c_int != 0
        && do_source(
            init_lua_path,
            true_0 != 0,
            DOSO_VIMRC as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ) != 0
    {
        if os_path_exists(user_vimrc) {
            semsg(
                (e_conflicting_configs.ptr() as *const _) as *const ::core::ffi::c_char,
                init_lua_path,
                user_vimrc,
            );
        }
        xfree(user_vimrc as *mut ::core::ffi::c_void);
        xfree(init_lua_path as *mut ::core::ffi::c_void);
        do_exrc = p_exrc.get() != 0;
        return do_exrc;
    }
    xfree(init_lua_path as *mut ::core::ffi::c_void);
    if do_source(
        user_vimrc,
        true_0 != 0,
        DOSO_VIMRC as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    ) != FAIL
    {
        do_exrc = p_exrc.get() != 0;
        if do_exrc {
            do_exrc = path_full_compare(
                VIMRC_FILE.as_ptr() as *mut ::core::ffi::c_char,
                user_vimrc,
                false_0 != 0,
                true_0 != 0,
            ) as ::core::ffi::c_uint
                != kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        xfree(user_vimrc as *mut ::core::ffi::c_void);
        return do_exrc;
    }
    xfree(user_vimrc as *mut ::core::ffi::c_void);
    let config_dirs: *mut ::core::ffi::c_char = stdpaths_get_xdg_var(kXDGConfigDirs);
    if !config_dirs.is_null() {
        let mut appname: *const ::core::ffi::c_char = get_appname(false_0 != 0);
        let mut appname_len: size_t = strlen(appname);
        let mut iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
        loop {
            let mut dir: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            let mut dir_len: size_t = 0;
            iter = vim_env_iter(
                ':' as ::core::ffi::c_char,
                config_dirs,
                iter,
                &raw mut dir,
                &raw mut dir_len,
            );
            if dir.is_null() || dir_len == 0 as size_t {
                break;
            }
            let init_lua_suffix: [::core::ffi::c_char; 10] = [
                PATHSEP as ::core::ffi::c_char,
                'i' as ::core::ffi::c_char,
                'n' as ::core::ffi::c_char,
                'i' as ::core::ffi::c_char,
                't' as ::core::ffi::c_char,
                '.' as ::core::ffi::c_char,
                'l' as ::core::ffi::c_char,
                'u' as ::core::ffi::c_char,
                'a' as ::core::ffi::c_char,
                NUL as ::core::ffi::c_char,
            ];
            let mut init_lua_len: size_t = dir_len
                .wrapping_add(1 as size_t)
                .wrapping_add(appname_len)
                .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 10]>());
            let mut init_lua: *mut ::core::ffi::c_char =
                xmalloc(init_lua_len) as *mut ::core::ffi::c_char;
            memcpy(
                init_lua as *mut ::core::ffi::c_void,
                dir as *const ::core::ffi::c_void,
                dir_len,
            );
            *init_lua.offset(dir_len as isize) = PATHSEP as ::core::ffi::c_char;
            memcpy(
                init_lua
                    .offset(dir_len as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                appname as *const ::core::ffi::c_void,
                appname_len,
            );
            memcpy(
                init_lua
                    .offset(dir_len as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    .offset(appname_len as isize) as *mut ::core::ffi::c_void,
                &raw const init_lua_suffix as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>(),
            );
            let init_vim_suffix: [::core::ffi::c_char; 10] = [
                PATHSEP as ::core::ffi::c_char,
                'i' as ::core::ffi::c_char,
                'n' as ::core::ffi::c_char,
                'i' as ::core::ffi::c_char,
                't' as ::core::ffi::c_char,
                '.' as ::core::ffi::c_char,
                'v' as ::core::ffi::c_char,
                'i' as ::core::ffi::c_char,
                'm' as ::core::ffi::c_char,
                NUL as ::core::ffi::c_char,
            ];
            let mut init_vim_len: size_t = dir_len
                .wrapping_add(1 as size_t)
                .wrapping_add(appname_len)
                .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 10]>());
            let mut init_vim: *mut ::core::ffi::c_char =
                xmalloc(init_vim_len) as *mut ::core::ffi::c_char;
            memcpy(
                init_vim as *mut ::core::ffi::c_void,
                dir as *const ::core::ffi::c_void,
                dir_len,
            );
            *init_vim.offset(dir_len as isize) = PATHSEP as ::core::ffi::c_char;
            memcpy(
                init_vim
                    .offset(dir_len as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                appname as *const ::core::ffi::c_void,
                appname_len,
            );
            memcpy(
                init_vim
                    .offset(dir_len as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    .offset(appname_len as isize) as *mut ::core::ffi::c_void,
                &raw const init_vim_suffix as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>(),
            );
            if os_path_exists(init_lua) as ::core::ffi::c_int != 0
                && do_source(
                    init_lua,
                    true_0 != 0,
                    DOSO_VIMRC as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ) != 0
            {
                if os_path_exists(init_vim) {
                    semsg(
                        (e_conflicting_configs.ptr() as *const _) as *const ::core::ffi::c_char,
                        init_lua,
                        init_vim,
                    );
                }
                xfree(init_vim as *mut ::core::ffi::c_void);
                xfree(init_lua as *mut ::core::ffi::c_void);
                xfree(config_dirs as *mut ::core::ffi::c_void);
                do_exrc = p_exrc.get() != 0;
                return do_exrc;
            }
            xfree(init_lua as *mut ::core::ffi::c_void);
            if do_source(
                init_vim,
                true_0 != 0,
                DOSO_VIMRC as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ) != FAIL
            {
                do_exrc = p_exrc.get() != 0;
                if do_exrc {
                    do_exrc = path_full_compare(
                        VIMRC_FILE.as_ptr() as *mut ::core::ffi::c_char,
                        init_vim,
                        false_0 != 0,
                        true_0 != 0,
                    ) as ::core::ffi::c_uint
                        != kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint;
                }
                xfree(init_vim as *mut ::core::ffi::c_void);
                xfree(config_dirs as *mut ::core::ffi::c_void);
                return do_exrc;
            }
            xfree(init_vim as *mut ::core::ffi::c_void);
            if iter.is_null() {
                break;
            }
        }
        xfree(config_dirs as *mut ::core::ffi::c_void);
    }
    if execute_env(b"EXINIT\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char)
        == OK
    {
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
                b"L\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/main.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2207 as ::core::ffi::c_uint,
                b"void do_exrc_initialization(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    lua_getfield(
        L,
        LUA_GLOBALSINDEX,
        b"require\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushstring(
        L,
        b"vim._core.exrc\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if nlua_pcall(L, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 {
        fprintf(
            stderr,
            b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
            lua_tolstring(
                L,
                -1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<size_t>(),
            ),
        );
    }
}
unsafe extern "C" fn source_startup_scripts(parmp: *const mparm_T) {
    if !(*parmp).use_vimrc.is_null() {
        if !(strequal(
            (*parmp).use_vimrc,
            b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int
            != 0
            || strequal(
                (*parmp).use_vimrc,
                b"NORC\0".as_ptr() as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int
                != 0)
        {
            if do_source(
                (*parmp).use_vimrc,
                false_0 != 0,
                DOSO_NONE as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ) != OK
            {
                semsg(
                    gettext(
                        (e_cannot_read_from_str_2.ptr() as *const _) as *const ::core::ffi::c_char,
                    ),
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
            b"sourcing vimrc file(s)\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
unsafe extern "C" fn execute_env(mut env: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut initstr: *mut ::core::ffi::c_char = os_getenv(env);
    if initstr.is_null() {
        return FAIL;
    }
    estack_push(ETYPE_ENV, env, 0 as linenr_T);
    let save_current_sctx: sctx_T = current_sctx.get();
    (*current_sctx.ptr()).sc_sid = SID_ENV as scid_T;
    (*current_sctx.ptr()).sc_seq = 0 as ::core::ffi::c_int;
    (*current_sctx.ptr()).sc_lnum = 0 as ::core::ffi::c_int as linenr_T;
    do_cmdline_cmd(initstr);
    estack_pop();
    current_sctx.set(save_current_sctx);
    xfree(initstr as *mut ::core::ffi::c_void);
    return OK;
}
unsafe extern "C" fn mainerr(
    mut msg1: *const ::core::ffi::c_char,
    mut msg2: *const ::core::ffi::c_char,
    mut msg3: *const ::core::ffi::c_char,
) -> ! {
    print_mainerr(msg1, msg2, msg3);
    os_exit(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn print_mainerr(
    mut msg1: *const ::core::ffi::c_char,
    mut msg2: *const ::core::ffi::c_char,
    mut msg3: *const ::core::ffi::c_char,
) {
    let mut prgname: *mut ::core::ffi::c_char = path_tail(argv0.get());
    signal_stop();
    fprintf(
        stderr,
        b"%s: %s\0".as_ptr() as *const ::core::ffi::c_char,
        prgname,
        gettext(msg1),
    );
    if !msg2.is_null() {
        fprintf(
            stderr,
            b": \"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
            msg2,
        );
    }
    if !msg3.is_null() {
        fprintf(
            stderr,
            b": \"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
            msg3,
        );
    }
    fprintf(
        stderr,
        gettext(b"\nMore info with \"\0".as_ptr() as *const ::core::ffi::c_char),
    );
    fprintf(
        stderr,
        b"%s -h\"\n\0".as_ptr() as *const ::core::ffi::c_char,
        prgname,
    );
}
unsafe extern "C" fn version() {
    nlua_init(
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
        -1 as ::core::ffi::c_int,
    );
    info_message.set(true_0 != 0);
    list_version();
    msg_putchar('\n' as ::core::ffi::c_int);
    msg_didout.set(false_0 != 0);
}
unsafe extern "C" fn usage() {
    signal_stop();
    printf(gettext(b"Usage:\n\0".as_ptr() as *const ::core::ffi::c_char));
    printf(gettext(
        b"  nvim [options] [file ...]\n\0".as_ptr() as *const ::core::ffi::c_char
    ));
    printf(gettext(
        b"\nOptions:\n\0".as_ptr() as *const ::core::ffi::c_char
    ));
    printf(gettext(
        b"  --cmd <cmd>           Execute <cmd> before any config\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  +<cmd>, -c <cmd>      Execute <cmd> after config and first file\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -l <script> [args...] Execute Lua <script> (with optional args)\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -S <session>          Source <session> after loading the first file\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -s <scriptin>         Read Normal mode commands from <scriptin>\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -u <config>           Use this config file\n\0".as_ptr() as *const ::core::ffi::c_char,
    ));
    printf(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
    printf(gettext(
        b"  -d                    Diff mode\n\0".as_ptr() as *const ::core::ffi::c_char
    ));
    printf(gettext(
        b"  -es, -Es              Silent (batch) mode\n\0".as_ptr() as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -h, --help            Print this help message\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -i <shada>            Use this shada file\n\0".as_ptr() as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -n                    No swap file, use memory only\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -o[N]                 Open N windows (default: one per file)\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -O[N]                 Open N vertical windows (default: one per file)\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -p[N]                 Open N tab pages (default: one per file)\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -R                    Read-only (view) mode\n\0".as_ptr() as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -v, --version         Print version information\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -V[N][file]           Verbose [level][file]\n\0".as_ptr() as *const ::core::ffi::c_char,
    ));
    printf(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
    printf(gettext(
        b"  --                    Only file names after this\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  --api-info            Write msgpack-encoded API metadata to stdout\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  --clean               \"Factory defaults\" (skip user config and plugins, shada)\n\0"
            .as_ptr() as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  --embed               Use stdin/stdout as a msgpack-rpc channel\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  --headless            Don't start a user interface\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  --listen <address>    Serve RPC API from this address\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  --remote[-subcommand] Execute commands remotely on a server\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  --server <address>    Connect to this Nvim server\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  --startuptime <file>  Write startup timing messages to <file>\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"\nSee \":help startup-options\" for all options.\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
}
unsafe extern "C" fn check_swap_exists_action() {
    if swap_exists_action.get() == SEA_QUIT {
        ui_call_error_exit(1 as Integer);
        getout(1 as ::core::ffi::c_int);
    }
    handle_swap_exists(::core::ptr::null_mut::<bufref_T>());
}
pub static tslua_query_parse_count: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub const MAX_ARG_CMDS: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub static namedfm: GlobalCell<[xfmark_T; 36]> = GlobalCell::new([
    xfmark_T {
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
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
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
]);
pub static ch_before_blocking_events: GlobalCell<*mut MultiQueue> =
    GlobalCell::new(::core::ptr::null_mut::<MultiQueue>());
pub static showcmd_buf: GlobalCell<[::core::ffi::c_char; 41]> = GlobalCell::new([0; 41]);
pub static repeat_luaref: GlobalCell<LuaRef> = GlobalCell::new(-2 as LuaRef);
pub static used_stdin: GlobalCell<bool> = GlobalCell::new(false);
pub static nvim_testing: GlobalCell<bool> = GlobalCell::new(false);
pub static pum_grid: GlobalCell<ScreenGrid> = GlobalCell::new(ScreenGrid {
    handle: 0 as handle_T,
    chars: ::core::ptr::null_mut::<schar_T>(),
    attrs: ::core::ptr::null_mut::<sattr_T>(),
    vcols: ::core::ptr::null_mut::<colnr_T>(),
    line_offset: ::core::ptr::null_mut::<size_t>(),
    dirty_col: ::core::ptr::null_mut::<::core::ffi::c_int>(),
    rows: 0 as ::core::ffi::c_int,
    cols: 0 as ::core::ffi::c_int,
    valid: false,
    throttled: false,
    blending: false,
    mouse_enabled: true,
    zindex: 0 as ::core::ffi::c_int,
    comp_row: 0 as ::core::ffi::c_int,
    comp_col: 0 as ::core::ffi::c_int,
    comp_width: 0 as ::core::ffi::c_int,
    comp_height: 0 as ::core::ffi::c_int,
    comp_index: 0 as size_t,
    comp_disabled: false,
    pending_comp_index_update: true,
});
#[no_mangle]
pub static pum_want: GlobalCell<C2Rust_Unnamed_46> = GlobalCell::new(C2Rust_Unnamed_46 {
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
pub static ui_client_error_exit: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(-1 as ::core::ffi::c_int);
pub static ui_client_exit_status: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static ui_client_attached: GlobalCell<bool> = GlobalCell::new(false);
pub static ui_client_forward_stdin: GlobalCell<bool> = GlobalCell::new(false);
pub static tabpage_move_disallowed: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub static float_anchor_str: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"NW\0".as_ptr() as *const ::core::ffi::c_char,
    b"NE\0".as_ptr() as *const ::core::ffi::c_char,
    b"SW\0".as_ptr() as *const ::core::ffi::c_char,
    b"SE\0".as_ptr() as *const ::core::ffi::c_char,
]);
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const WRITEBIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"wb\0") };
pub const APPENDBIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ab\0") };
pub fn main() {
    let mut args_strings: Vec<Vec<u8>> = ::std::env::args()
        .map(|arg| {
            ::std::ffi::CString::new(arg)
                .expect("Failed to convert argument into CString.")
                .into_bytes_with_nul()
        })
        .collect();
    let mut args_ptrs: Vec<*mut ::core::ffi::c_char> = args_strings
        .iter_mut()
        .map(|arg| arg.as_mut_ptr() as *mut ::core::ffi::c_char)
        .chain(::core::iter::once(::core::ptr::null_mut()))
        .collect();
    unsafe {
        ::std::process::exit(main_0(
            (args_ptrs.len() - 1) as ::core::ffi::c_int,
            args_ptrs.as_mut_ptr() as *mut *mut ::core::ffi::c_char,
        ) as i32)
    }
}
unsafe extern "C" fn c2rust_run_static_initializers() {
    kTVCstring.set((18446744073709551615 as size_t).wrapping_sub(1 as size_t));
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [c2rust_run_static_initializers];
