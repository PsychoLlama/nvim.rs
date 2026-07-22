use crate::src::nvim::api::private::converter::object_to_vim;
use crate::src::nvim::api::private::helpers::{
    arena_array, arena_dict, arena_string, cstr_as_string,
};
use crate::src::nvim::autocmd::{apply_autocmds, has_event};
use crate::src::nvim::eval::encode::{encode_list_write, encode_tv2json};
use crate::src::nvim::eval::typval::{
    callback_free, tv_clear, tv_dict_add_dict, tv_dict_add_list, tv_dict_find, tv_dict_free,
    tv_dict_set_keys_readonly, tv_list_alloc, tv_list_append_string, tv_list_unref,
};
use crate::src::nvim::eval_1::{
    callback_call, eval_fmt_source_name_line, get_v_event, restore_v_event,
};
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::event::libuv_proc::libuv_proc_init;
use crate::src::nvim::event::multiqueue::{
    multiqueue_free, multiqueue_new_child, multiqueue_put_event,
};
use crate::src::nvim::event::proc::{exit_on_closed_chan, proc_free, proc_spawn, proc_stop};
use crate::src::nvim::event::rstream::{
    rstream_init, rstream_init_fd, rstream_may_close, rstream_start, rstream_start_inner,
    rstream_stop_inner,
};
use crate::src::nvim::event::socket::{socket_connect, socket_watcher_accept};
use crate::src::nvim::event::stream::stream_may_close;
use crate::src::nvim::event::wstream::{
    wstream_init, wstream_init_fd, wstream_new_buffer, wstream_write,
};
use crate::src::nvim::garray::{ga_clear, ga_concat_len, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::lua::executor::api_free_luaref;
use crate::src::nvim::main::{
    channels, curbuf, e_invarg2, e_invchan, e_invstream, e_invstreamrpc, e_jobspawn, e_streamkey,
    embedded_mode, exiting, headless_mode, main_loop, ui_client_channel_id, IObuff,
};
use crate::src::nvim::map::{map_del_uint64_t_ptr_t, map_put_ref_uint64_t_ptr_t, mh_get_uint64_t};
use crate::src::nvim::memory::{arena_mem_free, xcalloc, xfree, xmemdup, xrealloc, xstrdup};
use crate::src::nvim::message::semsg;
use crate::src::nvim::msgpack_rpc::channel::rpc_init;
use crate::src::nvim::msgpack_rpc::server::server_owns_pipe_address;
use crate::src::nvim::os::fs::os_write;
use crate::src::nvim::os::libc::{
    __assert_fail, abort, dup2, fcntl, freopen, gettext, qsort, stderr, strlen,
};
use crate::src::nvim::os::pty_proc_unix::{
    pty_proc_close_master, pty_proc_init, pty_proc_resize, pty_proc_resume, pty_proc_tty_name,
};
use crate::src::nvim::os::shell::shell_free_argv;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, Arena, ArenaMem, Array, BoolVarValue, Boolean,
    BufUpdateCallbacks, Buffer, Callback, CallbackReader, CallbackType,
    Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, Channel, ChannelCallFrame, ChannelPart,
    ChannelStdinMode, ChannelStreamType, Channel_stream as C2Rust_Unnamed_33, ClientType, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_2, Dict, Error, ErrorType, Event, ExtmarkUndoObject,
    FileID, Float, FloatAnchor, FloatRelative, GridView, Integer, InternalState, Intersection,
    KeyValuePair, LibuvProc, ListLenSpecials, Loop, LuaRef, MTKey, MTNode, MTPos, MapHash,
    Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree,
    MultiQueue, Object, ObjectType, OptInt, PackerBuffer, PackerBufferFlush, Proc, ProcType,
    PtyProc, RStream, RemoteUI, RpcState, RpcState_call_stack as C2Rust_Unnamed_32,
    ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SocketWatcher, SpecialVarValue, StderrState, StdioPair, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, Stream, String_0, Terminal, TerminalOptions,
    Timestamp, Unpacker, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos,
    WBuffer, WinConfig, WinInfo, WinSplit, WinStyle, Window, _IO_codecvt, _IO_lock_t, _IO_marker,
    _IO_wide_data, __compar_fn_t, __gid_t, __off64_t, __off_t, __pthread_internal_list,
    __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t, __socklen_t, __time_t, __uid_t,
    addrinfo, alist_T, argv_callback, auto_event, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T,
    bufstate_T, chunksize_T, colnr_T, consumed_blk, dict_T, dictitem_T, dictvar_S, disptick_T,
    event_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_3, file_buffer_b_wininfo as C2Rust_Unnamed_11,
    file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, gid_t, handle_T,
    hash_T, hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, internal_proc_cb,
    intptr_t, key_value_pair, lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S,
    listwatch_S, listwatch_T, llpos_T, loop_0, loop_0_children as C2Rust_Unnamed_21, lpos_T,
    mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T,
    mtnode_inner_s, mtnode_s, multiqueue, object, object_data as C2Rust_Unnamed, packer_buffer_t,
    partial_S, partial_T, pos_T, pos_save_T, proc, proc_exit_cb, proc_state_cb, proftime_T,
    pthread_mutex_t, pthread_rwlock_t, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue,
    reg_extmatch_T, regmmatch_T, regprog, regprog_T, rstream, sa_family_t, sattr_T, save_v_event_T,
    schar_T, scid_T, sctx_T, size_t, sockaddr, socket_cb, socket_close_cb, socket_watcher,
    socket_watcher_uv as C2Rust_Unnamed_29, socket_watcher_uv_pipe as C2Rust_Unnamed_30,
    socket_watcher_uv_tcp as C2Rust_Unnamed_31, socklen_t, ssize_t, stream, stream_close_cb,
    stream_read_cb, stream_uv as C2Rust_Unnamed_23, stream_write_cb, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, taggy_T, terminal,
    terminal_close_cb, terminal_read_pause_cb, terminal_resize_cb, terminal_resume_cb,
    terminal_write_cb, time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header,
    u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8, u_header_uh_alt_prev as C2Rust_Unnamed_7,
    u_header_uh_next as C2Rust_Unnamed_10, u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T,
    uid_t, uint16_t, uint32_t, uint64_t, uint8_t, undo_object, uv__io_cb, uv__io_s, uv__io_t,
    uv__queue, uv_alloc_cb, uv_async_cb, uv_async_s, uv_async_s_u as C2Rust_Unnamed_18, uv_async_t,
    uv_buf_t, uv_close_cb, uv_connect_cb, uv_connect_s, uv_connect_t, uv_connection_cb, uv_exit_cb,
    uv_file, uv_gid_t, uv_handle_s, uv_handle_s_u as C2Rust_Unnamed_13, uv_handle_t,
    uv_handle_type, uv_idle_cb, uv_idle_s, uv_idle_s_u as C2Rust_Unnamed_24, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_17, uv_loop_s_timer_heap as C2Rust_Unnamed_16,
    uv_loop_t, uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_26, uv_pipe_t,
    uv_process_options_s, uv_process_options_t, uv_process_s, uv_process_s_u as C2Rust_Unnamed_27,
    uv_process_t, uv_read_cb, uv_req_type, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s,
    uv_shutdown_t, uv_signal_cb, uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed_14,
    uv_signal_s_u as C2Rust_Unnamed_15, uv_signal_t, uv_stdio_container_s,
    uv_stdio_container_s_data as C2Rust_Unnamed_28, uv_stdio_container_t, uv_stdio_flags,
    uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_22, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_25, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_19, uv_timer_s_u as C2Rust_Unnamed_20, uv_timer_t, uv_uid_t,
    varnumber_T, virt_line, visualinfo_T, wbuffer, wbuffer_data_finalizer, win_T, window_S,
    wininfo_S, winopt_T, winsize, wline_T, xfmark_T, FILE, QUEUE, _IO_FILE,
};
use crate::src::nvim::ui_client::ui_client_attach_to_restarted_server;
extern "C" {
    fn arena_finish(arena: *mut Arena) -> ArenaMem;
    fn arena_alloc(arena: *mut Arena, size: size_t, align: bool) -> *mut ::core::ffi::c_void;
    fn rpc_start(channel: *mut Channel);
    fn rpc_close(channel: *mut Channel);
    fn rpc_free(channel: *mut Channel);
    fn terminal_alloc(buf: *mut buf_T, opts: TerminalOptions) -> *mut Terminal;
    fn terminal_close(termpp: *mut *mut Terminal, status: ::core::ffi::c_int);
    fn terminal_set_state(term: *mut Terminal, suspended: bool);
    fn terminal_destroy(termpp: *mut *mut Terminal);
    fn terminal_receive(term: *mut Terminal, data: *const ::core::ffi::c_char, len: size_t);
    fn terminal_buf(term: *const Terminal) -> Buffer;
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
pub const kStlClickFuncRun: C2Rust_Unnamed_12 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_12 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_12 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_12 = 0;
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
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
pub const UV_OVERLAPPED_PIPE: uv_stdio_flags = 64;
pub const UV_NONBLOCK_PIPE: uv_stdio_flags = 64;
pub const UV_WRITABLE_PIPE: uv_stdio_flags = 32;
pub const UV_READABLE_PIPE: uv_stdio_flags = 16;
pub const UV_INHERIT_STREAM: uv_stdio_flags = 4;
pub const UV_INHERIT_FD: uv_stdio_flags = 2;
pub const UV_CREATE_PIPE: uv_stdio_flags = 1;
pub const UV_IGNORE: uv_stdio_flags = 0;
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
pub const kChannelStreamInternal: ChannelStreamType = 4;
pub const kChannelStreamStderr: ChannelStreamType = 3;
pub const kChannelStreamStdio: ChannelStreamType = 2;
pub const kChannelStreamSocket: ChannelStreamType = 1;
pub const kChannelStreamProc: ChannelStreamType = 0;
pub const kChannelPartAll: ChannelPart = 4;
pub const kChannelPartRpc: ChannelPart = 3;
pub const kChannelPartStderr: ChannelPart = 2;
pub const kChannelPartStdout: ChannelPart = 1;
pub const kChannelPartStdin: ChannelPart = 0;
pub const kChannelStdinNull: ChannelStdinMode = 1;
pub const kChannelStdinPipe: ChannelStdinMode = 0;
pub const kClientTypePlugin: ClientType = 4;
pub const kClientTypeHost: ClientType = 3;
pub const kClientTypeEmbedder: ClientType = 2;
pub const kClientTypeUi: ClientType = 1;
pub const kClientTypeMsgpackRpc: ClientType = 5;
pub const kClientTypeRemote: ClientType = 0;
pub const kClientTypeUnknown: ClientType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_34 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut int64_t,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const F_DUPFD_CLOEXEC: ::core::ffi::c_int = 1030 as ::core::ffi::c_int;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const LOGLVL_INF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_put_uint64_t_ptr_t(
    mut map: *mut Map_uint64_t_ptr_t,
    mut key: uint64_t,
    mut value: ptr_t,
) {
    let mut val: *mut ptr_t = map_put_ref_uint64_t_ptr_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut uint64_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
#[inline]
unsafe extern "C" fn map_get_uint64_t_ptr_t(
    mut map: *mut Map_uint64_t_ptr_t,
    mut key: uint64_t,
) -> ptr_t {
    let mut k: uint32_t = mh_get_uint64_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
pub const CHAN_STDIO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CHAN_STDERR: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn callback_reader_set(mut reader: CallbackReader) -> bool {
    return reader.cb.type_0 as ::core::ffi::c_uint
        != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
        || !reader.self_0.is_null();
}
#[inline]
unsafe extern "C" fn find_channel(mut id: uint64_t) -> *mut Channel {
    return map_get_uint64_t_ptr_t(channels.ptr(), id) as *mut Channel;
}
#[inline]
unsafe extern "C" fn channel_instream(mut chan: *mut Channel) -> *mut Stream {
    match (*chan).streamtype as ::core::ffi::c_uint {
        0 => return &raw mut (*chan).stream.proc.in_0,
        1 => return &raw mut (*chan).stream.socket.s,
        2 => return &raw mut (*chan).stream.stdio.out,
        4 | 3 => {
            abort();
        }
        _ => {}
    }
    abort();
}
static did_stdio: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static next_chan_id: GlobalCell<uint64_t> =
    GlobalCell::new((CHAN_STDERR + 1 as ::core::ffi::c_int) as uint64_t);
pub unsafe extern "C" fn channel_teardown() {
    let mut chan: *mut Channel = ::core::ptr::null_mut::<Channel>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*channels.ptr()).set.h.n_keys {
        chan = *(*channels.ptr()).values.offset(__i as isize) as *mut Channel;
        channel_close(
            (*chan).id,
            kChannelPartAll,
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
        );
        __i = __i.wrapping_add(1);
    }
}
pub unsafe extern "C" fn channel_close(
    mut id: uint64_t,
    mut part: ChannelPart,
    mut error: *mut *const ::core::ffi::c_char,
) -> bool {
    let mut chan: *mut Channel = ::core::ptr::null_mut::<Channel>();
    let mut proc: *mut Proc = ::core::ptr::null_mut::<Proc>();
    let mut dummy: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if error.is_null() {
        error = &raw mut dummy;
    }
    chan = find_channel(id);
    if chan.is_null() {
        if id < next_chan_id.get() {
            return true_0 != 0;
        }
        *error = &raw const e_invchan as *const ::core::ffi::c_char;
        return false_0 != 0;
    }
    let mut close_main: bool = false_0 != 0;
    if part as ::core::ffi::c_uint == kChannelPartRpc as ::core::ffi::c_int as ::core::ffi::c_uint
        || part as ::core::ffi::c_uint
            == kChannelPartAll as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        close_main = true_0 != 0;
        if (*chan).is_rpc {
            rpc_close(chan);
        } else if part as ::core::ffi::c_uint
            == kChannelPartRpc as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            *error = &raw const e_invstream as *const ::core::ffi::c_char;
            return false_0 != 0;
        }
    } else if (part as ::core::ffi::c_uint
        == kChannelPartStdin as ::core::ffi::c_int as ::core::ffi::c_uint
        || part as ::core::ffi::c_uint
            == kChannelPartStdout as ::core::ffi::c_int as ::core::ffi::c_uint)
        && (*chan).is_rpc as ::core::ffi::c_int != 0
    {
        *error = &raw const e_invstreamrpc as *const ::core::ffi::c_char;
        return false_0 != 0;
    }
    match (*chan).streamtype as ::core::ffi::c_uint {
        1 => {
            if !close_main {
                *error = &raw const e_invstream as *const ::core::ffi::c_char;
                return false_0 != 0;
            }
            rstream_may_close(&raw mut (*chan).stream.socket);
        }
        0 => {
            proc = &raw mut (*chan).stream.proc;
            if part as ::core::ffi::c_uint
                == kChannelPartStdin as ::core::ffi::c_int as ::core::ffi::c_uint
                || close_main as ::core::ffi::c_int != 0
            {
                stream_may_close(&raw mut (*proc).in_0);
            }
            if part as ::core::ffi::c_uint
                == kChannelPartStdout as ::core::ffi::c_int as ::core::ffi::c_uint
                || close_main as ::core::ffi::c_int != 0
            {
                rstream_may_close(&raw mut (*proc).out);
            }
            if part as ::core::ffi::c_uint
                == kChannelPartStderr as ::core::ffi::c_int as ::core::ffi::c_uint
                || part as ::core::ffi::c_uint
                    == kChannelPartAll as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                rstream_may_close(&raw mut (*proc).err);
            }
            if (*proc).type_0 as ::core::ffi::c_uint
                == kProcTypePty as ::core::ffi::c_int as ::core::ffi::c_uint
                && part as ::core::ffi::c_uint
                    == kChannelPartAll as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                pty_proc_close_master(&raw mut (*chan).stream.pty);
            }
        }
        2 => {
            if part as ::core::ffi::c_uint
                == kChannelPartStdin as ::core::ffi::c_int as ::core::ffi::c_uint
                || close_main as ::core::ffi::c_int != 0
            {
                rstream_may_close(&raw mut (*chan).stream.stdio.in_0);
            }
            if part as ::core::ffi::c_uint
                == kChannelPartStdout as ::core::ffi::c_int as ::core::ffi::c_uint
                || close_main as ::core::ffi::c_int != 0
            {
                stream_may_close(&raw mut (*chan).stream.stdio.out);
            }
            if part as ::core::ffi::c_uint
                == kChannelPartStderr as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                *error = &raw const e_invstream as *const ::core::ffi::c_char;
                return false_0 != 0;
            }
        }
        3 => {
            if part as ::core::ffi::c_uint
                != kChannelPartAll as ::core::ffi::c_int as ::core::ffi::c_uint
                && part as ::core::ffi::c_uint
                    != kChannelPartStderr as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                *error = &raw const e_invstream as *const ::core::ffi::c_char;
                return false_0 != 0;
            }
            if !(*chan).stream.err.closed {
                (*chan).stream.err.closed = true_0 != 0;
                if !exiting.get() {
                    freopen(
                        b"/dev/null\0".as_ptr() as *const ::core::ffi::c_char,
                        b"w\0".as_ptr() as *const ::core::ffi::c_char,
                        stderr,
                    );
                }
                channel_decref(chan);
            }
        }
        4 => {
            if !close_main {
                *error = &raw const e_invstream as *const ::core::ffi::c_char;
                return false_0 != 0;
            }
            if !(*chan).term.is_null() {
                api_free_luaref((*chan).stream.internal.cb);
                (*chan).stream.internal.cb = LUA_NOREF as LuaRef;
                (*chan).stream.internal.closed = true_0 != 0;
                terminal_close(&raw mut (*chan).term, 0 as ::core::ffi::c_int);
                (*chan).exit_status = 0 as ::core::ffi::c_int;
            } else {
                channel_decref(chan);
            }
        }
        _ => {}
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn channel_init() {
    channel_alloc(kChannelStreamStderr);
    rpc_init();
}
#[no_mangle]
pub unsafe extern "C" fn channel_alloc(mut type_0: ChannelStreamType) -> *mut Channel {
    let mut chan: *mut Channel =
        xcalloc(1 as size_t, ::core::mem::size_of::<Channel>()) as *mut Channel;
    if type_0 as ::core::ffi::c_uint
        == kChannelStreamStdio as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*chan).id = CHAN_STDIO as uint64_t;
    } else if type_0 as ::core::ffi::c_uint
        == kChannelStreamStderr as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*chan).id = CHAN_STDERR as uint64_t;
    } else {
        let c2rust_fresh0 = next_chan_id.get();
        next_chan_id.set((*next_chan_id.ptr()).wrapping_add(1));
        (*chan).id = c2rust_fresh0;
    }
    (*chan).events = multiqueue_new_child((*main_loop.ptr()).events);
    (*chan).refcount = 1 as size_t;
    (*chan).exit_status = -1 as ::core::ffi::c_int;
    (*chan).streamtype = type_0;
    (*chan).detach = false_0 != 0;
    '_c2rust_label: {
        if (*chan).id <= 9223372036854775807 as uint64_t {
        } else {
            __assert_fail(
                b"chan->id <= VARNUMBER_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/channel.rs\0".as_ptr() as *const ::core::ffi::c_char,
                230 as ::core::ffi::c_uint,
                b"Channel *channel_alloc(ChannelStreamType)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    map_put_uint64_t_ptr_t(channels.ptr(), (*chan).id, chan as ptr_t);
    return chan;
}
pub unsafe extern "C" fn channel_create_event(
    mut chan: *mut Channel,
    mut ext_source: *const ::core::ffi::c_char,
) {
    let mut source: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !ext_source.is_null() {
        source = ext_source;
    } else {
        eval_fmt_source_name_line(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
        );
        source = IObuff.ptr() as *mut ::core::ffi::c_char;
    }
    '_c2rust_label: {
        if (*chan).id <= 9223372036854775807 as uint64_t {
        } else {
            __assert_fail(
                b"chan->id <= VARNUMBER_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/channel.rs\0".as_ptr() as *const ::core::ffi::c_char,
                249 as ::core::ffi::c_uint,
                b"void channel_create_event(Channel *, const char *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut info: Dict = channel_info((*chan).id, &raw mut arena);
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    object_to_vim(
        object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: info },
        },
        &raw mut tv,
        ::core::ptr::null_mut::<Error>(),
    );
    '_c2rust_label_0: {
        if tv.v_type as ::core::ffi::c_uint == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"tv.v_type == VAR_DICT\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/channel.rs\0".as_ptr() as *const ::core::ffi::c_char,
                256 as ::core::ffi::c_uint,
                b"void channel_create_event(Channel *, const char *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut str: *mut ::core::ffi::c_char =
        encode_tv2json(&raw mut tv, ::core::ptr::null_mut::<size_t>());
    logmsg(
        LOGLVL_INF,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"channel_create_event\0".as_ptr() as *const ::core::ffi::c_char,
        258 as ::core::ffi::c_int,
        true_0 != 0,
        b"new channel %lu (%s) : %s\0".as_ptr() as *const ::core::ffi::c_char,
        (*chan).id,
        source,
        str,
    );
    xfree(str as *mut ::core::ffi::c_void);
    arena_mem_free(arena_finish(&raw mut arena));
    channel_info_changed(chan, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn channel_incref(mut chan: *mut Channel) {
    (*chan).refcount = (*chan).refcount.wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn channel_decref(mut chan: *mut Channel) {
    (*chan).refcount = (*chan).refcount.wrapping_sub(1);
    if (*chan).refcount == 0 {
        multiqueue_put_event(
            (*main_loop.ptr()).events,
            Event {
                handler: Some(
                    free_channel_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    chan as *mut ::core::ffi::c_void,
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
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
    }
}
pub unsafe extern "C" fn callback_reader_free(mut reader: *mut CallbackReader) {
    callback_free(&raw mut (*reader).cb);
    ga_clear(&raw mut (*reader).buffer);
}
pub unsafe extern "C" fn callback_reader_start(
    mut reader: *mut CallbackReader,
    mut type_0: *const ::core::ffi::c_char,
) {
    ga_init(
        &raw mut (*reader).buffer,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        32 as ::core::ffi::c_int,
    );
    (*reader).type_0 = type_0;
}
unsafe extern "C" fn channel_destroy(mut chan: *mut Channel) {
    if (*chan).is_rpc {
        rpc_free(chan);
    }
    if (*chan).streamtype as ::core::ffi::c_uint
        == kChannelStreamProc as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        proc_free(&raw mut (*chan).stream.proc);
    }
    callback_reader_free(&raw mut (*chan).on_data);
    callback_reader_free(&raw mut (*chan).on_stderr);
    callback_free(&raw mut (*chan).on_exit);
    multiqueue_free((*chan).events);
    xfree(chan as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn free_channel_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut chan: *mut Channel = *argv.offset(0 as ::core::ffi::c_int as isize) as *mut Channel;
    map_del_uint64_t_ptr_t(
        channels.ptr(),
        (*chan).id,
        ::core::ptr::null_mut::<uint64_t>(),
    );
    channel_destroy(chan);
}
unsafe extern "C" fn channel_destroy_early(mut chan: *mut Channel) {
    next_chan_id.set((*next_chan_id.ptr()).wrapping_sub(1));
    if (*chan).id != next_chan_id.get() {
        abort();
    }
    map_del_uint64_t_ptr_t(
        channels.ptr(),
        (*chan).id,
        ::core::ptr::null_mut::<uint64_t>(),
    );
    (*chan).id = 0 as uint64_t;
    (*chan).refcount = (*chan).refcount.wrapping_sub(1);
    if (*chan).refcount != 0 as size_t {
        abort();
    }
    multiqueue_put_event(
        (*main_loop.ptr()).events,
        Event {
            handler: Some(
                free_channel_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                chan as *mut ::core::ffi::c_void,
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
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
}
unsafe extern "C" fn close_cb(mut _stream: *mut Stream, mut data: *mut ::core::ffi::c_void) {
    channel_decref(data as *mut Channel);
}
pub unsafe extern "C" fn channel_job_start(
    mut argv: *mut *mut ::core::ffi::c_char,
    mut exepath: *const ::core::ffi::c_char,
    mut on_stdout: CallbackReader,
    mut on_stderr: CallbackReader,
    mut on_exit: Callback,
    mut pty: bool,
    mut rpc: bool,
    mut overlapped: bool,
    mut detach: bool,
    mut stdin_mode: ChannelStdinMode,
    mut cwd: *const ::core::ffi::c_char,
    mut pty_width: uint16_t,
    mut pty_height: uint16_t,
    mut env: *mut dict_T,
    mut status_out: *mut varnumber_T,
) -> *mut Channel {
    let mut chan: *mut Channel = channel_alloc(kChannelStreamProc);
    (*chan).on_data = on_stdout;
    (*chan).on_stderr = on_stderr;
    (*chan).on_exit = on_exit;
    if pty {
        if detach {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                b"terminal/pty job cannot be detached\0".as_ptr() as *const ::core::ffi::c_char,
            );
            shell_free_argv(argv);
            if !env.is_null() {
                tv_dict_free(env);
            }
            channel_destroy_early(chan);
            *status_out = 0 as varnumber_T;
            return ::core::ptr::null_mut::<Channel>();
        }
        (*chan).stream.pty = pty_proc_init(main_loop.ptr(), chan as *mut ::core::ffi::c_void);
        if pty_width as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
            (*chan).stream.pty.width = pty_width;
        }
        if pty_height as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
            (*chan).stream.pty.height = pty_height;
        }
    } else {
        (*chan).stream.uv = libuv_proc_init(main_loop.ptr(), chan as *mut ::core::ffi::c_void);
    }
    let mut proc: *mut Proc = &raw mut (*chan).stream.proc;
    (*proc).argv = argv;
    (*proc).exepath = exepath;
    (*proc).cb = Some(
        channel_proc_exit_cb
            as unsafe extern "C" fn(*mut Proc, ::core::ffi::c_int, *mut ::core::ffi::c_void) -> (),
    ) as proc_exit_cb;
    (*proc).state_cb = Some(
        channel_proc_state_cb
            as unsafe extern "C" fn(*mut Proc, bool, *mut ::core::ffi::c_void) -> (),
    ) as proc_state_cb;
    (*proc).events = (*chan).events;
    (*proc).detach = detach;
    (*proc).cwd = cwd;
    (*proc).env = env;
    (*proc).overlapped = overlapped;
    let mut cmd: *mut ::core::ffi::c_char = xstrdup(proc_get_exepath(proc));
    let mut has_out: bool = false;
    let mut has_err: bool = false;
    if (*proc).type_0 as ::core::ffi::c_uint
        == kProcTypePty as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        has_out = true_0 != 0;
        has_err = false_0 != 0;
    } else {
        has_out = rpc as ::core::ffi::c_int != 0
            || callback_reader_set((*chan).on_data) as ::core::ffi::c_int != 0;
        has_err = callback_reader_set((*chan).on_stderr);
        (*proc).fwd_err = (*chan).on_stderr.fwd_err;
    }
    let mut has_in: bool = stdin_mode as ::core::ffi::c_uint
        == kChannelStdinPipe as ::core::ffi::c_int as ::core::ffi::c_uint;
    let mut status: ::core::ffi::c_int = proc_spawn(proc, has_in, has_out, has_err);
    if status != 0 {
        semsg(
            gettext(&raw const e_jobspawn as *const ::core::ffi::c_char),
            uv_strerror(status),
            cmd,
        );
        xfree(cmd as *mut ::core::ffi::c_void);
        if !(*proc).env.is_null() {
            tv_dict_free((*proc).env);
        }
        channel_destroy_early(chan);
        *status_out = (*proc).status as varnumber_T;
        return ::core::ptr::null_mut::<Channel>();
    }
    xfree(cmd as *mut ::core::ffi::c_void);
    if !(*proc).env.is_null() {
        tv_dict_free((*proc).env);
    }
    if has_in {
        wstream_init(&raw mut (*proc).in_0, 0 as size_t);
    }
    if has_out {
        rstream_init(&raw mut (*proc).out);
    }
    if rpc {
        rpc_start(chan);
    } else if has_out {
        callback_reader_start(
            &raw mut (*chan).on_data,
            b"stdout\0".as_ptr() as *const ::core::ffi::c_char,
        );
        rstream_start(
            &raw mut (*proc).out,
            Some(
                on_channel_data
                    as unsafe extern "C" fn(
                        *mut RStream,
                        *const ::core::ffi::c_char,
                        size_t,
                        *mut ::core::ffi::c_void,
                        bool,
                    ) -> size_t,
            ),
            chan as *mut ::core::ffi::c_void,
        );
    }
    if has_err {
        callback_reader_start(
            &raw mut (*chan).on_stderr,
            b"stderr\0".as_ptr() as *const ::core::ffi::c_char,
        );
        rstream_init(&raw mut (*proc).err);
        rstream_start(
            &raw mut (*proc).err,
            Some(
                on_job_stderr
                    as unsafe extern "C" fn(
                        *mut RStream,
                        *const ::core::ffi::c_char,
                        size_t,
                        *mut ::core::ffi::c_void,
                        bool,
                    ) -> size_t,
            ),
            chan as *mut ::core::ffi::c_void,
        );
    }
    *status_out = (*chan).id as varnumber_T;
    return chan;
}
pub unsafe extern "C" fn channel_connect(
    mut tcp: bool,
    mut address: *const ::core::ffi::c_char,
    mut rpc: bool,
    mut on_output: CallbackReader,
    mut timeout: ::core::ffi::c_int,
    mut error: *mut *const ::core::ffi::c_char,
) -> uint64_t {
    let mut channel: *mut Channel = ::core::ptr::null_mut::<Channel>();
    '_end: {
        if !tcp && rpc as ::core::ffi::c_int != 0 {
            if server_owns_pipe_address(address) {
                channel = channel_alloc(kChannelStreamInternal);
                (*channel).stream.internal.cb = LUA_NOREF as LuaRef;
                rpc_start(channel);
                break '_end;
            }
        }
        channel = channel_alloc(kChannelStreamSocket);
        if !socket_connect(
            main_loop.ptr(),
            &raw mut (*channel).stream.socket,
            tcp,
            address,
            timeout,
            error,
        ) {
            channel_decref(channel);
            return 0 as uint64_t;
        }
        (*channel).stream.socket.s.internal_close_cb =
            Some(close_cb as unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> ())
                as stream_close_cb;
        (*channel).stream.socket.s.internal_data = channel as *mut ::core::ffi::c_void;
        wstream_init(&raw mut (*channel).stream.socket.s, 0 as size_t);
        rstream_init(&raw mut (*channel).stream.socket);
        if rpc {
            rpc_start(channel);
        } else {
            (*channel).on_data = on_output;
            callback_reader_start(
                &raw mut (*channel).on_data,
                b"data\0".as_ptr() as *const ::core::ffi::c_char,
            );
            rstream_start(
                &raw mut (*channel).stream.socket,
                Some(
                    on_channel_data
                        as unsafe extern "C" fn(
                            *mut RStream,
                            *const ::core::ffi::c_char,
                            size_t,
                            *mut ::core::ffi::c_void,
                            bool,
                        ) -> size_t,
                ),
                channel as *mut ::core::ffi::c_void,
            );
        }
    }
    channel_create_event(channel, address);
    return (*channel).id;
}
pub unsafe extern "C" fn channel_from_connection(mut watcher: *mut SocketWatcher) {
    let mut channel: *mut Channel = channel_alloc(kChannelStreamSocket);
    socket_watcher_accept(watcher, &raw mut (*channel).stream.socket);
    (*channel).stream.socket.s.internal_close_cb =
        Some(close_cb as unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> ())
            as stream_close_cb;
    (*channel).stream.socket.s.internal_data = channel as *mut ::core::ffi::c_void;
    wstream_init(&raw mut (*channel).stream.socket.s, 0 as size_t);
    rstream_init(&raw mut (*channel).stream.socket);
    rpc_start(channel);
    channel_create_event(
        channel,
        &raw mut (*watcher).addr as *mut ::core::ffi::c_char,
    );
}
pub unsafe extern "C" fn channel_from_stdio(
    mut rpc: bool,
    mut on_output: CallbackReader,
    mut error: *mut *const ::core::ffi::c_char,
) -> uint64_t {
    if !headless_mode.get() && !embedded_mode.get() {
        *error = gettext(
            b"can only be opened in headless mode\0".as_ptr() as *const ::core::ffi::c_char
        );
        return 0 as uint64_t;
    }
    if did_stdio.get() {
        *error = gettext(b"channel was already open\0".as_ptr() as *const ::core::ffi::c_char);
        return 0 as uint64_t;
    }
    did_stdio.set(true_0 != 0);
    let mut channel: *mut Channel = channel_alloc(kChannelStreamStdio);
    let mut stdin_dup_fd: ::core::ffi::c_int = STDIN_FILENO;
    let mut stdout_dup_fd: ::core::ffi::c_int = STDOUT_FILENO;
    if embedded_mode.get() {
        stdin_dup_fd = fcntl(
            STDIN_FILENO,
            F_DUPFD_CLOEXEC,
            STDERR_FILENO + 1 as ::core::ffi::c_int,
        );
        stdout_dup_fd = fcntl(
            STDOUT_FILENO,
            F_DUPFD_CLOEXEC,
            STDERR_FILENO + 1 as ::core::ffi::c_int,
        );
        dup2(STDERR_FILENO, STDOUT_FILENO);
        dup2(STDERR_FILENO, STDIN_FILENO);
    }
    rstream_init_fd(
        main_loop.ptr(),
        &raw mut (*channel).stream.stdio.in_0,
        stdin_dup_fd,
    );
    wstream_init_fd(
        main_loop.ptr(),
        &raw mut (*channel).stream.stdio.out,
        stdout_dup_fd,
        0 as size_t,
    );
    if rpc {
        rpc_start(channel);
    } else {
        (*channel).on_data = on_output;
        callback_reader_start(
            &raw mut (*channel).on_data,
            b"stdin\0".as_ptr() as *const ::core::ffi::c_char,
        );
        rstream_start(
            &raw mut (*channel).stream.stdio.in_0,
            Some(
                on_channel_data
                    as unsafe extern "C" fn(
                        *mut RStream,
                        *const ::core::ffi::c_char,
                        size_t,
                        *mut ::core::ffi::c_void,
                        bool,
                    ) -> size_t,
            ),
            channel as *mut ::core::ffi::c_void,
        );
    }
    return (*channel).id;
}
pub unsafe extern "C" fn channel_send(
    mut id: uint64_t,
    mut data: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut data_owned: bool,
    mut error: *mut *const ::core::ffi::c_char,
) -> size_t {
    let mut in_0: *mut Stream = ::core::ptr::null_mut::<Stream>();
    let mut buf: *mut WBuffer = ::core::ptr::null_mut::<WBuffer>();
    let mut chan: *mut Channel = find_channel(id);
    let mut written: size_t = 0 as size_t;
    if chan.is_null() {
        *error = gettext(&raw const e_invchan as *const ::core::ffi::c_char);
    } else if (*chan).streamtype as ::core::ffi::c_uint
        == kChannelStreamStderr as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*chan).stream.err.closed {
            *error = gettext(
                b"Can't send data to closed stream\0".as_ptr() as *const ::core::ffi::c_char
            );
        } else {
            let mut wres: ptrdiff_t = os_write(STDERR_FILENO, data, len, false_0 != 0);
            if wres >= 0 as ptrdiff_t {
                written = wres as size_t;
            }
        }
    } else if (*chan).streamtype as ::core::ffi::c_uint
        == kChannelStreamInternal as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*chan).is_rpc {
            *error = gettext(
                b"Can't send raw data to rpc channel\0".as_ptr() as *const ::core::ffi::c_char
            );
        } else if (*chan).term.is_null()
            || (*chan).stream.internal.closed as ::core::ffi::c_int != 0
        {
            *error = gettext(
                b"Can't send data to closed stream\0".as_ptr() as *const ::core::ffi::c_char
            );
        } else {
            terminal_receive((*chan).term, data, len);
            written = len;
        }
    } else {
        in_0 = channel_instream(chan);
        if (*in_0).closed {
            *error = gettext(
                b"Can't send data to closed stream\0".as_ptr() as *const ::core::ffi::c_char
            );
        } else if (*chan).is_rpc {
            *error = gettext(
                b"Can't send raw data to rpc channel\0".as_ptr() as *const ::core::ffi::c_char
            );
        } else {
            buf = wstream_new_buffer(
                (if data_owned as ::core::ffi::c_int != 0 {
                    data as *mut ::core::ffi::c_void
                } else {
                    xmemdup(data as *const ::core::ffi::c_void, len)
                }) as *mut ::core::ffi::c_char,
                len,
                1 as size_t,
                Some(xfree as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
            );
            return if wstream_write(in_0, buf) == 0 as ::core::ffi::c_int {
                len
            } else {
                0 as size_t
            };
        }
    }
    if data_owned {
        xfree(data as *mut ::core::ffi::c_void);
    }
    return written;
}
#[inline(always)]
unsafe extern "C" fn buffer_to_tv_list(
    buf: *const ::core::ffi::c_char,
    len: size_t,
) -> *mut list_T {
    let l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    tv_list_append_string(
        l,
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        0 as ssize_t,
    );
    if len > 0 as size_t {
        encode_list_write(l as *mut ::core::ffi::c_void, buf, len);
    }
    return l;
}
pub unsafe extern "C" fn on_channel_data(
    mut stream: *mut RStream,
    mut buf: *const ::core::ffi::c_char,
    mut count: size_t,
    mut data: *mut ::core::ffi::c_void,
    mut eof: bool,
) -> size_t {
    let mut chan: *mut Channel = data as *mut Channel;
    return on_channel_output(stream, chan, buf, count, eof, &raw mut (*chan).on_data);
}
pub unsafe extern "C" fn on_job_stderr(
    mut stream: *mut RStream,
    mut buf: *const ::core::ffi::c_char,
    mut count: size_t,
    mut data: *mut ::core::ffi::c_void,
    mut eof: bool,
) -> size_t {
    let mut chan: *mut Channel = data as *mut Channel;
    return on_channel_output(stream, chan, buf, count, eof, &raw mut (*chan).on_stderr);
}
unsafe extern "C" fn on_channel_output(
    mut _stream: *mut RStream,
    mut chan: *mut Channel,
    mut buf: *const ::core::ffi::c_char,
    mut count: size_t,
    mut eof: bool,
    mut reader: *mut CallbackReader,
) -> size_t {
    if !(*chan).term.is_null() {
        terminal_receive((*chan).term, buf, count);
    }
    if eof {
        (*reader).eof = true_0 != 0;
    }
    if callback_reader_set(*reader) {
        ga_concat_len(&raw mut (*reader).buffer, buf, count);
        schedule_channel_event(chan);
    }
    return count;
}
unsafe extern "C" fn schedule_channel_event(mut chan: *mut Channel) {
    if !(*chan).callback_scheduled {
        if !(*chan).callback_busy {
            multiqueue_put_event(
                (*chan).events,
                Event {
                    handler: Some(
                        on_channel_event
                            as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                    ),
                    argv: [
                        chan as *mut ::core::ffi::c_void,
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
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
            channel_incref(chan);
        }
        (*chan).callback_scheduled = true_0 != 0;
    }
}
unsafe extern "C" fn on_channel_event(mut args: *mut *mut ::core::ffi::c_void) {
    let mut chan: *mut Channel = *args.offset(0 as ::core::ffi::c_int as isize) as *mut Channel;
    (*chan).callback_busy = true_0 != 0;
    (*chan).callback_scheduled = false_0 != 0;
    let mut exit_status: ::core::ffi::c_int = (*chan).exit_status;
    channel_reader_callbacks(chan, &raw mut (*chan).on_data);
    channel_reader_callbacks(chan, &raw mut (*chan).on_stderr);
    if exit_status > -1 as ::core::ffi::c_int {
        channel_callback_call(chan, ::core::ptr::null_mut::<CallbackReader>());
        (*chan).exit_status = -1 as ::core::ffi::c_int;
    }
    (*chan).callback_busy = false_0 != 0;
    if (*chan).callback_scheduled {
        multiqueue_put_event(
            (*chan).events,
            Event {
                handler: Some(
                    on_channel_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    chan as *mut ::core::ffi::c_void,
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
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
        channel_incref(chan);
    }
    channel_decref(chan);
}
pub unsafe extern "C" fn channel_reader_callbacks(
    mut chan: *mut Channel,
    mut reader: *mut CallbackReader,
) {
    if (*reader).buffered {
        if (*reader).eof {
            if !(*reader).self_0.is_null() {
                if tv_dict_find((*reader).self_0, (*reader).type_0, -1 as ptrdiff_t).is_null() {
                    let mut data: *mut list_T = buffer_to_tv_list(
                        (*reader).buffer.ga_data as *const ::core::ffi::c_char,
                        (*reader).buffer.ga_len as size_t,
                    );
                    tv_dict_add_list(
                        (*reader).self_0,
                        (*reader).type_0,
                        strlen((*reader).type_0),
                        data,
                    );
                } else {
                    semsg(
                        gettext(&raw const e_streamkey as *const ::core::ffi::c_char),
                        (*reader).type_0,
                        (*chan).id,
                    );
                }
            } else {
                channel_callback_call(chan, reader);
            }
            (*reader).eof = false_0 != 0;
        }
    } else {
        let mut is_eof: bool = (*reader).eof;
        if (*reader).buffer.ga_len > 0 as ::core::ffi::c_int {
            channel_callback_call(chan, reader);
        }
        if is_eof {
            channel_callback_call(chan, reader);
            (*reader).eof = false_0 != 0;
        }
    };
}
unsafe extern "C" fn channel_proc_exit_cb(
    mut _proc: *mut Proc,
    mut status: ::core::ffi::c_int,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut chan: *mut Channel = data as *mut Channel;
    if !(*chan).term.is_null() {
        terminal_close(&raw mut (*chan).term, status);
    }
    if !exiting.get() && ui_client_channel_id.get() == (*chan).id {
        ui_client_attach_to_restarted_server();
        if ui_client_channel_id.get() == (*chan).id {
            exit_on_closed_chan(status);
        }
    }
    let mut exited: bool = status >= 0 as ::core::ffi::c_int;
    if exited as ::core::ffi::c_int != 0
        && (*chan).on_exit.type_0 as ::core::ffi::c_uint
            != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        schedule_channel_event(chan);
    }
    (*chan).exit_status = if exited as ::core::ffi::c_int != 0 {
        status
    } else {
        (*chan).exit_status
    };
    channel_decref(chan);
}
unsafe extern "C" fn channel_proc_state_cb(
    mut _proc: *mut Proc,
    mut suspended: bool,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut chan: *mut Channel = data as *mut Channel;
    if !(*chan).term.is_null() {
        terminal_set_state((*chan).term, suspended);
    }
}
unsafe extern "C" fn channel_callback_call(
    mut chan: *mut Channel,
    mut reader: *mut CallbackReader,
) {
    let mut cb: *mut Callback = ::core::ptr::null_mut::<Callback>();
    let mut argv: [typval_T; 4] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 4];
    argv[0 as ::core::ffi::c_int as usize].v_type = VAR_NUMBER;
    argv[0 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
    argv[0 as ::core::ffi::c_int as usize].vval.v_number = (*chan).id as varnumber_T;
    if !reader.is_null() {
        argv[1 as ::core::ffi::c_int as usize].v_type = VAR_LIST;
        argv[1 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
        argv[1 as ::core::ffi::c_int as usize].vval.v_list = buffer_to_tv_list(
            (*reader).buffer.ga_data as *const ::core::ffi::c_char,
            (*reader).buffer.ga_len as size_t,
        );
        tv_list_ref(argv[1 as ::core::ffi::c_int as usize].vval.v_list);
        ga_clear(&raw mut (*reader).buffer);
        cb = &raw mut (*reader).cb;
        argv[2 as ::core::ffi::c_int as usize].vval.v_string =
            (*reader).type_0 as *mut ::core::ffi::c_char;
    } else {
        argv[1 as ::core::ffi::c_int as usize].v_type = VAR_NUMBER;
        argv[1 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
        argv[1 as ::core::ffi::c_int as usize].vval.v_number = (*chan).exit_status as varnumber_T;
        cb = &raw mut (*chan).on_exit;
        argv[2 as ::core::ffi::c_int as usize].vval.v_string =
            b"exit\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    argv[2 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
    argv[2 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    callback_call(
        cb,
        3 as ::core::ffi::c_int,
        &raw mut argv as *mut typval_T,
        &raw mut rettv,
    );
    tv_clear(&raw mut rettv);
    if !reader.is_null() {
        tv_list_unref(argv[1 as ::core::ffi::c_int as usize].vval.v_list);
    }
}
pub unsafe extern "C" fn channel_terminal_alloc(mut buf: *mut buf_T, mut chan: *mut Channel) {
    let mut topts: TerminalOptions = TerminalOptions {
        data: chan as *mut ::core::ffi::c_void,
        width: (*chan).stream.pty.width,
        height: (*chan).stream.pty.height,
        read_pause_cb: Some(
            term_read_pause as unsafe extern "C" fn(bool, *mut ::core::ffi::c_void) -> (),
        ),
        write_cb: Some(
            term_write
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_char,
                    size_t,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        resize_cb: Some(
            term_resize as unsafe extern "C" fn(uint16_t, uint16_t, *mut ::core::ffi::c_void) -> (),
        ),
        resume_cb: Some(term_resume as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        close_cb: Some(term_close as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        force_crlf: false_0 != 0,
    };
    (*buf).b_p_channel = (*chan).id as OptInt;
    channel_incref(chan);
    (*chan).term = terminal_alloc(buf, topts);
}
unsafe extern "C" fn term_read_pause(mut pause: bool, mut data: *mut ::core::ffi::c_void) {
    let mut chan: *mut Channel = data as *mut Channel;
    if (*chan).stream.proc.out.s.closed {
        return;
    }
    if pause {
        rstream_stop_inner(&raw mut (*chan).stream.proc.out);
    } else {
        rstream_start_inner(&raw mut (*chan).stream.proc.out);
    };
}
unsafe extern "C" fn term_write(
    mut buf: *const ::core::ffi::c_char,
    mut size: size_t,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut chan: *mut Channel = data as *mut Channel;
    if (*chan).stream.proc.in_0.closed {
        logmsg(
            LOGLVL_INF,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"term_write\0".as_ptr() as *const ::core::ffi::c_char,
            918 as ::core::ffi::c_int,
            true_0 != 0,
            b"write failed: stream is closed\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut wbuf: *mut WBuffer = wstream_new_buffer(
        xmemdup(buf as *const ::core::ffi::c_void, size) as *mut ::core::ffi::c_char,
        size,
        1 as size_t,
        Some(xfree as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
    );
    wstream_write(&raw mut (*chan).stream.proc.in_0, wbuf);
}
unsafe extern "C" fn term_resize(
    mut width: uint16_t,
    mut height: uint16_t,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut chan: *mut Channel = data as *mut Channel;
    pty_proc_resize(&raw mut (*chan).stream.pty, width, height);
}
unsafe extern "C" fn term_resume(mut data: *mut ::core::ffi::c_void) {
    let mut chan: *mut Channel = data as *mut Channel;
    pty_proc_resume(&raw mut (*chan).stream.pty);
}
#[inline]
unsafe extern "C" fn term_delayed_free(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut chan: *mut Channel = *argv.offset(0 as ::core::ffi::c_int as isize) as *mut Channel;
    if (*chan).stream.proc.in_0.pending_reqs != 0 || (*chan).stream.proc.out.s.pending_reqs != 0 {
        multiqueue_put_event(
            (*chan).events,
            Event {
                handler: Some(
                    term_delayed_free as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    chan as *mut ::core::ffi::c_void,
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
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
        return;
    }
    if !(*chan).term.is_null() {
        terminal_destroy(&raw mut (*chan).term);
    }
    channel_decref(chan);
}
unsafe extern "C" fn term_close(mut data: *mut ::core::ffi::c_void) {
    let mut chan: *mut Channel = data as *mut Channel;
    proc_stop(&raw mut (*chan).stream.proc);
    multiqueue_put_event(
        (*chan).events,
        Event {
            handler: Some(
                term_delayed_free as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                data,
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
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
}
#[no_mangle]
pub unsafe extern "C" fn channel_info_changed(mut chan: *mut Channel, mut new_chan: bool) {
    let mut event: event_T = (if new_chan as ::core::ffi::c_int != 0 {
        EVENT_CHANOPEN as ::core::ffi::c_int
    } else {
        EVENT_CHANINFO as ::core::ffi::c_int
    }) as event_T;
    if has_event(event) {
        channel_incref(chan);
        multiqueue_put_event(
            (*main_loop.ptr()).events,
            Event {
                handler: Some(
                    set_info_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    chan as *mut ::core::ffi::c_void,
                    ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                        event as intptr_t as usize,
                    ),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
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
    }
}
unsafe extern "C" fn set_info_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut chan: *mut Channel = *argv.offset(0 as ::core::ffi::c_int as isize) as *mut Channel;
    let mut event: event_T =
        (*argv.offset(1 as ::core::ffi::c_int as isize)).expose_addr() as ptrdiff_t as event_T;
    let mut save_v_event: save_v_event_T = save_v_event_T {
        sve_did_save: false,
        sve_hashtab: hashtab_T {
            ht_mask: 0,
            ht_used: 0,
            ht_filled: 0,
            ht_changed: 0,
            ht_locked: 0,
            ht_array: ::core::ptr::null_mut::<hashitem_T>(),
            ht_smallarray: [hashitem_T {
                hi_hash: 0,
                hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            }; 16],
        },
    };
    let mut dict: *mut dict_T = get_v_event(&raw mut save_v_event);
    let mut arena: Arena = ARENA_EMPTY;
    let mut info: Dict = channel_info((*chan).id, &raw mut arena);
    let mut retval: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    object_to_vim(
        object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: info },
        },
        &raw mut retval,
        ::core::ptr::null_mut::<Error>(),
    );
    '_c2rust_label: {
        if retval.v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"retval.v_type == VAR_DICT\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/channel.rs\0".as_ptr() as *const ::core::ffi::c_char,
                978 as ::core::ffi::c_uint,
                b"void set_info_event(void **)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    tv_dict_add_dict(
        dict,
        b"info\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        retval.vval.v_dict,
    );
    tv_dict_set_keys_readonly(dict);
    apply_autocmds(
        event,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0 != 0,
        curbuf.get(),
    );
    restore_v_event(dict, &raw mut save_v_event);
    arena_mem_free(arena_finish(&raw mut arena));
    channel_decref(chan);
}
pub unsafe extern "C" fn channel_job_running(mut id: uint64_t) -> bool {
    let mut chan: *mut Channel = find_channel(id);
    return !chan.is_null()
        && (*chan).streamtype as ::core::ffi::c_uint
            == kChannelStreamProc as ::core::ffi::c_int as ::core::ffi::c_uint
        && !proc_is_stopped(&raw mut (*chan).stream.proc);
}
pub unsafe extern "C" fn channel_info(mut id: uint64_t, mut arena: *mut Arena) -> Dict {
    let mut chan: *mut Channel = find_channel(id);
    if chan.is_null() {
        return Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
    }
    let mut info: Dict = arena_dict(arena, 9 as size_t);
    let c2rust_fresh1 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh1 as isize) = key_value_pair {
        key: cstr_as_string(b"id\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*chan).id as Integer,
            },
        },
    };
    let mut stream_desc: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut mode_desc: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    's_131: {
        match (*chan).streamtype as ::core::ffi::c_uint {
            0 => {
                stream_desc = b"job\0".as_ptr() as *const ::core::ffi::c_char;
                if (*chan).stream.proc.type_0 as ::core::ffi::c_uint
                    == kProcTypePty as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut name: *const ::core::ffi::c_char =
                        pty_proc_tty_name(&raw mut (*chan).stream.pty);
                    let c2rust_fresh2 = info.size;
                    info.size = info.size.wrapping_add(1);
                    *info.items.offset(c2rust_fresh2 as isize) = key_value_pair {
                        key: cstr_as_string(b"pty\0".as_ptr() as *const ::core::ffi::c_char),
                        value: object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed {
                                string: arena_string(arena, cstr_as_string(name)),
                            },
                        },
                    };
                }
                let mut args: *mut *mut ::core::ffi::c_char = (*chan).stream.proc.argv;
                let mut argv: Array = Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
                if !args.is_null() {
                    let mut n: size_t = 0;
                    n = 0 as size_t;
                    while !(*args.offset(n as isize)).is_null() {
                        n = n.wrapping_add(1);
                    }
                    argv = arena_array(arena, n);
                    let mut i: size_t = 0 as size_t;
                    while i < n {
                        let c2rust_fresh3 = argv.size;
                        argv.size = argv.size.wrapping_add(1);
                        *argv.items.offset(c2rust_fresh3 as isize) = object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed {
                                string: cstr_as_string(*args.offset(i as isize)),
                            },
                        };
                        i = i.wrapping_add(1);
                    }
                }
                let c2rust_fresh4 = info.size;
                info.size = info.size.wrapping_add(1);
                *info.items.offset(c2rust_fresh4 as isize) = key_value_pair {
                    key: cstr_as_string(b"argv\0".as_ptr() as *const ::core::ffi::c_char),
                    value: object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed { array: argv },
                    },
                };
                break 's_131;
            }
            2 => {
                stream_desc = b"stdio\0".as_ptr() as *const ::core::ffi::c_char;
                break 's_131;
            }
            3 => {
                stream_desc = b"stderr\0".as_ptr() as *const ::core::ffi::c_char;
                break 's_131;
            }
            4 => {
                let c2rust_fresh5 = info.size;
                info.size = info.size.wrapping_add(1);
                *info.items.offset(c2rust_fresh5 as isize) = key_value_pair {
                    key: cstr_as_string(b"internal\0".as_ptr() as *const ::core::ffi::c_char),
                    value: object {
                        type_0: kObjectTypeBoolean,
                        data: C2Rust_Unnamed { boolean: true },
                    },
                };
            }
            1 => {}
            _ => {
                break 's_131;
            }
        }
        stream_desc = b"socket\0".as_ptr() as *const ::core::ffi::c_char;
    }
    let c2rust_fresh6 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh6 as isize) = key_value_pair {
        key: cstr_as_string(b"stream\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(stream_desc),
            },
        },
    };
    if (*chan).is_rpc {
        mode_desc = b"rpc\0".as_ptr() as *const ::core::ffi::c_char;
        let c2rust_fresh7 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh7 as isize) = key_value_pair {
            key: cstr_as_string(b"client\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed {
                    dict: (*chan).rpc.info,
                },
            },
        };
    } else if !(*chan).term.is_null() {
        mode_desc = b"terminal\0".as_ptr() as *const ::core::ffi::c_char;
        let c2rust_fresh8 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh8 as isize) = key_value_pair {
            key: cstr_as_string(b"buf\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBuffer,
                data: C2Rust_Unnamed {
                    integer: terminal_buf((*chan).term) as Integer,
                },
            },
        };
        let c2rust_fresh9 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh9 as isize) = key_value_pair {
            key: cstr_as_string(b"buffer\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBuffer,
                data: C2Rust_Unnamed {
                    integer: terminal_buf((*chan).term) as Integer,
                },
            },
        };
        let c2rust_fresh10 = info.size;
        info.size = info.size.wrapping_add(1);
        *info.items.offset(c2rust_fresh10 as isize) = key_value_pair {
            key: cstr_as_string(b"exitcode\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*chan).exit_status as Integer,
                },
            },
        };
    } else {
        mode_desc = b"bytes\0".as_ptr() as *const ::core::ffi::c_char;
    }
    let c2rust_fresh11 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh11 as isize) = key_value_pair {
        key: cstr_as_string(b"mode\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(mode_desc),
            },
        },
    };
    return info;
}
unsafe extern "C" fn int64_t_cmp(
    mut pa: *const ::core::ffi::c_void,
    mut pb: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let a: int64_t = *(pa as *const int64_t);
    let b: int64_t = *(pb as *const int64_t);
    return if a == b {
        0 as ::core::ffi::c_int
    } else if a > b {
        1 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
}
pub unsafe extern "C" fn channel_all_info(mut arena: *mut Arena) -> Array {
    let mut ids: C2Rust_Unnamed_34 = C2Rust_Unnamed_34 {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<int64_t>(),
    };
    ids.capacity = (*channels.ptr()).set.h.size as size_t;
    ids.items = arena_alloc(
        arena,
        ::core::mem::size_of::<int64_t>().wrapping_mul(ids.capacity),
        true_0 != 0,
    ) as *mut int64_t;
    let mut id: uint64_t = 0;
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*channels.ptr()).set.h.n_keys {
        id = *(*channels.ptr()).set.keys.offset(__i as isize);
        if ids.size == ids.capacity {
            ids.capacity = if ids.capacity != 0 {
                ids.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            ids.items = xrealloc(
                ids.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<int64_t>().wrapping_mul(ids.capacity),
            ) as *mut int64_t;
        } else {
        };
        let c2rust_fresh12 = ids.size;
        ids.size = ids.size.wrapping_add(1);
        *ids.items.offset(c2rust_fresh12 as isize) = id as int64_t;
        __i = __i.wrapping_add(1);
    }
    qsort(
        ids.items as *mut ::core::ffi::c_void,
        ids.size,
        ::core::mem::size_of::<int64_t>(),
        Some(
            int64_t_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    let mut ret: Array = arena_array(arena, ids.size);
    let mut i: size_t = 0 as size_t;
    while i < ids.size {
        let c2rust_fresh13 = ret.size;
        ret.size = ret.size.wrapping_add(1);
        *ret.items.offset(c2rust_fresh13 as isize) = object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed {
                dict: channel_info(*ids.items.offset(i as isize) as uint64_t, arena),
            },
        };
        i = i.wrapping_add(1);
    }
    return ret;
}
#[inline(always)]
unsafe extern "C" fn tv_list_ref(l: *mut list_T) {
    if l.is_null() {
        return;
    }
    (*l).lv_refcount += 1;
}
#[inline]
unsafe extern "C" fn proc_get_exepath(mut proc: *mut Proc) -> *const ::core::ffi::c_char {
    return if !(*proc).exepath.is_null() {
        (*proc).exepath
    } else {
        *(*proc).argv.offset(0 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_char
    };
}
#[inline]
unsafe extern "C" fn proc_is_stopped(mut proc: *mut Proc) -> bool {
    let mut exited: bool = (*proc).status >= 0 as ::core::ffi::c_int;
    return exited as ::core::ffi::c_int != 0 || (*proc).stopped_time != 0 as uint64_t;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
