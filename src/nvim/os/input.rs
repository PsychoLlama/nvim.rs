use crate::src::nvim::autocmd::{apply_autocmds, trigger_cursorhold};
use crate::src::nvim::event::libuv::uv_guess_handle;
use crate::src::nvim::event::multiqueue::{
    multiqueue_empty, multiqueue_process_events, multiqueue_put_event,
};
use crate::src::nvim::event::r#loop::loop_poll_events;
use crate::src::nvim::event::rstream::{
    rstream_init_fd, rstream_may_close, rstream_start, rstream_stop,
};
use crate::src::nvim::getchar::{before_blocking, typebuf_changed};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::keycodes::trans_special;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::main::{
    ch_before_blocking_events, ctrl_c_interrupts, curbuf, current_ui, did_cursorhold, do_profiling,
    getout, got_int, main_loop, mapped_ctrl_c, mouse_col, mouse_grid, mouse_row, p_mouset, p_ut,
    preserve_exit, silent_mode, typebuf_was_filled, used_stdin, Columns, Rows, State,
};
use crate::src::nvim::os::libc::{__assert_fail, gettext, memcpy, memmove, sscanf};
use crate::src::nvim::os::time::os_hrtime;
use crate::src::nvim::profile::{prof_input_end, prof_input_start};
use crate::src::nvim::state::get_real_state;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_16, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_13, Event,
    ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative, GridView, Intersection, Loop, LuaRef,
    MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, MultiQueue, OptInt, Proc, ProcType, RStream, ScopeDictDictItem,
    ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue,
    StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_23, Stream, String_0, Terminal,
    Timestamp, TriState, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos,
    WinConfig, WinInfo, WinSplit, WinStyle, Window, __pthread_internal_list, __pthread_list_t,
    __pthread_mutex_s, __pthread_rwlock_arch_t, __time_t, alist_T, argv_callback, auto_event,
    bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, colnr_T, dict_T,
    dictvar_S, disptick_T, event_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_14, file_buffer_b_wininfo as C2Rust_Unnamed_22,
    file_buffer_update_callbacks as C2Rust_Unnamed_11,
    file_buffer_update_channels as C2Rust_Unnamed_12, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_17, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, internal_proc_cb, key_extra,
    lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T,
    llpos_T, loop_0, loop_0_children as C2Rust_Unnamed_24, lpos_T, mapblock, mapblock_T, match_T,
    matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s, mtnode_s, multiqueue,
    partial_S, partial_T, pos_T, pos_save_T, proc, proc_exit_cb, proc_state_cb, proftime_T,
    pthread_mutex_t, pthread_rwlock_t, ptr_t, qf_info_S, qf_info_T, queue, reg_extmatch_T,
    regmmatch_T, regprog, regprog_T, rstream, sattr_T, schar_T, scid_T, sctx_T, size_t, ssize_t,
    stream, stream_close_cb, stream_read_cb, stream_uv as C2Rust_Unnamed_25, stream_write_cb,
    syn_state, syn_state_sst_union as C2Rust_Unnamed_15, syn_time_T, synblock_T, synstate_T,
    taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header,
    u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_19,
    u_header_uh_alt_prev as C2Rust_Unnamed_18, u_header_uh_next as C2Rust_Unnamed_21,
    u_header_uh_prev as C2Rust_Unnamed_20, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, uv__io_cb, uv__io_s, uv__io_t, uv__queue, uv_alloc_cb, uv_async_cb, uv_async_s,
    uv_async_s_u as C2Rust_Unnamed_3, uv_async_t, uv_buf_t, uv_close_cb, uv_connect_cb,
    uv_connect_s, uv_connect_t, uv_connection_cb, uv_file, uv_handle_s,
    uv_handle_s_u as C2Rust_Unnamed_0, uv_handle_t, uv_handle_type, uv_idle_cb, uv_idle_s,
    uv_idle_s_u as C2Rust_Unnamed_10, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_4, uv_loop_s_timer_heap as C2Rust_Unnamed_2, uv_loop_t,
    uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_7, uv_pipe_t, uv_read_cb, uv_req_type,
    uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t, uv_signal_cb, uv_signal_s,
    uv_signal_s_tree_entry as C2Rust_Unnamed, uv_signal_s_u as C2Rust_Unnamed_1, uv_signal_t,
    uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_5, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_6, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_8, uv_timer_s_u as C2Rust_Unnamed_9, uv_timer_t, varnumber_T,
    virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
};
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
pub const kStlClickFuncRun: C2Rust_Unnamed_23 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_23 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_23 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_23 = 0;
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
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_26 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_26 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_26 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_26 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_26 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_26 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_26 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_26 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_26 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_26 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_26 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_26 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_26 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_26 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_26 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_26 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_26 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_26 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_26 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_26 = 1;
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
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const FSK_SIMPLIFY: C2Rust_Unnamed_27 = 8;
pub const FSK_IN_STRING: C2Rust_Unnamed_27 = 4;
pub const FSK_KEEP_X_KEY: C2Rust_Unnamed_27 = 2;
pub const FSK_KEYCODE: C2Rust_Unnamed_27 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const EOF: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PROF_YES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
pub const KE_FILLER: ::core::ffi::c_int = 'X' as ::core::ffi::c_int;
pub const MOD_MASK_CTRL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MOD_MASK_2CLICK: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const MOD_MASK_3CLICK: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const MOD_MASK_4CLICK: ::core::ffi::c_int = 0x60 as ::core::ffi::c_int;
pub const MAX_KEY_CODE_LEN: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const READ_BUFFER_SIZE: ::core::ffi::c_int = 0xfff as ::core::ffi::c_int;
pub const INPUT_BUFFER_SIZE: ::core::ffi::c_int =
    READ_BUFFER_SIZE * 4 as ::core::ffi::c_int + MAX_KEY_CODE_LEN;
static read_stream: GlobalCell<RStream> = GlobalCell::new(rstream {
    s: stream {
        closed: true_0 != 0,
        uv: C2Rust_Unnamed_25 {
            pipe: uv_pipe_t {
                data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
                type_0: UV_UNKNOWN_HANDLE,
                close_cb: None,
                handle_queue: uv__queue {
                    next: ::core::ptr::null_mut::<uv__queue>(),
                    prev: ::core::ptr::null_mut::<uv__queue>(),
                },
                u: C2Rust_Unnamed_7 { fd: 0 },
                next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
                flags: 0,
                write_queue_size: 0,
                alloc_cb: None,
                read_cb: None,
                connect_req: ::core::ptr::null_mut::<uv_connect_t>(),
                shutdown_req: ::core::ptr::null_mut::<uv_shutdown_t>(),
                io_watcher: uv__io_t {
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
                write_queue: uv__queue {
                    next: ::core::ptr::null_mut::<uv__queue>(),
                    prev: ::core::ptr::null_mut::<uv__queue>(),
                },
                write_completed_queue: uv__queue {
                    next: ::core::ptr::null_mut::<uv__queue>(),
                    prev: ::core::ptr::null_mut::<uv__queue>(),
                },
                connection_cb: None,
                delayed_error: 0,
                accepted_fd: 0,
                queued_fds: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ipc: 0,
                pipe_fname: ::core::ptr::null::<::core::ffi::c_char>(),
            },
        },
        uvstream: ::core::ptr::null_mut::<uv_stream_t>(),
        fd: 0,
        fpos: 0,
        cb_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        before_close_cb: None,
        close_cb: None,
        internal_close_cb: None,
        close_cb_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        internal_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        pending_reqs: 0,
        events: ::core::ptr::null_mut::<MultiQueue>(),
        write_cb: None,
        curmem: 0,
        maxmem: 0,
    },
    did_eof: false,
    want_read: false,
    pending_read: false,
    paused_full: false,
    buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    uvbuf: uv_buf_t {
        base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        len: 0,
    },
    read_cb: None,
    num_bytes: 0,
});
static input_buffer: GlobalCell<[::core::ffi::c_char; 16386]> = GlobalCell::new([0; 16386]);
static input_read_pos: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new((input_buffer.as_raw() as *const _) as *mut ::core::ffi::c_char);
static input_write_pos: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new((input_buffer.as_raw() as *const _) as *mut ::core::ffi::c_char);
static input_eof: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static blocking: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static cursorhold_time: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static cursorhold_tb_change_cnt: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
#[no_mangle]
pub unsafe extern "C" fn input_start() {
    if !(*read_stream.ptr()).s.closed {
        return;
    }
    used_stdin.set(true_0 != 0);
    rstream_init_fd(main_loop.ptr(), read_stream.ptr(), STDIN_FILENO);
    rstream_start(
        read_stream.ptr(),
        Some(
            input_read_cb
                as unsafe extern "C" fn(
                    *mut RStream,
                    *const ::core::ffi::c_char,
                    size_t,
                    *mut ::core::ffi::c_void,
                    bool,
                ) -> size_t,
        ),
        NULL,
    );
}
#[no_mangle]
pub unsafe extern "C" fn input_stop() {
    if (*read_stream.ptr()).s.closed {
        return;
    }
    rstream_stop(read_stream.ptr());
    rstream_may_close(read_stream.ptr());
}
unsafe extern "C" fn cursorhold_event(mut _argv: *mut *mut ::core::ffi::c_void) {
    let mut event: event_T = (if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 {
        EVENT_CURSORHOLDI as ::core::ffi::c_int
    } else {
        EVENT_CURSORHOLD as ::core::ffi::c_int
    }) as event_T;
    apply_autocmds(
        event,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    did_cursorhold.set(true_0 != 0);
}
unsafe extern "C" fn create_cursorhold_event(mut events_enabled: bool) {
    '_c2rust_label: {
        if !events_enabled || multiqueue_empty((*main_loop.ptr()).events) as ::core::ffi::c_int != 0
        {
        } else {
            __assert_fail(
                b"!events_enabled || multiqueue_empty(main_loop.events)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/os/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                83 as ::core::ffi::c_uint,
                b"void create_cursorhold_event(_Bool)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    multiqueue_put_event(
        (*main_loop.ptr()).events,
        Event {
            handler: Some(
                cursorhold_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
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
unsafe extern "C" fn reset_cursorhold_wait(mut tb_change_cnt: ::core::ffi::c_int) {
    cursorhold_time.set(0 as ::core::ffi::c_int);
    cursorhold_tb_change_cnt.set(tb_change_cnt);
}
#[no_mangle]
pub unsafe extern "C" fn input_get(
    mut buf: *mut uint8_t,
    mut maxlen: ::core::ffi::c_int,
    mut ms: ::core::ffi::c_int,
    mut tb_change_cnt: ::core::ffi::c_int,
    mut events: *mut MultiQueue,
) -> ::core::ffi::c_int {
    if tb_change_cnt != cursorhold_tb_change_cnt.get() {
        reset_cursorhold_wait(tb_change_cnt);
    }
    if maxlen != 0 && input_available() != 0 {
        reset_cursorhold_wait(tb_change_cnt);
        '_c2rust_label: {
            if maxlen >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"maxlen >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/os/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    129 as ::core::ffi::c_uint,
                    b"int input_get(uint8_t *, int, int, int, MultiQueue *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut to_read: size_t = if (maxlen as size_t) < input_available() {
            maxlen as size_t
        } else {
            input_available()
        };
        memcpy(
            buf as *mut ::core::ffi::c_void,
            input_read_pos.get() as *const ::core::ffi::c_void,
            to_read,
        );
        input_read_pos.set((*input_read_pos.ptr()).offset(to_read as isize));
        '_c2rust_label_0: {
            if to_read <= 2147483647 as ::core::ffi::c_int as size_t {
            } else {
                __assert_fail(
                    b"to_read <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/os/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    129 as ::core::ffi::c_uint,
                    b"int input_get(uint8_t *, int, int, int, MultiQueue *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return to_read as ::core::ffi::c_int;
    }
    if (mapped_ctrl_c.get() | (*curbuf.get()).b_mapped_ctrl_c) & get_real_state() != 0 {
        ctrl_c_interrupts.set(false_0 != 0);
    }
    let mut result: TriState = kFalse;
    if ms >= 0 as ::core::ffi::c_int {
        result = inbuf_poll(ms, events);
        if result as ::core::ffi::c_int == kFalse as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
    } else {
        let mut wait_start: uint64_t = os_hrtime();
        cursorhold_time.set(
            if cursorhold_time.get() < p_ut.get() as ::core::ffi::c_int {
                cursorhold_time.get()
            } else {
                p_ut.get() as ::core::ffi::c_int
            },
        );
        result = inbuf_poll(
            p_ut.get() as ::core::ffi::c_int - cursorhold_time.get(),
            events,
        );
        if result as ::core::ffi::c_int == kFalse as ::core::ffi::c_int {
            if (*read_stream.ptr()).s.closed as ::core::ffi::c_int != 0
                && silent_mode.get() as ::core::ffi::c_int != 0
            {
                read_error_exit();
            }
            reset_cursorhold_wait(tb_change_cnt);
            if trigger_cursorhold() as ::core::ffi::c_int != 0 && !typebuf_changed(tb_change_cnt) {
                create_cursorhold_event(events == (*main_loop.ptr()).events);
            } else {
                before_blocking();
                result = inbuf_poll(-1 as ::core::ffi::c_int, events);
            }
        } else {
            (*cursorhold_time.ptr()) += os_hrtime()
                .wrapping_sub(wait_start)
                .wrapping_div(1000000 as uint64_t)
                as ::core::ffi::c_int;
        }
    }
    ctrl_c_interrupts.set(true_0 != 0);
    if typebuf_changed(tb_change_cnt) {
        return 0 as ::core::ffi::c_int;
    }
    if maxlen != 0 && input_available() != 0 {
        reset_cursorhold_wait(tb_change_cnt);
        '_c2rust_label_1: {
            if maxlen >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"maxlen >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/os/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    168 as ::core::ffi::c_uint,
                    b"int input_get(uint8_t *, int, int, int, MultiQueue *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut to_read_0: size_t = if (maxlen as size_t) < input_available() {
            maxlen as size_t
        } else {
            input_available()
        };
        memcpy(
            buf as *mut ::core::ffi::c_void,
            input_read_pos.get() as *const ::core::ffi::c_void,
            to_read_0,
        );
        input_read_pos.set((*input_read_pos.ptr()).offset(to_read_0 as isize));
        '_c2rust_label_2: {
            if to_read_0 <= 2147483647 as ::core::ffi::c_int as size_t {
            } else {
                __assert_fail(
                    b"to_read <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/os/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    168 as ::core::ffi::c_uint,
                    b"int input_get(uint8_t *, int, int, int, MultiQueue *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return to_read_0 as ::core::ffi::c_int;
    }
    if maxlen != 0 && pending_events(events) as ::core::ffi::c_int != 0 {
        return push_event_key(buf, maxlen);
    }
    if result as ::core::ffi::c_int == kNone as ::core::ffi::c_int && ms != 0 as ::core::ffi::c_int
    {
        read_error_exit();
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn os_char_avail() -> bool {
    return inbuf_poll(
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<MultiQueue>(),
    ) as ::core::ffi::c_int
        == kTrue as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn os_breakcheck() {
    if got_int.get() {
        return;
    }
    loop_poll_events(main_loop.ptr(), 0 as int64_t);
}
pub const BREAKCHECK_SKIP: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;
static breakcheck_count: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
#[no_mangle]
pub unsafe extern "C" fn line_breakcheck() {
    (*breakcheck_count.ptr()) += 1;
    if breakcheck_count.get() >= BREAKCHECK_SKIP {
        breakcheck_count.set(0 as ::core::ffi::c_int);
        os_breakcheck();
    }
}
#[no_mangle]
pub unsafe extern "C" fn fast_breakcheck() {
    (*breakcheck_count.ptr()) += 1;
    if breakcheck_count.get() >= BREAKCHECK_SKIP * 10 as ::core::ffi::c_int {
        breakcheck_count.set(0 as ::core::ffi::c_int);
        os_breakcheck();
    }
}
#[no_mangle]
pub unsafe extern "C" fn veryfast_breakcheck() {
    (*breakcheck_count.ptr()) += 1;
    if breakcheck_count.get() >= BREAKCHECK_SKIP * 100 as ::core::ffi::c_int {
        breakcheck_count.set(0 as ::core::ffi::c_int);
        os_breakcheck();
    }
}
#[no_mangle]
pub unsafe extern "C" fn os_isatty(mut fd: ::core::ffi::c_int) -> bool {
    return uv_guess_handle(fd as uv_file) as ::core::ffi::c_uint
        == UV_TTY as ::core::ffi::c_int as ::core::ffi::c_uint;
}
#[no_mangle]
pub unsafe extern "C" fn input_available() -> size_t {
    return (*input_write_pos.ptr()).offset_from(input_read_pos.get()) as size_t;
}
unsafe extern "C" fn input_space() -> size_t {
    return (input_buffer.ptr() as *mut ::core::ffi::c_char)
        .offset(INPUT_BUFFER_SIZE as isize)
        .offset_from(input_write_pos.get()) as size_t;
}
#[no_mangle]
pub unsafe extern "C" fn input_enqueue_raw(mut data: *const ::core::ffi::c_char, mut size: size_t) {
    if input_read_pos.get() > input_buffer.ptr() as *mut ::core::ffi::c_char {
        let mut available: size_t = input_available();
        memmove(
            input_buffer.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            input_read_pos.get() as *const ::core::ffi::c_void,
            available,
        );
        input_read_pos.set(input_buffer.ptr() as *mut ::core::ffi::c_char);
        input_write_pos
            .set((input_buffer.ptr() as *mut ::core::ffi::c_char).offset(available as isize));
    }
    let mut to_write: size_t = if size < input_space() {
        size
    } else {
        input_space()
    };
    memcpy(
        input_write_pos.get() as *mut ::core::ffi::c_void,
        data as *const ::core::ffi::c_void,
        to_write,
    );
    input_write_pos.set((*input_write_pos.ptr()).offset(to_write as isize));
}
#[no_mangle]
pub unsafe extern "C" fn input_enqueue(mut chan_id: uint64_t, mut keys: String_0) -> size_t {
    current_ui.set(chan_id);
    let mut ptr: *const ::core::ffi::c_char = keys.data;
    let mut end: *const ::core::ffi::c_char = ptr.offset(keys.size as isize);
    while input_space() >= 19 as size_t && ptr < end {
        let mut buf: [uint8_t; 19] = [
            0 as uint8_t,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
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
        let mut new_size: ::core::ffi::c_uint = trans_special(
            &raw mut ptr,
            end.offset_from(ptr) as size_t,
            &raw mut buf as *mut uint8_t as *mut ::core::ffi::c_char,
            FSK_KEYCODE as ::core::ffi::c_int,
            true_0 != 0,
            ::core::ptr::null_mut::<bool>(),
        );
        if new_size > 0 as ::core::ffi::c_uint {
            new_size = handle_mouse_event(&raw mut ptr, &raw mut buf as *mut uint8_t, new_size);
            if new_size > 0 as ::core::ffi::c_uint {
                input_enqueue_raw(
                    &raw mut buf as *mut uint8_t as *mut ::core::ffi::c_char,
                    new_size as size_t,
                );
            }
        } else if *ptr as ::core::ffi::c_int == '<' as ::core::ffi::c_int {
            let mut old_ptr: *const ::core::ffi::c_char = ptr;
            loop {
                ptr = ptr.offset(1);
                if !(ptr < end && *ptr as ::core::ffi::c_int != '>' as ::core::ffi::c_int) {
                    break;
                }
            }
            if *ptr as ::core::ffi::c_int != '>' as ::core::ffi::c_int {
                ptr = old_ptr;
                break;
            } else {
                ptr = ptr.offset(1);
            }
        } else {
            if *ptr as uint8_t as ::core::ffi::c_int == K_SPECIAL {
                let mut c2rust_lvalue: uint8_t = K_SPECIAL as uint8_t;
                input_enqueue_raw(
                    &raw mut c2rust_lvalue as *mut ::core::ffi::c_char,
                    1 as size_t,
                );
                let mut c2rust_lvalue_0: uint8_t = KS_SPECIAL as uint8_t;
                input_enqueue_raw(
                    &raw mut c2rust_lvalue_0 as *mut ::core::ffi::c_char,
                    1 as size_t,
                );
                let mut c2rust_lvalue_1: uint8_t = KE_FILLER as uint8_t;
                input_enqueue_raw(
                    &raw mut c2rust_lvalue_1 as *mut ::core::ffi::c_char,
                    1 as size_t,
                );
            } else {
                input_enqueue_raw(ptr, 1 as size_t);
            }
            ptr = ptr.offset(1);
        }
    }
    let mut rv: size_t = ptr.offset_from(keys.data) as size_t;
    process_ctrl_c();
    return rv;
}
unsafe extern "C" fn check_multiclick(
    mut code: ::core::ffi::c_int,
    mut grid: ::core::ffi::c_int,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut skip_event: *mut bool,
) -> uint8_t {
    static orig_num_clicks: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    static orig_mouse_code: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    static orig_mouse_grid: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    static orig_mouse_col: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    static orig_mouse_row: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    static orig_mouse_time: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
    if code >= KE_MOUSEDOWN as ::core::ffi::c_int && code <= KE_MOUSERIGHT as ::core::ffi::c_int {
        return 0 as uint8_t;
    }
    let mut no_move: bool =
        orig_mouse_grid.get() == grid && orig_mouse_col.get() == col && orig_mouse_row.get() == row;
    if code == KE_MOUSEMOVE as ::core::ffi::c_int {
        if no_move {
            *skip_event = true_0 != 0;
            return 0 as uint8_t;
        }
    } else if code == KE_LEFTMOUSE as ::core::ffi::c_int
        || code == KE_RIGHTMOUSE as ::core::ffi::c_int
        || code == KE_MIDDLEMOUSE as ::core::ffi::c_int
        || code == KE_X1MOUSE as ::core::ffi::c_int
        || code == KE_X2MOUSE as ::core::ffi::c_int
    {
        let mut mouse_time: uint64_t = os_hrtime();
        let mut timediff: uint64_t = mouse_time.wrapping_sub(orig_mouse_time.get());
        let mut mouset: uint64_t = (p_mouset.get() as uint64_t).wrapping_mul(1000000 as uint64_t);
        if code == orig_mouse_code.get()
            && no_move as ::core::ffi::c_int != 0
            && timediff < mouset
            && orig_num_clicks.get() != 4 as ::core::ffi::c_int
        {
            (*orig_num_clicks.ptr()) += 1;
        } else {
            orig_num_clicks.set(1 as ::core::ffi::c_int);
        }
        orig_mouse_code.set(code);
        orig_mouse_time.set(mouse_time);
    }
    orig_mouse_grid.set(grid);
    orig_mouse_col.set(col);
    orig_mouse_row.set(row);
    let mut modifiers: uint8_t = 0 as uint8_t;
    if code != KE_MOUSEMOVE as ::core::ffi::c_int {
        if orig_num_clicks.get() == 2 as ::core::ffi::c_int {
            modifiers = (modifiers as ::core::ffi::c_int | MOD_MASK_2CLICK) as uint8_t;
        } else if orig_num_clicks.get() == 3 as ::core::ffi::c_int {
            modifiers = (modifiers as ::core::ffi::c_int | MOD_MASK_3CLICK) as uint8_t;
        } else if orig_num_clicks.get() == 4 as ::core::ffi::c_int {
            modifiers = (modifiers as ::core::ffi::c_int | MOD_MASK_4CLICK) as uint8_t;
        }
    }
    return modifiers;
}
unsafe extern "C" fn handle_mouse_event(
    mut ptr: *mut *const ::core::ffi::c_char,
    mut buf: *mut uint8_t,
    mut bufsize: ::core::ffi::c_uint,
) -> ::core::ffi::c_uint {
    let mut mouse_code: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut type_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if bufsize == 3 as ::core::ffi::c_uint {
        mouse_code = *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
        type_0 = *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
    } else if bufsize == 6 as ::core::ffi::c_uint {
        mouse_code = *buf.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
        type_0 = *buf.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
    }
    if type_0 != KS_EXTRA
        || !(mouse_code >= KE_LEFTMOUSE as ::core::ffi::c_int
            && mouse_code <= KE_RIGHTRELEASE as ::core::ffi::c_int
            || mouse_code >= KE_X1MOUSE as ::core::ffi::c_int
                && mouse_code <= KE_X2RELEASE as ::core::ffi::c_int
            || mouse_code >= KE_MOUSEDOWN as ::core::ffi::c_int
                && mouse_code <= KE_MOUSERIGHT as ::core::ffi::c_int
            || mouse_code == KE_MOUSEMOVE as ::core::ffi::c_int)
    {
        return bufsize;
    }
    let mut col: ::core::ffi::c_int = 0;
    let mut row: ::core::ffi::c_int = 0;
    let mut advance: ::core::ffi::c_int = 0;
    if sscanf(
        *ptr,
        b"<%d,%d>%n\0".as_ptr() as *const ::core::ffi::c_char,
        &raw mut col,
        &raw mut row,
        &raw mut advance,
    ) != EOF
        && advance != 0
    {
        if col >= 0 as ::core::ffi::c_int && row >= 0 as ::core::ffi::c_int {
            if col >= Columns.get() {
                col = Columns.get() - 1 as ::core::ffi::c_int;
            }
            if row >= Rows.get() {
                row = Rows.get() - 1 as ::core::ffi::c_int;
            }
            mouse_grid.set(0 as ::core::ffi::c_int);
            mouse_row.set(row);
            mouse_col.set(col);
        }
        *ptr = (*ptr).offset(advance as isize);
    }
    let mut skip_event: bool = false_0 != 0;
    let mut modifiers: uint8_t = check_multiclick(
        mouse_code,
        mouse_grid.get(),
        mouse_row.get(),
        mouse_col.get(),
        &raw mut skip_event,
    );
    if skip_event {
        return 0 as ::core::ffi::c_uint;
    }
    if modifiers != 0 {
        if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != KS_MODIFIER {
            memcpy(
                buf.offset(3 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                buf as *const ::core::ffi::c_void,
                3 as size_t,
            );
            *buf.offset(0 as ::core::ffi::c_int as isize) = K_SPECIAL as uint8_t;
            *buf.offset(1 as ::core::ffi::c_int as isize) = KS_MODIFIER as uint8_t;
            *buf.offset(2 as ::core::ffi::c_int as isize) = modifiers;
            bufsize = bufsize.wrapping_add(3 as ::core::ffi::c_uint);
        } else {
            *buf.offset(2 as ::core::ffi::c_int as isize) =
                (*buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    | modifiers as ::core::ffi::c_int) as uint8_t;
        }
    }
    return bufsize;
}
#[no_mangle]
pub unsafe extern "C" fn input_enqueue_mouse(
    mut code: ::core::ffi::c_int,
    mut modifier: uint8_t,
    mut grid: ::core::ffi::c_int,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
) {
    let mut skip_event: bool = false_0 != 0;
    modifier = (modifier as ::core::ffi::c_int
        | check_multiclick(code, grid, row, col, &raw mut skip_event) as ::core::ffi::c_int)
        as uint8_t;
    if skip_event {
        return;
    }
    let mut buf: [uint8_t; 7] = [0; 7];
    let mut p: *mut uint8_t = &raw mut buf as *mut uint8_t;
    if modifier != 0 {
        *p.offset(0 as ::core::ffi::c_int as isize) = K_SPECIAL as uint8_t;
        *p.offset(1 as ::core::ffi::c_int as isize) = KS_MODIFIER as uint8_t;
        *p.offset(2 as ::core::ffi::c_int as isize) = modifier;
        p = p.offset(3 as ::core::ffi::c_int as isize);
    }
    *p.offset(0 as ::core::ffi::c_int as isize) = K_SPECIAL as uint8_t;
    *p.offset(1 as ::core::ffi::c_int as isize) = KS_EXTRA as uint8_t;
    *p.offset(2 as ::core::ffi::c_int as isize) = code as uint8_t;
    mouse_grid.set(grid);
    mouse_row.set(row);
    mouse_col.set(col);
    let mut written: size_t =
        (3 as size_t).wrapping_add(p.offset_from(&raw mut buf as *mut uint8_t) as size_t);
    input_enqueue_raw(
        &raw mut buf as *mut uint8_t as *mut ::core::ffi::c_char,
        written,
    );
}
#[no_mangle]
pub unsafe extern "C" fn input_blocking() -> bool {
    return blocking.get();
}
unsafe extern "C" fn inbuf_poll(
    mut ms: ::core::ffi::c_int,
    mut events: *mut MultiQueue,
) -> TriState {
    if os_input_ready(events) {
        return kTrue;
    }
    if do_profiling.get() == PROF_YES && ms != 0 {
        prof_input_start();
    }
    if (ms == -1 as ::core::ffi::c_int || ms > 0 as ::core::ffi::c_int)
        && events != (*main_loop.ptr()).events
        && !input_eof.get()
    {
        blocking.set(true_0 != 0);
        multiqueue_process_events(ch_before_blocking_events.get());
    }
    logmsg(
        LOGLVL_DBG,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"inbuf_poll\0".as_ptr() as *const ::core::ffi::c_char,
        514 as ::core::ffi::c_int,
        true_0 != 0,
        b"blocking... events=%s\0".as_ptr() as *const ::core::ffi::c_char,
        if !events.is_null() {
            b"true\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"false\0".as_ptr() as *const ::core::ffi::c_char
        },
    );
    let mut remaining: int64_t = ms as int64_t;
    let mut before: uint64_t = if remaining > 0 as int64_t {
        os_hrtime()
    } else {
        0 as uint64_t
    };
    while !(os_input_ready(events) as ::core::ffi::c_int != 0
        || input_eof.get() as ::core::ffi::c_int != 0)
    {
        if !::core::ptr::null_mut::<::core::ffi::c_void>().is_null()
            && !multiqueue_empty(::core::ptr::null_mut::<MultiQueue>())
        {
            multiqueue_process_events(::core::ptr::null_mut::<MultiQueue>());
        } else {
            loop_poll_events(main_loop.ptr(), remaining);
        }
        if remaining == 0 as int64_t {
            break;
        }
        if remaining <= 0 as int64_t {
            continue;
        }
        let mut now: uint64_t = os_hrtime();
        remaining -= now.wrapping_sub(before).wrapping_div(1000000 as uint64_t) as int64_t;
        before = now;
        if remaining <= 0 as int64_t {
            break;
        }
    }
    blocking.set(false_0 != 0);
    if do_profiling.get() == PROF_YES && ms != 0 {
        prof_input_end();
    }
    if os_input_ready(events) {
        return kTrue;
    }
    return (if input_eof.get() as ::core::ffi::c_int != 0 {
        kNone as ::core::ffi::c_int
    } else {
        kFalse as ::core::ffi::c_int
    }) as TriState;
}
unsafe extern "C" fn input_read_cb(
    mut _stream: *mut RStream,
    mut buf: *const ::core::ffi::c_char,
    mut c: size_t,
    mut _data: *mut ::core::ffi::c_void,
    mut at_eof: bool,
) -> size_t {
    if at_eof {
        input_eof.set(true_0 != 0);
    }
    '_c2rust_label: {
        if input_space() >= c {
        } else {
            __assert_fail(
                b"input_space() >= c\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                534 as ::core::ffi::c_uint,
                b"size_t input_read_cb(RStream *, const char *, size_t, void *, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    input_enqueue_raw(buf, c);
    return c;
}
unsafe extern "C" fn process_ctrl_c() {
    if !ctrl_c_interrupts.get() {
        return;
    }
    let mut available: size_t = input_available();
    let mut i: ssize_t = 0;
    i = available as ssize_t - 1 as ssize_t;
    while i >= 0 as ssize_t {
        let mut c: uint8_t = *(*input_read_pos.ptr()).offset(i as isize) as uint8_t;
        if c as ::core::ffi::c_int == Ctrl_C
            || c as ::core::ffi::c_int == 'C' as ::core::ffi::c_int
                && i >= 3 as ssize_t
                && *(*input_read_pos.ptr()).offset((i - 3 as ssize_t) as isize) as uint8_t
                    as ::core::ffi::c_int
                    == K_SPECIAL
                && *(*input_read_pos.ptr()).offset((i - 2 as ssize_t) as isize) as uint8_t
                    as ::core::ffi::c_int
                    == KS_MODIFIER
                && *(*input_read_pos.ptr()).offset((i - 1 as ssize_t) as isize) as uint8_t
                    as ::core::ffi::c_int
                    == MOD_MASK_CTRL
        {
            *(*input_read_pos.ptr()).offset(i as isize) = Ctrl_C as ::core::ffi::c_char;
            got_int.set(true_0 != 0);
            break;
        } else {
            i -= 1;
        }
    }
    if got_int.get() as ::core::ffi::c_int != 0 && i > 0 as ssize_t {
        input_read_pos.set((*input_read_pos.ptr()).offset(i as isize));
    }
}
unsafe extern "C" fn push_event_key(
    mut buf: *mut uint8_t,
    mut maxlen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    static key: GlobalCell<[uint8_t; 3]> = GlobalCell::new([
        K_SPECIAL as uint8_t,
        KS_EXTRA as uint8_t,
        KE_EVENT as ::core::ffi::c_int as uint8_t,
    ]);
    static key_idx: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    let mut buf_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        let c2rust_fresh0 = key_idx.get();
        key_idx.set(key_idx.get() + 1);
        let c2rust_fresh1 = buf_idx;
        buf_idx = buf_idx + 1;
        *buf.offset(c2rust_fresh1 as isize) = (*key.ptr())[c2rust_fresh0 as usize];
        (*key_idx.ptr()) %= 3 as ::core::ffi::c_int;
        if !(key_idx.get() > 0 as ::core::ffi::c_int && buf_idx < maxlen) {
            break;
        }
    }
    return buf_idx;
}
#[no_mangle]
pub unsafe extern "C" fn os_input_ready(mut events: *mut MultiQueue) -> bool {
    return typebuf_was_filled.get() as ::core::ffi::c_int != 0
        || input_available() != 0
        || pending_events(events) as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn read_error_exit() -> ! {
    if silent_mode.get() {
        getout(0 as ::core::ffi::c_int);
    }
    preserve_exit(gettext(
        b"Nvim: Error reading input, exiting...\n\0".as_ptr() as *const ::core::ffi::c_char,
    ));
}
unsafe extern "C" fn pending_events(mut events: *mut MultiQueue) -> bool {
    return !events.is_null() && !multiqueue_empty(events);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
