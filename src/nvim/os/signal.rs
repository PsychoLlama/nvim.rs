use crate::src::nvim::autocmd::apply_autocmds;
use crate::src::nvim::eval::vars::set_vim_var_nr;
use crate::src::nvim::event::signal::{
    signal_watcher_close, signal_watcher_init, signal_watcher_start, signal_watcher_stop,
};
use crate::src::nvim::ex_cmds2::autowrite_all;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::main::{curbuf, main_loop, p_awa, preserve_exit, v_dying, IObuff};
use crate::src::nvim::memline::ml_sync_all;
use crate::src::nvim::os::libc::{__assert_fail, snprintf};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_4, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_1,
    ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative, GridView, Intersection, Loop, LuaRef,
    MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, MultiQueue, OptInt, Proc, ProcType, RStream, ScopeDictDictItem,
    ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SignalWatcher, SpecialVarValue,
    StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_11, Stream, Terminal,
    Timestamp, VarLockStatus, VarType, VimVarIndex, VirtLines, VirtText, VirtTextChunk,
    VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle, Window, __pthread_internal_list,
    __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t, __time_t, alist_T, auto_event,
    bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, colnr_T, dict_T,
    dictvar_S, disptick_T, event_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_2, file_buffer_b_wininfo as C2Rust_Unnamed_10,
    file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, internal_proc_cb, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, loop_0,
    loop_0_children as C2Rust_Unnamed_20, lpos_T, mapblock, mapblock_T, match_T, matchitem,
    matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s, mtnode_s, multiqueue, partial_S,
    partial_T, pos_T, pos_save_T, proc, proc_exit_cb, proc_state_cb, proftime_T, pthread_mutex_t,
    pthread_rwlock_t, ptr_t, qf_info_S, qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog,
    regprog_T, rstream, sattr_T, schar_T, scid_T, sctx_T, signal_cb, signal_close_cb,
    signal_watcher, size_t, ssize_t, stream, stream_close_cb, stream_read_cb,
    stream_uv as C2Rust_Unnamed_22, stream_write_cb, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T, synblock_T, synstate_T, taggy_T, terminal,
    time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_7, u_header_uh_alt_prev as C2Rust_Unnamed_6,
    u_header_uh_next as C2Rust_Unnamed_9, u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, uv__io_cb, uv__io_s, uv__io_t, uv__queue,
    uv_alloc_cb, uv_async_cb, uv_async_s, uv_async_s_u as C2Rust_Unnamed_17, uv_async_t, uv_buf_t,
    uv_close_cb, uv_connect_cb, uv_connect_s, uv_connect_t, uv_connection_cb, uv_file, uv_handle_s,
    uv_handle_s_u as C2Rust_Unnamed_12, uv_handle_t, uv_handle_type, uv_idle_cb, uv_idle_s,
    uv_idle_s_u as C2Rust_Unnamed_23, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_16, uv_loop_s_timer_heap as C2Rust_Unnamed_15,
    uv_loop_t, uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_25, uv_pipe_t, uv_read_cb,
    uv_req_type, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t, uv_signal_cb,
    uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed_13, uv_signal_s_u as C2Rust_Unnamed_14,
    uv_signal_t, uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_21, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_24, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_18, uv_timer_s_u as C2Rust_Unnamed_19, uv_timer_t,
    varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T,
    QUEUE,
};
extern "C" {
    fn sigemptyset(__set: *mut sigset_t) -> ::core::ffi::c_int;
    fn pthread_sigmask(
        __how: ::core::ffi::c_int,
        __newmask: *const __sigset_t,
        __oldmask: *mut __sigset_t,
    ) -> ::core::ffi::c_int;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sigset_t {
    pub __val: [::core::ffi::c_ulong; 16],
}
pub type sigset_t = __sigset_t;
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
pub const kStlClickFuncRun: C2Rust_Unnamed_11 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_11 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_11 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_11 = 0;
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
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 45] = unsafe {
    ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
        *b"void on_signal(SignalWatcher *, int, void *)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const SIG_SETMASK: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const LOGLVL_INF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
static sint: GlobalCell<SignalWatcher> = GlobalCell::new(SignalWatcher {
    uv: uv_signal_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_14 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        signal_cb: None,
        signum: 0,
        tree_entry: C2Rust_Unnamed_13 {
            rbe_left: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_right: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_parent: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_color: 0,
        },
        caught_signals: 0,
        dispatched_signals: 0,
    },
    data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    cb: None,
    close_cb: None,
    events: ::core::ptr::null_mut::<MultiQueue>(),
});
static spipe: GlobalCell<SignalWatcher> = GlobalCell::new(SignalWatcher {
    uv: uv_signal_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_14 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        signal_cb: None,
        signum: 0,
        tree_entry: C2Rust_Unnamed_13 {
            rbe_left: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_right: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_parent: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_color: 0,
        },
        caught_signals: 0,
        dispatched_signals: 0,
    },
    data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    cb: None,
    close_cb: None,
    events: ::core::ptr::null_mut::<MultiQueue>(),
});
static squit: GlobalCell<SignalWatcher> = GlobalCell::new(SignalWatcher {
    uv: uv_signal_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_14 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        signal_cb: None,
        signum: 0,
        tree_entry: C2Rust_Unnamed_13 {
            rbe_left: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_right: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_parent: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_color: 0,
        },
        caught_signals: 0,
        dispatched_signals: 0,
    },
    data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    cb: None,
    close_cb: None,
    events: ::core::ptr::null_mut::<MultiQueue>(),
});
static sterm: GlobalCell<SignalWatcher> = GlobalCell::new(SignalWatcher {
    uv: uv_signal_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_14 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        signal_cb: None,
        signum: 0,
        tree_entry: C2Rust_Unnamed_13 {
            rbe_left: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_right: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_parent: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_color: 0,
        },
        caught_signals: 0,
        dispatched_signals: 0,
    },
    data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    cb: None,
    close_cb: None,
    events: ::core::ptr::null_mut::<MultiQueue>(),
});
static ststp: GlobalCell<SignalWatcher> = GlobalCell::new(SignalWatcher {
    uv: uv_signal_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_14 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        signal_cb: None,
        signum: 0,
        tree_entry: C2Rust_Unnamed_13 {
            rbe_left: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_right: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_parent: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_color: 0,
        },
        caught_signals: 0,
        dispatched_signals: 0,
    },
    data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    cb: None,
    close_cb: None,
    events: ::core::ptr::null_mut::<MultiQueue>(),
});
static swinch: GlobalCell<SignalWatcher> = GlobalCell::new(SignalWatcher {
    uv: uv_signal_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_14 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        signal_cb: None,
        signum: 0,
        tree_entry: C2Rust_Unnamed_13 {
            rbe_left: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_right: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_parent: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_color: 0,
        },
        caught_signals: 0,
        dispatched_signals: 0,
    },
    data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    cb: None,
    close_cb: None,
    events: ::core::ptr::null_mut::<MultiQueue>(),
});
static susr1: GlobalCell<SignalWatcher> = GlobalCell::new(SignalWatcher {
    uv: uv_signal_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_14 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        signal_cb: None,
        signum: 0,
        tree_entry: C2Rust_Unnamed_13 {
            rbe_left: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_right: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_parent: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_color: 0,
        },
        caught_signals: 0,
        dispatched_signals: 0,
    },
    data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    cb: None,
    close_cb: None,
    events: ::core::ptr::null_mut::<MultiQueue>(),
});
static shup: GlobalCell<SignalWatcher> = GlobalCell::new(SignalWatcher {
    uv: uv_signal_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_14 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        signal_cb: None,
        signum: 0,
        tree_entry: C2Rust_Unnamed_13 {
            rbe_left: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_right: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_parent: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_color: 0,
        },
        caught_signals: 0,
        dispatched_signals: 0,
    },
    data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    cb: None,
    close_cb: None,
    events: ::core::ptr::null_mut::<MultiQueue>(),
});
static spwr: GlobalCell<SignalWatcher> = GlobalCell::new(SignalWatcher {
    uv: uv_signal_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_14 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        signal_cb: None,
        signum: 0,
        tree_entry: C2Rust_Unnamed_13 {
            rbe_left: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_right: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_parent: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_color: 0,
        },
        caught_signals: 0,
        dispatched_signals: 0,
    },
    data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    cb: None,
    close_cb: None,
    events: ::core::ptr::null_mut::<MultiQueue>(),
});
static rejecting_deadly: GlobalCell<bool> = GlobalCell::new(false);
#[no_mangle]
pub unsafe extern "C" fn signal_init() {
    let mut mask: sigset_t = sigset_t { __val: [0; 16] };
    sigemptyset(&raw mut mask);
    if pthread_sigmask(
        SIG_SETMASK,
        &raw mut mask,
        ::core::ptr::null_mut::<__sigset_t>(),
    ) != 0 as ::core::ffi::c_int
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"signal_init\0".as_ptr() as *const ::core::ffi::c_char,
            47 as ::core::ffi::c_int,
            true_0 != 0,
            b"Could not unblock signals, nvim might behave strangely.\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
    }
    signal_watcher_init(main_loop.ptr(), spipe.ptr(), NULL);
    signal_watcher_init(main_loop.ptr(), shup.ptr(), NULL);
    signal_watcher_init(main_loop.ptr(), sint.ptr(), NULL);
    signal_watcher_init(main_loop.ptr(), squit.ptr(), NULL);
    signal_watcher_init(main_loop.ptr(), sterm.ptr(), NULL);
    signal_watcher_init(main_loop.ptr(), ststp.ptr(), NULL);
    signal_watcher_init(main_loop.ptr(), spwr.ptr(), NULL);
    signal_watcher_init(main_loop.ptr(), susr1.ptr(), NULL);
    signal_watcher_init(main_loop.ptr(), swinch.ptr(), NULL);
    signal_start();
}
#[no_mangle]
pub unsafe extern "C" fn signal_teardown() {
    signal_stop();
    signal_watcher_close(spipe.ptr(), None);
    signal_watcher_close(shup.ptr(), None);
    signal_watcher_close(sint.ptr(), None);
    signal_watcher_close(squit.ptr(), None);
    signal_watcher_close(sterm.ptr(), None);
    signal_watcher_close(ststp.ptr(), None);
    signal_watcher_close(spwr.ptr(), None);
    signal_watcher_close(susr1.ptr(), None);
    signal_watcher_close(swinch.ptr(), None);
}
#[no_mangle]
pub unsafe extern "C" fn signal_start() {
    signal_watcher_start(
        spipe.ptr(),
        Some(
            on_signal
                as unsafe extern "C" fn(
                    *mut SignalWatcher,
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        SIGPIPE,
    );
    signal_watcher_start(
        shup.ptr(),
        Some(
            on_signal
                as unsafe extern "C" fn(
                    *mut SignalWatcher,
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        SIGHUP,
    );
    signal_watcher_start(
        sint.ptr(),
        Some(
            on_signal
                as unsafe extern "C" fn(
                    *mut SignalWatcher,
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        SIGINT,
    );
    signal_watcher_start(
        squit.ptr(),
        Some(
            on_signal
                as unsafe extern "C" fn(
                    *mut SignalWatcher,
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        SIGQUIT,
    );
    signal_watcher_start(
        sterm.ptr(),
        Some(
            on_signal
                as unsafe extern "C" fn(
                    *mut SignalWatcher,
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        SIGTERM,
    );
    signal_watcher_start(
        ststp.ptr(),
        Some(
            on_signal
                as unsafe extern "C" fn(
                    *mut SignalWatcher,
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        SIGTSTP,
    );
    signal_watcher_start(
        spwr.ptr(),
        Some(
            on_signal
                as unsafe extern "C" fn(
                    *mut SignalWatcher,
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        SIGPWR,
    );
    signal_watcher_start(
        susr1.ptr(),
        Some(
            on_signal
                as unsafe extern "C" fn(
                    *mut SignalWatcher,
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        SIGUSR1,
    );
    signal_watcher_start(
        swinch.ptr(),
        Some(
            on_signal
                as unsafe extern "C" fn(
                    *mut SignalWatcher,
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        SIGWINCH,
    );
}
#[no_mangle]
pub unsafe extern "C" fn signal_stop() {
    signal_watcher_stop(spipe.ptr());
    signal_watcher_stop(shup.ptr());
    signal_watcher_stop(sint.ptr());
    signal_watcher_stop(squit.ptr());
    signal_watcher_stop(sterm.ptr());
    signal_watcher_stop(ststp.ptr());
    signal_watcher_stop(spwr.ptr());
    signal_watcher_stop(susr1.ptr());
    signal_watcher_stop(swinch.ptr());
}
#[no_mangle]
pub unsafe extern "C" fn signal_reject_deadly() {
    rejecting_deadly.set(true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn signal_accept_deadly() {
    rejecting_deadly.set(false_0 != 0);
}
unsafe extern "C" fn signal_name(mut signum: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    match signum {
        SIGPWR => {
            return b"SIGPWR\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        SIGPIPE => {
            return b"SIGPIPE\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        SIGTERM => {
            return b"SIGTERM\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        SIGTSTP => {
            return b"SIGTSTP\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        SIGQUIT => {
            return b"SIGQUIT\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        SIGHUP => {
            return b"SIGHUP\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        SIGINT => {
            return b"SIGINT\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        SIGUSR1 => {
            return b"SIGUSR1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        SIGWINCH => {
            return b"SIGWINCH\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        _ => {
            return b"Unknown\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
    };
}
unsafe extern "C" fn deadly_signal(mut signum: ::core::ffi::c_int) -> ! {
    set_vim_var_nr(VV_DYING, 1 as varnumber_T);
    v_dying.set(1 as ::core::ffi::c_int);
    logmsg(
        LOGLVL_INF,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"deadly_signal\0".as_ptr() as *const ::core::ffi::c_char,
        196 as ::core::ffi::c_int,
        true_0 != 0,
        b"got signal %d (%s)\0".as_ptr() as *const ::core::ffi::c_char,
        signum,
        signal_name(signum),
    );
    snprintf(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        b"Nvim: Caught deadly signal '%s'\n\0".as_ptr() as *const ::core::ffi::c_char,
        signal_name(signum),
    );
    if p_awa.get() != 0 && signum != SIGTERM && signum != SIGINT {
        autowrite_all();
    }
    preserve_exit(IObuff.ptr() as *mut ::core::ffi::c_char);
}
unsafe extern "C" fn on_signal(
    mut _handle: *mut SignalWatcher,
    mut signum: ::core::ffi::c_int,
    mut _data: *mut ::core::ffi::c_void,
) {
    '_c2rust_label: {
        if signum >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"signum >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/signal.rs\0".as_ptr() as *const ::core::ffi::c_char,
                210 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    match signum {
        SIGPWR => {
            ml_sync_all(false_0, false_0, true_0 != 0);
        }
        SIGPIPE => {}
        SIGTSTP => {
            if p_awa.get() != 0 {
                autowrite_all();
            }
        }
        SIGHUP | SIGINT | SIGTERM | SIGQUIT => {
            if !rejecting_deadly.get() {
                deadly_signal(signum);
            }
        }
        SIGUSR1 => {
            apply_autocmds(
                EVENT_SIGNAL,
                b"SIGUSR1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                (*curbuf.get()).b_fname,
                true_0 != 0,
                curbuf.get(),
            );
        }
        SIGWINCH => {
            apply_autocmds(
                EVENT_SIGNAL,
                b"SIGWINCH\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                (*curbuf.get()).b_fname,
                true_0 != 0,
                curbuf.get(),
            );
        }
        _ => {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"on_signal\0".as_ptr() as *const ::core::ffi::c_char,
                254 as ::core::ffi::c_int,
                true_0 != 0,
                b"invalid signal: %d\0".as_ptr() as *const ::core::ffi::c_char,
                signum,
            );
        }
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SIGINT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SIGTERM: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const SIGHUP: ::core::ffi::c_int = 1;
pub const SIGQUIT: ::core::ffi::c_int = 3;
pub const SIGPIPE: ::core::ffi::c_int = 13;
pub const SIGPWR: ::core::ffi::c_int = 30;
pub const SIGTSTP: ::core::ffi::c_int = 20;
pub const SIGUSR1: ::core::ffi::c_int = 10;
pub const SIGWINCH: ::core::ffi::c_int = 28 as ::core::ffi::c_int;
