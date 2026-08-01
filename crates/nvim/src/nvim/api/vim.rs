use crate::src::nvim::api::buffer::{api_buf_ensure_loaded, nvim_buf_del_keymap};
use crate::src::nvim::api::deprecated::{buffer_del_line, buffer_get_line, buffer_set_line};
use crate::src::nvim::api::private::converter::vim_to_object;
use crate::src::nvim::api::private::helpers::{
    api_metadata, api_set_error, api_set_sctx, api_typename, arena_array, arena_dict, arena_string,
    arena_take_arraybuilder, copy_array, copy_dict, copy_object, copy_string, cstr_as_string,
    dict_get_value, dict_set_var, find_buffer_by_handle, find_tab_by_handle, find_window_by_handle,
    get_default_stl_hl, parse_hl_msg, set_mark, string_to_array, try_enter, try_leave,
};
use crate::src::nvim::api::private::validate::{api_err_exp, api_err_invalid, api_err_required};
use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::autocmd::{
    EVENT_BUFADD, EVENT_BUFNEW, apply_autocmds, block_autocmds, may_trigger_vim_suspend_resume,
    unblock_autocmds,
};
use crate::src::nvim::buffer::buf_get_changedtick;
use crate::src::nvim::buffer::{
    buf_close_terminal, buflist_new, buflist_nr2name, bufref_valid, do_buffer, read_buffer_into,
    set_bufref,
};
use crate::src::nvim::channel::channel_internal;
use crate::src::nvim::channel::find_channel;
use crate::src::nvim::channel::{channel_all_info, channel_info, channel_send};
use crate::src::nvim::channel::{channel_alloc, channel_decref, channel_incref};
use crate::src::nvim::context::{
    ctx_free, ctx_from_dict, ctx_restore, ctx_save, ctx_to_dict, kCtxAll,
};
use crate::src::nvim::cursor::get_cursor_rel_lnum;
use crate::src::nvim::decoration::decor_redraw_signs;
use crate::src::nvim::drawline::use_cursor_line_highlight;
use crate::src::nvim::drawscreen::{
    UPD_CLEAR, UPD_NOT_VALID, UPD_VALID, redraw_all_later, redraw_buf_later,
    redraw_buf_range_later, redraw_later, setcursor_mayforce, update_screen, win_update_cursorline,
};
use crate::src::nvim::eval::typval::tv_dict_find;
use crate::src::nvim::eval::vars::{get_globvar_dict, get_vimvar_dict, set_vim_var_nr};
use crate::src::nvim::ex_docmd::{changedir_func, exec_normal};
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::fold::fold_info;
use crate::src::nvim::getchar::{ins_typebuf, paste_store};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{
    get_win_by_grid_handle, schar_cache_clear, schar_get, win_grid_alloc,
};
use crate::src::nvim::highlight::{
    dict2hlattrs, highlight_use_hlstate, hl_check_ns, hl_get_attr_by_id, hl_inspect,
    hl_ns_get_attrs, ns_hl_def, win_check_ns_hl,
};
use crate::src::nvim::highlight_group::{
    COLOR_NAMES, HLF_CLN, HLF_CLS, HLF_LNA, HLF_LNB, HLF_N, HLF_NONE, HLF_SC, name_to_color,
    ns_get_hl_defs, syn_check_group, syn_id2name,
};
use crate::src::nvim::insexpand::get_cot_flags;
use crate::src::nvim::keycodes::{name_to_mod_mask, replace_termcodes, vim_strsave_escape_ks};
use crate::src::nvim::kvec::_memcpy_free;
use crate::src::nvim::log::{LOGLVL_DBG, logmsg};
use crate::src::nvim::lua::executor::{
    api_free_luaref, nlua_call_ref, nlua_exec, nlua_get_global_ref_count, nlua_is_deferred_safe,
};
use crate::src::nvim::main::{
    Columns, RedrawingDisabled, VIsual_active, arena_alloc_count, cmdpreview, cmdwin_buf, curbuf,
    current_sctx, curtab, curwin, default_grid, did_emsg, e_cmdwin, e_invchan, ex_normal_busy,
    first_tabpage, firstbuf, firstwin, g_stats, lines_left, msg_didany, msg_no_more, msg_scroll,
    msg_silent, must_redraw, need_wait_return, no_wait_return, ns_hl_fast, ns_hl_global, p_cpo,
    p_lz, pum_grid, redraw_tabline, textlock, tslua_query_parse_count, typebuf, typebuf_was_filled,
    vgetc_busy,
};
use crate::src::nvim::mapping::{keymap_array, modify_keymap};
use crate::src::nvim::mark::mark_get_global;
use crate::src::nvim::mbyte::{mb_string2cells, utfc_ptr2len, utfc_ptr2schar};
use crate::src::nvim::memline::ml_open;
use crate::src::nvim::memory::{
    arena_alloc, arena_strdup, memchrsub, strequal, xfree, xmalloc, xrealloc,
};
use crate::src::nvim::message::{
    do_autocmd_progress, hl_msg_free, msg_id_exists, msg_multihl, verbose_enter, verbose_leave,
    verbose_stop,
};
use crate::src::nvim::r#move::{
    changed_window_setting, update_topline, validate_cursor, win_col_off,
};
use crate::src::nvim::msgpack_rpc::channel::rpc_set_client_info;
use crate::src::nvim::msgpack_rpc::unpacker::unpack;
use crate::src::nvim::normal::reset_VIsual_and_resel;
use crate::src::nvim::option::{buf_copy_options, set_option_direct_for};
use crate::src::nvim::options::{kOptBufhidden, kOptBuftype, kOptCotFlagPopup, kOptInvalid};
use crate::src::nvim::optionstr::check_stl_option;
use crate::src::nvim::os::input::{
    input_blocking, input_enqueue, input_enqueue_mouse, input_enqueue_raw,
};
use crate::src::nvim::os::libc::{__assert_fail, labs, memcmp, memcpy, snprintf, strlen};
use crate::src::nvim::os::proc::os_proc_children;
use crate::src::nvim::popupmenu::{pum_ext_select_item, pum_set_info};
use crate::src::nvim::register::{do_put, finish_yankreg_from_object, prepare_yankreg_from_object};
use crate::src::nvim::runtime::{
    do_in_runtimepath, do_source, get_lib_dir, runtime_get_named, runtime_inspect, script_autoload,
};
use crate::src::nvim::search::{BACKWARD, FORWARD};
use crate::src::nvim::state::get_mode;
use crate::src::nvim::statusline::{
    build_stl_str_hl, draw_tabline, fillchar_status, win_redr_status, win_redr_winbar,
};
use crate::src::nvim::terminal::{
    terminal_alloc, terminal_buf, terminal_check_size, terminal_destroy, terminal_open,
    terminal_running, terminal_set_streamed_paste,
};
use crate::src::nvim::types::api::{kErrorTypeException, kErrorTypeNone, kErrorTypeValidation};
pub use crate::src::nvim::types::{
    __gid_t, __pthread_internal_list, __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t,
    __time_t, __uid_t, AdditionalData, AlignTextPos, ApiDispatchWrapper, Arena, ArenaMem, Array,
    ArrayBuilder, BoolVarValue, Boolean, BufUpdateCallbacks, Buffer, Callback,
    Callback_data as C2Rust_Unnamed_5, CallbackReader, CallbackType, CdScope, ChangedtickDictItem,
    Channel, Channel_stream, ChannelCallFrame, ChannelStreamType, ClientType, Context, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_2, Dict, DoInRuntimepathCB, Error, ErrorType,
    ExtmarkUndoObject, FileID, Float, FloatAnchor, FloatRelative, GridLineEvent, GridView,
    HLGroupID, HlAttrs, HlMessage, HlMessageChunk, Integer, InternalState, Intersection,
    KeyDict_complete_set, KeyDict_context, KeyDict_echo_opts, KeyDict_empty,
    KeyDict_eval_statusline, KeyDict_get_highlight, KeyDict_get_ns, KeyDict_highlight,
    KeyDict_keymap, KeyDict_open_term, KeyDict_redraw, KeyDict_runtime, KeyValuePair, LibuvProc,
    Loop, LuaRef, LuaRetMode, MTKey, MTNode, MTPos, Map_int64_t_int64_t, Map_int64_t_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MapHash, MarkTree, MessageData, MessageType,
    MotionType, MsgpackRpcRequestHandler, MultiQueue, NS, Object, ObjectType, OptIndex, OptInt,
    OptScope, OptVal, OptValData, OptValType, OptionalKeys, PackerBuffer, PackerBufferFlush, Proc,
    ProcType, PtyProc, QUEUE, RStream, RemapValues, RemoteUI, RgbValue, RpcState,
    ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SignTextAttrs, SpecialVarValue, StderrState, StdioPair, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, StlClickRecord, StlFlag, Stream, String_0,
    StringBuilder, Tabpage, Terminal, TerminalOptions, Timestamp, TriState, TryState,
    UIClientHandler, Unpacker, VarLockStatus, VarType, VimVarIndex, VirtLines, VirtText,
    VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle, Window, alist_T,
    auto_event, bhdr_T, bln_values, blob_T, blobvar_S, blocknr_T, buf_T, bufref_T, bufstate_T,
    chunksize_T, colnr_T, color_name_table_T, consumed_blk, dict_T, dictitem_T, dictvar_S, diff_T,
    diffblock_S, disptick_T, dobuf_action_values, dobuf_start_values, event_T, except_T,
    except_type_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_3, file_buffer_b_wininfo as C2Rust_Unnamed_11,
    file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, foldinfo_T,
    frame_S, frame_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T,
    gid_t, handle_T, hash_T, hashitem_T, hashtab_T, hlf_T, infoptr_T, int16_t, int32_t, int64_t,
    internal_proc_cb, kObjectTypeArray, kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict,
    kObjectTypeInteger, kObjectTypeNil, kObjectTypeString, kObjectTypeTabpage, kObjectTypeWindow,
    key_extra, key_value_pair, lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S,
    listwatch_S, listwatch_T, llpos_T, loop_0, lpos_T, mapblock, mapblock_T, match_T, matchitem,
    matchitem_T, memfile_T, memline_T, mfdirty_T, mpack_data_t, mpack_node_s, mpack_node_t,
    mpack_parser_t, mpack_sintmax_t, mpack_tokbuf_s, mpack_tokbuf_t, mpack_token_s,
    mpack_token_s_data as C2Rust_Unnamed_31, mpack_token_t, mpack_token_type_t, mpack_uint32_t,
    mpack_uintmax_t, mpack_value_s, mpack_value_t, msg_data, msglist, msglist_T, mtnode_inner_s,
    mtnode_s, multiqueue, nvim_stats_s, object, object_data as C2Rust_Unnamed, packer_buffer_t,
    partial_S, partial_T, pos_T, pos_save_T, proc, proc_exit_cb, proc_state_cb, proftime_T,
    pthread_mutex_t, pthread_rwlock_t, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue,
    reg_extmatch_T, regmmatch_T, regprog, regprog_T, rstream, sattr_T, schar_T, scid_T, sctx_T,
    size_t, ssize_t, statuscol_T, stl_hlrec, stl_hlrec_t, stream, stream_close_cb, stream_read_cb,
    stream_uv as C2Rust_Unnamed_23, stream_write_cb, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, tabpage_S,
    tabpage_T, taggy_T, terminal, terminal_close_cb, terminal_read_pause_cb, terminal_resize_cb,
    terminal_resume_cb, terminal_write_cb, time_t, typebuf_T, typval_T, typval_vval_union, u_entry,
    u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8,
    u_header_uh_alt_prev as C2Rust_Unnamed_7, u_header_uh_next as C2Rust_Unnamed_10,
    u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T, uid_t, uint8_t, uint16_t, uint32_t,
    uint64_t, undo_object, uv__io_cb, uv__io_s, uv__io_t, uv__queue, uv_alloc_cb, uv_async_cb,
    uv_async_s, uv_async_s_u as C2Rust_Unnamed_18, uv_async_t, uv_buf_t, uv_close_cb,
    uv_connect_cb, uv_connect_s, uv_connect_t, uv_connection_cb, uv_exit_cb, uv_file, uv_gid_t,
    uv_handle_s, uv_handle_s_u as C2Rust_Unnamed_13, uv_handle_t, uv_handle_type, uv_idle_cb,
    uv_idle_s, uv_idle_s_u as C2Rust_Unnamed_24, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_17, uv_loop_s_timer_heap as C2Rust_Unnamed_16,
    uv_loop_t, uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_26, uv_pipe_t,
    uv_process_options_s, uv_process_options_t, uv_process_s, uv_process_s_u as C2Rust_Unnamed_28,
    uv_process_t, uv_read_cb, uv_req_type, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s,
    uv_shutdown_t, uv_signal_cb, uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed_14,
    uv_signal_s_u as C2Rust_Unnamed_15, uv_signal_t, uv_stdio_container_s,
    uv_stdio_container_s_data as C2Rust_Unnamed_29, uv_stdio_container_t, uv_stdio_flags,
    uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_22, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_25, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_19, uv_timer_s_u as C2Rust_Unnamed_20, uv_timer_t, uv_uid_t,
    varnumber_T, vim_exception, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T,
    winsize, wline_T, xfmark_T, yankreg_T,
};
use crate::src::nvim::ui::{ui_array, ui_call_screenshot, ui_flush};
use crate::src::nvim::window::{
    global_stl_height, goto_tabpage_tp, goto_tabpage_win, win_find_tabpage,
};
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub const kVPosWinCol: VirtTextPos = 5;
pub const kVPosRightAlign: VirtTextPos = 4;
pub const kVPosOverlay: VirtTextPos = 3;
pub const kVPosInline: VirtTextPos = 2;
pub const kVPosEndOfLineRightAlign: VirtTextPos = 1;
pub const kVPosEndOfLine: VirtTextPos = 0;
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
pub const kMessageTypeRedrawEvent: MessageType = 3;
pub const kMessageTypeNotification: MessageType = 2;
pub const kMessageTypeResponse: MessageType = 1;
pub const kMessageTypeRequest: MessageType = 0;
pub const kMessageTypeUnknown: MessageType = -1;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_int;
pub const kDirectionNotSet: C2Rust_Unnamed_27 = 0;
pub const kCdScopeGlobal: CdScope = 2;
pub const kCdScopeTabpage: CdScope = 1;
pub const kCdScopeWindow: CdScope = 0;
pub const kCdScopeInvalid: CdScope = -1;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
pub const kOptScopeBuf: OptScope = 2;
pub const kOptScopeWin: OptScope = 1;
pub const kOptScopeGlobal: OptScope = 0;
pub const UV_OVERLAPPED_PIPE: uv_stdio_flags = 64;
pub const UV_NONBLOCK_PIPE: uv_stdio_flags = 64;
pub const UV_WRITABLE_PIPE: uv_stdio_flags = 32;
pub const UV_READABLE_PIPE: uv_stdio_flags = 16;
pub const UV_INHERIT_STREAM: uv_stdio_flags = 4;
pub const UV_INHERIT_FD: uv_stdio_flags = 2;
pub const UV_CREATE_PIPE: uv_stdio_flags = 1;
pub const UV_IGNORE: uv_stdio_flags = 0;
pub const STL_CLICK_FUNC: StlFlag = 64;
pub const STL_TABCLOSENR: StlFlag = 88;
pub const STL_TABPAGENR: StlFlag = 84;
pub const STL_HIGHLIGHT_COMB: StlFlag = 36;
pub const STL_HIGHLIGHT: StlFlag = 35;
pub const STL_USER_HL: StlFlag = 42;
pub const STL_TRUNCMARK: StlFlag = 60;
pub const STL_SEPARATE: StlFlag = 61;
pub const STL_VIM_EXPR: StlFlag = 123;
pub const STL_SIGNCOL: StlFlag = 115;
pub const STL_FOLDCOL: StlFlag = 67;
pub const STL_SHOWCMD: StlFlag = 83;
pub const STL_PAGENUM: StlFlag = 78;
pub const STL_ARGLISTSTAT: StlFlag = 97;
pub const STL_ALTPERCENT: StlFlag = 80;
pub const STL_PERCENTAGE: StlFlag = 112;
pub const STL_QUICKFIX: StlFlag = 113;
pub const STL_MODIFIED_ALT: StlFlag = 77;
pub const STL_MODIFIED: StlFlag = 109;
pub const STL_PREVIEWFLAG_ALT: StlFlag = 87;
pub const STL_PREVIEWFLAG: StlFlag = 119;
pub const STL_FILETYPE_ALT: StlFlag = 89;
pub const STL_FILETYPE: StlFlag = 121;
pub const STL_HELPFLAG_ALT: StlFlag = 72;
pub const STL_HELPFLAG: StlFlag = 104;
pub const STL_ROFLAG_ALT: StlFlag = 82;
pub const STL_ROFLAG: StlFlag = 114;
pub const STL_BYTEVAL_X: StlFlag = 66;
pub const STL_BYTEVAL: StlFlag = 98;
pub const STL_OFFSET_X: StlFlag = 79;
pub const STL_OFFSET: StlFlag = 111;
pub const STL_KEYMAP: StlFlag = 107;
pub const STL_BUFNO: StlFlag = 110;
pub const STL_NUMLINES: StlFlag = 76;
pub const STL_LINE: StlFlag = 108;
pub const STL_VIRTCOL_ALT: StlFlag = 86;
pub const STL_VIRTCOL: StlFlag = 118;
pub const STL_COLUMN: StlFlag = 99;
pub const STL_FILENAME: StlFlag = 116;
pub const STL_FULLPATH: StlFlag = 70;
pub const STL_FILEPATH: StlFlag = 102;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub const KE_LEFTMOUSE: key_extra = 44;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const REPTERM_NO_SPECIAL: C2Rust_Unnamed_36 = 4;
pub const REPTERM_DO_LT: C2Rust_Unnamed_36 = 2;
pub const REPTERM_FROM_PART: C2Rust_Unnamed_36 = 1;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RuntimeCookie {
    pub rv: ArrayBuilder,
    pub arena: *mut Arena,
}
pub const DIP_ALL: C2Rust_Unnamed_41 = 1;
pub const DIP_DIRFILE: C2Rust_Unnamed_41 = 512;
pub const DOSO_NONE: C2Rust_Unnamed_40 = 0;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const DOBUF_GOTO: dobuf_action_values = 0;
pub const BLN_LISTED: bln_values = 2;
pub const BLN_NEW: bln_values = 8;
pub const BLN_NOOPT: bln_values = 16;
pub const NUM_EVENTS: auto_event = 145;
pub const OPT_LOCAL: C2Rust_Unnamed_38 = 2;
pub const BCO_NOHELP: C2Rust_Unnamed_37 = 4;
pub const BCO_ENTER: C2Rust_Unnamed_37 = 1;
pub const kClientTypePlugin: ClientType = 4;
pub const kClientTypeHost: ClientType = 3;
pub const kClientTypeEmbedder: ClientType = 2;
pub const kClientTypeUi: ClientType = 1;
pub const kClientTypeMsgpackRpc: ClientType = 5;
pub const kClientTypeRemote: ClientType = 0;
pub const kClientTypeUnknown: ClientType = -1;
pub const MPACK_TOKEN_EXT: mpack_token_type_t = 11;
pub const MPACK_TOKEN_STR: mpack_token_type_t = 10;
pub const MPACK_TOKEN_BIN: mpack_token_type_t = 9;
pub const MPACK_TOKEN_MAP: mpack_token_type_t = 8;
pub const MPACK_TOKEN_ARRAY: mpack_token_type_t = 7;
pub const MPACK_TOKEN_CHUNK: mpack_token_type_t = 6;
pub const MPACK_TOKEN_FLOAT: mpack_token_type_t = 5;
pub const MPACK_TOKEN_SINT: mpack_token_type_t = 4;
pub const MPACK_TOKEN_UINT: mpack_token_type_t = 3;
pub const MPACK_TOKEN_BOOLEAN: mpack_token_type_t = 2;
pub const MPACK_TOKEN_NIL: mpack_token_type_t = 1;
pub const kChannelStreamInternal: ChannelStreamType = 4;
pub const kChannelStreamStderr: ChannelStreamType = 3;
pub const kChannelStreamStdio: ChannelStreamType = 2;
pub const kChannelStreamSocket: ChannelStreamType = 1;
pub const kChannelStreamProc: ChannelStreamType = 0;
pub const PUT_CURSEND: C2Rust_Unnamed_39 = 2;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub const kCtxFuncs: C2Rust_Unnamed_33 = 32;
pub const kCtxSFuncs: C2Rust_Unnamed_33 = 16;
pub const kCtxGVars: C2Rust_Unnamed_33 = 8;
pub const kCtxBufs: C2Rust_Unnamed_33 = 4;
pub const kCtxJumps: C2Rust_Unnamed_33 = 2;
pub const kCtxRegs: C2Rust_Unnamed_33 = 1;
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
pub const BLN_NOCURWIN: bln_values = 128;
pub const BLN_DUMMY: bln_values = 4;
pub const BLN_CURBUF: bln_values = 1;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_SPLIT: dobuf_action_values = 1;
pub const DOBUF_MOD: dobuf_start_values = 3;
pub const DOBUF_LAST: dobuf_start_values = 2;
pub const DOBUF_CURRENT: dobuf_start_values = 0;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const REMAP_SKIP: RemapValues = -3;
pub const REMAP_SCRIPT: RemapValues = -2;
pub const KE_WILD: key_extra = 108;
pub const KE_COMMAND: key_extra = 104;
pub const KE_LUA: key_extra = 103;
pub const KE_EVENT: key_extra = 102;
pub const KE_NOP: key_extra = 97;
pub const KE_DROP: key_extra = 95;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_C_END: key_extra = 88;
pub const KE_C_HOME: key_extra = 87;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_CMDWIN: key_extra = 84;
pub const KE_PLUG: key_extra = 83;
pub const KE_SNR: key_extra = 82;
pub const KE_KDEL: key_extra = 80;
pub const KE_KINS: key_extra = 79;
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
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_MIDDLEDRAG: key_extra = 48;
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
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub const REPTERM_NO_SIMPLIFY: C2Rust_Unnamed_36 = 8;
pub type C2Rust_Unnamed_37 = ::core::ffi::c_uint;
pub const BCO_ALWAYS: C2Rust_Unnamed_37 = 2;
pub type C2Rust_Unnamed_38 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_38 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_38 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_38 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_38 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_38 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_38 = 4;
pub const OPT_GLOBAL: C2Rust_Unnamed_38 = 1;
pub type C2Rust_Unnamed_39 = ::core::ffi::c_uint;
pub const PUT_BLOCK_INNER: C2Rust_Unnamed_39 = 64;
pub const PUT_LINE_FORWARD: C2Rust_Unnamed_39 = 32;
pub const PUT_LINE_SPLIT: C2Rust_Unnamed_39 = 16;
pub const PUT_LINE: C2Rust_Unnamed_39 = 8;
pub const PUT_CURSLINE: C2Rust_Unnamed_39 = 4;
pub const PUT_FIXINDENT: C2Rust_Unnamed_39 = 1;
pub type C2Rust_Unnamed_40 = ::core::ffi::c_uint;
pub const DOSO_VIMRC: C2Rust_Unnamed_40 = 1;
pub type C2Rust_Unnamed_41 = ::core::ffi::c_uint;
pub const DIP_AFTER: C2Rust_Unnamed_41 = 128;
pub const DIP_NOAFTER: C2Rust_Unnamed_41 = 64;
pub const DIP_NORTP: C2Rust_Unnamed_41 = 32;
pub const DIP_OPT: C2Rust_Unnamed_41 = 16;
pub const DIP_START: C2Rust_Unnamed_41 = 8;
pub const DIP_ERR: C2Rust_Unnamed_41 = 4;
pub const DIP_DIR: C2Rust_Unnamed_41 = 2;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const MAX_SCHAR_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const STRING_INIT: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const INTERNAL_CALL_MASK: uint64_t = (1 as ::core::ffi::c_int as uint64_t)
    << ::core::mem::size_of::<uint64_t>()
        .wrapping_mul(8 as usize)
        .wrapping_sub(1 as usize);
#[inline(always)]
unsafe extern "C" fn is_internal_call(channel_id: uint64_t) -> bool {
    return channel_id & INTERNAL_CALL_MASK != 0;
}
pub const KEYSET_OPTIDX_context__types: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_eval_statusline__fillchar: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_eval_statusline__maxwidth: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_eval_statusline__use_statuscol_lnum: ::core::ffi::c_int =
    7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__url: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__update: ::core::ffi::c_int = 13 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_ns__winid: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_open_term__on_input: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_open_term__force_crlf: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_complete_set__info: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_redraw__buf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_redraw__win: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_redraw__flush: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_redraw__range: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_redraw__valid: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const NULL_STRING: String_0 = STRING_INIT;
pub unsafe extern "C" fn nvim_get_hl_id_by_name(mut name: String_0) -> Integer {
    return syn_check_group(name.data, name.size) as Integer;
}
pub unsafe extern "C" fn nvim_get_hl(
    mut ns_id: Integer,
    mut opts: *mut KeyDict_get_highlight,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    return ns_get_hl_defs(ns_id as NS, opts, arena, err);
}
pub unsafe extern "C" fn nvim_set_hl(
    mut channel_id: uint64_t,
    mut ns_id: Integer,
    mut name: String_0,
    mut val: *mut KeyDict_highlight,
    mut err: *mut Error,
) {
    let mut hl_id: ::core::ffi::c_int = syn_check_group(name.data, name.size);
    if !(hl_id != 0 as ::core::ffi::c_int) {
        api_err_invalid(
            err,
            b"highlight name\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return;
    }
    let mut link_id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if (*val).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__url
        != 0 as ::core::ffi::c_ulonglong
    {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Invalid key: 'url'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut update: bool = (*val).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__update
        != 0 as ::core::ffi::c_ulonglong
        && (*val).update as ::core::ffi::c_int != 0;
    let mut base: Option<&HlAttrs> = None;
    let mut base_attrs: HlAttrs = HlAttrs {
        rgb_ae_attr: 0,
        cterm_ae_attr: 0,
        rgb_fg_color: 0,
        rgb_bg_color: 0,
        rgb_sp_color: 0,
        cterm_fg_color: 0,
        cterm_bg_color: 0,
        hl_blend: 0,
        url: 0,
    };
    if update as ::core::ffi::c_int != 0 {
        if let Some(attrs) = hl_ns_get_attrs(ns_id as ::core::ffi::c_int, hl_id, None) {
            base_attrs = attrs;
            base = Some(&base_attrs);
        }
    }
    let mut attrs: HlAttrs = dict2hlattrs(&*val, true_0 != 0, Some(&mut link_id), base, err);
    if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        let save_current_sctx: sctx_T = api_set_sctx(channel_id);
        ns_hl_def(ns_id as NS, hl_id, attrs, link_id, val);
        current_sctx.set(save_current_sctx);
    }
}
pub unsafe extern "C" fn nvim_get_hl_ns(
    mut opts: *mut KeyDict_get_ns,
    mut err: *mut Error,
) -> Integer {
    if (*opts).is_set__get_ns_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_ns__winid
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut win: *mut win_T = find_window_by_handle((*opts).winid, err);
        if win.is_null() {
            return 0 as Integer;
        }
        return (*win).w_ns_hl as Integer;
    } else {
        return ns_hl_global.get() as Integer;
    };
}
pub unsafe extern "C" fn nvim_set_hl_ns(mut ns_id: Integer, mut err: *mut Error) {
    if !(ns_id >= 0 as Integer) {
        api_err_invalid(
            err,
            b"namespace\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ns_id as int64_t,
            false_0 != 0,
        );
        return;
    }
    ns_hl_global.set(ns_id as NS);
    hl_check_ns();
    redraw_all_later(UPD_NOT_VALID);
}
pub unsafe extern "C" fn nvim_set_hl_ns_fast(mut ns_id: Integer, mut _err: *mut Error) {
    ns_hl_fast.set(ns_id as NS);
    hl_check_ns();
}
pub unsafe extern "C" fn nvim_feedkeys(
    mut keys: String_0,
    mut mode: String_0,
    mut escape_ks: Boolean,
) {
    let mut remap: bool = true_0 != 0;
    let mut insert: bool = false_0 != 0;
    let mut typed: bool = false_0 != 0;
    let mut execute: bool = false_0 != 0;
    let mut dangerous: bool = false_0 != 0;
    let mut lowlevel: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < mode.size {
        match *mode.data.offset(i as isize) as ::core::ffi::c_int {
            110 => {
                remap = false_0 != 0;
            }
            109 => {
                remap = true_0 != 0;
            }
            116 => {
                typed = true_0 != 0;
            }
            105 => {
                insert = true_0 != 0;
            }
            120 => {
                execute = true_0 != 0;
            }
            33 => {
                dangerous = true_0 != 0;
            }
            76 => {
                lowlevel = true_0 != 0;
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
    if keys.size == 0 as size_t && !execute {
        return;
    }
    let mut keys_esc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if escape_ks {
        keys_esc = vim_strsave_escape_ks(keys.data);
    } else {
        keys_esc = keys.data;
    }
    if lowlevel {
        input_enqueue_raw(keys_esc, strlen(keys_esc));
    } else {
        ins_typebuf(
            keys_esc,
            if remap as ::core::ffi::c_int != 0 {
                REMAP_YES as ::core::ffi::c_int
            } else {
                REMAP_NONE as ::core::ffi::c_int
            },
            if insert as ::core::ffi::c_int != 0 {
                0 as ::core::ffi::c_int
            } else {
                (*typebuf.ptr()).tb_len
            },
            !typed,
            false_0 != 0,
        );
        if vgetc_busy.get() != 0 {
            typebuf_was_filled.set(true_0 != 0);
        }
    }
    if escape_ks {
        xfree(keys_esc as *mut ::core::ffi::c_void);
    }
    if execute {
        let mut save_msg_scroll: ::core::ffi::c_int = msg_scroll.get();
        msg_scroll.set(false_0);
        if !dangerous {
            (*ex_normal_busy.ptr()) += 1;
        }
        exec_normal(true_0 != 0, lowlevel);
        if !dangerous {
            (*ex_normal_busy.ptr()) -= 1;
        }
        (*msg_scroll.ptr()) |= save_msg_scroll;
    }
}
pub unsafe extern "C" fn nvim_input(mut channel_id: uint64_t, mut keys: String_0) -> Integer {
    may_trigger_vim_suspend_resume(false_0 != 0);
    return input_enqueue(channel_id, keys) as Integer;
}
pub unsafe extern "C" fn nvim_input_mouse(
    mut button: String_0,
    mut action: String_0,
    mut modifier: String_0,
    mut grid: Integer,
    mut row: Integer,
    mut col: Integer,
    mut err: *mut Error,
) {
    let mut code: ::core::ffi::c_int = 0;
    let mut modmask: ::core::ffi::c_int = 0;
    may_trigger_vim_suspend_resume(false_0 != 0);
    '_error: {
        if !(button.data.is_null() || action.data.is_null()) {
            code = 0 as ::core::ffi::c_int;
            if strequal(
                button.data,
                b"left\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                code = KE_LEFTMOUSE as ::core::ffi::c_int;
            } else if strequal(
                button.data,
                b"middle\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                code = KE_MIDDLEMOUSE as ::core::ffi::c_int;
            } else if strequal(
                button.data,
                b"right\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                code = KE_RIGHTMOUSE as ::core::ffi::c_int;
            } else if strequal(
                button.data,
                b"wheel\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                code = KE_MOUSEDOWN as ::core::ffi::c_int;
            } else if strequal(button.data, b"x1\0".as_ptr() as *const ::core::ffi::c_char) {
                code = KE_X1MOUSE as ::core::ffi::c_int;
            } else if strequal(button.data, b"x2\0".as_ptr() as *const ::core::ffi::c_char) {
                code = KE_X2MOUSE as ::core::ffi::c_int;
            } else if strequal(
                button.data,
                b"move\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                code = KE_MOUSEMOVE as ::core::ffi::c_int;
            } else {
                break '_error;
            }
            if code == KE_MOUSEDOWN as ::core::ffi::c_int {
                if strequal(
                    action.data,
                    b"down\0".as_ptr() as *const ::core::ffi::c_char,
                ) {
                    code = KE_MOUSEUP as ::core::ffi::c_int;
                } else if !strequal(action.data, b"up\0".as_ptr() as *const ::core::ffi::c_char) {
                    if strequal(
                        action.data,
                        b"left\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        code = KE_MOUSERIGHT as ::core::ffi::c_int;
                    } else if strequal(
                        action.data,
                        b"right\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        code = KE_MOUSELEFT as ::core::ffi::c_int;
                    } else {
                        break '_error;
                    }
                }
            } else if code != KE_MOUSEMOVE as ::core::ffi::c_int {
                if !strequal(
                    action.data,
                    b"press\0".as_ptr() as *const ::core::ffi::c_char,
                ) {
                    if strequal(
                        action.data,
                        b"drag\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        code +=
                            KE_LEFTDRAG as ::core::ffi::c_int - KE_LEFTMOUSE as ::core::ffi::c_int;
                    } else if strequal(
                        action.data,
                        b"release\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        code += KE_LEFTRELEASE as ::core::ffi::c_int
                            - KE_LEFTMOUSE as ::core::ffi::c_int;
                    } else {
                        break '_error;
                    }
                }
            }
            modmask = 0 as ::core::ffi::c_int;
            let mut i: size_t = 0 as size_t;
            while i < modifier.size {
                let mut byte: ::core::ffi::c_char = *modifier.data.offset(i as isize);
                if byte as ::core::ffi::c_int != '-' as ::core::ffi::c_int {
                    let mut mod_0: ::core::ffi::c_int =
                        name_to_mod_mask(byte as ::core::ffi::c_int);
                    if !(mod_0 != 0 as ::core::ffi::c_int) {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            b"Invalid modifier: %c\0".as_ptr() as *const ::core::ffi::c_char,
                            byte as ::core::ffi::c_int,
                        );
                        return;
                    }
                    modmask |= mod_0;
                }
                i = i.wrapping_add(1);
            }
            input_enqueue_mouse(
                code,
                modmask as uint8_t,
                grid as ::core::ffi::c_int,
                row as ::core::ffi::c_int,
                col as ::core::ffi::c_int,
            );
            return;
        }
    }
    api_set_error(
        err,
        kErrorTypeValidation,
        b"invalid button or action\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
pub unsafe extern "C" fn nvim_replace_termcodes(
    mut str: String_0,
    mut from_part: Boolean,
    mut do_lt: Boolean,
    mut special: Boolean,
) -> String_0 {
    if str.size == 0 as size_t {
        return String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0 as size_t,
        };
    }
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if from_part {
        flags |= REPTERM_FROM_PART as ::core::ffi::c_int;
    }
    if do_lt {
        flags |= REPTERM_DO_LT as ::core::ffi::c_int;
    }
    if !special {
        flags |= REPTERM_NO_SPECIAL as ::core::ffi::c_int;
    }
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    replace_termcodes(
        str.data,
        str.size,
        &raw mut ptr,
        0 as scid_T,
        flags,
        ::core::ptr::null_mut::<bool>(),
        p_cpo.get(),
    );
    return cstr_as_string(ptr);
}
pub unsafe extern "C" fn nvim_exec_lua(
    mut code: String_0,
    mut args: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    return nlua_exec(
        code,
        ::core::ptr::null::<::core::ffi::c_char>(),
        args,
        kRetObject,
        arena,
        err,
    );
}
pub unsafe extern "C" fn nvim__exec_lua_fast(
    mut code: String_0,
    mut args: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    return nvim_exec_lua(code, args, arena, err);
}
pub unsafe extern "C" fn nvim_strwidth(mut text: String_0, mut err: *mut Error) -> Integer {
    if !(text.size <= 2147483647 as ::core::ffi::c_int as size_t) {
        api_err_invalid(
            err,
            b"text length\0".as_ptr() as *const ::core::ffi::c_char,
            b"(too long)\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            true_0 != 0,
        );
        return 0 as Integer;
    }
    return mb_string2cells(text.data) as Integer;
}
pub unsafe extern "C" fn nvim_list_runtime_paths(
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    return nvim_get_runtime_file(NULL_STRING, true_0 != 0, arena, err);
}
pub unsafe extern "C" fn nvim__runtime_inspect(mut arena: *mut Arena) -> Array {
    return runtime_inspect(arena);
}
pub unsafe extern "C" fn nvim_get_runtime_file(
    mut name: String_0,
    mut all: Boolean,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut cookie: RuntimeCookie = RuntimeCookie {
        rv: ArrayBuilder {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
            init_array: [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            }; 16],
        },
        arena: arena,
    };
    cookie.rv.capacity = ::core::mem::size_of::<[Object; 16]>()
        .wrapping_div(::core::mem::size_of::<Object>())
        .wrapping_div(
            (::core::mem::size_of::<[Object; 16]>().wrapping_rem(::core::mem::size_of::<Object>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    cookie.rv.size = 0 as size_t;
    cookie.rv.items = &raw mut cookie.rv.init_array as *mut Object;
    let mut flags: ::core::ffi::c_int = DIP_DIRFILE as ::core::ffi::c_int
        | (if all as ::core::ffi::c_int != 0 {
            DIP_ALL as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    do_in_runtimepath(
        (if name.size != 0 {
            name.data as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char,
        flags,
        Some(
            find_runtime_cb
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *mut ::core::ffi::c_char,
                    bool,
                    *mut ::core::ffi::c_void,
                ) -> bool,
        ),
        &raw mut cookie as *mut ::core::ffi::c_void,
    );
    try_leave(&raw mut tstate, err);
    return arena_take_arraybuilder(arena, &raw mut cookie.rv);
}
unsafe extern "C" fn find_runtime_cb(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut c: *mut ::core::ffi::c_void,
) -> bool {
    let mut cookie: *mut RuntimeCookie = c as *mut RuntimeCookie;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_fnames {
        if (*cookie).rv.size == (*cookie).rv.capacity {
            (*cookie).rv.capacity = if (*cookie).rv.capacity << 1 as ::core::ffi::c_int
                > ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                (*cookie).rv.capacity << 1 as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            (*cookie).rv.items = (if (*cookie).rv.capacity
                == ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if (*cookie).rv.items == &raw mut (*cookie).rv.init_array as *mut Object {
                    (*cookie).rv.items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut (*cookie).rv.init_array as *mut Object as *mut ::core::ffi::c_void,
                        (*cookie).rv.items as *mut ::core::ffi::c_void,
                        (*cookie)
                            .rv
                            .size
                            .wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                }
            } else {
                if (*cookie).rv.items == &raw mut (*cookie).rv.init_array as *mut Object {
                    memcpy(
                        xmalloc(
                            (*cookie)
                                .rv
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<Object>()),
                        ),
                        (*cookie).rv.items as *const ::core::ffi::c_void,
                        (*cookie)
                            .rv
                            .size
                            .wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                } else {
                    xrealloc(
                        (*cookie).rv.items as *mut ::core::ffi::c_void,
                        (*cookie)
                            .rv
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                }
            }) as *mut Object;
        } else {
        };
        let c2rust_fresh0 = (*cookie).rv.size;
        (*cookie).rv.size = (*cookie).rv.size.wrapping_add(1);
        *(*cookie).rv.items.offset(c2rust_fresh0 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: arena_string((*cookie).arena, cstr_as_string(*fnames.offset(i as isize))),
            },
        };
        if !all {
            return true_0 != 0;
        }
        i += 1;
    }
    return num_fnames > 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn nvim__get_lib_dir() -> String_0 {
    return cstr_as_string(get_lib_dir());
}
pub unsafe extern "C" fn nvim__get_runtime(
    mut pat: Array,
    mut all: Boolean,
    mut opts: *mut KeyDict_runtime,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    if !(!(*opts).do_source || nlua_is_deferred_safe() as ::core::ffi::c_int != 0) {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"'do_source' used in fast callback\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
    }
    let mut res: Array = runtime_get_named((*opts).is_lua as bool, pat, all as bool, arena);
    if (*opts).do_source {
        let mut i: size_t = 0 as size_t;
        while i < res.size {
            let mut name: String_0 = (*res.items.offset(i as isize)).data.string;
            do_source(
                name.data,
                false_0 != 0,
                DOSO_NONE as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            );
            i = i.wrapping_add(1);
        }
    }
    return res;
}
pub unsafe extern "C" fn nvim_set_current_dir(mut dir: String_0, mut err: *mut Error) {
    if !(dir.size < 4096 as size_t) {
        api_err_invalid(
            err,
            b"directory name\0".as_ptr() as *const ::core::ffi::c_char,
            b"(too long)\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            true_0 != 0,
        );
        return;
    }
    let mut string: [::core::ffi::c_char; 4096] = [0; 4096];
    memcpy(
        &raw mut string as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        dir.data as *const ::core::ffi::c_void,
        dir.size,
    );
    string[dir.size as usize] = NUL as ::core::ffi::c_char;
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    changedir_func(&raw mut string as *mut ::core::ffi::c_char, kCdScopeGlobal);
    try_leave(&raw mut tstate, err);
}
pub unsafe extern "C" fn nvim_get_current_line(
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> String_0 {
    return buffer_get_line(
        (*curbuf.get()).handle as Buffer,
        ((*curwin.get()).w_cursor.lnum - 1 as linenr_T) as Integer,
        arena,
        err,
    );
}
pub unsafe extern "C" fn nvim_set_current_line(
    mut line: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    buffer_set_line(
        (*curbuf.get()).handle as Buffer,
        ((*curwin.get()).w_cursor.lnum - 1 as linenr_T) as Integer,
        line,
        arena,
        err,
    );
}
pub unsafe extern "C" fn nvim_del_current_line(mut arena: *mut Arena, mut err: *mut Error) {
    buffer_del_line(
        (*curbuf.get()).handle as Buffer,
        ((*curwin.get()).w_cursor.lnum - 1 as linenr_T) as Integer,
        arena,
        err,
    );
}
pub unsafe extern "C" fn nvim_get_var(
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut di: *mut dictitem_T =
        tv_dict_find(get_globvar_dict(), name.data, name.size as ptrdiff_t);
    if di.is_null() {
        let mut found: bool =
            script_autoload(name.data, name.size, false_0 != 0) as ::core::ffi::c_int != 0
                && !aborting();
        if !found {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"Key not found: %s\0".as_ptr() as *const ::core::ffi::c_char,
                name.data,
            );
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
        di = tv_dict_find(get_globvar_dict(), name.data, name.size as ptrdiff_t);
    }
    if di.is_null() {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Key not found: %s\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
        );
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return vim_to_object(&raw mut (*di).di_tv, arena, true_0 != 0);
}
pub unsafe extern "C" fn nvim_set_var(mut name: String_0, mut value: Object, mut err: *mut Error) {
    dict_set_var(
        get_globvar_dict(),
        name,
        value,
        false_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
}
pub unsafe extern "C" fn nvim_del_var(mut name: String_0, mut err: *mut Error) {
    dict_set_var(
        get_globvar_dict(),
        name,
        object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        true_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
}
pub unsafe extern "C" fn nvim_get_vvar(
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    return dict_get_value(get_vimvar_dict(), name, arena, err);
}
pub unsafe extern "C" fn nvim_set_vvar(mut name: String_0, mut value: Object, mut err: *mut Error) {
    dict_set_var(
        get_vimvar_dict(),
        name,
        value,
        false_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
}
pub unsafe extern "C" fn nvim_echo(
    mut chunks: Array,
    mut history: Boolean,
    mut opts: *mut KeyDict_echo_opts,
    mut err: *mut Error,
) -> Object {
    let mut kind: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut is_progress: bool = false;
    let mut needs_clear: bool = false;
    let mut msg_data: MessageData = MessageData {
        source: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        percent: 0,
        title: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        status: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        data: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
    };
    let mut save_nwr: bool = false;
    let mut save_lines_left: ::core::ffi::c_int = 0;
    let mut save_msg_didany: bool = false;
    let mut id: Object = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: -1 as Integer,
        },
    };
    let mut hl_msg: HlMessage = parse_hl_msg(chunks, (*opts).err as bool, err);
    if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
        kind = (*opts).kind.data;
        if (*opts).verbose {
            verbose_enter();
        } else if kind.is_null() {
            kind = (if (*opts).err as ::core::ffi::c_int != 0 {
                b"echoerr\0".as_ptr() as *const ::core::ffi::c_char
            } else if history as ::core::ffi::c_int != 0 {
                b"echomsg\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"echo\0".as_ptr() as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char;
        }
        is_progress = strequal(kind, b"progress\0".as_ptr() as *const ::core::ffi::c_char);
        needs_clear = !history;
        if !(is_progress as ::core::ffi::c_int != 0
            || (*opts).status.size == 0 as size_t
                && (*opts).title.size == 0 as size_t
                && (*opts).percent == 0 as Integer
                && (*opts).data.size == 0 as size_t
                && (*opts).source.size == 0 as size_t)
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"Conflict: title/source/status/percent/data not allowed with kind='%s'\0".as_ptr()
                    as *const ::core::ffi::c_char,
                kind,
            );
        } else if !(!is_progress
            || strequal(
                (*opts).status.data,
                b"success\0".as_ptr() as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int
                != 0
            || strequal(
                (*opts).status.data,
                b"failed\0".as_ptr() as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int
                != 0
            || strequal(
                (*opts).status.data,
                b"running\0".as_ptr() as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int
                != 0
            || strequal(
                (*opts).status.data,
                b"cancel\0".as_ptr() as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int
                != 0)
        {
            api_err_exp(
                err,
                b"status\0".as_ptr() as *const ::core::ffi::c_char,
                b"success|failed|running|cancel\0".as_ptr() as *const ::core::ffi::c_char,
                (*opts).status.data,
            );
        } else if !(!is_progress
            || (*opts).percent >= 0 as Integer && (*opts).percent <= 100 as Integer)
        {
            api_err_invalid(
                err,
                b"percent\0".as_ptr() as *const ::core::ffi::c_char,
                b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                0 as int64_t,
                false_0 != 0,
            );
        } else if !(!is_progress || (*opts).source.size != 0 as size_t) {
            api_err_required(err, b"opts.source\0".as_ptr() as *const ::core::ffi::c_char);
        } else if !((*opts).id.type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            || msg_id_exists((*opts).id.data.integer as int64_t) as ::core::ffi::c_int != 0)
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"Invalid 'id': %ld\0".as_ptr() as *const ::core::ffi::c_char,
                (*opts).id.data.integer,
            );
        } else {
            msg_data = msg_data {
                source: (*opts).source,
                percent: (*opts).percent,
                title: (*opts).title,
                status: (*opts).status,
                data: (*opts).data,
            };
            save_nwr = need_wait_return.get();
            save_lines_left = lines_left.get();
            save_msg_didany = msg_didany.get();
            if (*opts)._truncate {
                (*no_wait_return.ptr()) += 1;
                lines_left.set(0 as ::core::ffi::c_int);
                msg_didany.set(true_0 != 0);
                msg_no_more.set(true_0 != 0);
            }
            id = msg_multihl(
                (*opts).id,
                hl_msg,
                kind,
                history as bool,
                (*opts).err as bool,
                &raw mut msg_data,
                &raw mut needs_clear,
            );
            if (*opts)._truncate {
                msg_no_more.set(false_0 != 0);
                msg_didany.set(save_msg_didany);
                lines_left.set(save_lines_left);
                (*no_wait_return.ptr()) -= 1;
                need_wait_return.set(save_nwr);
            }
            if (*opts).verbose {
                verbose_leave();
                verbose_stop();
            }
            if is_progress {
                do_autocmd_progress(id, hl_msg, &raw mut msg_data);
            }
            if !needs_clear {
                return id;
            }
        }
    }
    hl_msg_free(hl_msg);
    return id;
}
pub unsafe extern "C" fn nvim_list_bufs(mut arena: *mut Arena) -> Array {
    let mut n: size_t = 0 as size_t;
    let mut b: *mut buf_T = firstbuf.get();
    while !b.is_null() {
        n = n.wrapping_add(1);
        b = (*b).b_next;
    }
    let mut rv: Array = arena_array(arena, n);
    let mut b_0: *mut buf_T = firstbuf.get();
    while !b_0.is_null() {
        let c2rust_fresh1 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh1 as isize) = object {
            type_0: kObjectTypeBuffer,
            data: C2Rust_Unnamed {
                integer: (*b_0).handle as Integer,
            },
        };
        b_0 = (*b_0).b_next;
    }
    return rv;
}
pub unsafe extern "C" fn nvim_get_current_buf() -> Buffer {
    return (*curbuf.get()).handle as Buffer;
}
pub unsafe extern "C" fn nvim_set_current_buf(mut buf: Buffer, mut err: *mut Error) {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return;
    }
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    do_buffer(
        DOBUF_GOTO as ::core::ffi::c_int,
        DOBUF_FIRST as ::core::ffi::c_int,
        FORWARD as ::core::ffi::c_int,
        (*b).handle as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    try_leave(&raw mut tstate, err);
}
pub unsafe extern "C" fn nvim_list_wins(mut arena: *mut Arena) -> Array {
    let mut n: size_t = 0 as size_t;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            n = n.wrapping_add(1);
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    let mut rv: Array = arena_array(arena, n);
    let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp_0.is_null() {
        let mut wp_0: *mut win_T = if tp_0 == curtab.get() {
            firstwin.get()
        } else {
            (*tp_0).tp_firstwin
        };
        while !wp_0.is_null() {
            let c2rust_fresh2 = rv.size;
            rv.size = rv.size.wrapping_add(1);
            *rv.items.offset(c2rust_fresh2 as isize) = object {
                type_0: kObjectTypeWindow,
                data: C2Rust_Unnamed {
                    integer: (*wp_0).handle as Integer,
                },
            };
            wp_0 = (*wp_0).w_next;
        }
        tp_0 = (*tp_0).tp_next as *mut tabpage_T;
    }
    return rv;
}
pub unsafe extern "C" fn nvim_get_current_win() -> Window {
    return (*curwin.get()).handle as Window;
}
pub unsafe extern "C" fn nvim_set_current_win(mut win: Window, mut err: *mut Error) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return;
    }
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    if (*w).w_buffer != curbuf.get() {
        reset_VIsual_and_resel();
    }
    goto_tabpage_win(win_find_tabpage(w), w);
    try_leave(&raw mut tstate, err);
}
pub unsafe extern "C" fn nvim_create_buf(
    mut listed: Boolean,
    mut scratch: Boolean,
    mut err: *mut Error,
) -> Buffer {
    let mut ret: Buffer = 0 as Buffer;
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    block_autocmds();
    let mut buf: *mut buf_T = buflist_new(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        0 as linenr_T,
        BLN_NOOPT as ::core::ffi::c_int
            | BLN_NEW as ::core::ffi::c_int
            | (if listed as ::core::ffi::c_int != 0 {
                BLN_LISTED as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }),
    );
    if buf.is_null() {
        unblock_autocmds();
    } else if ml_open(buf) == 0 as ::core::ffi::c_int {
        unblock_autocmds();
    } else {
        (*buf).b_last_changedtick = buf_get_changedtick(buf);
        (*buf).b_last_changedtick_i = buf_get_changedtick(buf);
        (*buf).b_last_changedtick_pum = buf_get_changedtick(buf);
        buf_copy_options(
            buf,
            BCO_ENTER as ::core::ffi::c_int | BCO_NOHELP as ::core::ffi::c_int,
        );
        if scratch {
            set_option_direct_for(
                kOptBufhidden,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: String_0 {
                            data: b"hide\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                                .wrapping_sub(1 as size_t),
                        },
                    },
                },
                OPT_LOCAL as ::core::ffi::c_int,
                0 as scid_T,
                kOptScopeBuf,
                buf as *mut ::core::ffi::c_void,
            );
            set_option_direct_for(
                kOptBuftype,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: String_0 {
                            data: b"nofile\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                .wrapping_sub(1 as size_t),
                        },
                    },
                },
                OPT_LOCAL as ::core::ffi::c_int,
                0 as scid_T,
                kOptScopeBuf,
                buf as *mut ::core::ffi::c_void,
            );
            '_c2rust_label: {
                if (*(*buf).b_ml.ml_mfp).mf_fd < 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"buf->b_ml.ml_mfp->mf_fd < 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/vim.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1077 as ::core::ffi::c_uint,
                        b"Buffer nvim_create_buf(Boolean, Boolean, Error *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            (*buf).b_p_swf = 0 as ::core::ffi::c_int;
            (*buf).b_p_ml = 0 as ::core::ffi::c_int;
        }
        unblock_autocmds();
        let mut bufref: bufref_T = bufref_T::default();
        set_bufref(&raw mut bufref, buf);
        if !(apply_autocmds(
            EVENT_BUFNEW,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false,
            buf,
        ) as ::core::ffi::c_int
            != 0
            && !bufref_valid(&raw mut bufref))
        {
            if !(listed as ::core::ffi::c_int != 0
                && apply_autocmds(
                    EVENT_BUFADD,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false,
                    buf,
                ) as ::core::ffi::c_int
                    != 0
                && !bufref_valid(&raw mut bufref))
            {
                ret = (*buf).handle as Buffer;
            }
        }
    }
    try_leave(&raw mut tstate, err);
    if ret == 0 as ::core::ffi::c_int
        && !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
    {
        api_set_error(
            err,
            kErrorTypeException,
            b"Failed to create buffer\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return ret;
}
pub unsafe extern "C" fn nvim_open_term(
    mut buf: Buffer,
    mut opts: *mut KeyDict_open_term,
    mut err: *mut Error,
) -> Integer {
    let mut b: *mut buf_T = api_buf_ensure_loaded(buf, err);
    if b.is_null() {
        return 0 as Integer;
    }
    if b == cmdwin_buf.get() {
        api_set_error(
            err,
            kErrorTypeException,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw const e_cmdwin as *const ::core::ffi::c_char,
        );
        return 0 as Integer;
    }
    let mut may_read_buffer: bool = true_0 != 0;
    if !(*b).terminal.is_null() {
        if terminal_running((*b).terminal) {
            api_set_error(
                err,
                kErrorTypeException,
                b"Terminal already connected to buffer %d\0".as_ptr() as *const ::core::ffi::c_char,
                (*b).handle,
            );
            return 0 as Integer;
        }
        buf_close_terminal(b);
        may_read_buffer = false_0 != 0;
    }
    let mut cb: LuaRef = LUA_NOREF;
    if (*opts).is_set__open_term_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_open_term__on_input
        != 0 as ::core::ffi::c_ulonglong
    {
        cb = (*opts).on_input;
        (*opts).on_input = LUA_NOREF as LuaRef;
    }
    let mut chan: *mut Channel = channel_alloc(kChannelStreamInternal);
    (*channel_internal(chan)).cb = cb;
    (*channel_internal(chan)).closed = false_0 != 0;
    let mut topts: TerminalOptions = TerminalOptions {
        data: chan as *mut ::core::ffi::c_void,
        width: (if (*curwin.get()).w_view_width - win_col_off(curwin.get())
            > 0 as ::core::ffi::c_int
        {
            (*curwin.get()).w_view_width - win_col_off(curwin.get())
        } else {
            0 as ::core::ffi::c_int
        }) as uint16_t,
        height: (*curwin.get()).w_view_height as uint16_t,
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
        force_crlf: if (*opts).is_set__open_term_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_open_term__force_crlf
            != 0 as ::core::ffi::c_ulonglong
        {
            (*opts).force_crlf as ::core::ffi::c_int
        } else {
            true_0
        } != 0,
    };
    let mut contents: StringBuilder = StringBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    if may_read_buffer {
        read_buffer_into(b, 1 as linenr_T, (*b).b_ml.ml_line_count, &raw mut contents);
    }
    channel_incref(chan);
    (*chan).term = terminal_alloc(b, topts);
    terminal_open(&raw mut (*chan).term, b);
    if !(*chan).term.is_null() {
        terminal_check_size((*chan).term);
    }
    channel_decref(chan);
    if contents.size > 0 as size_t {
        let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        channel_send(
            (*chan).id,
            contents.items,
            contents.size,
            true_0 != 0,
            &raw mut error,
        );
        if !error.is_null() {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                error,
            );
        }
    }
    return (*chan).id as Integer;
}
unsafe extern "C" fn term_read_pause(mut _pause: bool, mut _data: *mut ::core::ffi::c_void) {}
unsafe extern "C" fn term_write(
    mut buf: *const ::core::ffi::c_char,
    mut size: size_t,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut chan: *mut Channel = data as *mut Channel;
    let mut cb: LuaRef = (*channel_internal(chan)).cb;
    if cb == LUA_NOREF {
        return;
    }
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 3] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 3];
    args.capacity = 3 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh3 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh3 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: (*chan).id as Integer,
        },
    };
    let c2rust_fresh4 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh4 as isize) = object {
        type_0: kObjectTypeBuffer,
        data: C2Rust_Unnamed {
            integer: terminal_buf((*chan).term) as Integer,
        },
    };
    let c2rust_fresh5 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh5 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: String_0 {
                data: buf as *mut ::core::ffi::c_char,
                size: size,
            },
        },
    };
    (*textlock.ptr()) += 1;
    nlua_call_ref(
        cb,
        b"input\0".as_ptr() as *const ::core::ffi::c_char,
        args,
        kRetNilBool,
        ::core::ptr::null_mut::<Arena>(),
        ::core::ptr::null_mut::<Error>(),
    );
    (*textlock.ptr()) -= 1;
}
unsafe extern "C" fn term_resize(
    mut _width: uint16_t,
    mut _height: uint16_t,
    mut _data: *mut ::core::ffi::c_void,
) {
}
unsafe extern "C" fn term_resume(mut _data: *mut ::core::ffi::c_void) {}
unsafe extern "C" fn term_close(mut data: *mut ::core::ffi::c_void) {
    let mut chan: *mut Channel = data as *mut Channel;
    terminal_destroy(&raw mut (*chan).term);
    api_free_luaref((*channel_internal(chan)).cb);
    (*channel_internal(chan)).cb = LUA_NOREF as LuaRef;
    channel_decref(chan);
}
pub unsafe extern "C" fn nvim_chan_send(
    mut chan: Integer,
    mut data: String_0,
    mut err: *mut Error,
) {
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if data.size == 0 {
        return;
    }
    channel_send(
        chan as uint64_t,
        data.data,
        data.size,
        false_0 != 0,
        &raw mut error,
    );
    if !error.is_null() {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            error,
        );
    }
}
pub unsafe extern "C" fn nvim_list_tabpages(mut arena: *mut Arena) -> Array {
    let mut n: size_t = 0 as size_t;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        n = n.wrapping_add(1);
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    let mut rv: Array = arena_array(arena, n);
    let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp_0.is_null() {
        let c2rust_fresh6 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh6 as isize) = object {
            type_0: kObjectTypeTabpage,
            data: C2Rust_Unnamed {
                integer: (*tp_0).handle as Integer,
            },
        };
        tp_0 = (*tp_0).tp_next as *mut tabpage_T;
    }
    return rv;
}
pub unsafe extern "C" fn nvim_get_current_tabpage() -> Tabpage {
    return (*curtab.get()).handle as Tabpage;
}
pub unsafe extern "C" fn nvim_set_current_tabpage(mut tabpage: Tabpage, mut err: *mut Error) {
    let mut tp: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tp.is_null() {
        return;
    }
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    goto_tabpage_tp(tp, true, true);
    try_leave(&raw mut tstate, err);
}
pub unsafe extern "C" fn nvim_paste(
    mut channel_id: uint64_t,
    mut data: String_0,
    mut crlf: Boolean,
    mut phase: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Boolean {
    let mut lines: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 2] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 2];
    let mut rv: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    static cancelled: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if !(phase >= -1 as Integer && phase <= 3 as Integer) {
        api_err_invalid(
            err,
            b"phase\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            phase as int64_t,
            false_0 != 0,
        );
        return false;
    }
    's_151: {
        if phase == -1 as Integer || phase == 1 as Integer {
            cancelled.set(false_0 != 0);
            if !(*curbuf.get()).terminal.is_null() {
                terminal_set_streamed_paste((*curbuf.get()).terminal, true_0 != 0);
            }
        } else if cancelled.get() {
            break 's_151;
        }
        lines = string_to_array(data, crlf as bool, arena);
        args = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        args__items = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 2];
        args.capacity = 2 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh7 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh7 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: lines },
        };
        let c2rust_fresh8 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh8 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: phase },
        };
        rv = nlua_exec(
            String_0 {
                data: b"return vim.paste(...)\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 22]>().wrapping_sub(1 as size_t),
            },
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetNilBool,
            arena,
            err,
        );
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
            || rv.type_0 as ::core::ffi::c_uint
                == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                && !rv.data.boolean
        {
            cancelled.set(true_0 != 0);
        }
        if (phase == -1 as Integer
            || phase == 3 as Integer
            || cancelled.get() as ::core::ffi::c_int != 0)
            && !(*curbuf.get()).terminal.is_null()
        {
            terminal_set_streamed_paste((*curbuf.get()).terminal, false_0 != 0);
        }
        if !cancelled.get() && (phase == -1 as Integer || phase == 1 as Integer) {
            paste_store(channel_id, kFalse, NULL_STRING, crlf as bool);
        }
        if !cancelled.get() {
            paste_store(channel_id, kNone, data, crlf as bool);
        }
        if phase == 3 as Integer
            || phase
                == (if cancelled.get() as ::core::ffi::c_int != 0 {
                    2 as ::core::ffi::c_int
                } else {
                    -1 as ::core::ffi::c_int
                }) as Integer
        {
            paste_store(channel_id, kTrue, NULL_STRING, crlf as bool);
        }
    }
    let mut retval: bool = !cancelled.get();
    if phase == -1 as Integer || phase == 3 as Integer {
        cancelled.set(false_0 != 0);
    }
    return retval as Boolean;
}
pub unsafe extern "C" fn nvim_put(
    mut lines: Array,
    mut type_0: String_0,
    mut after: Boolean,
    mut follow: Boolean,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    let mut reg: [yankreg_T; 1] = [yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    }];
    if !prepare_yankreg_from_object(&raw mut reg as *mut yankreg_T, type_0, lines.size) {
        api_err_invalid(
            err,
            b"type\0".as_ptr() as *const ::core::ffi::c_char,
            type_0.data,
            0 as int64_t,
            true_0 != 0,
        );
        return;
    }
    if lines.size == 0 as size_t {
        return;
    }
    (*(&raw mut reg as *mut yankreg_T)).y_array = arena_alloc(
        arena,
        lines.size.wrapping_mul(::core::mem::size_of::<String_0>()),
        true_0 != 0,
    ) as *mut String_0;
    (*(&raw mut reg as *mut yankreg_T)).y_size = lines.size;
    let mut i: size_t = 0 as size_t;
    while i < lines.size {
        if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            != (*lines.items.offset(i as isize)).type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                b"line\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(kObjectTypeString),
                api_typename((*lines.items.offset(i as isize)).type_0),
            );
            return;
        }
        let mut line: String_0 = (*lines.items.offset(i as isize)).data.string;
        *(*(&raw mut reg as *mut yankreg_T))
            .y_array
            .offset(i as isize) = copy_string(line, arena);
        memchrsub(
            (*(*(&raw mut reg as *mut yankreg_T))
                .y_array
                .offset(i as isize))
            .data as *mut ::core::ffi::c_void,
            NUL as ::core::ffi::c_char,
            NL as ::core::ffi::c_char,
            line.size,
        );
        i = i.wrapping_add(1);
    }
    finish_yankreg_from_object(&raw mut reg as *mut yankreg_T, false_0 != 0);
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    let mut VIsual_was_active: bool = VIsual_active.get();
    (*msg_silent.ptr()) += 1;
    do_put(
        0 as ::core::ffi::c_int,
        &raw mut reg as *mut yankreg_T,
        if after as ::core::ffi::c_int != 0 {
            FORWARD as ::core::ffi::c_int
        } else {
            BACKWARD as ::core::ffi::c_int
        },
        1 as ::core::ffi::c_int,
        if follow as ::core::ffi::c_int != 0 {
            PUT_CURSEND as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
    );
    (*msg_silent.ptr()) -= 1;
    VIsual_active.set(VIsual_was_active);
    try_leave(&raw mut tstate, err);
}
pub unsafe extern "C" fn nvim_get_color_by_name(mut name: String_0) -> Integer {
    // An API string is NUL-terminated.
    return name_to_color(::core::ffi::CStr::from_ptr(name.data)).0 as Integer;
}
pub unsafe extern "C" fn nvim_get_color_map(mut arena: *mut Arena) -> Dict {
    let mut colors: Dict = arena_dict(arena, COLOR_NAMES.len() as size_t);
    for entry in &COLOR_NAMES {
        let c2rust_fresh9 = colors.size;
        colors.size = colors.size.wrapping_add(1);
        *colors.items.offset(c2rust_fresh9 as isize) = key_value_pair {
            key: cstr_as_string(entry.name.as_ptr()),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: entry.color as Integer,
                },
            },
        };
    }
    return colors;
}
pub unsafe extern "C" fn nvim_get_context(
    mut opts: *mut KeyDict_context,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut types: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    if (*opts).is_set__context_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_context__types
        != 0 as ::core::ffi::c_ulonglong
    {
        types = (*opts).types;
    }
    let mut int_types: ::core::ffi::c_int = if types.size > 0 as size_t {
        0 as ::core::ffi::c_int
    } else {
        kCtxAll.get()
    };
    if types.size > 0 as size_t {
        let mut i: size_t = 0 as size_t;
        while i < types.size {
            if (*types.items.offset(i as isize)).type_0 as ::core::ffi::c_uint
                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let s: *const ::core::ffi::c_char =
                    (*types.items.offset(i as isize)).data.string.data;
                if strequal(s, b"regs\0".as_ptr() as *const ::core::ffi::c_char) {
                    int_types |= kCtxRegs as ::core::ffi::c_int;
                } else if strequal(s, b"jumps\0".as_ptr() as *const ::core::ffi::c_char) {
                    int_types |= kCtxJumps as ::core::ffi::c_int;
                } else if strequal(s, b"bufs\0".as_ptr() as *const ::core::ffi::c_char) {
                    int_types |= kCtxBufs as ::core::ffi::c_int;
                } else if strequal(s, b"gvars\0".as_ptr() as *const ::core::ffi::c_char) {
                    int_types |= kCtxGVars as ::core::ffi::c_int;
                } else if strequal(s, b"sfuncs\0".as_ptr() as *const ::core::ffi::c_char) {
                    int_types |= kCtxSFuncs as ::core::ffi::c_int;
                } else if strequal(s, b"funcs\0".as_ptr() as *const ::core::ffi::c_char) {
                    int_types |= kCtxFuncs as ::core::ffi::c_int;
                } else if true {
                    api_err_invalid(
                        err,
                        b"type\0".as_ptr() as *const ::core::ffi::c_char,
                        s,
                        0 as int64_t,
                        true_0 != 0,
                    );
                    return Dict {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<KeyValuePair>(),
                    };
                }
            }
            i = i.wrapping_add(1);
        }
    }
    let mut ctx: Context = CONTEXT_INIT;
    ctx_save(&raw mut ctx, int_types);
    let mut dict: Dict = ctx_to_dict(&raw mut ctx, arena);
    ctx_free(&raw mut ctx);
    return dict;
}
pub unsafe extern "C" fn nvim_load_context(mut dict: Dict, mut err: *mut Error) -> Object {
    let mut ctx: Context = CONTEXT_INIT;
    let mut save_did_emsg: ::core::ffi::c_int = did_emsg.get();
    did_emsg.set(false_0);
    ctx_from_dict(dict, &raw mut ctx, err);
    if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        ctx_restore(&raw mut ctx, kCtxAll.get());
    }
    ctx_free(&raw mut ctx);
    did_emsg.set(save_did_emsg);
    return object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
}
pub unsafe extern "C" fn nvim_get_mode(mut arena: *mut Arena) -> Dict {
    let mut rv: Dict = arena_dict(arena, 2 as size_t);
    let mut modestr: *mut ::core::ffi::c_char =
        arena_alloc(arena, MODE_MAX_LENGTH as size_t, false_0 != 0) as *mut ::core::ffi::c_char;
    get_mode(modestr);
    let mut blocked: bool = input_blocking();
    let c2rust_fresh10 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh10 as isize) = key_value_pair {
        key: cstr_as_string(b"mode\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(modestr),
            },
        },
    };
    let c2rust_fresh11 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh11 as isize) = key_value_pair {
        key: cstr_as_string(b"blocking\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed { boolean: blocked },
        },
    };
    return rv;
}
pub unsafe extern "C" fn nvim_get_keymap(mut mode: String_0, mut arena: *mut Arena) -> Array {
    return keymap_array(mode, ::core::ptr::null_mut::<buf_T>(), arena);
}
pub unsafe extern "C" fn nvim_set_keymap(
    mut channel_id: uint64_t,
    mut mode: String_0,
    mut lhs: String_0,
    mut rhs: String_0,
    mut opts: *mut KeyDict_keymap,
    mut err: *mut Error,
) {
    modify_keymap(
        channel_id,
        -1 as Buffer,
        false_0 != 0,
        mode,
        lhs,
        rhs,
        opts,
        err,
    );
}
pub unsafe extern "C" fn nvim_del_keymap(
    mut channel_id: uint64_t,
    mut mode: String_0,
    mut lhs: String_0,
    mut err: *mut Error,
) {
    nvim_buf_del_keymap(channel_id, -1 as Buffer, mode, lhs, err);
}
pub unsafe extern "C" fn nvim_get_api_info(
    mut channel_id: uint64_t,
    mut arena: *mut Arena,
) -> Array {
    let mut rv: Array = arena_array(arena, 2 as size_t);
    '_c2rust_label: {
        if channel_id <= 9223372036854775807 as uint64_t {
        } else {
            __assert_fail(
                b"channel_id <= INT64_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/api/vim.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1658 as ::core::ffi::c_uint,
                b"Array nvim_get_api_info(uint64_t, Arena *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let c2rust_fresh12 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh12 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: channel_id as int64_t,
        },
    };
    let c2rust_fresh13 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh13 as isize) = api_metadata();
    return rv;
}
pub unsafe extern "C" fn nvim_set_client_info(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut version: Dict,
    mut type_0: String_0,
    mut methods: Dict,
    mut attributes: Dict,
    mut arena: *mut Arena,
    mut _err: *mut Error,
) {
    let mut info: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut info__items: [KeyValuePair; 5] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    }; 5];
    info.capacity = 5 as size_t;
    info.items = &raw mut info__items as *mut KeyValuePair;
    let c2rust_fresh14 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh14 as isize) = key_value_pair {
        key: cstr_as_string(b"name\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed { string: name },
        },
    };
    let mut has_major: bool = false_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < version.size {
        if strequal(
            (*version.items.offset(i as isize)).key.data,
            b"major\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            has_major = true_0 != 0;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if !has_major {
        let mut v: Dict = arena_dict(arena, version.size.wrapping_add(1 as size_t));
        if version.size != 0 {
            memcpy(
                v.items as *mut ::core::ffi::c_void,
                version.items as *const ::core::ffi::c_void,
                version
                    .size
                    .wrapping_mul(::core::mem::size_of::<KeyValuePair>()),
            );
            v.size = version.size;
        }
        let c2rust_fresh15 = v.size;
        v.size = v.size.wrapping_add(1);
        *v.items.offset(c2rust_fresh15 as isize) = key_value_pair {
            key: cstr_as_string(b"major\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: 0 as Integer,
                },
            },
        };
        version = v;
    }
    let c2rust_fresh16 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh16 as isize) = key_value_pair {
        key: cstr_as_string(b"version\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: version },
        },
    };
    let c2rust_fresh17 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh17 as isize) = key_value_pair {
        key: cstr_as_string(b"type\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed { string: type_0 },
        },
    };
    let c2rust_fresh18 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh18 as isize) = key_value_pair {
        key: cstr_as_string(b"methods\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: methods },
        },
    };
    let c2rust_fresh19 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh19 as isize) = key_value_pair {
        key: cstr_as_string(b"attributes\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: attributes },
        },
    };
    rpc_set_client_info(
        channel_id,
        copy_dict(info, ::core::ptr::null_mut::<Arena>()),
    );
}
pub unsafe extern "C" fn nvim__chan_set_detach(
    mut channel_id: uint64_t,
    mut detach: Boolean,
    mut err: *mut Error,
) {
    let mut chan: *mut Channel = find_channel(channel_id);
    if chan.is_null() {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw const e_invchan as *const ::core::ffi::c_char,
        );
        return;
    }
    (*chan).detach = detach;
}
pub unsafe extern "C" fn nvim_get_chan_info(
    mut channel_id: uint64_t,
    mut chan: Integer,
    mut arena: *mut Arena,
    mut _err: *mut Error,
) -> Dict {
    if chan < 0 as Integer {
        return Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
    }
    if chan == 0 as Integer && !is_internal_call(channel_id) {
        '_c2rust_label: {
            if channel_id <= 9223372036854775807 as uint64_t {
            } else {
                __assert_fail(
                    b"channel_id <= INT64_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/api/vim.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1800 as ::core::ffi::c_uint,
                    b"Dict nvim_get_chan_info(uint64_t, Integer, Arena *, Error *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        chan = channel_id as Integer;
    }
    return channel_info(chan as uint64_t, arena);
}
pub unsafe extern "C" fn nvim_list_chans(mut arena: *mut Arena) -> Array {
    return channel_all_info(arena);
}
pub unsafe extern "C" fn nvim__id(mut obj: Object, mut arena: *mut Arena) -> Object {
    return copy_object(obj, arena);
}
pub unsafe extern "C" fn nvim__id_array(mut arr: Array, mut arena: *mut Arena) -> Array {
    return copy_array(arr, arena);
}
pub unsafe extern "C" fn nvim__id_dict(mut dct: Dict, mut arena: *mut Arena) -> Dict {
    return copy_dict(dct, arena);
}
pub unsafe extern "C" fn nvim__id_float(mut flt: Float) -> Float {
    return flt;
}
pub unsafe extern "C" fn nvim__stats(mut arena: *mut Arena) -> Dict {
    let mut rv: Dict = arena_dict(arena, 6 as size_t);
    let c2rust_fresh20 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh20 as isize) = key_value_pair {
        key: cstr_as_string(b"fsync\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*g_stats.ptr()).fsync,
            },
        },
    };
    let c2rust_fresh21 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh21 as isize) = key_value_pair {
        key: cstr_as_string(b"log_skip\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*g_stats.ptr()).log_skip as Integer,
            },
        },
    };
    let c2rust_fresh22 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh22 as isize) = key_value_pair {
        key: cstr_as_string(b"lua_refcount\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: nlua_get_global_ref_count() as Integer,
            },
        },
    };
    let c2rust_fresh23 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh23 as isize) = key_value_pair {
        key: cstr_as_string(b"redraw\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*g_stats.ptr()).redraw,
            },
        },
    };
    let c2rust_fresh24 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh24 as isize) = key_value_pair {
        key: cstr_as_string(b"arena_alloc_count\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: arena_alloc_count.get() as Integer,
            },
        },
    };
    let c2rust_fresh25 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh25 as isize) = key_value_pair {
        key: cstr_as_string(b"ts_query_parse_count\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: tslua_query_parse_count.get() as Integer,
            },
        },
    };
    return rv;
}
pub unsafe extern "C" fn nvim_list_uis(mut arena: *mut Arena) -> Array {
    return ui_array(arena);
}
pub unsafe extern "C" fn nvim_get_proc_children(
    mut pid: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut rv: ::core::ffi::c_int = 0;
    let mut rvobj: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut children: Vec<::core::ffi::c_int> = Vec::new();
    if !(pid > 0 as Integer && pid <= 2147483647 as Integer) {
        api_err_invalid(
            err,
            b"pid\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            pid as int64_t,
            false_0 != 0,
        );
    } else {
        match os_proc_children(pid as ::core::ffi::c_int) {
            Some(pids) => children = pids,
            // Only "could not inspect" is reachable on this platform.
            None => rv = 2 as ::core::ffi::c_int,
        }
        if rv == 2 as ::core::ffi::c_int {
            logmsg(
                LOGLVL_DBG,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"nvim_get_proc_children\0".as_ptr() as *const ::core::ffi::c_char,
                1924 as ::core::ffi::c_int,
                true_0 != 0,
                b"fallback to vim._os_proc_children()\0".as_ptr() as *const ::core::ffi::c_char,
            );
            let mut a: Array = Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
            let mut a__items: [Object; 1] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            }; 1];
            a.capacity = 1 as size_t;
            a.items = &raw mut a__items as *mut Object;
            let c2rust_fresh26 = a.size;
            a.size = a.size.wrapping_add(1);
            *a.items.offset(c2rust_fresh26 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed { integer: pid },
            };
            let mut o: Object = nlua_exec(
                String_0 {
                    data: b"return vim._os_proc_children(...)\0".as_ptr()
                        as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 34]>()
                        .wrapping_sub(1 as size_t),
                },
                ::core::ptr::null::<::core::ffi::c_char>(),
                a,
                kRetObject,
                arena,
                err,
            );
            if o.type_0 as ::core::ffi::c_uint
                == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                rvobj = o.data.array;
            } else if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
            {
                api_set_error(
                    err,
                    kErrorTypeException,
                    b"Failed to get process children. pid=%ld error=%d\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    pid,
                    rv,
                );
            }
        } else {
            rvobj = arena_array(arena, children.len() as size_t);
            for pid in children {
                let c2rust_fresh27 = rvobj.size;
                rvobj.size = rvobj.size.wrapping_add(1);
                *rvobj.items.offset(c2rust_fresh27 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: pid as Integer,
                    },
                };
            }
        }
    }
    return rvobj;
}
pub unsafe extern "C" fn nvim_get_proc(
    mut pid: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut rvobj: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    if !(pid > 0 as Integer && pid <= 2147483647 as Integer) {
        api_err_invalid(
            err,
            b"pid\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            pid as int64_t,
            false_0 != 0,
        );
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    let mut a: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut a__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 1];
    a.capacity = 1 as size_t;
    a.items = &raw mut a__items as *mut Object;
    if a.size == a.capacity {
        a.capacity = if a.capacity != 0 {
            a.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        a.items = xrealloc(
            a.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<Object>().wrapping_mul(a.capacity),
        ) as *mut Object;
    } else {
    };
    let c2rust_fresh28 = a.size;
    a.size = a.size.wrapping_add(1);
    *a.items.offset(c2rust_fresh28 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: pid },
    };
    let mut o: Object = nlua_exec(
        String_0 {
            data: b"return vim._os_proc_info(...)\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: ::core::mem::size_of::<[::core::ffi::c_char; 30]>().wrapping_sub(1 as size_t),
        },
        ::core::ptr::null::<::core::ffi::c_char>(),
        a,
        kRetObject,
        arena,
        err,
    );
    if o.type_0 as ::core::ffi::c_uint
        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        && o.data.array.size == 0 as size_t
    {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    } else if o.type_0 as ::core::ffi::c_uint
        == kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        rvobj = o;
    } else if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        api_set_error(
            err,
            kErrorTypeException,
            b"Failed to get process info. pid=%ld\0".as_ptr() as *const ::core::ffi::c_char,
            pid,
        );
    }
    return rvobj;
}
pub unsafe extern "C" fn nvim_select_popupmenu_item(
    mut item: Integer,
    mut insert: Boolean,
    mut finish: Boolean,
    mut _opts: *mut KeyDict_empty,
    mut _err: *mut Error,
) {
    if finish {
        insert = true_0 != 0;
    }
    pum_ext_select_item(item as ::core::ffi::c_int, insert as bool, finish as bool);
}
pub unsafe extern "C" fn nvim__inspect_cell(
    mut grid: Integer,
    mut row: Integer,
    mut col: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut ret: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut g: *mut ScreenGrid = default_grid.ptr();
    if grid == (*pum_grid.ptr()).handle as Integer {
        g = pum_grid.ptr();
    } else if grid > 1 as Integer {
        let mut wp: *mut win_T = get_win_by_grid_handle(grid as handle_T);
        if !(!wp.is_null() && !(*wp).w_grid_alloc.chars.is_null()) {
            api_err_invalid(
                err,
                b"grid handle\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
                grid as int64_t,
                false_0 != 0,
            );
            return ret;
        }
        g = &raw mut (*wp).w_grid_alloc;
    }
    if row < 0 as Integer
        || row >= (*g).rows as Integer
        || col < 0 as Integer
        || col >= (*g).cols as Integer
    {
        return ret;
    }
    ret = arena_array(arena, 3 as size_t);
    let mut off: size_t =
        (*(*g).line_offset.offset(row as size_t as isize)).wrapping_add(col as size_t);
    let mut sc_buf: *mut ::core::ffi::c_char =
        arena_alloc(arena, MAX_SCHAR_SIZE as size_t, false_0 != 0) as *mut ::core::ffi::c_char;
    schar_get(sc_buf, *(*g).chars.offset(off as isize));
    let c2rust_fresh29 = ret.size;
    ret.size = ret.size.wrapping_add(1);
    *ret.items.offset(c2rust_fresh29 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstr_as_string(sc_buf),
        },
    };
    let mut attr: ::core::ffi::c_int = *(*g).attrs.offset(off as isize) as ::core::ffi::c_int;
    let c2rust_fresh30 = ret.size;
    ret.size = ret.size.wrapping_add(1);
    *ret.items.offset(c2rust_fresh30 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed {
            dict: hl_get_attr_by_id(attr as Integer, true, arena, err),
        },
    };
    if !highlight_use_hlstate() {
        let c2rust_fresh31 = ret.size;
        ret.size = ret.size.wrapping_add(1);
        *ret.items.offset(c2rust_fresh31 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed {
                array: hl_inspect(attr, arena),
            },
        };
    }
    return ret;
}
pub unsafe extern "C" fn nvim__screenshot(mut path: String_0) {
    ui_call_screenshot(path);
}
pub unsafe extern "C" fn nvim__invalidate_glyph_cache() {
    schar_cache_clear();
    must_redraw.set(UPD_CLEAR);
}
pub unsafe extern "C" fn nvim__unpack(
    mut str: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    return unpack(str.data, str.size, arena, err);
}
pub unsafe extern "C" fn nvim_del_mark(mut name: String_0, mut err: *mut Error) -> Boolean {
    let mut res: bool = false_0 != 0;
    if !(name.size == 1 as size_t) {
        api_err_invalid(
            err,
            b"mark name (must be a single char)\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return res as Boolean;
    }
    if !(*name.data as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *name.data as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || ascii_isdigit(*name.data as ::core::ffi::c_int) as ::core::ffi::c_int != 0)
    {
        api_err_invalid(
            err,
            b"mark name (must be file/uppercase)\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return res as Boolean;
    }
    res = set_mark(
        ::core::ptr::null_mut::<buf_T>(),
        name,
        0 as Integer,
        0 as Integer,
        err,
    );
    return res as Boolean;
}
pub unsafe extern "C" fn nvim_get_mark(
    mut name: String_0,
    mut _opts: *mut KeyDict_empty,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut rv: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    if !(name.size == 1 as size_t) {
        api_err_invalid(
            err,
            b"mark name (must be a single char)\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return rv;
    }
    if !(*name.data as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *name.data as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || ascii_isdigit(*name.data as ::core::ffi::c_int) as ::core::ffi::c_int != 0)
    {
        api_err_invalid(
            err,
            b"mark name (must be file/uppercase)\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return rv;
    }
    let mut mark: *mut xfmark_T = mark_get_global(false_0 != 0, *name.data as ::core::ffi::c_int);
    let mut pos: pos_T = (*mark).fmark.mark;
    let mut allocated: bool = false_0 != 0;
    let mut bufnr: ::core::ffi::c_int = 0;
    let mut filename: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*mark).fmark.fnum != 0 as ::core::ffi::c_int {
        bufnr = (*mark).fmark.fnum;
        filename = buflist_nr2name(bufnr, true_0, true_0);
        allocated = true_0 != 0;
    } else {
        filename = (*mark).fname;
        bufnr = 0 as ::core::ffi::c_int;
    }
    let mut exists: bool = !filename.is_null();
    let mut row: Integer = 0;
    let mut col: Integer = 0;
    if !exists || pos.lnum <= 0 as linenr_T {
        if allocated {
            xfree(filename as *mut ::core::ffi::c_void);
            allocated = false_0 != 0;
        }
        filename = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        bufnr = 0 as ::core::ffi::c_int;
        row = 0 as Integer;
        col = 0 as Integer;
    } else {
        row = pos.lnum as Integer;
        col = pos.col as Integer;
    }
    rv = arena_array(arena, 4 as size_t);
    let c2rust_fresh32 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh32 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: row },
    };
    let c2rust_fresh33 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh33 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: col },
    };
    let c2rust_fresh34 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh34 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: bufnr as Integer,
        },
    };
    let c2rust_fresh35 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh35 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: arena_string(arena, cstr_as_string(filename)),
        },
    };
    if allocated {
        xfree(filename as *mut ::core::ffi::c_void);
    }
    return rv;
}
pub unsafe extern "C" fn nvim_eval_statusline(
    mut str: String_0,
    mut opts: *mut KeyDict_eval_statusline,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut result: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut maxwidth: ::core::ffi::c_int = 0;
    let mut fillchar: schar_T = 0 as schar_T;
    let mut statuscol_lnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if str.size < 2 as size_t
        || memcmp(
            str.data as *const ::core::ffi::c_void,
            b"%!\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            2 as size_t,
        ) != 0 as ::core::ffi::c_int
    {
        let errmsg: *const ::core::ffi::c_char = check_stl_option(str.data);
        if !errmsg.is_null() {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                errmsg,
            );
            return result;
        }
    }
    let mut window: Window = (*opts).winid;
    if (*opts).is_set__eval_statusline_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_eval_statusline__fillchar
        != 0 as ::core::ffi::c_ulonglong
    {
        if !(*(*opts).fillchar.data as ::core::ffi::c_int != 0 as ::core::ffi::c_int
            && utfc_ptr2len((*opts).fillchar.data) as size_t == (*opts).fillchar.size)
        {
            api_err_exp(
                err,
                b"fillchar\0".as_ptr() as *const ::core::ffi::c_char,
                b"single character\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
            return result;
        }
        let mut c: ::core::ffi::c_int = 0;
        fillchar = utfc_ptr2schar((*opts).fillchar.data, &raw mut c);
    }
    let mut use_bools: ::core::ffi::c_int =
        (*opts).use_winbar as ::core::ffi::c_int + (*opts).use_tabline as ::core::ffi::c_int;
    let mut wp: *mut win_T = if (*opts).use_tabline as ::core::ffi::c_int != 0 {
        curwin.get()
    } else {
        find_window_by_handle(window, err)
    };
    if wp.is_null() {
        api_set_error(
            err,
            kErrorTypeException,
            b"unknown winid %d\0".as_ptr() as *const ::core::ffi::c_char,
            window,
        );
        return result;
    }
    if (*opts).is_set__eval_statusline_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_eval_statusline__use_statuscol_lnum
        != 0 as ::core::ffi::c_ulonglong
    {
        statuscol_lnum = (*opts).use_statuscol_lnum as ::core::ffi::c_int;
        if !(statuscol_lnum > 0 as ::core::ffi::c_int
            && statuscol_lnum as linenr_T <= (*(*wp).w_buffer).b_ml.ml_line_count)
        {
            api_err_invalid(
                err,
                b"use_statuscol_lnum\0".as_ptr() as *const ::core::ffi::c_char,
                b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                0 as int64_t,
                false_0 != 0,
            );
            return result;
        }
        use_bools += 1;
    }
    if !(use_bools <= 1 as ::core::ffi::c_int) {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"Can only use one of 'use_winbar', 'use_tabline' and 'use_statuscol_lnum'\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        return result;
    }
    let mut stc_hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut scl_hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut statuscol: statuscol_T = statuscol_T {
        width: 0 as ::core::ffi::c_int,
        lnum: 0,
        sign_cul_id: 0,
        draw: false,
        hlrec: ::core::ptr::null_mut::<stl_hlrec_t>(),
        foldinfo: foldinfo_T {
            fi_lnum: 0,
            fi_level: 0,
            fi_low_level: 0,
            fi_lines: 0,
        },
        fold_vcol: [0; 9],
        sattrs: ::core::ptr::null_mut::<SignTextAttrs>(),
    };
    let mut sattrs: [SignTextAttrs; 9] = [
        SignTextAttrs {
            text: [0 as schar_T, 0],
            hl_id: 0,
        },
        SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        },
        SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        },
        SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        },
        SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        },
        SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        },
        SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        },
        SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        },
        SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        },
    ];
    if statuscol_lnum != 0 {
        let mut line_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut cul_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut num_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut lnum: linenr_T = statuscol_lnum as linenr_T;
        let mut cursorline_fi: foldinfo_T = foldinfo_T {
            fi_lnum: 0 as linenr_T,
            fi_level: 0,
            fi_low_level: 0,
            fi_lines: 0,
        };
        decor_redraw_signs(
            wp,
            (*wp).w_buffer,
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            &raw mut sattrs as *mut SignTextAttrs,
            &raw mut line_id,
            &raw mut cul_id,
            &raw mut num_id,
        );
        statuscol.sattrs = &raw mut sattrs as *mut SignTextAttrs;
        statuscol.foldinfo = fold_info(wp, lnum);
        win_update_cursorline(wp, &raw mut cursorline_fi);
        statuscol.sign_cul_id = if use_cursor_line_highlight(wp, lnum) as ::core::ffi::c_int != 0 {
            cul_id
        } else {
            0 as ::core::ffi::c_int
        };
        scl_hl_id = if use_cursor_line_highlight(wp, lnum) as ::core::ffi::c_int != 0 {
            HLF_CLS
        } else {
            HLF_SC
        };
        if num_id != 0 {
            stc_hl_id = num_id;
        } else if use_cursor_line_highlight(wp, lnum) {
            stc_hl_id = HLF_CLN;
        } else if (*wp).w_onebuf_opt.wo_rnu != 0 {
            stc_hl_id = if lnum < (*wp).w_cursor.lnum {
                HLF_LNA
            } else {
                HLF_LNB
            };
        } else {
            stc_hl_id = HLF_N;
        }
        set_vim_var_nr(VV_LNUM, lnum as varnumber_T);
        set_vim_var_nr(
            VV_RELNUM,
            labs(get_cursor_rel_lnum(wp, lnum) as ::core::ffi::c_long) as varnumber_T,
        );
        set_vim_var_nr(VV_VIRTNUM, 0 as varnumber_T);
    } else if fillchar == 0 as schar_T && !(*opts).use_tabline {
        if (*opts).use_winbar {
            fillchar = (*wp).w_p_fcs_chars.wbr;
        } else {
            let mut group: hlf_T = HLF_NONE;
            fillchar = fillchar_status(&raw mut group, wp);
        }
    }
    if (*opts).is_set__eval_statusline_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_eval_statusline__maxwidth
        != 0 as ::core::ffi::c_ulonglong
    {
        maxwidth = (*opts).maxwidth as ::core::ffi::c_int;
    } else {
        maxwidth = if statuscol_lnum != 0 {
            win_col_off(wp)
        } else if (*opts).use_tabline as ::core::ffi::c_int != 0
            || !(*opts).use_winbar && global_stl_height() > 0 as ::core::ffi::c_int
        {
            Columns.get()
        } else {
            (*wp).w_width
        };
    }
    result = arena_dict(arena, 3 as size_t);
    let mut buf: *mut ::core::ffi::c_char =
        arena_alloc(arena, MAXPATHL as size_t, false_0 != 0) as *mut ::core::ffi::c_char;
    let mut hltab: *mut stl_hlrec_t = ::core::ptr::null_mut::<stl_hlrec_t>();
    let mut hltab_len: size_t = 0 as size_t;
    let mut p_crb_save: ::core::ffi::c_int = (*wp).w_onebuf_opt.wo_crb;
    (*wp).w_onebuf_opt.wo_crb = false_0;
    let mut width: ::core::ffi::c_int = build_stl_str_hl(
        wp,
        buf,
        MAXPATHL as size_t,
        str.data,
        kOptInvalid,
        0 as ::core::ffi::c_int,
        fillchar,
        maxwidth,
        if (*opts).highlights as ::core::ffi::c_int != 0 {
            &raw mut hltab
        } else {
            ::core::ptr::null_mut::<*mut stl_hlrec_t>()
        },
        &raw mut hltab_len,
        ::core::ptr::null_mut::<*mut StlClickRecord>(),
        if statuscol_lnum != 0 {
            &raw mut statuscol
        } else {
            ::core::ptr::null_mut::<statuscol_T>()
        },
    );
    let c2rust_fresh36 = result.size;
    result.size = result.size.wrapping_add(1);
    *result.items.offset(c2rust_fresh36 as isize) = key_value_pair {
        key: cstr_as_string(b"width\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: width as Integer,
            },
        },
    };
    (*wp).w_onebuf_opt.wo_crb = p_crb_save;
    if (*opts).highlights {
        let mut hl_values: Array = arena_array(arena, hltab_len.wrapping_add(1 as size_t));
        let mut user_group: [::core::ffi::c_char; 15] = [0; 15];
        let mut dfltname: *const ::core::ffi::c_char = get_default_stl_hl(
            if (*opts).use_tabline as ::core::ffi::c_int != 0 {
                ::core::ptr::null_mut::<win_T>()
            } else {
                wp
            },
            (*opts).use_winbar as bool,
            stc_hl_id,
        );
        if (*hltab).start.is_null() || (*hltab).start.offset_from(buf) != 0 as isize {
            let mut hl_info: Dict = arena_dict(arena, 3 as size_t);
            let c2rust_fresh37 = hl_info.size;
            hl_info.size = hl_info.size.wrapping_add(1);
            *hl_info.items.offset(c2rust_fresh37 as isize) = key_value_pair {
                key: cstr_as_string(b"start\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: 0 as Integer,
                    },
                },
            };
            let c2rust_fresh38 = hl_info.size;
            hl_info.size = hl_info.size.wrapping_add(1);
            *hl_info.items.offset(c2rust_fresh38 as isize) = key_value_pair {
                key: cstr_as_string(b"group\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(dfltname),
                    },
                },
            };
            let mut groups: Array = arena_array(arena, 1 as size_t);
            let c2rust_fresh39 = groups.size;
            groups.size = groups.size.wrapping_add(1);
            *groups.items.offset(c2rust_fresh39 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(dfltname),
                },
            };
            let c2rust_fresh40 = hl_info.size;
            hl_info.size = hl_info.size.wrapping_add(1);
            *hl_info.items.offset(c2rust_fresh40 as isize) = key_value_pair {
                key: cstr_as_string(b"groups\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed { array: groups },
                },
            };
            let c2rust_fresh41 = hl_values.size;
            hl_values.size = hl_values.size.wrapping_add(1);
            *hl_values.items.offset(c2rust_fresh41 as isize) = object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: hl_info },
            };
        }
        let mut sp: *mut stl_hlrec_t = hltab;
        while !(*sp).start.is_null() {
            let mut grpname: *const ::core::ffi::c_char =
                ::core::ptr::null::<::core::ffi::c_char>();
            if (*sp).userhl == 0 as ::core::ffi::c_int {
                grpname = get_default_stl_hl(
                    if (*opts).use_tabline as ::core::ffi::c_int != 0 {
                        ::core::ptr::null_mut::<win_T>()
                    } else {
                        wp
                    },
                    (*opts).use_winbar as bool,
                    stc_hl_id,
                );
            } else if (*sp).userhl < 0 as ::core::ffi::c_int {
                grpname = syn_id2name(-(*sp).userhl);
            } else {
                snprintf(
                    &raw mut user_group as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 15]>(),
                    b"User%d\0".as_ptr() as *const ::core::ffi::c_char,
                    (*sp).userhl,
                );
                grpname = arena_strdup(arena, &raw mut user_group as *mut ::core::ffi::c_char);
            }
            let mut combine: *const ::core::ffi::c_char = if (*sp).item as ::core::ffi::c_uint
                == STL_SIGNCOL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                syn_id2name(scl_hl_id) as *const ::core::ffi::c_char
            } else if (*sp).item as ::core::ffi::c_uint
                == STL_FOLDCOL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                grpname
            } else {
                dfltname
            };
            let mut hl_info_0: Dict = arena_dict(arena, 3 as size_t);
            let c2rust_fresh42 = hl_info_0.size;
            hl_info_0.size = hl_info_0.size.wrapping_add(1);
            *hl_info_0.items.offset(c2rust_fresh42 as isize) = key_value_pair {
                key: cstr_as_string(b"start\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (*sp).start.offset_from(buf) as i64,
                    },
                },
            };
            let c2rust_fresh43 = hl_info_0.size;
            hl_info_0.size = hl_info_0.size.wrapping_add(1);
            *hl_info_0.items.offset(c2rust_fresh43 as isize) = key_value_pair {
                key: cstr_as_string(b"group\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(grpname),
                    },
                },
            };
            let mut groups_0: Array = arena_array(
                arena,
                (1 as ::core::ffi::c_int + (combine != grpname) as ::core::ffi::c_int) as size_t,
            );
            if combine != grpname {
                let c2rust_fresh44 = groups_0.size;
                groups_0.size = groups_0.size.wrapping_add(1);
                *groups_0.items.offset(c2rust_fresh44 as isize) = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(combine),
                    },
                };
            }
            let c2rust_fresh45 = groups_0.size;
            groups_0.size = groups_0.size.wrapping_add(1);
            *groups_0.items.offset(c2rust_fresh45 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(grpname),
                },
            };
            let c2rust_fresh46 = hl_info_0.size;
            hl_info_0.size = hl_info_0.size.wrapping_add(1);
            *hl_info_0.items.offset(c2rust_fresh46 as isize) = key_value_pair {
                key: cstr_as_string(b"groups\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed { array: groups_0 },
                },
            };
            let c2rust_fresh47 = hl_values.size;
            hl_values.size = hl_values.size.wrapping_add(1);
            *hl_values.items.offset(c2rust_fresh47 as isize) = object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: hl_info_0 },
            };
            sp = sp.offset(1);
        }
        let c2rust_fresh48 = result.size;
        result.size = result.size.wrapping_add(1);
        *result.items.offset(c2rust_fresh48 as isize) = key_value_pair {
            key: cstr_as_string(b"highlights\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: hl_values },
            },
        };
    }
    let c2rust_fresh49 = result.size;
    result.size = result.size.wrapping_add(1);
    *result.items.offset(c2rust_fresh49 as isize) = key_value_pair {
        key: cstr_as_string(b"str\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(buf),
            },
        },
    };
    return result;
}
pub unsafe extern "C" fn nvim__complete_set(
    mut index: Integer,
    mut opts: *mut KeyDict_complete_set,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut rv: Dict = arena_dict(arena, 2 as size_t);
    if get_cot_flags() & kOptCotFlagPopup as ::core::ffi::c_int as ::core::ffi::c_uint
        == 0 as ::core::ffi::c_uint
    {
        api_set_error(
            err,
            kErrorTypeException,
            b"completeopt option does not include popup\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return rv;
    }
    if (*opts).is_set__complete_set_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_complete_set__info
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut wp: *mut win_T = pum_set_info(index as ::core::ffi::c_int, (*opts).info.data);
        if !wp.is_null() {
            let c2rust_fresh50 = rv.size;
            rv.size = rv.size.wrapping_add(1);
            *rv.items.offset(c2rust_fresh50 as isize) = key_value_pair {
                key: cstr_as_string(b"winid\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeWindow,
                    data: C2Rust_Unnamed {
                        integer: (*wp).handle as Integer,
                    },
                },
            };
            let c2rust_fresh51 = rv.size;
            rv.size = rv.size.wrapping_add(1);
            *rv.items.offset(c2rust_fresh51 as isize) = key_value_pair {
                key: cstr_as_string(b"bufnr\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBuffer,
                    data: C2Rust_Unnamed {
                        integer: (*(*wp).w_buffer).handle as Integer,
                    },
                },
            };
        }
    }
    return rv;
}
unsafe extern "C" fn redraw_status(
    mut wp: *mut win_T,
    mut opts: *mut KeyDict_redraw,
    mut flush: *mut bool,
) {
    if (*opts).statuscolumn as ::core::ffi::c_int != 0
        && *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL
    {
        (*wp).w_nrwidth_line_count = 0 as ::core::ffi::c_int as linenr_T;
        changed_window_setting(wp);
    }
    let mut old_row_offset: ::core::ffi::c_int = (*wp).w_grid.row_offset;
    win_grid_alloc(wp);
    if (*wp).w_lines_valid == 0 as ::core::ffi::c_int || (*wp).w_grid.row_offset != old_row_offset {
        *flush = true_0 != 0;
    }
    if *flush as ::core::ffi::c_int != 0
        && ((*opts).statusline as ::core::ffi::c_int != 0
            || (*opts).winbar as ::core::ffi::c_int != 0)
    {
        (*wp).w_redr_status = true_0 != 0;
    } else if (*opts).statusline as ::core::ffi::c_int != 0
        || (*opts).winbar as ::core::ffi::c_int != 0
    {
        win_check_ns_hl(wp);
        if (*opts).winbar {
            win_redr_winbar(wp);
        }
        if (*opts).statusline {
            win_redr_status(wp);
        }
        win_check_ns_hl(::core::ptr::null_mut::<win_T>());
    }
}
pub unsafe extern "C" fn nvim__redraw(mut opts: *mut KeyDict_redraw, mut err: *mut Error) {
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    if (*opts).is_set__redraw_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_redraw__win
        != 0 as ::core::ffi::c_ulonglong
    {
        win = find_window_by_handle((*opts).win, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return;
        }
    }
    if (*opts).is_set__redraw_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_redraw__buf
        != 0 as ::core::ffi::c_ulonglong
    {
        if !win.is_null() {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"cannot use both 'buf' and 'win'\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        buf = find_buffer_by_handle((*opts).buf, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return;
        }
    }
    let mut count: ::core::ffi::c_uint = (!win.is_null() as ::core::ffi::c_int
        + !buf.is_null() as ::core::ffi::c_int)
        as ::core::ffi::c_uint;
    if !(((*opts).is_set__redraw_ as uint64_t).count_ones() > count) {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"at least one action required\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if (*opts).is_set__redraw_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_redraw__valid
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut type_0: ::core::ffi::c_int = if (*opts).valid as ::core::ffi::c_int != 0 {
            UPD_VALID
        } else {
            UPD_NOT_VALID
        };
        if !win.is_null() {
            redraw_later(win, type_0);
        } else if !buf.is_null() {
            redraw_buf_later(buf, type_0);
        } else {
            redraw_all_later(type_0);
        }
    }
    if (*opts).is_set__redraw_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_redraw__range
        != 0 as ::core::ffi::c_ulonglong
    {
        if !((*opts).range.size == 2 as size_t
            && (*(*opts).range.items.offset(0 as ::core::ffi::c_int as isize)).type_0
                as ::core::ffi::c_uint
                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*(*opts).range.items.offset(1 as ::core::ffi::c_int as isize)).type_0
                as ::core::ffi::c_uint
                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*(*opts).range.items.offset(0 as ::core::ffi::c_int as isize))
                .data
                .integer
                >= 0 as Integer
            && (*(*opts).range.items.offset(1 as ::core::ffi::c_int as isize))
                .data
                .integer
                >= -1 as Integer)
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"Invalid 'range': Expected 2-tuple of Integers\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            return;
        }
        let mut begin_raw: int64_t = (*(*opts).range.items.offset(0 as ::core::ffi::c_int as isize))
            .data
            .integer as int64_t;
        let mut end_raw: int64_t = (*(*opts).range.items.offset(1 as ::core::ffi::c_int as isize))
            .data
            .integer as int64_t;
        let mut rbuf: *mut buf_T = if !win.is_null() {
            (*win).w_buffer
        } else if !buf.is_null() {
            buf
        } else {
            curbuf.get()
        };
        let mut line_count: linenr_T = (*rbuf).b_ml.ml_line_count;
        let mut begin: ::core::ffi::c_int = (if begin_raw < line_count as int64_t {
            begin_raw
        } else {
            line_count as int64_t
        }) as ::core::ffi::c_int;
        let mut end: ::core::ffi::c_int = 0;
        if end_raw == -1 as int64_t {
            end = line_count as ::core::ffi::c_int;
        } else {
            end = (if (if begin as int64_t > end_raw {
                begin as int64_t
            } else {
                end_raw
            }) < line_count as int64_t
            {
                if begin as int64_t > end_raw {
                    begin as int64_t
                } else {
                    end_raw
                }
            } else {
                line_count as int64_t
            }) as ::core::ffi::c_int;
        }
        if begin < end {
            redraw_buf_range_later(rbuf, 1 as linenr_T + begin as linenr_T, end as linenr_T);
        }
    }
    if (*opts).is_set__redraw_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_redraw__valid
        != 0 as ::core::ffi::c_ulonglong
        || (*opts).is_set__redraw_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_redraw__range
            != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).flush = if (*opts).is_set__redraw_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_redraw__flush
            != 0 as ::core::ffi::c_ulonglong
        {
            (*opts).flush as ::core::ffi::c_int
        } else {
            true_0
        } != 0;
    }
    let mut flush_ui: bool = (*opts).flush as bool;
    if (*opts).tabline {
        if redraw_tabline.get() as ::core::ffi::c_int != 0
            && (*firstwin.get()).w_lines_valid == 0 as ::core::ffi::c_int
        {
            (*opts).flush = true_0 != 0;
        } else {
            draw_tabline();
        }
        flush_ui = true_0 != 0;
    }
    let mut save_lz: bool = p_lz.get() != 0;
    let mut save_rd: ::core::ffi::c_int = RedrawingDisabled.get();
    RedrawingDisabled.set(0 as ::core::ffi::c_int);
    p_lz.set(false_0);
    if (*opts).statuscolumn as ::core::ffi::c_int != 0
        || (*opts).statusline as ::core::ffi::c_int != 0
        || (*opts).winbar as ::core::ffi::c_int != 0
    {
        if win.is_null() {
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                if buf.is_null() || (*wp).w_buffer == buf {
                    redraw_status(wp, opts, &raw mut (*opts).flush);
                }
                wp = (*wp).w_next;
            }
        } else {
            redraw_status(win, opts, &raw mut (*opts).flush);
        }
        flush_ui = true_0 != 0;
    }
    let mut cwin: *mut win_T = if !win.is_null() { win } else { curwin.get() };
    if (*opts).cursor as ::core::ffi::c_int != 0
        && ((*cwin).w_grid.target.is_null() || !(*(*cwin).w_grid.target).valid)
    {
        (*opts).flush = true_0 != 0;
    }
    if (*opts).flush as ::core::ffi::c_int != 0 && !cmdpreview.get() {
        validate_cursor(curwin.get());
        update_topline(curwin.get());
        update_screen();
    }
    if (*opts).cursor {
        setcursor_mayforce(cwin, true_0 != 0);
        flush_ui = true_0 != 0;
    }
    if flush_ui {
        ui_flush();
    }
    RedrawingDisabled.set(save_rd);
    p_lz.set(save_lz as ::core::ffi::c_int);
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CONTEXT_INIT: Context = Context {
    regs: STRING_INIT,
    jumps: STRING_INIT,
    bufs: STRING_INIT,
    gvars: STRING_INIT,
    funcs: Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    },
};
pub const MODE_MAX_LENGTH: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
