use crate::src::nvim::autocmd::{apply_autocmds, has_event};
use crate::src::nvim::drawscreen::{setcursor, update_screen};
use crate::src::nvim::eval::typval::{tv_dict_add_str, tv_dict_set_keys_readonly};
use crate::src::nvim::eval_1::{get_v_event, restore_v_event};
use crate::src::nvim::event::multiqueue::{multiqueue_empty, multiqueue_get};
use crate::src::nvim::ex_getln::{cmdline_overstrike, get_cmdline_info};
use crate::src::nvim::getchar::{
    check_end_reg_executing, may_sync_undo, safe_vgetc, stuff_empty, using_script, vpeekc,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::insexpand::{ctrl_x_mode_not_defined_yet, ins_compl_active};
use crate::src::nvim::keycodes::get_special_key_name;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::main::{
    curbuf, debug_mode, exmode_active, finish_op, global_busy, got_int, last_mode, main_loop,
    mod_mask, motion_force, must_redraw, need_wait_return, restart_VIsual_select, restart_edit,
    typebuf, virtual_op, State, VIsual_active, VIsual_mode, VIsual_select,
};
use crate::src::nvim::option::get_ve_flags;
use crate::src::nvim::os::input::{input_available, input_get, os_breakcheck};
use crate::src::nvim::os::libc::{strcmp, strcpy};
use crate::src::nvim::strings::vim_snprintf;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_4, ChangedtickDictItem, CmdRedraw, CmdlineColorChunk,
    CmdlineColors, CmdlineInfo, ColoredCmdline, DecorExt, DecorHighlightInline, DecorInlineData,
    DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_1, Direction, Event,
    ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative, GridView, Intersection, Loop, LuaRef,
    MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, MultiQueue, OptInt, Proc, ProcType, RStream, ScopeDictDictItem,
    ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue,
    StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_11, Stream, Terminal,
    Timestamp, TriState, VarLockStatus, VarType, VimState, VirtLines, VirtText, VirtTextChunk,
    VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle, Window, __pthread_internal_list,
    __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t, __time_t, alist_T, argv_callback,
    auto_event, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, cmdline_info,
    colnr_T, dict_T, dictvar_S, disptick_T, event_T, expand_T, extmark_undo_vec_t, fcs_chars_T,
    file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_2,
    file_buffer_b_wininfo as C2Rust_Unnamed_10, file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, internal_proc_cb, key_extra,
    lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T,
    llpos_T, loop_0, loop_0_children as C2Rust_Unnamed_20, lpos_T, mapblock, mapblock_T, match_T,
    matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s, mtnode_s, multiqueue,
    partial_S, partial_T, pos_T, pos_save_T, proc, proc_exit_cb, proc_state_cb, proftime_T,
    pthread_mutex_t, pthread_rwlock_t, ptr_t, qf_info_S, qf_info_T, queue, reg_extmatch_T,
    regmmatch_T, regprog, regprog_T, rstream, sattr_T, save_v_event_T, schar_T, scid_T, sctx_T,
    size_t, ssize_t, state_check_callback, state_execute_callback, stream, stream_close_cb,
    stream_read_cb, stream_uv as C2Rust_Unnamed_22, stream_write_cb, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T, synblock_T, synstate_T, taggy_T, terminal,
    time_t, typebuf_T, typval_T, typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
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
    varnumber_T, vim_state, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T,
    xfmark_T, xp_prefix_T, QUEUE,
};
use crate::src::nvim::ui::ui_flush;
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
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
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
pub const kCmdRedrawAll: CmdRedraw = 2;
pub const kCmdRedrawPos: CmdRedraw = 1;
pub const kCmdRedrawNone: CmdRedraw = 0;
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
pub const kOptVeFlagNoneU: C2Rust_Unnamed_27 = 32;
pub const kOptVeFlagNone: C2Rust_Unnamed_27 = 16;
pub const kOptVeFlagOnemore: C2Rust_Unnamed_27 = 8;
pub const kOptVeFlagInsert: C2Rust_Unnamed_27 = 6;
pub const kOptVeFlagBlock: C2Rust_Unnamed_27 = 5;
pub const kOptVeFlagAll: C2Rust_Unnamed_27 = 4;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub unsafe extern "C" fn state_enter(mut s: *mut VimState) {
    's_132: loop {
        let mut check_result: ::core::ffi::c_int = if (*s).check.is_some() {
            (*s).check.expect("non-null function pointer")(s)
        } else {
            1 as ::core::ffi::c_int
        };
        if check_result == 0 {
            break;
        }
        if check_result == -1 as ::core::ffi::c_int {
            continue;
        }
        let mut key: ::core::ffi::c_int = 0;
        loop {
            if vpeekc() != NUL || (*typebuf.ptr()).tb_len > 0 as ::core::ffi::c_int {
                key = safe_vgetc();
            } else if !multiqueue_empty((*main_loop.ptr()).events) {
                ui_flush();
                key = -(253 as ::core::ffi::c_int
                    + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
            } else {
                if must_redraw.get() != 0 as ::core::ffi::c_int
                    && !need_wait_return.get()
                    && State.get() & MODE_CMDLINE as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                {
                    update_screen();
                    setcursor();
                }
                ui_flush();
                input_get(
                    ::core::ptr::null_mut::<uint8_t>(),
                    0 as ::core::ffi::c_int,
                    -1 as ::core::ffi::c_int,
                    (*typebuf.ptr()).tb_change_cnt,
                    (*main_loop.ptr()).events,
                );
                if !(input_available() == 0 && !multiqueue_empty((*main_loop.ptr()).events)) {
                    continue;
                }
                key = -(253 as ::core::ffi::c_int
                    + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
            }
            if key
                == -(253 as ::core::ffi::c_int
                    + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                check_end_reg_executing(true_0 != 0);
                may_sync_undo();
            }
            let mut keyname: *mut ::core::ffi::c_char = (if key
                == -(253 as ::core::ffi::c_int
                    + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                b"K_EVENT\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                get_special_key_name(key, mod_mask.get()) as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char;
            logmsg(
                LOGLVL_DBG,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"state_enter\0".as_ptr() as *const ::core::ffi::c_char,
                97 as ::core::ffi::c_int,
                true_0 != 0,
                b"input: %s\0".as_ptr() as *const ::core::ffi::c_char,
                keyname,
            );
            let mut execute_result: ::core::ffi::c_int =
                (*s).execute.expect("non-null function pointer")(s, key);
            if execute_result == 0 {
                break 's_132;
            }
            if execute_result != -1 as ::core::ffi::c_int {
                break;
            }
        }
    }
}
pub unsafe extern "C" fn state_handle_k_event() {
    loop {
        let mut event: Event = multiqueue_get((*main_loop.ptr()).events);
        if event.handler.is_some() {
            event.handler.expect("non-null function pointer")(
                &raw mut event.argv as *mut *mut ::core::ffi::c_void,
            );
        }
        if multiqueue_empty((*main_loop.ptr()).events) {
            return;
        }
        os_breakcheck();
        if input_available() != 0 || got_int.get() as ::core::ffi::c_int != 0 {
            return;
        }
    }
}
pub unsafe extern "C" fn virtual_active(mut wp: *mut win_T) -> bool {
    if virtual_op.get() as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
        return virtual_op.get() as u64 != 0;
    }
    if State.get() & MODE_TERMINAL as ::core::ffi::c_int != 0 {
        return true_0 != 0;
    }
    let mut cur_ve_flags: ::core::ffi::c_uint = get_ve_flags(wp);
    return cur_ve_flags == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
        || cur_ve_flags & kOptVeFlagBlock as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && VIsual_active.get() as ::core::ffi::c_int != 0
            && VIsual_mode.get() == Ctrl_V
        || cur_ve_flags & kOptVeFlagInsert as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && State.get() & MODE_INSERT as ::core::ffi::c_int != 0;
}
pub unsafe extern "C" fn get_real_state() -> ::core::ffi::c_int {
    if State.get() & MODE_NORMAL as ::core::ffi::c_int != 0 {
        if VIsual_active.get() {
            if VIsual_select.get() {
                return MODE_SELECT as ::core::ffi::c_int;
            }
            return MODE_VISUAL as ::core::ffi::c_int;
        } else if finish_op.get() {
            return MODE_OP_PENDING as ::core::ffi::c_int;
        }
    }
    return State.get();
}
pub unsafe extern "C" fn get_mode(mut buf: *mut ::core::ffi::c_char) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if State.get() == MODE_HITRETURN as ::core::ffi::c_int
        || State.get() == MODE_ASKMORE as ::core::ffi::c_int
        || State.get() == MODE_SETWSIZE as ::core::ffi::c_int
        || State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0
            && (*get_cmdline_info()).one_key as ::core::ffi::c_int != 0
    {
        let c2rust_fresh0 = i;
        i = i + 1;
        *buf.offset(c2rust_fresh0 as isize) = 'r' as ::core::ffi::c_char;
        if State.get() == MODE_ASKMORE as ::core::ffi::c_int {
            let c2rust_fresh1 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh1 as isize) = 'm' as ::core::ffi::c_char;
        } else if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0 {
            let c2rust_fresh2 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh2 as isize) = '?' as ::core::ffi::c_char;
        }
    } else if State.get() == MODE_EXTERNCMD as ::core::ffi::c_int {
        let c2rust_fresh3 = i;
        i = i + 1;
        *buf.offset(c2rust_fresh3 as isize) = '!' as ::core::ffi::c_char;
    } else if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 {
        if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            let c2rust_fresh4 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh4 as isize) = 'R' as ::core::ffi::c_char;
            let c2rust_fresh5 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh5 as isize) = 'v' as ::core::ffi::c_char;
        } else if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
            let c2rust_fresh6 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh6 as isize) = 'R' as ::core::ffi::c_char;
        } else {
            let c2rust_fresh7 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh7 as isize) = 'i' as ::core::ffi::c_char;
        }
        if ins_compl_active() {
            let c2rust_fresh8 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh8 as isize) = 'c' as ::core::ffi::c_char;
        } else if ctrl_x_mode_not_defined_yet() {
            let c2rust_fresh9 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh9 as isize) = 'x' as ::core::ffi::c_char;
        }
    } else if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0
        || exmode_active.get() as ::core::ffi::c_int != 0
    {
        let c2rust_fresh10 = i;
        i = i + 1;
        *buf.offset(c2rust_fresh10 as isize) = 'c' as ::core::ffi::c_char;
        if exmode_active.get() {
            let c2rust_fresh11 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh11 as isize) = 'v' as ::core::ffi::c_char;
        }
        if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0
            && cmdline_overstrike() as ::core::ffi::c_int != 0
        {
            let c2rust_fresh12 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh12 as isize) = 'r' as ::core::ffi::c_char;
        }
    } else if State.get() & MODE_TERMINAL as ::core::ffi::c_int != 0 {
        let c2rust_fresh13 = i;
        i = i + 1;
        *buf.offset(c2rust_fresh13 as isize) = 't' as ::core::ffi::c_char;
    } else if VIsual_active.get() {
        if VIsual_select.get() {
            let c2rust_fresh14 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh14 as isize) = (VIsual_mode.get() + 's' as ::core::ffi::c_int
                - 'v' as ::core::ffi::c_int)
                as ::core::ffi::c_char;
        } else {
            let c2rust_fresh15 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh15 as isize) = VIsual_mode.get() as ::core::ffi::c_char;
            if restart_VIsual_select.get() != 0 {
                let c2rust_fresh16 = i;
                i = i + 1;
                *buf.offset(c2rust_fresh16 as isize) = 's' as ::core::ffi::c_char;
            }
        }
    } else {
        let c2rust_fresh17 = i;
        i = i + 1;
        *buf.offset(c2rust_fresh17 as isize) = 'n' as ::core::ffi::c_char;
        if finish_op.get() {
            let c2rust_fresh18 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh18 as isize) = 'o' as ::core::ffi::c_char;
            let c2rust_fresh19 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh19 as isize) = motion_force.get() as ::core::ffi::c_char;
        } else if !(*curbuf.get()).terminal.is_null() {
            let c2rust_fresh20 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh20 as isize) = 't' as ::core::ffi::c_char;
            if restart_edit.get() == 'I' as ::core::ffi::c_int {
                let c2rust_fresh21 = i;
                i = i + 1;
                *buf.offset(c2rust_fresh21 as isize) = 'T' as ::core::ffi::c_char;
            }
        } else if restart_edit.get() == 'I' as ::core::ffi::c_int
            || restart_edit.get() == 'R' as ::core::ffi::c_int
            || restart_edit.get() == 'V' as ::core::ffi::c_int
        {
            let c2rust_fresh22 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh22 as isize) = 'i' as ::core::ffi::c_char;
            let c2rust_fresh23 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh23 as isize) = restart_edit.get() as ::core::ffi::c_char;
        }
    }
    *buf.offset(i as isize) = NUL as ::core::ffi::c_char;
}
pub unsafe extern "C" fn may_trigger_modechanged() {
    if !has_event(EVENT_MODECHANGED) || got_int.get() as ::core::ffi::c_int != 0 {
        return;
    }
    let mut curr_mode: [::core::ffi::c_char; 4] = [0; 4];
    let mut pattern_buf: [::core::ffi::c_char; 8] = [0; 8];
    get_mode(&raw mut curr_mode as *mut ::core::ffi::c_char);
    if strcmp(
        &raw mut curr_mode as *mut ::core::ffi::c_char,
        last_mode.ptr() as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        return;
    }
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
    let mut v_event: *mut dict_T = get_v_event(&raw mut save_v_event);
    tv_dict_add_str(
        v_event,
        b"new_mode\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        &raw mut curr_mode as *mut ::core::ffi::c_char,
    );
    tv_dict_add_str(
        v_event,
        b"old_mode\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        last_mode.ptr() as *mut ::core::ffi::c_char,
    );
    tv_dict_set_keys_readonly(v_event);
    vim_snprintf(
        &raw mut pattern_buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>(),
        b"%s:%s\0".as_ptr() as *const ::core::ffi::c_char,
        last_mode.ptr() as *mut ::core::ffi::c_char,
        &raw mut curr_mode as *mut ::core::ffi::c_char,
    );
    apply_autocmds(
        EVENT_MODECHANGED,
        &raw mut pattern_buf as *mut ::core::ffi::c_char,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    strcpy(
        last_mode.ptr() as *mut ::core::ffi::c_char,
        &raw mut curr_mode as *mut ::core::ffi::c_char,
    );
    restore_v_event(v_event, &raw mut save_v_event);
}
static was_safe: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
unsafe extern "C" fn is_safe_now() -> bool {
    return stuff_empty() as ::core::ffi::c_int != 0
        && (*typebuf.ptr()).tb_len == 0 as ::core::ffi::c_int
        && using_script() == 0
        && global_busy.get() == 0
        && !debug_mode.get();
}
pub unsafe extern "C" fn may_trigger_safestate(mut safe: bool) {
    let mut is_safe: bool =
        safe as ::core::ffi::c_int != 0 && is_safe_now() as ::core::ffi::c_int != 0;
    if was_safe.get() as ::core::ffi::c_int != is_safe as ::core::ffi::c_int {
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"may_trigger_safestate\0".as_ptr() as *const ::core::ffi::c_char,
            305 as ::core::ffi::c_int,
            true_0 != 0,
            if is_safe as ::core::ffi::c_int != 0 {
                b"SafeState: Start triggering\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"SafeState: Stop triggering\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
    }
    if is_safe {
        apply_autocmds(
            EVENT_SAFESTATE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    }
    was_safe.set(is_safe);
}
pub unsafe extern "C" fn state_no_longer_safe(mut reason: *const ::core::ffi::c_char) {
    if was_safe.get() as ::core::ffi::c_int != 0 && !reason.is_null() {
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"state_no_longer_safe\0".as_ptr() as *const ::core::ffi::c_char,
            319 as ::core::ffi::c_int,
            true_0 != 0,
            b"SafeState reset: %s\0".as_ptr() as *const ::core::ffi::c_char,
            reason,
        );
    }
    was_safe.set(false_0 != 0);
}
pub unsafe extern "C" fn get_was_safe_state() -> bool {
    return was_safe.get();
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
