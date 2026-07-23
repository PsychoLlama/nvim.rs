use crate::src::nvim::buffer::read_buffer_into;
use crate::src::nvim::charset::{backslash_halve, skipwhite};
use crate::src::nvim::eval::vars::set_vim_var_nr;
use crate::src::nvim::event::libuv::{uv_err_name, uv_strerror};
use crate::src::nvim::event::libuv_proc::libuv_proc_init;
use crate::src::nvim::event::multiqueue::{
    multiqueue_empty, multiqueue_free, multiqueue_new_child, multiqueue_put_event,
};
use crate::src::nvim::event::proc::{proc_spawn, proc_stop, proc_wait};
use crate::src::nvim::event::r#loop::loop_poll_events;
use crate::src::nvim::event::rstream::{rstream_init, rstream_start};
use crate::src::nvim::event::stream::stream_may_close;
use crate::src::nvim::event::wstream::{
    wstream_init, wstream_new_buffer, wstream_set_write_cb, wstream_write,
};
use crate::src::nvim::ex_cmds::{check_secure, make_filter_cmd};
use crate::src::nvim::fileio::vim_tempname;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    cmdline_row, curbuf, curwin, do_profiling, e_cannot_read_from_str_2, e_cant_read_file_str,
    e_notmp, e_shellempty, e_wildexpand, emsg_silent, got_int, lines_left, main_loop, msg_no_more,
    no_check_timestamps, no_wait_return, p_sh, p_shcf, p_sxe, p_sxq, p_verbose, sandbox, secure,
    Rows, State,
};
use crate::src::nvim::mbyte::{utf8len_tab_zero, utfc_ptr2len_len};
use crate::src::nvim::memline::ml_append;
use crate::src::nvim::memory::{
    xcalloc, xfree, xmalloc, xmemdupz, xrealloc, xstrdup, xstrlcat, xstrlcpy,
};
use crate::src::nvim::message::{
    emsg, msg, msg_end, msg_ext_set_append, msg_ext_set_kind, msg_multiline, msg_outnum,
    msg_outtrans, msg_putchar, msg_puts, msg_sb_eol, msg_schedule_semsg, msg_start, semsg, smsg,
    verbose_enter, verbose_leave,
};
use crate::src::nvim::os::fs::{os_can_exe, os_fopen, os_isdir, os_path_exists, os_remove};
use crate::src::nvim::os::libc::{
    __assert_fail, fclose, fopen, fread, fseek, ftell, gettext, memcpy, memmove, strcat, strcmp,
    strcpy, strlen, strncmp, strstr,
};
use crate::src::nvim::os::signal::{signal_accept_deadly, signal_reject_deadly};
use crate::src::nvim::os::time::{os_delay, os_hrtime};
use crate::src::nvim::path::{add_pathsep, invocation_path_tail, path_has_wildcard, path_tail};
use crate::src::nvim::profile::{prof_child_enter, prof_child_exit};
use crate::src::nvim::strings::{
    vim_snprintf, vim_strchr, vim_strnsave_unquoted, vim_strsave_escaped_ext,
};
use crate::src::nvim::tag::tag_freematch;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_18, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_15, Event,
    ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative, GridView, Intersection, LibuvProc, Loop,
    LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MultiQueue, OptInt, Proc, ProcType,
    RStream, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_25, Stream,
    StringBuilder, String_0, Terminal, Timestamp, UIExtension, VarLockStatus, VarType, VimVarIndex,
    VirtLines, VirtText, VirtTextChunk, VirtTextPos, WBuffer, WinConfig, WinInfo, WinSplit,
    WinStyle, Window, _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, __gid_t, __off64_t,
    __off_t, __pthread_internal_list, __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t,
    __time_t, __uid_t, alist_T, argv_callback, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T,
    bufstate_T, chunksize_T, colnr_T, dict_T, dictvar_S, disptick_T, extmark_undo_vec_t,
    fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_16,
    file_buffer_b_wininfo as C2Rust_Unnamed_24, file_buffer_update_callbacks as C2Rust_Unnamed_13,
    file_buffer_update_channels as C2Rust_Unnamed_14, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_19, funccall_T, garray_T, gid_t, handle_T,
    hash_T, hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, internal_proc_cb,
    intptr_t, lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S,
    listwatch_T, llpos_T, loop_0, loop_0_children as C2Rust_Unnamed_26, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s,
    mtnode_s, multiqueue, partial_S, partial_T, pos_T, pos_save_T, proc, proc_exit_cb,
    proc_state_cb, proftime_T, pthread_mutex_t, pthread_rwlock_t, ptr_t, qf_info_S, qf_info_T,
    queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, rstream, sattr_T, schar_T, scid_T,
    sctx_T, size_t, ssize_t, stream, stream_close_cb, stream_read_cb,
    stream_uv as C2Rust_Unnamed_27, stream_write_cb, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_17, syn_time_T, synblock_T, synstate_T, taggy_T,
    terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_21, u_header_uh_alt_prev as C2Rust_Unnamed_20,
    u_header_uh_next as C2Rust_Unnamed_23, u_header_uh_prev as C2Rust_Unnamed_22, ufunc_S, ufunc_T,
    uid_t, uint16_t, uint32_t, uint64_t, uint8_t, undo_object, uv__io_cb, uv__io_s, uv__io_t,
    uv__queue, uv_alloc_cb, uv_async_cb, uv_async_s, uv_async_s_u as C2Rust_Unnamed_3, uv_async_t,
    uv_buf_t, uv_close_cb, uv_connect_cb, uv_connect_s, uv_connect_t, uv_connection_cb, uv_exit_cb,
    uv_file, uv_gid_t, uv_handle_s, uv_handle_s_u as C2Rust_Unnamed_0, uv_handle_t, uv_handle_type,
    uv_idle_cb, uv_idle_s, uv_idle_s_u as C2Rust_Unnamed_10, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_4, uv_loop_s_timer_heap as C2Rust_Unnamed_2, uv_loop_t,
    uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_7, uv_pipe_t, uv_process_options_s,
    uv_process_options_t, uv_process_s, uv_process_s_u as C2Rust_Unnamed_11, uv_process_t,
    uv_read_cb, uv_req_type, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t,
    uv_signal_cb, uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed,
    uv_signal_s_u as C2Rust_Unnamed_1, uv_signal_t, uv_stdio_container_s,
    uv_stdio_container_s_data as C2Rust_Unnamed_12, uv_stdio_container_t, uv_stdio_flags,
    uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_5, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_6, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_8, uv_timer_s_u as C2Rust_Unnamed_9, uv_timer_t, uv_uid_t,
    varnumber_T, virt_line, visualinfo_T, wbuffer, wbuffer_data_finalizer, win_T, window_S,
    wininfo_S, winopt_T, wline_T, xfmark_T, FILE, QUEUE, _IO_FILE,
};
use crate::src::nvim::ui::{ui_busy_start, ui_busy_stop, ui_flush, ui_has};
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
pub const UV_OVERLAPPED_PIPE: uv_stdio_flags = 64;
pub const UV_NONBLOCK_PIPE: uv_stdio_flags = 64;
pub const UV_WRITABLE_PIPE: uv_stdio_flags = 32;
pub const UV_READABLE_PIPE: uv_stdio_flags = 16;
pub const UV_INHERIT_STREAM: uv_stdio_flags = 4;
pub const UV_INHERIT_FD: uv_stdio_flags = 2;
pub const UV_CREATE_PIPE: uv_stdio_flags = 1;
pub const UV_IGNORE: uv_stdio_flags = 0;
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
pub const kStlClickFuncRun: C2Rust_Unnamed_25 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_25 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_25 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_25 = 0;
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
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_28 = 76;
pub const HLF_PRE: C2Rust_Unnamed_28 = 75;
pub const HLF_OK: C2Rust_Unnamed_28 = 74;
pub const HLF_SO: C2Rust_Unnamed_28 = 73;
pub const HLF_SE: C2Rust_Unnamed_28 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_28 = 71;
pub const HLF_TS: C2Rust_Unnamed_28 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_28 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_28 = 68;
pub const HLF_CU: C2Rust_Unnamed_28 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_28 = 66;
pub const HLF_WBR: C2Rust_Unnamed_28 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_28 = 64;
pub const HLF_MSG: C2Rust_Unnamed_28 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_28 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_28 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_28 = 60;
pub const HLF_0: C2Rust_Unnamed_28 = 59;
pub const HLF_QFL: C2Rust_Unnamed_28 = 58;
pub const HLF_MC: C2Rust_Unnamed_28 = 57;
pub const HLF_CUL: C2Rust_Unnamed_28 = 56;
pub const HLF_CUC: C2Rust_Unnamed_28 = 55;
pub const HLF_TPF: C2Rust_Unnamed_28 = 54;
pub const HLF_TPS: C2Rust_Unnamed_28 = 53;
pub const HLF_TP: C2Rust_Unnamed_28 = 52;
pub const HLF_PBR: C2Rust_Unnamed_28 = 51;
pub const HLF_PST: C2Rust_Unnamed_28 = 50;
pub const HLF_PSB: C2Rust_Unnamed_28 = 49;
pub const HLF_PSX: C2Rust_Unnamed_28 = 48;
pub const HLF_PNX: C2Rust_Unnamed_28 = 47;
pub const HLF_PSK: C2Rust_Unnamed_28 = 46;
pub const HLF_PNK: C2Rust_Unnamed_28 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_28 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_28 = 43;
pub const HLF_PSI: C2Rust_Unnamed_28 = 42;
pub const HLF_PNI: C2Rust_Unnamed_28 = 41;
pub const HLF_SPL: C2Rust_Unnamed_28 = 40;
pub const HLF_SPR: C2Rust_Unnamed_28 = 39;
pub const HLF_SPC: C2Rust_Unnamed_28 = 38;
pub const HLF_SPB: C2Rust_Unnamed_28 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_28 = 36;
pub const HLF_SC: C2Rust_Unnamed_28 = 35;
pub const HLF_TXA: C2Rust_Unnamed_28 = 34;
pub const HLF_TXD: C2Rust_Unnamed_28 = 33;
pub const HLF_DED: C2Rust_Unnamed_28 = 32;
pub const HLF_CHD: C2Rust_Unnamed_28 = 31;
pub const HLF_ADD: C2Rust_Unnamed_28 = 30;
pub const HLF_FC: C2Rust_Unnamed_28 = 29;
pub const HLF_FL: C2Rust_Unnamed_28 = 28;
pub const HLF_WM: C2Rust_Unnamed_28 = 27;
pub const HLF_W: C2Rust_Unnamed_28 = 26;
pub const HLF_VNC: C2Rust_Unnamed_28 = 25;
pub const HLF_V: C2Rust_Unnamed_28 = 24;
pub const HLF_T: C2Rust_Unnamed_28 = 23;
pub const HLF_VSP: C2Rust_Unnamed_28 = 22;
pub const HLF_C: C2Rust_Unnamed_28 = 21;
pub const HLF_SNC: C2Rust_Unnamed_28 = 20;
pub const HLF_S: C2Rust_Unnamed_28 = 19;
pub const HLF_R: C2Rust_Unnamed_28 = 18;
pub const HLF_CLF: C2Rust_Unnamed_28 = 17;
pub const HLF_CLS: C2Rust_Unnamed_28 = 16;
pub const HLF_CLN: C2Rust_Unnamed_28 = 15;
pub const HLF_LNB: C2Rust_Unnamed_28 = 14;
pub const HLF_LNA: C2Rust_Unnamed_28 = 13;
pub const HLF_N: C2Rust_Unnamed_28 = 12;
pub const HLF_CM: C2Rust_Unnamed_28 = 11;
pub const HLF_M: C2Rust_Unnamed_28 = 10;
pub const HLF_LC: C2Rust_Unnamed_28 = 9;
pub const HLF_L: C2Rust_Unnamed_28 = 8;
pub const HLF_I: C2Rust_Unnamed_28 = 7;
pub const HLF_E: C2Rust_Unnamed_28 = 6;
pub const HLF_D: C2Rust_Unnamed_28 = 5;
pub const HLF_AT: C2Rust_Unnamed_28 = 4;
pub const HLF_TERM: C2Rust_Unnamed_28 = 3;
pub const HLF_EOB: C2Rust_Unnamed_28 = 2;
pub const HLF_8: C2Rust_Unnamed_28 = 1;
pub const HLF_NONE: C2Rust_Unnamed_28 = 0;
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
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_29 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_29 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_29 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_29 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_29 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_29 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_29 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_29 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_29 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_29 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_29 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_29 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_29 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_29 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_29 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_29 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_29 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_29 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_29 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_29 = 1;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const kShellOptHideMess: C2Rust_Unnamed_30 = 64;
pub const kShellOptWrite: C2Rust_Unnamed_30 = 32;
pub const kShellOptRead: C2Rust_Unnamed_30 = 16;
pub const kShellOptSilent: C2Rust_Unnamed_30 = 8;
pub const kShellOptDoOut: C2Rust_Unnamed_30 = 4;
pub const kShellOptExpand: C2Rust_Unnamed_30 = 2;
pub const kShellOptFilter: C2Rust_Unnamed_30 = 1;
pub const EW_NOTFOUND: C2Rust_Unnamed_31 = 4;
pub const EW_SHELLCMD: C2Rust_Unnamed_31 = 8192;
pub const EW_EXEC: C2Rust_Unnamed_31 = 64;
pub const EW_FILE: C2Rust_Unnamed_31 = 2;
pub const EW_DIR: C2Rust_Unnamed_31 = 1;
pub const EW_SILENT: C2Rust_Unnamed_31 = 32;
pub const kUIExtCount: UIExtension = 10;
pub const kUIFloatDebug: UIExtension = 9;
pub const kUITermColors: UIExtension = 8;
pub const kUIHlState: UIExtension = 7;
pub const kUIMultigrid: UIExtension = 6;
pub const kUILinegrid: UIExtension = 5;
pub const kUIMessages: UIExtension = 4;
pub const kUIWildmenu: UIExtension = 3;
pub const kUITabline: UIExtension = 2;
pub const kUIPopupmenu: UIExtension = 1;
pub const kUICmdline: UIExtension = 0;
pub const EW_KEEPDOLLAR: C2Rust_Unnamed_31 = 2048;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const EW_NOBREAK: C2Rust_Unnamed_31 = 262144;
pub const EW_CDPATH: C2Rust_Unnamed_31 = 131072;
pub const EW_NOTENV: C2Rust_Unnamed_31 = 65536;
pub const EW_EMPTYOK: C2Rust_Unnamed_31 = 32768;
pub const EW_DODOT: C2Rust_Unnamed_31 = 16384;
pub const EW_ALLLINKS: C2Rust_Unnamed_31 = 4096;
pub const EW_NOTWILD: C2Rust_Unnamed_31 = 1024;
pub const EW_NOERROR: C2Rust_Unnamed_31 = 512;
pub const EW_ICASE: C2Rust_Unnamed_31 = 256;
pub const EW_PATH: C2Rust_Unnamed_31 = 128;
pub const EW_KEEPALL: C2Rust_Unnamed_31 = 16;
pub const EW_ADDSLASH: C2Rust_Unnamed_31 = 8;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const KV_INITIAL_VALUE: StringBuilder = StringBuilder {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PROF_YES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NS_1_SECOND: ::core::ffi::c_uint = 1000000000 as ::core::ffi::c_uint;
pub const OUT_DATA_THRESHOLD: ::core::ffi::c_uint =
    (1024 as ::core::ffi::c_uint).wrapping_mul(10 as ::core::ffi::c_uint);
pub const SHELL_SPECIAL: [::core::ffi::c_char; 15] = unsafe {
    ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\t \"&'$;<>()\\|\n\0")
};
unsafe extern "C" fn save_patterns(
    mut num_pat: ::core::ffi::c_int,
    mut pat: *mut *mut ::core::ffi::c_char,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
) {
    *file = xmalloc(
        (num_pat as size_t).wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
    ) as *mut *mut ::core::ffi::c_char;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_pat {
        let mut s: *mut ::core::ffi::c_char = xstrdup(*pat.offset(i as isize));
        backslash_halve(s);
        *(*file).offset(i as isize) = s;
        i += 1;
    }
    *num_file = num_pat;
}
unsafe extern "C" fn have_wildcard(
    mut num: ::core::ffi::c_int,
    mut file: *mut *mut ::core::ffi::c_char,
) -> bool {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num {
        if path_has_wildcard(*file.offset(i as isize)) {
            return true_0 != 0;
        }
        i += 1;
    }
    return false_0 != 0;
}
unsafe extern "C" fn have_dollars(
    mut num: ::core::ffi::c_int,
    mut file: *mut *mut ::core::ffi::c_char,
) -> bool {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num {
        if !vim_strchr(*file.offset(i as isize), '$' as ::core::ffi::c_int).is_null() {
            return true_0 != 0;
        }
        i += 1;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn os_expand_wildcards(
    mut num_pat: ::core::ffi::c_int,
    mut pat: *mut *mut ::core::ffi::c_char,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut fseek_res: ::core::ffi::c_int = 0;
    let mut templen: int64_t = 0;
    let mut buffer: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut readlen: size_t = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut len: size_t = 0;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut extra_shell_arg: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut shellopts: ::core::ffi::c_int =
        kShellOptExpand as ::core::ffi::c_int | kShellOptSilent as ::core::ffi::c_int;
    let mut j: ::core::ffi::c_int = 0;
    let mut tempname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut shell_style: ::core::ffi::c_int = STYLE_ECHO;
    let mut check_spaces: ::core::ffi::c_int = 0;
    static did_find_nul: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut ampersand: bool = false_0 != 0;
    static sh_vimglob_func: GlobalCell<*mut ::core::ffi::c_char> = GlobalCell::new(
        b"vimglob() { while [ $# -ge 1 ]; do echo \"$1\"; shift; done }; vimglob >\0".as_ptr()
            as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
    static sh_globstar_opt: GlobalCell<*mut ::core::ffi::c_char> = GlobalCell::new(
        b"[[ ${BASH_VERSINFO[0]} -ge 4 ]] && shopt -s globstar; \0".as_ptr()
            as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
    let mut is_fish_shell: bool = strncmp(
        invocation_path_tail(p_sh.get(), ::core::ptr::null_mut::<size_t>()),
        b"fish\0".as_ptr() as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int;
    *num_file = 0 as ::core::ffi::c_int;
    *file = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    if !have_wildcard(num_pat, pat) {
        save_patterns(num_pat, pat, num_file, file);
        return OK;
    }
    if sandbox.get() != 0 as ::core::ffi::c_int && check_secure() as ::core::ffi::c_int != 0 {
        return FAIL;
    }
    if secure.get() != 0 {
        i = 0 as ::core::ffi::c_int;
        while i < num_pat {
            if !vim_strchr(*pat.offset(i as isize), '`' as ::core::ffi::c_int).is_null()
                && check_secure() as ::core::ffi::c_int != 0
            {
                return FAIL;
            }
            i += 1;
        }
    }
    tempname = vim_tempname();
    if tempname.is_null() {
        emsg(gettext(&raw const e_notmp as *const ::core::ffi::c_char));
        return FAIL;
    }
    if num_pat == 1 as ::core::ffi::c_int
        && **pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '`' as ::core::ffi::c_int
        && {
            len = strlen(*pat.offset(0 as ::core::ffi::c_int as isize));
            len > 2 as size_t
        }
        && *(*pat.offset(0 as ::core::ffi::c_int as isize))
            .offset(len as isize)
            .offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
            == '`' as ::core::ffi::c_int
    {
        shell_style = STYLE_BT;
    } else {
        len = strlen(p_sh.get());
        if len >= 3 as size_t {
            if strcmp(
                (*p_sh.ptr())
                    .offset(len as isize)
                    .offset(-(3 as ::core::ffi::c_int as isize)),
                b"csh\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                shell_style = STYLE_GLOB;
            } else if strcmp(
                (*p_sh.ptr())
                    .offset(len as isize)
                    .offset(-(3 as ::core::ffi::c_int as isize)),
                b"zsh\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                shell_style = STYLE_PRINT;
            }
        }
    }
    if shell_style == STYLE_ECHO {
        if !strstr(
            path_tail(p_sh.get()),
            b"bash\0".as_ptr() as *const ::core::ffi::c_char,
        )
        .is_null()
        {
            shell_style = STYLE_GLOBSTAR;
        } else if !strstr(
            path_tail(p_sh.get()),
            b"sh\0".as_ptr() as *const ::core::ffi::c_char,
        )
        .is_null()
        {
            shell_style = STYLE_VIMGLOB;
        }
    }
    len = strlen(tempname).wrapping_add(29 as size_t);
    if shell_style == STYLE_VIMGLOB {
        len = len.wrapping_add(strlen(sh_vimglob_func.get()));
    } else if shell_style == STYLE_GLOBSTAR {
        len = len.wrapping_add(
            strlen(sh_vimglob_func.get()).wrapping_add(strlen(sh_globstar_opt.get())),
        );
    }
    i = 0 as ::core::ffi::c_int;
    while i < num_pat {
        len = len.wrapping_add(1);
        j = 0 as ::core::ffi::c_int;
        while *(*pat.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int != NUL {
            if !vim_strchr(
                SHELL_SPECIAL.as_ptr(),
                *(*pat.offset(i as isize)).offset(j as isize) as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
            {
                len = len.wrapping_add(1);
            }
            len = len.wrapping_add(1);
            j += 1;
        }
        i += 1;
    }
    if is_fish_shell {
        len = (len as ::core::ffi::c_ulong).wrapping_add(
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize)
                as ::core::ffi::c_ulong,
        ) as size_t;
    }
    let mut command: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    if shell_style == STYLE_BT {
        if is_fish_shell {
            strcpy(
                command,
                b"begin; \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
        } else {
            strcpy(
                command,
                b"(\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
        }
        strcat(
            command,
            (*pat.offset(0 as ::core::ffi::c_int as isize))
                .offset(1 as ::core::ffi::c_int as isize),
        );
        p = command
            .offset(strlen(command) as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
        if is_fish_shell {
            let c2rust_fresh0 = p;
            p = p.offset(-1);
            *c2rust_fresh0 = ';' as ::core::ffi::c_char;
            strcat(command, b" end\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            let c2rust_fresh1 = p;
            p = p.offset(-1);
            *c2rust_fresh1 = ')' as ::core::ffi::c_char;
        }
        while p > command && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
            p = p.offset(-1);
        }
        if *p as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
            ampersand = true_0 != 0;
            *p = ' ' as ::core::ffi::c_char;
        }
        strcat(command, b">\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        strcpy(
            command,
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if shell_style == STYLE_GLOB {
            if flags & EW_NOTFOUND as ::core::ffi::c_int != 0 {
                strcat(
                    command,
                    b"set nonomatch; \0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                strcat(
                    command,
                    b"unset nonomatch; \0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        }
        if shell_style == STYLE_GLOB {
            strcat(command, b"glob >\0".as_ptr() as *const ::core::ffi::c_char);
        } else if shell_style == STYLE_PRINT {
            strcat(
                command,
                b"print -N >\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else if shell_style == STYLE_VIMGLOB {
            strcat(command, sh_vimglob_func.get());
        } else if shell_style == STYLE_GLOBSTAR {
            strcat(command, sh_globstar_opt.get());
            strcat(command, sh_vimglob_func.get());
        } else {
            strcat(command, b"echo >\0".as_ptr() as *const ::core::ffi::c_char);
        }
    }
    strcat(command, tempname);
    if shell_style != STYLE_BT {
        i = 0 as ::core::ffi::c_int;
        while i < num_pat {
            let mut intick: bool = false_0 != 0;
            p = command.offset(strlen(command) as isize);
            let c2rust_fresh2 = p;
            p = p.offset(1);
            *c2rust_fresh2 = ' ' as ::core::ffi::c_char;
            j = 0 as ::core::ffi::c_int;
            while *(*pat.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int != NUL {
                if *(*pat.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int
                    == '`' as ::core::ffi::c_int
                {
                    intick = !intick;
                } else if *(*pat.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *(*pat.offset(i as isize)).offset((j + 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        != NUL
                {
                    if intick as ::core::ffi::c_int != 0
                        || !vim_strchr(
                            SHELL_SPECIAL.as_ptr(),
                            *(*pat.offset(i as isize))
                                .offset((j + 1 as ::core::ffi::c_int) as isize)
                                as uint8_t as ::core::ffi::c_int,
                        )
                        .is_null()
                        || *(*pat.offset(i as isize)).offset((j + 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            == '`' as ::core::ffi::c_int
                    {
                        let c2rust_fresh3 = p;
                        p = p.offset(1);
                        *c2rust_fresh3 = '\\' as ::core::ffi::c_char;
                    }
                    j += 1;
                } else if !intick
                    && (flags & EW_KEEPDOLLAR as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                        || *(*pat.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int
                            != '$' as ::core::ffi::c_int)
                    && !vim_strchr(
                        SHELL_SPECIAL.as_ptr(),
                        *(*pat.offset(i as isize)).offset(j as isize) as uint8_t
                            as ::core::ffi::c_int,
                    )
                    .is_null()
                {
                    let c2rust_fresh4 = p;
                    p = p.offset(1);
                    *c2rust_fresh4 = '\\' as ::core::ffi::c_char;
                }
                let c2rust_fresh5 = p;
                p = p.offset(1);
                *c2rust_fresh5 = *(*pat.offset(i as isize)).offset(j as isize);
                j += 1;
            }
            *p = NUL as ::core::ffi::c_char;
            i += 1;
        }
    }
    if flags & EW_SILENT as ::core::ffi::c_int != 0 {
        shellopts |= kShellOptHideMess as ::core::ffi::c_int;
    }
    if ampersand {
        strcat(command, b"&\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if shell_style == STYLE_PRINT {
        extra_shell_arg =
            b"-G\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else if shell_style == STYLE_GLOB && !have_dollars(num_pat, pat) {
        extra_shell_arg =
            b"-f\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    i = call_shell(command, shellopts, extra_shell_arg);
    if ampersand {
        os_delay(10 as uint64_t, true_0 != 0);
    }
    xfree(command as *mut ::core::ffi::c_void);
    if i != 0 {
        os_remove(tempname);
        xfree(tempname as *mut ::core::ffi::c_void);
        if flags & EW_SILENT as ::core::ffi::c_int == 0 {
            msg_putchar('\n' as ::core::ffi::c_int);
            cmdline_row.set(Rows.get() - 1 as ::core::ffi::c_int);
            msg(
                gettext(&raw const e_wildexpand as *const ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
            );
            msg_start();
        }
        if shell_style == STYLE_BT {
            return FAIL;
        }
    } else {
        fd = fopen(tempname, READBIN.as_ptr()) as *mut FILE;
        if fd.is_null() {
            if flags & EW_SILENT as ::core::ffi::c_int == 0 {
                msg(
                    gettext(&raw const e_wildexpand as *const ::core::ffi::c_char),
                    0 as ::core::ffi::c_int,
                );
                msg_start();
            }
            xfree(tempname as *mut ::core::ffi::c_void);
        } else {
            fseek_res = fseek(fd, 0 as ::core::ffi::c_long, SEEK_END);
            if fseek_res < 0 as ::core::ffi::c_int {
                xfree(tempname as *mut ::core::ffi::c_void);
                fclose(fd);
                return FAIL;
            }
            templen = ftell(fd) as int64_t;
            if templen < 0 as int64_t {
                xfree(tempname as *mut ::core::ffi::c_void);
                fclose(fd);
                return FAIL;
            }
            len = templen as size_t;
            fseek(fd, 0 as ::core::ffi::c_long, SEEK_SET);
            buffer = xmalloc(len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
            readlen = fread(buffer as *mut ::core::ffi::c_void, 1 as size_t, len, fd) as size_t;
            fclose(fd);
            os_remove(tempname);
            if readlen != len {
                semsg(
                    gettext(&raw const e_cant_read_file_str as *const ::core::ffi::c_char),
                    tempname,
                );
                xfree(tempname as *mut ::core::ffi::c_void);
                xfree(buffer as *mut ::core::ffi::c_void);
                return FAIL;
            }
            xfree(tempname as *mut ::core::ffi::c_void);
            if shell_style == STYLE_ECHO {
                *buffer.offset(len as isize) = '\n' as ::core::ffi::c_char;
                p = buffer;
                i = 0 as ::core::ffi::c_int;
                while *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int {
                    while *p as ::core::ffi::c_int != ' ' as ::core::ffi::c_int
                        && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                    {
                        p = p.offset(1);
                    }
                    p = skipwhite(p);
                    i += 1;
                }
            } else if shell_style == STYLE_BT
                || shell_style == STYLE_VIMGLOB
                || shell_style == STYLE_GLOBSTAR
            {
                *buffer.offset(len as isize) = NUL as ::core::ffi::c_char;
                p = buffer;
                i = 0 as ::core::ffi::c_int;
                while *p as ::core::ffi::c_int != NUL {
                    while *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                        && *p as ::core::ffi::c_int != NUL
                    {
                        p = p.offset(1);
                    }
                    if *p as ::core::ffi::c_int != NUL {
                        p = p.offset(1);
                    }
                    p = skipwhite(p);
                    i += 1;
                }
            } else {
                check_spaces = false_0;
                if shell_style == STYLE_PRINT && !did_find_nul.get() {
                    *buffer.offset(len as isize) = NUL as ::core::ffi::c_char;
                    if len != 0
                        && (strlen(buffer) as ::core::ffi::c_int) < len as ::core::ffi::c_int
                    {
                        did_find_nul.set(true_0 != 0);
                    } else {
                        check_spaces = true_0;
                    }
                }
                if len != 0
                    && *buffer.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                        == NUL
                {
                    len = len.wrapping_sub(1);
                } else {
                    *buffer.offset(len as isize) = NUL as ::core::ffi::c_char;
                }
                p = buffer;
                while p < buffer.offset(len as isize) {
                    if *p as ::core::ffi::c_int == NUL
                        || *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                            && check_spaces != 0
                    {
                        i += 1;
                        *p = NUL as ::core::ffi::c_char;
                    }
                    p = p.offset(1);
                }
                if len != 0 {
                    i += 1;
                }
            }
            '_c2rust_label: {
                if *buffer.offset(len as isize) as ::core::ffi::c_int == '\0' as ::core::ffi::c_int
                    || *buffer.offset(len as isize) as ::core::ffi::c_int
                        == '\n' as ::core::ffi::c_int
                {
                } else {
                    __assert_fail(
                        b"buffer[len] == NUL || buffer[len] == '\\n'\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/os/shell.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        487 as ::core::ffi::c_uint,
                        b"int os_expand_wildcards(int, char **, int *, char ***, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            if i == 0 as ::core::ffi::c_int {
                xfree(buffer as *mut ::core::ffi::c_void);
            } else {
                *num_file = i;
                *file = xmalloc(
                    ::core::mem::size_of::<*mut ::core::ffi::c_char>().wrapping_mul(i as size_t),
                ) as *mut *mut ::core::ffi::c_char;
                p = buffer;
                i = 0 as ::core::ffi::c_int;
                while i < *num_file {
                    *(*file).offset(i as isize) = p;
                    if shell_style == STYLE_ECHO
                        || shell_style == STYLE_BT
                        || shell_style == STYLE_VIMGLOB
                        || shell_style == STYLE_GLOBSTAR
                    {
                        while !(shell_style == STYLE_ECHO
                            && *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int)
                            && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                            && *p as ::core::ffi::c_int != NUL
                        {
                            p = p.offset(1);
                        }
                        if p == buffer.offset(len as isize) {
                            *p = NUL as ::core::ffi::c_char;
                        } else {
                            let c2rust_fresh6 = p;
                            p = p.offset(1);
                            *c2rust_fresh6 = NUL as ::core::ffi::c_char;
                            p = skipwhite(p);
                        }
                    } else {
                        while *p as ::core::ffi::c_int != 0 && p < buffer.offset(len as isize) {
                            p = p.offset(1);
                        }
                        p = p.offset(1);
                    }
                    i += 1;
                }
                j = 0 as ::core::ffi::c_int;
                i = 0 as ::core::ffi::c_int;
                while i < *num_file {
                    if !(flags & EW_NOTFOUND as ::core::ffi::c_int == 0
                        && !os_path_exists(*(*file).offset(i as isize)))
                    {
                        let mut dir: bool = os_isdir(*(*file).offset(i as isize));
                        if !(dir as ::core::ffi::c_int != 0
                            && flags & EW_DIR as ::core::ffi::c_int == 0
                            || !dir && flags & EW_FILE as ::core::ffi::c_int == 0)
                        {
                            if !(!dir
                                && flags & EW_EXEC as ::core::ffi::c_int != 0
                                && !os_can_exe(
                                    *(*file).offset(i as isize),
                                    ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                                    flags & EW_SHELLCMD as ::core::ffi::c_int == 0,
                                ))
                            {
                                p = xmalloc(
                                    strlen(*(*file).offset(i as isize))
                                        .wrapping_add(1 as size_t)
                                        .wrapping_add(dir as size_t),
                                ) as *mut ::core::ffi::c_char;
                                strcpy(p, *(*file).offset(i as isize));
                                if dir {
                                    add_pathsep(p);
                                }
                                let c2rust_fresh7 = j;
                                j = j + 1;
                                let c2rust_lvalue_ptr =
                                    &raw mut *(*file).offset(c2rust_fresh7 as isize);
                                *c2rust_lvalue_ptr = p;
                            }
                        }
                    }
                    i += 1;
                }
                xfree(buffer as *mut ::core::ffi::c_void);
                *num_file = j;
                if *num_file == 0 as ::core::ffi::c_int {
                    let mut ptr_: *mut *mut ::core::ffi::c_void =
                        file as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL;
                    let _ = *ptr_;
                } else {
                    return OK;
                }
            }
        }
    }
    if flags & EW_NOTFOUND as ::core::ffi::c_int != 0 {
        save_patterns(num_pat, pat, num_file, file);
        return OK;
    }
    return FAIL;
}
pub const STYLE_ECHO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const STYLE_GLOB: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STYLE_VIMGLOB: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const STYLE_PRINT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const STYLE_BT: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const STYLE_GLOBSTAR: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn shell_build_argv(
    mut cmd: *const ::core::ffi::c_char,
    mut extra_args: *const ::core::ffi::c_char,
) -> *mut *mut ::core::ffi::c_char {
    let mut argc: size_t = tokenize(
        p_sh.get(),
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    )
    .wrapping_add(if !cmd.is_null() {
        tokenize(
            p_shcf.get(),
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        )
    } else {
        0 as size_t
    });
    let mut rv: *mut *mut ::core::ffi::c_char = xmalloc(
        argc.wrapping_add(4 as size_t)
            .wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
    ) as *mut *mut ::core::ffi::c_char;
    let mut i: size_t = tokenize(p_sh.get(), rv);
    if !extra_args.is_null() {
        let c2rust_fresh8 = i;
        i = i.wrapping_add(1);
        let c2rust_lvalue_ptr = &raw mut *rv.offset(c2rust_fresh8 as isize);
        *c2rust_lvalue_ptr = xstrdup(extra_args);
    }
    if !cmd.is_null() {
        i = i.wrapping_add(tokenize(p_shcf.get(), rv.offset(i as isize)));
        let c2rust_fresh9 = i;
        i = i.wrapping_add(1);
        let c2rust_lvalue_ptr_0 = &raw mut *rv.offset(c2rust_fresh9 as isize);
        *c2rust_lvalue_ptr_0 = shell_xescape_xquote(cmd);
    }
    *rv.offset(i as isize) = ::core::ptr::null_mut::<::core::ffi::c_char>();
    '_c2rust_label: {
        if !(*rv.offset(0 as ::core::ffi::c_int as isize)).is_null() {
        } else {
            __assert_fail(
                b"rv[0]\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/shell.rs\0".as_ptr() as *const ::core::ffi::c_char,
                596 as ::core::ffi::c_uint,
                b"char **shell_build_argv(const char *, const char *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    return rv;
}
pub unsafe extern "C" fn shell_free_argv(mut argv: *mut *mut ::core::ffi::c_char) {
    let mut p: *mut *mut ::core::ffi::c_char = argv;
    if p.is_null() {
        return;
    }
    while !(*p).is_null() {
        xfree(*p as *mut ::core::ffi::c_void);
        p = p.offset(1);
    }
    xfree(argv as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn shell_argv_to_str(
    argv: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut n: size_t = 0 as size_t;
    let mut p: *mut *mut ::core::ffi::c_char = argv;
    let mut rv: *mut ::core::ffi::c_char =
        xcalloc(256 as size_t, ::core::mem::size_of::<::core::ffi::c_char>())
            as *mut ::core::ffi::c_char;
    let maxsize: size_t =
        (256 as size_t).wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>());
    if (*p).is_null() {
        return rv;
    }
    while !(*p).is_null() {
        xstrlcat(rv, b"'\0".as_ptr() as *const ::core::ffi::c_char, maxsize);
        xstrlcat(rv, *p, maxsize);
        n = xstrlcat(rv, b"' \0".as_ptr() as *const ::core::ffi::c_char, maxsize);
        if n >= maxsize {
            break;
        }
        p = p.offset(1);
    }
    if n < maxsize {
        *rv.offset(n.wrapping_sub(1 as size_t) as isize) = NUL as ::core::ffi::c_char;
    } else {
        *rv.offset(maxsize.wrapping_sub(4 as size_t) as isize) = '.' as ::core::ffi::c_char;
        *rv.offset(maxsize.wrapping_sub(3 as size_t) as isize) = '.' as ::core::ffi::c_char;
        *rv.offset(maxsize.wrapping_sub(2 as size_t) as isize) = '.' as ::core::ffi::c_char;
        *rv.offset(maxsize.wrapping_sub(1 as size_t) as isize) = NUL as ::core::ffi::c_char;
    }
    return rv;
}
pub unsafe extern "C" fn os_call_shell(
    mut cmd: *mut ::core::ffi::c_char,
    mut opts: ::core::ffi::c_int,
    mut extra_args: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut input: StringBuilder = KV_INITIAL_VALUE;
    let mut output: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut output_ptr: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut current_state: ::core::ffi::c_int = State.get();
    let mut forward_output: bool = true_0 != 0;
    signal_reject_deadly();
    if opts & (kShellOptHideMess as ::core::ffi::c_int | kShellOptExpand as ::core::ffi::c_int) != 0
    {
        forward_output = false_0 != 0;
    } else {
        State.set(MODE_EXTERNCMD as ::core::ffi::c_int);
        if opts & kShellOptWrite as ::core::ffi::c_int != 0 {
            read_input(&raw mut input);
        }
        if opts & kShellOptRead as ::core::ffi::c_int != 0 {
            output_ptr = &raw mut output;
            forward_output = false_0 != 0;
        } else if opts & kShellOptDoOut as ::core::ffi::c_int != 0 {
            forward_output = false_0 != 0;
        }
    }
    let mut nread: size_t = 0;
    let mut exitcode: ::core::ffi::c_int = do_os_system(
        shell_build_argv(cmd, extra_args),
        input.items,
        input.size,
        output_ptr,
        &raw mut nread,
        emsg_silent.get() != 0,
        forward_output,
    );
    xfree(input.items as *mut ::core::ffi::c_void);
    input.capacity = 0 as size_t;
    input.size = input.capacity;
    input.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !output.is_null() {
        write_output(output, nread, true_0 != 0);
        xfree(output as *mut ::core::ffi::c_void);
    }
    if emsg_silent.get() == 0
        && exitcode != 0 as ::core::ffi::c_int
        && opts & kShellOptSilent as ::core::ffi::c_int == 0
    {
        msg_ext_set_kind(b"shell_ret\0".as_ptr() as *const ::core::ffi::c_char);
        if !ui_has(kUIMessages) {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        msg_puts(gettext(
            b"shell returned \0".as_ptr() as *const ::core::ffi::c_char
        ));
        msg_outnum(exitcode);
    }
    State.set(current_state);
    signal_accept_deadly();
    return exitcode;
}
pub unsafe extern "C" fn call_shell(
    mut cmd: *mut ::core::ffi::c_char,
    mut opts: ::core::ffi::c_int,
    mut extra_shell_arg: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = 0;
    let mut wait_time: proftime_T = 0;
    if p_verbose.get() > 3 as OptInt {
        verbose_enter();
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"Executing command: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
            if cmd.is_null() { p_sh.get() } else { cmd },
        );
        msg_putchar('\n' as ::core::ffi::c_int);
        verbose_leave();
    }
    if do_profiling.get() == PROF_YES {
        wait_time = prof_child_enter();
    }
    if *p_sh.get() as ::core::ffi::c_int == NUL {
        emsg(gettext(
            &raw const e_shellempty as *const ::core::ffi::c_char,
        ));
        retval = -1 as ::core::ffi::c_int;
    } else {
        tag_freematch();
        retval = os_call_shell(cmd, opts, extra_shell_arg);
    }
    set_vim_var_nr(VV_SHELL_ERROR, retval as varnumber_T);
    if do_profiling.get() == PROF_YES {
        prof_child_exit(wait_time);
    }
    return retval;
}
pub unsafe extern "C" fn get_cmd_output(
    mut cmd: *mut ::core::ffi::c_char,
    mut infile: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut ret_len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut len: size_t = 0;
    let mut i: size_t = 0;
    let mut buffer: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if check_secure() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut tempname: *mut ::core::ffi::c_char = vim_tempname();
    if tempname.is_null() {
        emsg(gettext(&raw const e_notmp as *const ::core::ffi::c_char));
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut command: *mut ::core::ffi::c_char =
        make_filter_cmd(cmd, infile, tempname, false_0 != 0);
    (*no_check_timestamps.ptr()) += 1;
    call_shell(
        command,
        kShellOptDoOut as ::core::ffi::c_int | kShellOptExpand as ::core::ffi::c_int | flags,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
    (*no_check_timestamps.ptr()) -= 1;
    xfree(command as *mut ::core::ffi::c_void);
    let mut fd: *mut FILE = os_fopen(tempname, READBIN.as_ptr());
    let mut len_l: ::core::ffi::c_long = 0;
    if fd.is_null()
        || fseek(fd, 0 as ::core::ffi::c_long, SEEK_END) == -1 as ::core::ffi::c_int
        || {
            len_l = ftell(fd);
            len_l == -1 as ::core::ffi::c_long
        }
        || fseek(fd, 0 as ::core::ffi::c_long, SEEK_SET) == -1 as ::core::ffi::c_int
    {
        semsg(
            gettext(&raw const e_cannot_read_from_str_2 as *const ::core::ffi::c_char),
            tempname,
        );
        if !fd.is_null() {
            fclose(fd);
        }
    } else {
        len = len_l as size_t;
        buffer = xmalloc(len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        i = fread(buffer as *mut ::core::ffi::c_void, 1 as size_t, len, fd) as size_t;
        fclose(fd);
        os_remove(tempname);
        if i != len {
            semsg(
                gettext(&raw const e_cant_read_file_str as *const ::core::ffi::c_char),
                tempname,
            );
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut buffer as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
        } else if ret_len.is_null() {
            i = 0 as size_t;
            while i < len {
                if *buffer.offset(i as isize) as ::core::ffi::c_int == NUL {
                    *buffer.offset(i as isize) = 1 as ::core::ffi::c_char;
                }
                i = i.wrapping_add(1);
            }
            *buffer.offset(len as isize) = NUL as ::core::ffi::c_char;
        } else {
            *ret_len = len;
        }
    }
    xfree(tempname as *mut ::core::ffi::c_void);
    return buffer;
}
#[no_mangle]
pub unsafe extern "C" fn os_system(
    mut argv: *mut *mut ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
    mut len: size_t,
    mut output: *mut *mut ::core::ffi::c_char,
    mut nread: *mut size_t,
) -> ::core::ffi::c_int {
    return do_os_system(argv, input, len, output, nread, true_0 != 0, false_0 != 0);
}
unsafe extern "C" fn do_os_system(
    mut argv: *mut *mut ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
    mut len: size_t,
    mut output: *mut *mut ::core::ffi::c_char,
    mut nread: *mut size_t,
    mut silent: bool,
    mut forward_output: bool,
) -> ::core::ffi::c_int {
    let mut exitcode: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    out_data_decide_throttle(0 as size_t);
    out_data_ring(::core::ptr::null::<::core::ffi::c_char>(), 0 as size_t);
    let mut has_input: bool = !input.is_null() && len > 0 as size_t;
    let mut buf: StringBuilder = KV_INITIAL_VALUE;
    let mut data_cb: stream_read_cb = Some(
        system_data_cb
            as unsafe extern "C" fn(
                *mut RStream,
                *const ::core::ffi::c_char,
                size_t,
                *mut ::core::ffi::c_void,
                bool,
            ) -> size_t,
    );
    if !nread.is_null() {
        *nread = 0 as size_t;
    }
    if forward_output {
        data_cb = Some(
            out_data_cb
                as unsafe extern "C" fn(
                    *mut RStream,
                    *const ::core::ffi::c_char,
                    size_t,
                    *mut ::core::ffi::c_void,
                    bool,
                ) -> size_t,
        ) as stream_read_cb;
    } else if output.is_null() {
        data_cb = None;
    }
    let mut prog: [::core::ffi::c_char; 4096] = [0; 4096];
    xstrlcpy(
        &raw mut prog as *mut ::core::ffi::c_char,
        *argv.offset(0 as ::core::ffi::c_int as isize),
        MAXPATHL as size_t,
    );
    let mut uvproc: LibuvProc =
        libuv_proc_init(main_loop.ptr(), &raw mut buf as *mut ::core::ffi::c_void);
    let mut proc: *mut Proc = &raw mut uvproc.proc;
    let mut events: *mut MultiQueue = multiqueue_new_child((*main_loop.ptr()).events);
    (*proc).events = events;
    (*proc).argv = argv;
    let mut status: ::core::ffi::c_int = proc_spawn(proc, has_input, true_0 != 0, true_0 != 0);
    '_end: {
        if status != 0 {
            loop_poll_events(main_loop.ptr(), 0 as int64_t);
            if !silent {
                msg_puts(gettext(
                    b"\nshell failed to start: \0".as_ptr() as *const ::core::ffi::c_char
                ));
                msg_outtrans(uv_strerror(status), 0 as ::core::ffi::c_int, false_0 != 0);
                msg_puts(b": \0".as_ptr() as *const ::core::ffi::c_char);
                msg_outtrans(
                    &raw mut prog as *mut ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
                msg_putchar('\n' as ::core::ffi::c_int);
            }
        } else {
            if has_input {
                wstream_init(&raw mut (*proc).in_0, 0 as size_t);
            }
            rstream_init(&raw mut (*proc).out);
            rstream_start(
                &raw mut (*proc).out,
                data_cb,
                &raw mut buf as *mut ::core::ffi::c_void,
            );
            rstream_init(&raw mut (*proc).err);
            rstream_start(
                &raw mut (*proc).err,
                data_cb,
                &raw mut buf as *mut ::core::ffi::c_void,
            );
            if has_input {
                let mut input_buffer: *mut WBuffer =
                    wstream_new_buffer(input as *mut ::core::ffi::c_char, len, 1 as size_t, None);
                if wstream_write(&raw mut (*proc).in_0, input_buffer) != 0 as ::core::ffi::c_int {
                    proc_stop(proc);
                    break '_end;
                } else {
                    wstream_set_write_cb(
                        &raw mut (*proc).in_0,
                        Some(
                            shell_write_cb
                                as unsafe extern "C" fn(
                                    *mut Stream,
                                    *mut ::core::ffi::c_void,
                                    ::core::ffi::c_int,
                                ) -> (),
                        ),
                        NULL,
                    );
                }
            }
            ui_busy_start();
            ui_flush();
            if forward_output {
                msg_sb_eol();
                msg_start();
                msg_no_more.set(true_0 != 0);
                lines_left.set(-1 as ::core::ffi::c_int);
            }
            exitcode = proc_wait(
                proc,
                -1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<MultiQueue>(),
            );
            if !got_int.get() && out_data_decide_throttle(0 as size_t) as ::core::ffi::c_int != 0 {
                out_data_ring(
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    SIZE_MAX as size_t,
                );
            }
            if forward_output {
                (*no_wait_return.ptr()) += 1;
                msg_end();
                (*no_wait_return.ptr()) -= 1;
                msg_no_more.set(false_0 != 0);
            }
            ui_busy_stop();
            if !output.is_null() {
                '_c2rust_label: {
                    if !nread.is_null() {
                    } else {
                        __assert_fail(
                            b"nread\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/os/shell.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            971 as ::core::ffi::c_uint,
                            b"int do_os_system(char **, const char *, size_t, char **, size_t *, _Bool, _Bool)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if buf.size == 0 as size_t {
                    *output = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    *nread = 0 as size_t;
                    xfree(buf.items as *mut ::core::ffi::c_void);
                    buf.capacity = 0 as size_t;
                    buf.size = buf.capacity;
                    buf.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
                } else {
                    *nread = buf.size;
                    if buf.size == buf.capacity {
                        buf.capacity = if buf.capacity != 0 {
                            buf.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        buf.items = xrealloc(
                            buf.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<::core::ffi::c_char>()
                                .wrapping_mul(buf.capacity),
                        ) as *mut ::core::ffi::c_char;
                    } else {
                    };
                    let c2rust_fresh10 = buf.size;
                    buf.size = buf.size.wrapping_add(1);
                    *buf.items.offset(c2rust_fresh10 as isize) = '\0' as ::core::ffi::c_char;
                    *output = buf.items;
                }
            }
            '_c2rust_label_0: {
                if multiqueue_empty(events) {
                } else {
                    __assert_fail(
                        b"multiqueue_empty(events)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/os/shell.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        985 as ::core::ffi::c_uint,
                        b"int do_os_system(char **, const char *, size_t, char **, size_t *, _Bool, _Bool)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
        }
    }
    multiqueue_free(events);
    return exitcode;
}
unsafe extern "C" fn system_data_cb(
    mut _stream: *mut RStream,
    mut buf: *const ::core::ffi::c_char,
    mut count: size_t,
    mut data: *mut ::core::ffi::c_void,
    mut _eof: bool,
) -> size_t {
    let mut dbuf: *mut StringBuilder = data as *mut StringBuilder;
    if count > 0 as size_t {
        if (*dbuf).capacity < (*dbuf).size.wrapping_add(count) {
            (*dbuf).capacity = (*dbuf).size.wrapping_add(count);
            (*dbuf).capacity = (*dbuf).capacity.wrapping_sub(1);
            (*dbuf).capacity |= (*dbuf).capacity >> 1 as ::core::ffi::c_int;
            (*dbuf).capacity |= (*dbuf).capacity >> 2 as ::core::ffi::c_int;
            (*dbuf).capacity |= (*dbuf).capacity >> 4 as ::core::ffi::c_int;
            (*dbuf).capacity |= (*dbuf).capacity >> 8 as ::core::ffi::c_int;
            (*dbuf).capacity |= (*dbuf).capacity >> 16 as ::core::ffi::c_int;
            (*dbuf).capacity = (*dbuf).capacity.wrapping_add(1);
            (*dbuf).capacity = (*dbuf).capacity;
            (*dbuf).items = xrealloc(
                (*dbuf).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*dbuf).capacity),
            ) as *mut ::core::ffi::c_char;
        }
        '_c2rust_label: {
            if !(*dbuf).items.is_null() {
            } else {
                __assert_fail(
                    b"(*dbuf).items\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/os/shell.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1002 as ::core::ffi::c_uint,
                    b"size_t system_data_cb(RStream *, const char *, size_t, void *, _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        memcpy(
            (*dbuf).items.offset((*dbuf).size as isize) as *mut ::core::ffi::c_void,
            buf as *const ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(count),
        );
        (*dbuf).size = (*dbuf).size.wrapping_add(count);
    }
    return count;
}
unsafe extern "C" fn out_data_decide_throttle(mut size: size_t) -> bool {
    static started: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
    static received: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
    static visit: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
    static pulse_msg: GlobalCell<[::core::ffi::c_char; 4]> = GlobalCell::new([
        ' ' as ::core::ffi::c_char,
        ' ' as ::core::ffi::c_char,
        ' ' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
    ]);
    if size == 0 {
        let mut previous_decision: bool = visit.get() > 0 as size_t;
        visit.set(0 as size_t);
        received.set(visit.get());
        started.set(received.get() as uint64_t);
        return previous_decision;
    }
    received.set((*received.ptr()).wrapping_add(size));
    if received.get() < OUT_DATA_THRESHOLD as size_t
        || started.get() == 0 && received.get() < size.wrapping_add(1000 as size_t)
    {
        return false_0 != 0;
    } else if visit.get() == 0 {
        started.set(os_hrtime());
    } else {
        let mut since: uint64_t = os_hrtime().wrapping_sub(started.get());
        if since
            < (visit.get() as uint64_t)
                .wrapping_mul(NS_1_SECOND.wrapping_div(10 as ::core::ffi::c_uint) as uint64_t)
        {
            return true_0 != 0;
        }
        if since > (3 as ::core::ffi::c_uint).wrapping_mul(NS_1_SECOND) as uint64_t {
            visit.set(0 as size_t);
            received.set(visit.get());
            return false_0 != 0;
        }
    }
    visit.set((*visit.ptr()).wrapping_add(1));
    let mut tick: size_t = (*visit.ptr()).wrapping_rem(4 as size_t);
    (*pulse_msg.ptr())[0 as ::core::ffi::c_int as usize] = (if tick > 0 as size_t {
        '.' as ::core::ffi::c_int
    } else {
        ' ' as ::core::ffi::c_int
    }) as ::core::ffi::c_char;
    (*pulse_msg.ptr())[1 as ::core::ffi::c_int as usize] = (if tick > 1 as size_t {
        '.' as ::core::ffi::c_int
    } else {
        ' ' as ::core::ffi::c_int
    }) as ::core::ffi::c_char;
    (*pulse_msg.ptr())[2 as ::core::ffi::c_int as usize] = (if tick > 2 as size_t {
        '.' as ::core::ffi::c_int
    } else {
        ' ' as ::core::ffi::c_int
    }) as ::core::ffi::c_char;
    if visit.get() == 1 as size_t {
        msg_puts(b"...\n\0".as_ptr() as *const ::core::ffi::c_char);
    }
    msg_putchar('\r' as ::core::ffi::c_int);
    msg_puts(pulse_msg.ptr() as *mut ::core::ffi::c_char);
    msg_putchar('\r' as ::core::ffi::c_int);
    ui_flush();
    return true_0 != 0;
}
unsafe extern "C" fn out_data_ring(mut output: *const ::core::ffi::c_char, mut size: size_t) {
    static last_skipped: GlobalCell<[::core::ffi::c_char; 5120]> = GlobalCell::new([0; 5120]);
    static last_skipped_len: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
    '_c2rust_label: {
        if !output.is_null() || (size == 0 as size_t || size == 18446744073709551615 as size_t) {
        } else {
            __assert_fail(
                b"output != NULL || (size == 0 || size == SIZE_MAX)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/os/shell.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1092 as ::core::ffi::c_uint,
                b"void out_data_ring(const char *, size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if output.is_null() && size == 0 as size_t {
        last_skipped_len.set(0 as size_t);
        return;
    }
    if output.is_null() && size == SIZE_MAX as size_t {
        out_data_append_to_screen(
            last_skipped.ptr() as *mut ::core::ffi::c_char,
            last_skipped_len.ptr(),
            STDOUT_FILENO,
            true_0 != 0,
        );
        return;
    }
    if size >= MAX_CHUNK_SIZE as size_t {
        let mut start: size_t = size.wrapping_sub(MAX_CHUNK_SIZE as size_t);
        memcpy(
            last_skipped.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            output.offset(start as isize) as *const ::core::ffi::c_void,
            MAX_CHUNK_SIZE as size_t,
        );
        last_skipped_len.set(MAX_CHUNK_SIZE as size_t);
    } else if size > 0 as size_t {
        let mut keep_len: size_t = if last_skipped_len.get()
            < ((1024 as ::core::ffi::c_uint)
                .wrapping_mul(10 as ::core::ffi::c_uint)
                .wrapping_div(2 as ::core::ffi::c_uint) as size_t)
                .wrapping_sub(size)
        {
            last_skipped_len.get()
        } else {
            ((1024 as ::core::ffi::c_uint)
                .wrapping_mul(10 as ::core::ffi::c_uint)
                .wrapping_div(2 as ::core::ffi::c_uint) as size_t)
                .wrapping_sub(size)
        };
        let mut keep_start: size_t = (*last_skipped_len.ptr()).wrapping_sub(keep_len);
        if keep_start != 0 {
            memmove(
                last_skipped.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                (last_skipped.ptr() as *mut ::core::ffi::c_char).offset(keep_start as isize)
                    as *const ::core::ffi::c_void,
                keep_len,
            );
        }
        memcpy(
            (last_skipped.ptr() as *mut ::core::ffi::c_char).offset(keep_len as isize)
                as *mut ::core::ffi::c_void,
            output as *const ::core::ffi::c_void,
            size,
        );
        last_skipped_len.set(keep_len.wrapping_add(size));
    }
}
pub const MAX_CHUNK_SIZE: ::core::ffi::c_uint =
    OUT_DATA_THRESHOLD.wrapping_div(2 as ::core::ffi::c_uint);
unsafe extern "C" fn out_data_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut need_clear: bool = true_0 != 0;
    let mut hl: ::core::ffi::c_int = if (*argv.offset(2 as ::core::ffi::c_int as isize))
        .expose_addr() as intptr_t as ::core::ffi::c_int
        == STDERR_FILENO
    {
        HLF_SE as ::core::ffi::c_int
    } else {
        HLF_SO as ::core::ffi::c_int
    };
    msg_ext_set_kind(
        if (*argv.offset(2 as ::core::ffi::c_int as isize)).expose_addr() as intptr_t
            as ::core::ffi::c_int
            == STDERR_FILENO
        {
            b"shell_err\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"shell_out\0".as_ptr() as *const ::core::ffi::c_char
        },
    );
    msg_ext_set_append(true_0 != 0);
    msg_multiline(
        String_0 {
            data: *argv.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char,
            size: (*argv.offset(1 as ::core::ffi::c_int as isize)).expose_addr() as size_t,
        },
        hl,
        false_0 != 0,
        false_0 != 0,
        &raw mut need_clear,
    );
    xfree(*argv.offset(0 as ::core::ffi::c_int as isize));
    ui_flush();
}
unsafe extern "C" fn out_data_append_to_screen(
    mut output: *const ::core::ffi::c_char,
    mut count: *mut size_t,
    mut fd: ::core::ffi::c_int,
    mut eof: bool,
) {
    let mut p: *const ::core::ffi::c_char = output;
    let mut end: *const ::core::ffi::c_char = output.offset(*count as isize);
    while p < end {
        let mut i: ::core::ffi::c_int = if *p as ::core::ffi::c_int != 0 {
            utfc_ptr2len_len(
                p,
                *count as ::core::ffi::c_int - p.offset_from(output) as ::core::ffi::c_int,
            )
        } else {
            1 as ::core::ffi::c_int
        };
        if !eof
            && i == 1 as ::core::ffi::c_int
            && (*utf8len_tab_zero.ptr())[*(p as *mut uint8_t) as usize] as isize
                > end.offset_from(p)
        {
            *count = p.offset_from(output) as size_t;
            break;
        } else {
            p = p.offset(i as isize);
        }
    }
    let mut str: *mut ::core::ffi::c_char =
        xmemdupz(output as *const ::core::ffi::c_void, *count) as *mut ::core::ffi::c_char;
    if ui_has(kUIMessages) {
        multiqueue_put_event(
            (*main_loop.ptr()).fast_events,
            Event {
                handler: Some(
                    out_data_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    str as *mut ::core::ffi::c_void,
                    ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(*count as usize),
                    ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                        fd as intptr_t as usize,
                    ),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ],
            },
        );
    } else {
        let mut c2rust_lvalue: [*mut ::core::ffi::c_void; 3] = [
            str as *mut ::core::ffi::c_void,
            ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(*count as usize),
            ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(fd as intptr_t as usize),
        ];
        out_data_event(&raw mut c2rust_lvalue as *mut *mut ::core::ffi::c_void);
    };
}
unsafe extern "C" fn out_data_cb(
    mut stream: *mut RStream,
    mut ptr: *const ::core::ffi::c_char,
    mut count: size_t,
    mut _data: *mut ::core::ffi::c_void,
    mut eof: bool,
) -> size_t {
    if count > 0 as size_t && out_data_decide_throttle(count) as ::core::ffi::c_int != 0 {
        out_data_ring(ptr, count);
    } else if count > 0 as size_t {
        out_data_append_to_screen(
            ptr,
            &raw mut count,
            (*stream).s.fd as ::core::ffi::c_int,
            eof,
        );
    }
    return count;
}
unsafe extern "C" fn tokenize(
    str: *const ::core::ffi::c_char,
    argv: *mut *mut ::core::ffi::c_char,
) -> size_t {
    let mut argc: size_t = 0 as size_t;
    let mut p: *const ::core::ffi::c_char = str;
    while *p as ::core::ffi::c_int != NUL {
        let len: size_t = word_length(p);
        if !argv.is_null() {
            *argv.offset(argc as isize) = vim_strnsave_unquoted(p, len);
        }
        argc = argc.wrapping_add(1);
        p = skipwhite(p.offset(len as isize));
    }
    return argc;
}
unsafe extern "C" fn word_length(mut str: *const ::core::ffi::c_char) -> size_t {
    let mut p: *const ::core::ffi::c_char = str;
    let mut inquote: bool = false_0 != 0;
    let mut length: size_t = 0 as size_t;
    while *p as ::core::ffi::c_int != 0
        && (inquote as ::core::ffi::c_int != 0
            || *p as ::core::ffi::c_int != ' ' as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != TAB)
    {
        if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
            inquote = !inquote;
        } else if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && inquote as ::core::ffi::c_int != 0
        {
            p = p.offset(1);
            length = length.wrapping_add(1);
        }
        p = p.offset(1);
        length = length.wrapping_add(1);
    }
    return length;
}
unsafe extern "C" fn read_input(mut buf: *mut StringBuilder) {
    read_buffer_into(
        curbuf.get(),
        (*curbuf.get()).b_op_start.lnum,
        (*curbuf.get()).b_op_end.lnum,
        buf,
    );
}
unsafe extern "C" fn write_output(
    mut output: *mut ::core::ffi::c_char,
    mut remaining: size_t,
    mut eof: bool,
) -> size_t {
    if output.is_null() {
        return 0 as size_t;
    }
    let mut start: *mut ::core::ffi::c_char = output;
    let mut off: size_t = 0 as size_t;
    while off < remaining {
        if *output.offset(off as isize) as ::core::ffi::c_int == CAR
            && *output.offset(off.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int == NL
            && (*curbuf.get()).b_p_bin == 0
        {
            *output.offset(off as isize) = NUL as ::core::ffi::c_char;
            let c2rust_fresh11 = (*curwin.get()).w_cursor.lnum;
            (*curwin.get()).w_cursor.lnum = (*curwin.get()).w_cursor.lnum + 1;
            ml_append(
                c2rust_fresh11,
                output,
                off as colnr_T + 1 as colnr_T,
                false_0 != 0,
            );
            let mut skip: size_t = off.wrapping_add(2 as size_t);
            output = output.offset(skip as isize);
            remaining = remaining.wrapping_sub(skip);
            off = 0 as size_t;
        } else if *output.offset(off as isize) as ::core::ffi::c_int == CAR
            && (*curbuf.get()).b_p_bin == 0
            || *output.offset(off as isize) as ::core::ffi::c_int == NL
        {
            *output.offset(off as isize) = NUL as ::core::ffi::c_char;
            let c2rust_fresh12 = (*curwin.get()).w_cursor.lnum;
            (*curwin.get()).w_cursor.lnum = (*curwin.get()).w_cursor.lnum + 1;
            ml_append(
                c2rust_fresh12,
                output,
                off as colnr_T + 1 as colnr_T,
                false_0 != 0,
            );
            let mut skip_0: size_t = off.wrapping_add(1 as size_t);
            output = output.offset(skip_0 as isize);
            remaining = remaining.wrapping_sub(skip_0);
            off = 0 as size_t;
        } else {
            if *output.offset(off as isize) as ::core::ffi::c_int == NUL {
                *output.offset(off as isize) = NL as ::core::ffi::c_char;
            }
            off = off.wrapping_add(1);
        }
    }
    if eof {
        if remaining != 0 {
            let c2rust_fresh13 = (*curwin.get()).w_cursor.lnum;
            (*curwin.get()).w_cursor.lnum = (*curwin.get()).w_cursor.lnum + 1;
            ml_append(c2rust_fresh13, output, 0 as colnr_T, false_0 != 0);
            (*curbuf.get()).b_no_eol_lnum = (*curwin.get()).w_cursor.lnum;
            output = output.offset(remaining as isize);
        } else {
            (*curbuf.get()).b_no_eol_lnum = 0 as ::core::ffi::c_int as linenr_T;
        }
    }
    ui_flush();
    return output.offset_from(start) as size_t;
}
unsafe extern "C" fn shell_write_cb(
    mut stream: *mut Stream,
    mut _data: *mut ::core::ffi::c_void,
    mut status: ::core::ffi::c_int,
) {
    if status != 0 {
        msg_schedule_semsg(
            gettext(
                b"E5677: Error writing input to shell-command: %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            uv_err_name(status),
        );
    }
    stream_may_close(stream);
}
unsafe extern "C" fn shell_xescape_xquote(
    mut cmd: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if *p_sxq.get() as ::core::ffi::c_int == NUL {
        return xstrdup(cmd);
    }
    let mut ecmd: *const ::core::ffi::c_char = cmd;
    if *p_sxe.get() as ::core::ffi::c_int != NUL
        && strcmp(p_sxq.get(), b"(\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        ecmd = vim_strsave_escaped_ext(cmd, p_sxe.get(), '^' as ::core::ffi::c_char, false_0 != 0);
    }
    let mut ncmd_size: size_t = strlen(ecmd)
        .wrapping_add(strlen(p_sxq.get()).wrapping_mul(2 as size_t))
        .wrapping_add(1 as size_t);
    let mut ncmd: *mut ::core::ffi::c_char = xmalloc(ncmd_size) as *mut ::core::ffi::c_char;
    if strcmp(p_sxq.get(), b"(\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        vim_snprintf(
            ncmd,
            ncmd_size,
            b"(%s)\0".as_ptr() as *const ::core::ffi::c_char,
            ecmd,
        );
    } else if strcmp(p_sxq.get(), b"\"(\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        vim_snprintf(
            ncmd,
            ncmd_size,
            b"\"(%s)\"\0".as_ptr() as *const ::core::ffi::c_char,
            ecmd,
        );
    } else {
        vim_snprintf(
            ncmd,
            ncmd_size,
            b"%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
            p_sxq.get(),
            ecmd,
            p_sxq.get(),
        );
    }
    if ecmd != cmd {
        xfree(ecmd as *mut ::core::ffi::c_void);
    }
    return ncmd;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const READBIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"rb\0") };
