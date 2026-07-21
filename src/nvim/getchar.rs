use crate::src::nvim::global_cell::{GlobalCell, SharedCell};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, ApiDispatchWrapper, Arena, ArenaMem, Array, BoolVarValue,
    Boolean, BufUpdateCallbacks, CSType, Callback, CallbackType, Callback_data as C2Rust_Unnamed_5,
    ChangedtickDictItem, CharInfo, CharSize, CharsizeArg, CmdRedraw, CmdlineColorChunk,
    CmdlineColors, CmdlineInfo, ColoredCmdline, DecorExt, DecorHighlightInline, DecorInlineData,
    DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, Dict, Direction, Error,
    ErrorType, EvalFuncData, ExtmarkUndoObject, FileDescriptor, FileID, Float, FloatAnchor,
    FloatRelative, GridView, Integer, Intersection, KeyValuePair, Loop, LuaRef, LuaRetMode, MTKey,
    MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, MarkTreeIter, MarkTreeIter_s as C2Rust_Unnamed_28, MotionType,
    MsgpackRpcRequestHandler, MultiQueue, Object, ObjectType, OptInt, Proc, ProcType, RStream,
    RemapValues, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_12,
    StrCharInfo, Stream, String_0, Terminal, Timestamp, TriState, VarLockStatus, VarType,
    VimVarIndex, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit,
    WinStyle, Window, _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, __off64_t, __off_t,
    __pthread_internal_list, __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t,
    __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, buffblock, buffblock_T,
    buffheader_T, bufstate_T, chunksize_T, cmdline_info, colnr_T, consumed_blk, dict_T, dictvar_S,
    disptick_T, expand_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_3, file_buffer_b_wininfo as C2Rust_Unnamed_11,
    file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, flush_buffers_T, fmark_T, fmarkv_T,
    frame_S, frame_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T,
    handle_T, hash_T, hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t,
    internal_proc_cb, key_extra, key_value_pair, lcs_chars_T, linenr_T, list_T, listitem_S,
    listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, loop_0,
    loop_0_children as C2Rust_Unnamed_21, lpos_T, mapblock, mapblock_T, match_T, matchitem,
    matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s, mtnode_s, multiqueue, object,
    object_data as C2Rust_Unnamed, oparg_T, partial_S, partial_T, pos_T, pos_save_T, proc,
    proc_exit_cb, proc_state_cb, proftime_T, pthread_mutex_t, pthread_rwlock_t, ptr_t, ptrdiff_t,
    qf_info_S, qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, rstream, sattr_T,
    save_redo_T, schar_T, scid_T, sctx_T, size_t, ssize_t, stream, stream_close_cb, stream_read_cb,
    stream_uv as C2Rust_Unnamed_23, stream_write_cb, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, taggy_T, tasave_T,
    terminal, time_t, typebuf_T, typval_T, typval_vval_union, u_entry, u_entry_T, u_header,
    u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8, u_header_uh_alt_prev as C2Rust_Unnamed_7,
    u_header_uh_next as C2Rust_Unnamed_10, u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, uintptr_t, undo_object, uv__io_cb, uv__io_s, uv__io_t,
    uv__queue, uv_alloc_cb, uv_async_cb, uv_async_s, uv_async_s_u as C2Rust_Unnamed_18, uv_async_t,
    uv_buf_t, uv_close_cb, uv_connect_cb, uv_connect_s, uv_connect_t, uv_connection_cb, uv_file,
    uv_handle_s, uv_handle_s_u as C2Rust_Unnamed_13, uv_handle_t, uv_handle_type, uv_idle_cb,
    uv_idle_s, uv_idle_s_u as C2Rust_Unnamed_24, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_17, uv_loop_s_timer_heap as C2Rust_Unnamed_16,
    uv_loop_t, uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_26, uv_pipe_t, uv_read_cb,
    uv_req_type, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t, uv_signal_cb,
    uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed_14, uv_signal_s_u as C2Rust_Unnamed_15,
    uv_signal_t, uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_22, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_25, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_19, uv_timer_s_u as C2Rust_Unnamed_20, uv_timer_t,
    varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T,
    xp_prefix_T, FILE, QUEUE, _IO_FILE,
};
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    static mut stderr: *mut FILE;
    fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn putc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xmemcpyz(
        dst: *mut ::core::ffi::c_void,
        src: *const ::core::ffi::c_void,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn arena_finish(arena: *mut Arena) -> ArenaMem;
    fn arena_mem_free(mem: ArenaMem);
    fn api_clear_error(value: *mut Error);
    fn nvim_paste(
        channel_id: uint64_t,
        data: String_0,
        crlf: Boolean,
        phase: Integer,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Boolean;
    static p_fs: GlobalCell<::core::ffi::c_int>;
    static p_langmap: GlobalCell<*mut ::core::ffi::c_char>;
    static p_lrm: GlobalCell<::core::ffi::c_int>;
    static p_lz: GlobalCell<::core::ffi::c_int>;
    static p_mmd: GlobalCell<OptInt>;
    static p_paste: GlobalCell<::core::ffi::c_int>;
    static p_sc: GlobalCell<::core::ffi::c_int>;
    static p_smd: GlobalCell<::core::ffi::c_int>;
    static p_timeout: GlobalCell<::core::ffi::c_int>;
    static p_tm: GlobalCell<OptInt>;
    static p_ttimeout: GlobalCell<::core::ffi::c_int>;
    static p_ttm: GlobalCell<OptInt>;
    static p_uc: GlobalCell<OptInt>;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn uv_strerror(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn get_cursor_line_ptr() -> *mut ::core::ffi::c_char;
    fn update_screen() -> ::core::ffi::c_int;
    fn setcursor();
    fn showmode() -> ::core::ffi::c_int;
    fn unshowmode(force: bool);
    fn edit_putchar(c: ::core::ffi::c_int, highlight: bool);
    fn edit_unputchar();
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static e_invarg2: [::core::ffi::c_char; 0];
    static e_invargNval: [::core::ffi::c_char; 0];
    static e_nesting: [::core::ffi::c_char; 0];
    static e_notopen_2: [::core::ffi::c_char; 0];
    static e_toocompl: [::core::ffi::c_char; 0];
    fn garbage_collect(testing: bool) -> bool;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn semsg_multiline(
        kind: *const ::core::ffi::c_char,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn iemsg(s: *const ::core::ffi::c_char);
    fn internal_error(where_0: *const ::core::ffi::c_char);
    fn tv_dict_has_key(d: *const dict_T, key: *const ::core::ffi::c_char) -> bool;
    fn tv_dict_get_bool(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        def: ::core::ffi::c_int,
    ) -> varnumber_T;
    fn tv_dict_get_string(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        save: bool,
    ) -> *mut ::core::ffi::c_char;
    fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T;
    fn tv_check_for_opt_dict_arg(
        args: *const typval_T,
        idx: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn set_vim_var_nr(idx: VimVarIndex, val: varnumber_T);
    fn multiqueue_empty(self_0: *mut MultiQueue) -> bool;
    fn check_secure() -> bool;
    fn update_topline_cursor();
    fn putcmdline(c: ::core::ffi::c_char, shift: bool);
    fn unputcmdline();
    fn redrawcmdline();
    fn redrawcmd();
    fn get_cmdline_info() -> *mut CmdlineInfo;
    static test_disable_char_avail: GlobalCell<bool>;
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn ga_concat_len(gap: *mut garray_T, s: *const ::core::ffi::c_char, len: size_t);
    fn ga_append(gap: *mut garray_T, c: uint8_t);
    static mod_mask: GlobalCell<::core::ffi::c_int>;
    static vgetc_mod_mask: GlobalCell<::core::ffi::c_int>;
    static vgetc_char: GlobalCell<::core::ffi::c_int>;
    static cmdline_row: GlobalCell<::core::ffi::c_int>;
    static redraw_cmdline: GlobalCell<bool>;
    static mode_displayed: GlobalCell<bool>;
    static cmdline_star: GlobalCell<::core::ffi::c_int>;
    static msg_col: GlobalCell<::core::ffi::c_int>;
    static msg_row: GlobalCell<::core::ffi::c_int>;
    static msg_scroll: GlobalCell<::core::ffi::c_int>;
    static msg_didout: GlobalCell<bool>;
    static did_emsg: GlobalCell<::core::ffi::c_int>;
    static called_emsg: GlobalCell<::core::ffi::c_int>;
    static need_wait_return: GlobalCell<bool>;
    static vgetc_busy: GlobalCell<::core::ffi::c_int>;
    static debug_did_msg: GlobalCell<bool>;
    static may_garbage_collect: GlobalCell<bool>;
    static want_garbage_collect: GlobalCell<bool>;
    static mouse_grid: GlobalCell<::core::ffi::c_int>;
    static mouse_row: GlobalCell<::core::ffi::c_int>;
    static mouse_col: GlobalCell<::core::ffi::c_int>;
    static firstwin: GlobalCell<*mut win_T>;
    static curwin: GlobalCell<*mut win_T>;
    static curbuf: GlobalCell<*mut buf_T>;
    static VIsual: GlobalCell<pos_T>;
    static VIsual_active: GlobalCell<bool>;
    static VIsual_select: GlobalCell<bool>;
    static VIsual_reselect: GlobalCell<::core::ffi::c_int>;
    static redo_VIsual_busy: GlobalCell<bool>;
    static did_ai: GlobalCell<bool>;
    static State: GlobalCell<::core::ffi::c_int>;
    static finish_op: GlobalCell<bool>;
    static exmode_active: GlobalCell<bool>;
    static pending_exmode_active: GlobalCell<bool>;
    static reg_recording: GlobalCell<::core::ffi::c_int>;
    static reg_executing: GlobalCell<::core::ffi::c_int>;
    static pending_end_reg_executing: GlobalCell<bool>;
    static no_mapping: GlobalCell<::core::ffi::c_int>;
    static no_zero_mapping: GlobalCell<::core::ffi::c_int>;
    static allow_keys: GlobalCell<::core::ffi::c_int>;
    static restart_edit: GlobalCell<::core::ffi::c_int>;
    static arrow_used: GlobalCell<bool>;
    static mapped_ctrl_c: GlobalCell<::core::ffi::c_int>;
    static ctrl_c_interrupts: GlobalCell<bool>;
    static msg_silent: GlobalCell<::core::ffi::c_int>;
    static emsg_silent: GlobalCell<::core::ffi::c_int>;
    static cmd_silent: GlobalCell<bool>;
    static NameBuff: GlobalCell<[::core::ffi::c_char; 4096]>;
    static typebuf: GlobalCell<typebuf_T>;
    static typebuf_was_empty: GlobalCell<bool>;
    static ex_normal_busy: GlobalCell<::core::ffi::c_int>;
    static ignore_script: GlobalCell<bool>;
    static KeyTyped: GlobalCell<bool>;
    static KeyStuffed: GlobalCell<::core::ffi::c_int>;
    static maptick: GlobalCell<::core::ffi::c_int>;
    static must_redraw: GlobalCell<::core::ffi::c_int>;
    static scriptout: GlobalCell<*mut FILE>;
    static got_int: GlobalCell<bool>;
    static did_outofmem_msg: GlobalCell<bool>;
    static did_swapwrite_msg: GlobalCell<bool>;
    static langmap_mapchar: GlobalCell<[uint8_t; 256]>;
    static cmdwin_type: GlobalCell<::core::ffi::c_int>;
    static typebuf_was_filled: GlobalCell<bool>;
    fn get_keystroke(events: *mut MultiQueue) -> ::core::ffi::c_int;
    fn ctrl_x_mode_not_default() -> bool;
    fn compl_status_local() -> bool;
    fn vim_is_ctrl_x_key(c: ::core::ffi::c_int) -> bool;
    fn special_to_buf(
        key: ::core::ffi::c_int,
        modifiers: ::core::ffi::c_int,
        escape_ks: bool,
        dst: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_uint;
    fn nlua_call_ref(
        ref_0: LuaRef,
        name: *const ::core::ffi::c_char,
        args: Array,
        mode: LuaRetMode,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn nlua_execute_on_key(c: ::core::ffi::c_int, typed_buf: *mut ::core::ffi::c_char) -> bool;
    fn get_maphash_list(state: ::core::ffi::c_int, c: ::core::ffi::c_int) -> *mut mapblock_T;
    fn get_buf_maphash_list(state: ::core::ffi::c_int, c: ::core::ffi::c_int) -> *mut mapblock_T;
    fn eval_map_expr(mp: *mut mapblock_T, c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn langmap_adjust_mb(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    static main_loop: SharedCell<Loop>;
    fn utf_ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2CharInfo_impl(p: *const uint8_t, len: uintptr_t) -> int32_t;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_cptr2char_adv(pp: *mut *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn utfc_next_impl(cur: StrCharInfo) -> StrCharInfo;
    fn mb_unescape(pp: *mut *const ::core::ffi::c_char) -> *const ::core::ffi::c_char;
    static utf8len_tab: [uint8_t; 256];
    fn ml_sync_all(check_file: ::core::ffi::c_int, check_char: ::core::ffi::c_int, do_fsync: bool);
    fn is_mouse_key(c: ::core::ffi::c_int) -> bool;
    fn mouse_comp_pos(
        win: *mut win_T,
        rowp: *mut ::core::ffi::c_int,
        colp: *mut ::core::ffi::c_int,
        lnump: *mut linenr_T,
    ) -> bool;
    fn mouse_find_win_inner(
        gridp: *mut ::core::ffi::c_int,
        rowp: *mut ::core::ffi::c_int,
        colp: *mut ::core::ffi::c_int,
    ) -> *mut win_T;
    fn validate_cursor(wp: *mut win_T);
    fn win_col_off(wp: *mut win_T) -> ::core::ffi::c_int;
    fn add_to_showcmd(c: ::core::ffi::c_int) -> bool;
    fn push_showcmd();
    fn pop_showcmd();
    fn normal_cmd(oap: *mut oparg_T, toplevel: bool);
    fn clear_oparg(oap: *mut oparg_T);
    static repeat_luaref: GlobalCell<LuaRef>;
    fn file_open(
        ret_fp: *mut FileDescriptor,
        fname: *const ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        mode: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn file_open_stdin(fp: *mut FileDescriptor) -> ::core::ffi::c_int;
    fn file_close(fp: *mut FileDescriptor, do_fsync: bool) -> ::core::ffi::c_int;
    fn file_read(
        fp: *mut FileDescriptor,
        ret_buf: *mut ::core::ffi::c_char,
        size: size_t,
    ) -> ptrdiff_t;
    fn input_get(
        buf: *mut uint8_t,
        maxlen: ::core::ffi::c_int,
        ms: ::core::ffi::c_int,
        tb_change_cnt: ::core::ffi::c_int,
        events: *mut MultiQueue,
    ) -> ::core::ffi::c_int;
    fn os_breakcheck();
    fn line_breakcheck();
    fn input_available() -> size_t;
    fn expand_env(
        src: *mut ::core::ffi::c_char,
        dst: *mut ::core::ffi::c_char,
        dstlen: ::core::ffi::c_int,
    ) -> size_t;
    fn init_charsize_arg(
        csarg: *mut CharsizeArg,
        wp: *mut win_T,
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
    ) -> CSType;
    fn charsize_regular(
        csarg: *mut CharsizeArg,
        cur: *mut ::core::ffi::c_char,
        vcol: colnr_T,
        cur_char: int32_t,
    ) -> CharSize;
    fn charsize_fast(
        csarg: *mut CharsizeArg,
        cur: *const ::core::ffi::c_char,
        vcol: colnr_T,
        cur_char: int32_t,
    ) -> CharSize;
    fn state_handle_k_event();
    fn get_real_state() -> ::core::ffi::c_int;
    fn state_no_longer_safe(reason: *const ::core::ffi::c_char);
    fn ui_busy_start();
    fn ui_busy_stop();
    fn vim_beep(val: ::core::ffi::c_uint);
    fn ui_cursor_goto(new_row: ::core::ffi::c_int, new_col: ::core::ffi::c_int);
    fn ui_flush();
    fn u_sync(force: bool);
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
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const MAXMAPLEN: C2Rust_Unnamed_27 = 50;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const kOptBoFlagWildmode: C2Rust_Unnamed_29 = 524288;
pub const kOptBoFlagTerm: C2Rust_Unnamed_29 = 262144;
pub const kOptBoFlagSpell: C2Rust_Unnamed_29 = 131072;
pub const kOptBoFlagShell: C2Rust_Unnamed_29 = 65536;
pub const kOptBoFlagRegister: C2Rust_Unnamed_29 = 32768;
pub const kOptBoFlagOperator: C2Rust_Unnamed_29 = 16384;
pub const kOptBoFlagShowmatch: C2Rust_Unnamed_29 = 8192;
pub const kOptBoFlagMess: C2Rust_Unnamed_29 = 4096;
pub const kOptBoFlagLang: C2Rust_Unnamed_29 = 2048;
pub const kOptBoFlagInsertmode: C2Rust_Unnamed_29 = 1024;
pub const kOptBoFlagHangul: C2Rust_Unnamed_29 = 512;
pub const kOptBoFlagEx: C2Rust_Unnamed_29 = 256;
pub const kOptBoFlagEsc: C2Rust_Unnamed_29 = 128;
pub const kOptBoFlagError: C2Rust_Unnamed_29 = 64;
pub const kOptBoFlagCtrlg: C2Rust_Unnamed_29 = 32;
pub const kOptBoFlagCopy: C2Rust_Unnamed_29 = 16;
pub const kOptBoFlagComplete: C2Rust_Unnamed_29 = 8;
pub const kOptBoFlagCursor: C2Rust_Unnamed_29 = 4;
pub const kOptBoFlagBackspace: C2Rust_Unnamed_29 = 2;
pub const kOptBoFlagAll: C2Rust_Unnamed_29 = 1;
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
pub const REMAP_SKIP: RemapValues = -3;
pub const REMAP_SCRIPT: RemapValues = -2;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub const kCmdRedrawAll: CmdRedraw = 2;
pub const kCmdRedrawPos: CmdRedraw = 1;
pub const kCmdRedrawNone: CmdRedraw = 0;
pub const FLUSH_INPUT: flush_buffers_T = 2;
pub const FLUSH_TYPEAHEAD: flush_buffers_T = 1;
pub const FLUSH_MINIMAL: flush_buffers_T = 0;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const NSCRIPT: C2Rust_Unnamed_30 = 15;
pub const MODE_HITRETURN: C2Rust_Unnamed_32 = 8193;
pub const RM_SCRIPT: C2Rust_Unnamed_36 = 2;
pub const RM_NONE: C2Rust_Unnamed_36 = 1;
pub const RM_YES: C2Rust_Unnamed_36 = 0;
pub const RM_ABBR: C2Rust_Unnamed_36 = 4;
pub const KE_IGNORE: key_extra = 53;
pub const MODE_CMDLINE: C2Rust_Unnamed_32 = 8;
pub const MODE_INSERT: C2Rust_Unnamed_32 = 16;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct gotchars_state_T {
    pub buf: [uint8_t; 67],
    pub prev_c: ::core::ffi::c_int,
    pub buflen: size_t,
    pub pending_special: ::core::ffi::c_uint,
    pub pending_mbyte: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_31 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_char,
    pub init_array: [::core::ffi::c_char; 51],
}
pub const KEYLEN_PART_KEY: C2Rust_Unnamed_37 = -1;
pub const SHOWCMD_COLS: C2Rust_Unnamed_33 = 10;
pub const MODE_LANGMAP: C2Rust_Unnamed_32 = 32;
pub const MODE_NORMAL: C2Rust_Unnamed_32 = 1;
pub const kCharsizeFast: C2Rust_Unnamed_35 = 1;
pub const map_result_get: map_result_T = 1;
pub type map_result_T = ::core::ffi::c_uint;
pub const map_result_nomatch: map_result_T = 3;
pub const map_result_retry: map_result_T = 2;
pub const map_result_fail: map_result_T = 0;
pub const MODE_VISUAL: C2Rust_Unnamed_32 = 2;
pub const MODE_TERMINAL: C2Rust_Unnamed_32 = 128;
pub const KEYLEN_PART_MAP: C2Rust_Unnamed_37 = -2;
pub const KE_SNR: key_extra = 82;
pub const MODE_SELECT: C2Rust_Unnamed_32 = 64;
pub const MODE_ASKMORE: C2Rust_Unnamed_32 = 12288;
pub const KE_PLUG: key_extra = 83;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub const kFileReadOnly: C2Rust_Unnamed_34 = 1;
pub const kFileNonBlocking: C2Rust_Unnamed_34 = 128;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub const KE_LUA: key_extra = 103;
pub const KE_COMMAND: key_extra = 104;
pub const KE_XRIGHT: key_extra = 68;
pub const KE_XLEFT: key_extra = 67;
pub const KE_XDOWN: key_extra = 66;
pub const KE_XUP: key_extra = 65;
pub const KE_C_END: key_extra = 88;
pub const KE_ZEND: key_extra = 62;
pub const KE_XEND: key_extra = 61;
pub const KE_C_HOME: key_extra = 87;
pub const KE_ZHOME: key_extra = 64;
pub const KE_XHOME: key_extra = 63;
pub const KE_MOUSEMOVE: key_extra = 100;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_32 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_32 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_32 = 16384;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_32 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_32 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_32 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_32 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_32 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_32 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_32 = 255;
pub const MODE_OP_PENDING: C2Rust_Unnamed_32 = 4;
pub const KE_WILD: key_extra = 108;
pub const KE_EVENT: key_extra = 102;
pub const KE_NOP: key_extra = 97;
pub const KE_DROP: key_extra = 95;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_CMDWIN: key_extra = 84;
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
pub const KE_XF4: key_extra = 60;
pub const KE_XF3: key_extra = 59;
pub const KE_XF2: key_extra = 58;
pub const KE_XF1: key_extra = 57;
pub const KE_S_TAB_OLD: key_extra = 55;
pub const KE_TAB: key_extra = 54;
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
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const kFileMkDir: C2Rust_Unnamed_34 = 256;
pub const kFileAppend: C2Rust_Unnamed_34 = 64;
pub const kFileTruncate: C2Rust_Unnamed_34 = 32;
pub const kFileCreateOnly: C2Rust_Unnamed_34 = 16;
pub const kFileNoSymlink: C2Rust_Unnamed_34 = 8;
pub const kFileWriteOnly: C2Rust_Unnamed_34 = 4;
pub const kFileCreate: C2Rust_Unnamed_34 = 2;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const kCharsizeRegular: C2Rust_Unnamed_35 = 0;
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_37 = ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
#[inline(always)]
unsafe extern "C" fn _memcpy_free(
    dest: *mut ::core::ffi::c_void,
    src: *mut ::core::ffi::c_void,
    size: size_t,
) -> *mut ::core::ffi::c_void {
    memcpy(dest, src, size);
    let mut ptr_: *mut *mut ::core::ffi::c_void = &raw const src as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    return dest;
}
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const INTERNAL_CALL_MASK: uint64_t = (1 as ::core::ffi::c_int as uint64_t)
    << ::core::mem::size_of::<uint64_t>()
        .wrapping_mul(8 as usize)
        .wrapping_sub(1 as usize);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL.wrapping_add(1 as uint64_t);
#[inline(always)]
unsafe extern "C" fn is_internal_call(channel_id: uint64_t) -> bool {
    return channel_id & INTERNAL_CALL_MASK != 0;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const NL_STR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"\n\0") };
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const DEL: ::core::ffi::c_int = 0x7f as ::core::ffi::c_int;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const Ctrl_N: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const Ctrl_O: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const Ctrl_P: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
static curscript: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
static scriptin: GlobalCell<[FileDescriptor; 15]> = GlobalCell::new([
    FileDescriptor {
        fd: 0 as ::core::ffi::c_int,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
    FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    },
]);
static redobuff: GlobalCell<buffheader_T> = GlobalCell::new(buffheader_T {
    bh_first: buffblock {
        b_next: ::core::ptr::null_mut::<buffblock>(),
        b_strlen: 0 as size_t,
        b_str: [NUL as ::core::ffi::c_char],
    },
    bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
    bh_index: 0 as size_t,
    bh_space: 0 as size_t,
    bh_create_newblock: false_0 != 0,
});
static old_redobuff: GlobalCell<buffheader_T> = GlobalCell::new(buffheader_T {
    bh_first: buffblock {
        b_next: ::core::ptr::null_mut::<buffblock>(),
        b_strlen: 0 as size_t,
        b_str: [NUL as ::core::ffi::c_char],
    },
    bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
    bh_index: 0 as size_t,
    bh_space: 0 as size_t,
    bh_create_newblock: false_0 != 0,
});
static recordbuff: GlobalCell<buffheader_T> = GlobalCell::new(buffheader_T {
    bh_first: buffblock {
        b_next: ::core::ptr::null_mut::<buffblock>(),
        b_strlen: 0 as size_t,
        b_str: [NUL as ::core::ffi::c_char],
    },
    bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
    bh_index: 0 as size_t,
    bh_space: 0 as size_t,
    bh_create_newblock: false_0 != 0,
});
static readbuf1: GlobalCell<buffheader_T> = GlobalCell::new(buffheader_T {
    bh_first: buffblock {
        b_next: ::core::ptr::null_mut::<buffblock>(),
        b_strlen: 0 as size_t,
        b_str: [NUL as ::core::ffi::c_char],
    },
    bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
    bh_index: 0 as size_t,
    bh_space: 0 as size_t,
    bh_create_newblock: false_0 != 0,
});
static readbuf2: GlobalCell<buffheader_T> = GlobalCell::new(buffheader_T {
    bh_first: buffblock {
        b_next: ::core::ptr::null_mut::<buffblock>(),
        b_strlen: 0 as size_t,
        b_str: [NUL as ::core::ffi::c_char],
    },
    bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
    bh_index: 0 as size_t,
    bh_space: 0 as size_t,
    bh_create_newblock: false_0 != 0,
});
static on_key_buf: GlobalCell<C2Rust_Unnamed_31> = GlobalCell::new(C2Rust_Unnamed_31 {
    size: 0,
    capacity: 0,
    items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    init_array: [0; 51],
});
static on_key_ignore_len: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
static typeahead_char: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static block_redo: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static KeyNoremap: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static typebuf_init: GlobalCell<[uint8_t; 265]> = GlobalCell::new([0; 265]);
static noremapbuf_init: GlobalCell<[uint8_t; 265]> = GlobalCell::new([0; 265]);
static last_recorded_len: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
static e_recursive_mapping: GlobalCell<[::core::ffi::c_char; 24]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E223: Recursive mapping\0")
});
static e_cmd_mapping_must_end_with_cr: GlobalCell<[::core::ffi::c_char; 40]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
            *b"E1255: <Cmd> mapping must end with <CR>\0",
        )
    });
static e_cmd_mapping_must_end_with_cr_before_second_cmd: GlobalCell<[::core::ffi::c_char; 60]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 60], [::core::ffi::c_char; 60]>(
            *b"E1136: <Cmd> mapping must end with <CR> before second <Cmd>\0",
        )
    });
unsafe extern "C" fn free_buff(mut buf: *mut buffheader_T) {
    let mut np: *mut buffblock_T = ::core::ptr::null_mut::<buffblock_T>();
    let mut p: *mut buffblock_T = (*buf).bh_first.b_next as *mut buffblock_T;
    while !p.is_null() {
        np = (*p).b_next as *mut buffblock_T;
        xfree(p as *mut ::core::ffi::c_void);
        p = np;
    }
    (*buf).bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
    (*buf).bh_curr = ::core::ptr::null_mut::<buffblock_T>();
}
unsafe extern "C" fn get_buffcont(
    mut buffer: *mut buffheader_T,
    mut dozero: ::core::ffi::c_int,
    mut len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut count: size_t = 0 as size_t;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: size_t = 0 as size_t;
    let mut bp: *const buffblock_T = (*buffer).bh_first.b_next;
    while !bp.is_null() {
        count = count.wrapping_add((*bp).b_strlen);
        bp = (*bp).b_next;
    }
    if count > 0 as size_t || dozero != 0 {
        p = xmalloc(count.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        let mut p2: *mut ::core::ffi::c_char = p;
        let mut bp_0: *const buffblock_T = (*buffer).bh_first.b_next;
        while !bp_0.is_null() {
            let mut str: *const ::core::ffi::c_char =
                &raw const (*bp_0).b_str as *const ::core::ffi::c_char;
            while *str != 0 {
                let c2rust_fresh0 = str;
                str = str.offset(1);
                let c2rust_fresh1 = p2;
                p2 = p2.offset(1);
                *c2rust_fresh1 = *c2rust_fresh0;
            }
            bp_0 = (*bp_0).b_next;
        }
        *p2 = NUL as ::core::ffi::c_char;
        i = p2.offset_from(p) as size_t;
    }
    if !len.is_null() {
        *len = i;
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn get_recorded() -> *mut ::core::ffi::c_char {
    let mut len: size_t = 0;
    let mut p: *mut ::core::ffi::c_char = get_buffcont(recordbuff.ptr(), true_0, &raw mut len);
    if p.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    free_buff(recordbuff.ptr());
    if len >= last_recorded_len.get() {
        len = len.wrapping_sub(last_recorded_len.get());
        *p.offset(len as isize) = NUL as ::core::ffi::c_char;
    }
    if len > 0 as size_t
        && restart_edit.get() != 0 as ::core::ffi::c_int
        && *p.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int == Ctrl_O
    {
        *p.offset(len.wrapping_sub(1 as size_t) as isize) = NUL as ::core::ffi::c_char;
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn get_inserted() -> String_0 {
    let mut len: size_t = 0 as size_t;
    let mut str: *mut ::core::ffi::c_char = get_buffcont(redobuff.ptr(), false_0, &raw mut len);
    return String_0 {
        data: str,
        size: len,
    };
}
unsafe extern "C" fn add_buff(
    buf: *mut buffheader_T,
    s: *const ::core::ffi::c_char,
    mut slen: ptrdiff_t,
) {
    if slen < 0 as ptrdiff_t {
        slen = strlen(s) as ptrdiff_t;
    }
    if slen == 0 as ptrdiff_t {
        return;
    }
    if (*buf).bh_first.b_next.is_null() {
        (*buf).bh_curr = &raw mut (*buf).bh_first;
        (*buf).bh_create_newblock = true_0 != 0;
    } else if (*buf).bh_curr.is_null() {
        iemsg(gettext(
            b"E222: Add to read buffer\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return;
    } else if (*buf).bh_index != 0 as size_t {
        memmove(
            &raw mut (*(*buf).bh_first.b_next).b_str as *mut ::core::ffi::c_char
                as *mut ::core::ffi::c_void,
            (&raw mut (*(*buf).bh_first.b_next).b_str as *mut ::core::ffi::c_char)
                .offset((*buf).bh_index as isize) as *const ::core::ffi::c_void,
            (*(*buf).bh_first.b_next)
                .b_strlen
                .wrapping_sub((*buf).bh_index)
                .wrapping_add(1 as size_t),
        );
        (*(*buf).bh_first.b_next).b_strlen = (*(*buf).bh_first.b_next)
            .b_strlen
            .wrapping_sub((*buf).bh_index);
        (*buf).bh_space = (*buf).bh_space.wrapping_add((*buf).bh_index);
    }
    (*buf).bh_index = 0 as size_t;
    if !(*buf).bh_create_newblock && (*buf).bh_space >= slen as size_t {
        xmemcpyz(
            (&raw mut (*(*buf).bh_curr).b_str as *mut ::core::ffi::c_char)
                .offset((*(*buf).bh_curr).b_strlen as isize)
                as *mut ::core::ffi::c_void,
            s as *const ::core::ffi::c_void,
            slen as size_t,
        );
        (*(*buf).bh_curr).b_strlen = (*(*buf).bh_curr).b_strlen.wrapping_add(slen as size_t);
        (*buf).bh_space = (*buf).bh_space.wrapping_sub(slen as size_t);
    } else {
        let mut len: size_t = if 20 as size_t > slen as size_t {
            20 as size_t
        } else {
            slen as size_t
        };
        let mut p: *mut buffblock_T =
            xmalloc((16 as size_t).wrapping_add(len).wrapping_add(1 as size_t)) as *mut buffblock_T;
        xmemcpyz(
            &raw mut (*p).b_str as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            s as *const ::core::ffi::c_void,
            slen as size_t,
        );
        (*p).b_strlen = slen as size_t;
        (*buf).bh_space = len.wrapping_sub(slen as size_t);
        (*buf).bh_create_newblock = false_0 != 0;
        (*p).b_next = (*(*buf).bh_curr).b_next;
        (*(*buf).bh_curr).b_next = p as *mut buffblock;
        (*buf).bh_curr = p;
    };
}
unsafe extern "C" fn delete_buff_tail(mut buf: *mut buffheader_T, mut slen: ::core::ffi::c_int) {
    if (*buf).bh_curr.is_null() {
        return;
    }
    if (*(*buf).bh_curr).b_strlen < slen as size_t {
        return;
    }
    *(&raw mut (*(*buf).bh_curr).b_str as *mut ::core::ffi::c_char)
        .offset((*(*buf).bh_curr).b_strlen.wrapping_sub(slen as size_t) as isize) =
        NUL as ::core::ffi::c_char;
    (*(*buf).bh_curr).b_strlen = (*(*buf).bh_curr).b_strlen.wrapping_sub(slen as size_t);
    (*buf).bh_space = (*buf).bh_space.wrapping_add(slen as size_t);
}
unsafe extern "C" fn add_num_buff(mut buf: *mut buffheader_T, mut n: ::core::ffi::c_int) {
    let mut number: [::core::ffi::c_char; 32] = [0; 32];
    let mut numberlen: ::core::ffi::c_int = snprintf(
        &raw mut number as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 32]>(),
        b"%d\0".as_ptr() as *const ::core::ffi::c_char,
        n,
    );
    add_buff(
        buf,
        &raw mut number as *mut ::core::ffi::c_char,
        numberlen as ptrdiff_t,
    );
}
unsafe extern "C" fn add_byte_buff(mut buf: *mut buffheader_T, mut c: ::core::ffi::c_int) {
    let mut temp: [::core::ffi::c_char; 4] = [0; 4];
    let mut templen: ptrdiff_t = 0;
    if c < 0 as ::core::ffi::c_int || c == K_SPECIAL || c == NUL {
        temp[0 as ::core::ffi::c_int as usize] = K_SPECIAL as ::core::ffi::c_char;
        temp[1 as ::core::ffi::c_int as usize] = (if c == K_SPECIAL {
            KS_SPECIAL
        } else if c == NUL {
            KS_ZERO
        } else {
            -c & 0xff as ::core::ffi::c_int
        }) as ::core::ffi::c_char;
        temp[2 as ::core::ffi::c_int as usize] = (if c == K_SPECIAL || c == NUL {
            KE_FILLER as ::core::ffi::c_uint
        } else {
            -c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint
        }) as ::core::ffi::c_char;
        temp[3 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        templen = 3 as ptrdiff_t;
    } else {
        temp[0 as ::core::ffi::c_int as usize] = c as ::core::ffi::c_char;
        temp[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        templen = 1 as ptrdiff_t;
    }
    add_buff(buf, &raw mut temp as *mut ::core::ffi::c_char, templen);
}
unsafe extern "C" fn add_char_buff(mut buf: *mut buffheader_T, mut c: ::core::ffi::c_int) {
    let mut bytes: [uint8_t; 22] = [0; 22];
    let mut len: ::core::ffi::c_int = 0;
    if c < 0 as ::core::ffi::c_int {
        len = 1 as ::core::ffi::c_int;
    } else {
        len = utf_char2bytes(
            c,
            &raw mut bytes as *mut uint8_t as *mut ::core::ffi::c_char,
        );
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < len {
        if !(c < 0 as ::core::ffi::c_int) {
            c = bytes[i as usize] as ::core::ffi::c_int;
        }
        add_byte_buff(buf, c);
        i += 1;
    }
}
unsafe extern "C" fn read_readbuffers(mut advance: bool) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = read_readbuf(readbuf1.ptr(), advance);
    if c == NUL {
        c = read_readbuf(readbuf2.ptr(), advance);
    }
    return c;
}
unsafe extern "C" fn read_readbuf(
    mut buf: *mut buffheader_T,
    mut advance: bool,
) -> ::core::ffi::c_int {
    if (*buf).bh_first.b_next.is_null() {
        return NUL;
    }
    let curr: *mut buffblock_T = (*buf).bh_first.b_next as *mut buffblock_T;
    let mut c: uint8_t = *(&raw mut (*curr).b_str as *mut ::core::ffi::c_char)
        .offset((*buf).bh_index as isize) as uint8_t;
    if advance {
        (*buf).bh_index = (*buf).bh_index.wrapping_add(1);
        if *(&raw mut (*curr).b_str as *mut ::core::ffi::c_char).offset((*buf).bh_index as isize)
            as ::core::ffi::c_int
            == NUL
        {
            (*buf).bh_first.b_next = (*curr).b_next;
            xfree(curr as *mut ::core::ffi::c_void);
            (*buf).bh_index = 0 as size_t;
        }
    }
    return c as ::core::ffi::c_int;
}
unsafe extern "C" fn start_stuff() {
    if !(*readbuf1.ptr()).bh_first.b_next.is_null() {
        (*readbuf1.ptr()).bh_curr = &raw mut (*readbuf1.ptr()).bh_first;
        (*readbuf1.ptr()).bh_create_newblock = true_0 != 0;
    }
    if !(*readbuf2.ptr()).bh_first.b_next.is_null() {
        (*readbuf2.ptr()).bh_curr = &raw mut (*readbuf2.ptr()).bh_first;
        (*readbuf2.ptr()).bh_create_newblock = true_0 != 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn stuff_empty() -> bool {
    return (*readbuf1.ptr()).bh_first.b_next.is_null()
        && (*readbuf2.ptr()).bh_first.b_next.is_null();
}
#[no_mangle]
pub unsafe extern "C" fn readbuf1_empty() -> bool {
    return (*readbuf1.ptr()).bh_first.b_next.is_null();
}
#[no_mangle]
pub unsafe extern "C" fn typeahead_noflush(mut c: ::core::ffi::c_int) {
    typeahead_char.set(c);
}
#[no_mangle]
pub unsafe extern "C" fn flush_buffers(mut flush_typeahead: flush_buffers_T) {
    init_typebuf();
    start_stuff();
    while read_readbuffers(true_0 != 0) != NUL {}
    if flush_typeahead as ::core::ffi::c_uint
        == FLUSH_MINIMAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*typebuf.ptr()).tb_off + (*typebuf.ptr()).tb_maplen >= (*typebuf.ptr()).tb_buflen {
            (*typebuf.ptr()).tb_off = MAXMAPLEN as ::core::ffi::c_int;
            (*typebuf.ptr()).tb_len = 0 as ::core::ffi::c_int;
        } else {
            (*typebuf.ptr()).tb_off += (*typebuf.ptr()).tb_maplen;
            (*typebuf.ptr()).tb_len -= (*typebuf.ptr()).tb_maplen;
        }
        if (*typebuf.ptr()).tb_len == 0 as ::core::ffi::c_int {
            typebuf_was_filled.set(false_0 != 0);
        }
    } else {
        if flush_typeahead as ::core::ffi::c_uint
            == FLUSH_INPUT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            while inchar(
                (*typebuf.ptr()).tb_buf,
                (*typebuf.ptr()).tb_buflen - 1 as ::core::ffi::c_int,
                10 as ::core::ffi::c_long,
            ) != 0 as ::core::ffi::c_int
            {}
        }
        (*typebuf.ptr()).tb_off = MAXMAPLEN as ::core::ffi::c_int;
        (*typebuf.ptr()).tb_len = 0 as ::core::ffi::c_int;
        typebuf_was_filled.set(false_0 != 0);
    }
    (*typebuf.ptr()).tb_maplen = 0 as ::core::ffi::c_int;
    (*typebuf.ptr()).tb_silent = 0 as ::core::ffi::c_int;
    cmd_silent.set(false_0 != 0);
    (*typebuf.ptr()).tb_no_abbr_cnt = 0 as ::core::ffi::c_int;
    (*typebuf.ptr()).tb_change_cnt += 1;
    if (*typebuf.ptr()).tb_change_cnt == 0 as ::core::ffi::c_int {
        (*typebuf.ptr()).tb_change_cnt = 1 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn beep_flush() {
    if emsg_silent.get() == 0 as ::core::ffi::c_int {
        flush_buffers(FLUSH_MINIMAL);
        vim_beep(kOptBoFlagError as ::core::ffi::c_int as ::core::ffi::c_uint);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ResetRedobuff() {
    if block_redo.get() {
        return;
    }
    free_buff(old_redobuff.ptr());
    old_redobuff.set(redobuff.get());
    (*redobuff.ptr()).bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
}
#[no_mangle]
pub unsafe extern "C" fn CancelRedo() {
    if block_redo.get() {
        return;
    }
    free_buff(redobuff.ptr());
    redobuff.set(old_redobuff.get());
    (*old_redobuff.ptr()).bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
    start_stuff();
    while read_readbuffers(true_0 != 0) != NUL {}
}
#[no_mangle]
pub unsafe extern "C" fn saveRedobuff(mut save_redo: *mut save_redo_T) {
    (*save_redo).sr_redobuff = redobuff.get();
    (*redobuff.ptr()).bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
    (*save_redo).sr_old_redobuff = old_redobuff.get();
    (*old_redobuff.ptr()).bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
    let mut slen: size_t = 0;
    let s: *mut ::core::ffi::c_char =
        get_buffcont(&raw mut (*save_redo).sr_redobuff, false_0, &raw mut slen);
    if s.is_null() {
        return;
    }
    add_buff(redobuff.ptr(), s, slen as ptrdiff_t);
    xfree(s as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn restoreRedobuff(mut save_redo: *mut save_redo_T) {
    free_buff(redobuff.ptr());
    redobuff.set((*save_redo).sr_redobuff);
    free_buff(old_redobuff.ptr());
    old_redobuff.set((*save_redo).sr_old_redobuff);
}
#[no_mangle]
pub unsafe extern "C" fn AppendToRedobuff(mut s: *const ::core::ffi::c_char) {
    if !block_redo.get() {
        add_buff(redobuff.ptr(), s, -1 as ptrdiff_t);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AppendToRedobuffLit(
    mut str: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) {
    if block_redo.get() {
        return;
    }
    let mut s: *const ::core::ffi::c_char = str;
    while if len < 0 as ::core::ffi::c_int {
        (*s as ::core::ffi::c_int != NUL) as ::core::ffi::c_int
    } else {
        (s.offset_from(str) < len as isize) as ::core::ffi::c_int
    } != 0
    {
        let mut start: *const ::core::ffi::c_char = s;
        while *s as ::core::ffi::c_int >= ' ' as ::core::ffi::c_int
            && (*s as ::core::ffi::c_int) < DEL
            && (len < 0 as ::core::ffi::c_int || s.offset_from(str) < len as isize)
        {
            s = s.offset(1);
        }
        if *s as ::core::ffi::c_int == NUL
            && (*s.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '0' as ::core::ffi::c_int
                || *s.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '^' as ::core::ffi::c_int)
        {
            s = s.offset(-1);
        }
        if s > start {
            add_buff(redobuff.ptr(), start, s.offset_from(start));
        }
        if *s as ::core::ffi::c_int == NUL
            || len >= 0 as ::core::ffi::c_int && s.offset_from(str) >= len as isize
        {
            break;
        }
        let c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut s);
        if c < ' ' as ::core::ffi::c_int
            || c == DEL
            || *s as ::core::ffi::c_int == NUL
                && (c == '0' as ::core::ffi::c_int || c == '^' as ::core::ffi::c_int)
        {
            add_char_buff(redobuff.ptr(), Ctrl_V);
        }
        if *s as ::core::ffi::c_int == NUL && c == '0' as ::core::ffi::c_int {
            add_buff(
                redobuff.ptr(),
                b"048\0".as_ptr() as *const ::core::ffi::c_char,
                3 as ptrdiff_t,
            );
        } else {
            add_char_buff(redobuff.ptr(), c);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn AppendToRedobuffSpec(mut s: *const ::core::ffi::c_char) {
    if block_redo.get() {
        return;
    }
    while *s as ::core::ffi::c_int != NUL {
        if *s as uint8_t as ::core::ffi::c_int == K_SPECIAL
            && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            add_buff(redobuff.ptr(), s, 3 as ptrdiff_t);
            s = s.offset(3 as ::core::ffi::c_int as isize);
        } else {
            add_char_buff(redobuff.ptr(), mb_cptr2char_adv(&raw mut s));
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn AppendCharToRedobuff(mut c: ::core::ffi::c_int) {
    if !block_redo.get() {
        add_char_buff(redobuff.ptr(), c);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AppendNumberToRedobuff(mut n: ::core::ffi::c_int) {
    if !block_redo.get() {
        add_num_buff(redobuff.ptr(), n);
    }
}
#[no_mangle]
pub unsafe extern "C" fn stuffReadbuff(mut s: *const ::core::ffi::c_char) {
    add_buff(readbuf1.ptr(), s, -1 as ptrdiff_t);
}
#[no_mangle]
pub unsafe extern "C" fn stuffRedoReadbuff(mut s: *const ::core::ffi::c_char) {
    add_buff(readbuf2.ptr(), s, -1 as ptrdiff_t);
}
#[no_mangle]
pub unsafe extern "C" fn stuffReadbuffLen(mut s: *const ::core::ffi::c_char, mut len: ptrdiff_t) {
    add_buff(readbuf1.ptr(), s, len);
}
#[no_mangle]
pub unsafe extern "C" fn stuffReadbuffSpec(mut s: *const ::core::ffi::c_char) {
    while *s as ::core::ffi::c_int != NUL {
        if *s as uint8_t as ::core::ffi::c_int == K_SPECIAL
            && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            stuffReadbuffLen(s, 3 as ptrdiff_t);
            s = s.offset(3 as ::core::ffi::c_int as isize);
        } else {
            let mut c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut s);
            if c == CAR || c == NL || c == ESC {
                c = ' ' as ::core::ffi::c_int;
            }
            stuffcharReadbuff(c);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn stuffcharReadbuff(mut c: ::core::ffi::c_int) {
    add_char_buff(readbuf1.ptr(), c);
}
#[no_mangle]
pub unsafe extern "C" fn stuffnumReadbuff(mut n: ::core::ffi::c_int) {
    add_num_buff(readbuf1.ptr(), n);
}
#[no_mangle]
pub unsafe extern "C" fn stuffescaped(mut arg: *const ::core::ffi::c_char, mut literally: bool) {
    while *arg as ::core::ffi::c_int != NUL {
        let start: *const ::core::ffi::c_char = arg;
        while *arg as ::core::ffi::c_int >= ' ' as ::core::ffi::c_int
            && (*arg as ::core::ffi::c_int) < DEL
            || *arg as uint8_t as ::core::ffi::c_int == K_SPECIAL && !literally
        {
            arg = arg.offset(1);
        }
        if arg > start {
            stuffReadbuffLen(start, arg.offset_from(start));
        }
        if *arg as ::core::ffi::c_int != NUL {
            let c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut arg);
            if literally as ::core::ffi::c_int != 0
                && (c < ' ' as ::core::ffi::c_int && c != TAB || c == DEL)
            {
                stuffcharReadbuff(Ctrl_V);
            }
            stuffcharReadbuff(c);
        }
    }
}
unsafe extern "C" fn read_redo(mut init: bool, mut old_redo: bool) -> ::core::ffi::c_int {
    static bp: GlobalCell<*mut buffblock_T> =
        GlobalCell::new(::core::ptr::null_mut::<buffblock_T>());
    static p: GlobalCell<*mut uint8_t> = GlobalCell::new(::core::ptr::null_mut::<uint8_t>());
    let mut c: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut buf: [uint8_t; 22] = [0; 22];
    if init {
        bp.set(
            (if old_redo as ::core::ffi::c_int != 0 {
                (*old_redobuff.ptr()).bh_first.b_next
            } else {
                (*redobuff.ptr()).bh_first.b_next
            }) as *mut buffblock_T,
        );
        if (*bp.ptr()).is_null() {
            return FAIL;
        }
        p.set(&raw mut (*bp.get()).b_str as *mut ::core::ffi::c_char as *mut uint8_t);
        return OK;
    }
    c = *p.get() as ::core::ffi::c_int;
    if c == NUL {
        return c;
    }
    if c != K_SPECIAL
        || *(*p.ptr()).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_SPECIAL
    {
        n = if c < 0 as ::core::ffi::c_int || c > 255 as ::core::ffi::c_int {
            1 as ::core::ffi::c_int
        } else {
            utf8len_tab[c as usize] as ::core::ffi::c_int
        };
    } else {
        n = 1 as ::core::ffi::c_int;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        if c == K_SPECIAL {
            c = if *(*p.ptr()).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == KS_SPECIAL
            {
                K_SPECIAL
            } else if *(*p.ptr()).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == KS_ZERO
            {
                K_ZERO
            } else {
                -(*(*p.ptr()).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ((*(*p.ptr()).offset(2 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int)
                        << 8 as ::core::ffi::c_int))
            };
            p.set((*p.ptr()).offset(2 as ::core::ffi::c_int as isize));
        }
        p.set((*p.ptr()).offset(1));
        if *p.get() as ::core::ffi::c_int == NUL && !(*bp.get()).b_next.is_null() {
            bp.set((*bp.get()).b_next as *mut buffblock_T);
            p.set(&raw mut (*bp.get()).b_str as *mut ::core::ffi::c_char as *mut uint8_t);
        }
        buf[i as usize] = c as uint8_t;
        if i == n - 1 as ::core::ffi::c_int {
            if n != 1 as ::core::ffi::c_int {
                c = utf_ptr2char(&raw mut buf as *mut uint8_t as *mut ::core::ffi::c_char);
            }
            break;
        } else {
            c = *p.get() as ::core::ffi::c_int;
            if c == NUL {
                break;
            }
            i += 1;
        }
    }
    return c;
}
unsafe extern "C" fn copy_redo(mut old_redo: bool) {
    let mut c: ::core::ffi::c_int = 0;
    loop {
        c = read_redo(false_0 != 0, old_redo);
        if c == NUL {
            break;
        }
        add_char_buff(readbuf2.ptr(), c);
    }
}
#[no_mangle]
pub unsafe extern "C" fn start_redo(
    mut count: ::core::ffi::c_int,
    mut old_redo: bool,
) -> ::core::ffi::c_int {
    if read_redo(true_0 != 0, old_redo) == FAIL {
        return FAIL;
    }
    let mut c: ::core::ffi::c_int = read_redo(false_0 != 0, old_redo);
    if c == '"' as ::core::ffi::c_int {
        add_buff(
            readbuf2.ptr(),
            b"\"\0".as_ptr() as *const ::core::ffi::c_char,
            1 as ptrdiff_t,
        );
        c = read_redo(false_0 != 0, old_redo);
        if c >= '1' as ::core::ffi::c_int && c < '9' as ::core::ffi::c_int {
            c += 1;
        }
        add_char_buff(readbuf2.ptr(), c);
        if c == '=' as ::core::ffi::c_int {
            add_char_buff(readbuf2.ptr(), CAR);
            cmd_silent.set(true_0 != 0);
        }
        c = read_redo(false_0 != 0, old_redo);
    }
    if c == 'v' as ::core::ffi::c_int {
        VIsual.set((*curwin.get()).w_cursor);
        VIsual_active.set(true_0 != 0);
        VIsual_select.set(false_0 != 0);
        VIsual_reselect.set(true_0);
        redo_VIsual_busy.set(true_0 != 0);
        c = read_redo(false_0 != 0, old_redo);
    }
    if count != 0 {
        while ascii_isdigit(c) {
            c = read_redo(false_0 != 0, old_redo);
        }
        add_num_buff(readbuf2.ptr(), count);
    }
    add_char_buff(readbuf2.ptr(), c);
    copy_redo(old_redo);
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn start_redo_ins() -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    if read_redo(true_0 != 0, false_0 != 0) == FAIL {
        return FAIL;
    }
    start_stuff();
    loop {
        c = read_redo(false_0 != 0, false_0 != 0);
        if c == NUL {
            break;
        }
        if vim_strchr(b"AaIiRrOo\0".as_ptr() as *const ::core::ffi::c_char, c).is_null() {
            continue;
        }
        if c == 'O' as ::core::ffi::c_int || c == 'o' as ::core::ffi::c_int {
            add_buff(readbuf2.ptr(), NL_STR.as_ptr(), -1 as ptrdiff_t);
        }
        break;
    }
    copy_redo(false_0 != 0);
    block_redo.set(true_0 != 0);
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn stop_redo_ins() {
    block_redo.set(false_0 != 0);
}
unsafe extern "C" fn init_typebuf() {
    if !(*typebuf.ptr()).tb_buf.is_null() {
        return;
    }
    (*typebuf.ptr()).tb_buf = typebuf_init.ptr() as *mut uint8_t;
    (*typebuf.ptr()).tb_noremap = noremapbuf_init.ptr() as *mut uint8_t;
    (*typebuf.ptr()).tb_buflen =
        5 as ::core::ffi::c_int * (MAXMAPLEN as ::core::ffi::c_int + 3 as ::core::ffi::c_int);
    (*typebuf.ptr()).tb_len = 0 as ::core::ffi::c_int;
    (*typebuf.ptr()).tb_off = MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int;
    (*typebuf.ptr()).tb_change_cnt = 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn noremap_keys() -> bool {
    return KeyNoremap.get() & (RM_NONE as ::core::ffi::c_int | RM_SCRIPT as ::core::ffi::c_int)
        != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ins_typebuf(
    mut str: *mut ::core::ffi::c_char,
    mut noremap: ::core::ffi::c_int,
    mut offset: ::core::ffi::c_int,
    mut nottyped: bool,
    mut silent: bool,
) -> ::core::ffi::c_int {
    let mut val: ::core::ffi::c_int = 0;
    let mut nrm: ::core::ffi::c_int = 0;
    init_typebuf();
    (*typebuf.ptr()).tb_change_cnt += 1;
    if (*typebuf.ptr()).tb_change_cnt == 0 as ::core::ffi::c_int {
        (*typebuf.ptr()).tb_change_cnt = 1 as ::core::ffi::c_int;
    }
    state_no_longer_safe(b"ins_typebuf()\0".as_ptr() as *const ::core::ffi::c_char);
    let mut addlen: ::core::ffi::c_int = strlen(str) as ::core::ffi::c_int;
    if offset == 0 as ::core::ffi::c_int && addlen <= (*typebuf.ptr()).tb_off {
        (*typebuf.ptr()).tb_off -= addlen;
        memmove(
            (*typebuf.ptr())
                .tb_buf
                .offset((*typebuf.ptr()).tb_off as isize) as *mut ::core::ffi::c_void,
            str as *const ::core::ffi::c_void,
            addlen as size_t,
        );
    } else if (*typebuf.ptr()).tb_len == 0 as ::core::ffi::c_int
        && (*typebuf.ptr()).tb_buflen
            >= addlen
                + 3 as ::core::ffi::c_int
                    * (MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int)
    {
        (*typebuf.ptr()).tb_off = ((*typebuf.ptr()).tb_buflen
            - addlen
            - 3 as ::core::ffi::c_int
                * (MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int))
            / 2 as ::core::ffi::c_int;
        memmove(
            (*typebuf.ptr())
                .tb_buf
                .offset((*typebuf.ptr()).tb_off as isize) as *mut ::core::ffi::c_void,
            str as *const ::core::ffi::c_void,
            addlen as size_t,
        );
    } else {
        let mut newoff: ::core::ffi::c_int =
            MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int;
        let mut extra: ::core::ffi::c_int = addlen
            + newoff
            + 4 as ::core::ffi::c_int * (MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int);
        if (*typebuf.ptr()).tb_len > INT_MAX - extra {
            emsg(gettext(&raw const e_toocompl as *const ::core::ffi::c_char));
            setcursor();
            return FAIL;
        }
        let mut newlen: ::core::ffi::c_int = (*typebuf.ptr()).tb_len + extra;
        let mut s1: *mut uint8_t = xmalloc(newlen as size_t) as *mut uint8_t;
        let mut s2: *mut uint8_t = xmalloc(newlen as size_t) as *mut uint8_t;
        (*typebuf.ptr()).tb_buflen = newlen;
        memmove(
            s1.offset(newoff as isize) as *mut ::core::ffi::c_void,
            (*typebuf.ptr())
                .tb_buf
                .offset((*typebuf.ptr()).tb_off as isize) as *const ::core::ffi::c_void,
            offset as size_t,
        );
        memmove(
            s1.offset(newoff as isize).offset(offset as isize) as *mut ::core::ffi::c_void,
            str as *const ::core::ffi::c_void,
            addlen as size_t,
        );
        let mut bytes: ::core::ffi::c_int =
            (*typebuf.ptr()).tb_len - offset + 1 as ::core::ffi::c_int;
        '_c2rust_label: {
            if bytes > 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"bytes > 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    978 as ::core::ffi::c_uint,
                    b"int ins_typebuf(char *, int, int, _Bool, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        memmove(
            s1.offset(newoff as isize)
                .offset(offset as isize)
                .offset(addlen as isize) as *mut ::core::ffi::c_void,
            (*typebuf.ptr())
                .tb_buf
                .offset((*typebuf.ptr()).tb_off as isize)
                .offset(offset as isize) as *const ::core::ffi::c_void,
            bytes as size_t,
        );
        if (*typebuf.ptr()).tb_buf != typebuf_init.ptr() as *mut uint8_t {
            xfree((*typebuf.ptr()).tb_buf as *mut ::core::ffi::c_void);
        }
        (*typebuf.ptr()).tb_buf = s1;
        memmove(
            s2.offset(newoff as isize) as *mut ::core::ffi::c_void,
            (*typebuf.ptr())
                .tb_noremap
                .offset((*typebuf.ptr()).tb_off as isize) as *const ::core::ffi::c_void,
            offset as size_t,
        );
        memmove(
            s2.offset(newoff as isize)
                .offset(offset as isize)
                .offset(addlen as isize) as *mut ::core::ffi::c_void,
            (*typebuf.ptr())
                .tb_noremap
                .offset((*typebuf.ptr()).tb_off as isize)
                .offset(offset as isize) as *const ::core::ffi::c_void,
            ((*typebuf.ptr()).tb_len - offset) as size_t,
        );
        if (*typebuf.ptr()).tb_noremap != noremapbuf_init.ptr() as *mut uint8_t {
            xfree((*typebuf.ptr()).tb_noremap as *mut ::core::ffi::c_void);
        }
        (*typebuf.ptr()).tb_noremap = s2;
        (*typebuf.ptr()).tb_off = newoff;
    }
    (*typebuf.ptr()).tb_len += addlen;
    if noremap == REMAP_SCRIPT as ::core::ffi::c_int {
        val = RM_SCRIPT as ::core::ffi::c_int;
    } else if noremap == REMAP_SKIP as ::core::ffi::c_int {
        val = RM_ABBR as ::core::ffi::c_int;
    } else {
        val = RM_NONE as ::core::ffi::c_int;
    }
    if noremap == REMAP_SKIP as ::core::ffi::c_int {
        nrm = 1 as ::core::ffi::c_int;
    } else if noremap < 0 as ::core::ffi::c_int {
        nrm = addlen;
    } else {
        nrm = noremap;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < addlen {
        nrm -= 1;
        *(*typebuf.ptr())
            .tb_noremap
            .offset(((*typebuf.ptr()).tb_off + i + offset) as isize) =
            (if nrm >= 0 as ::core::ffi::c_int {
                val
            } else {
                RM_YES as ::core::ffi::c_int
            }) as uint8_t;
        i += 1;
    }
    if nottyped as ::core::ffi::c_int != 0 || (*typebuf.ptr()).tb_maplen > offset {
        (*typebuf.ptr()).tb_maplen += addlen;
    }
    if silent as ::core::ffi::c_int != 0 || (*typebuf.ptr()).tb_silent > offset {
        (*typebuf.ptr()).tb_silent += addlen;
        cmd_silent.set(true_0 != 0);
    }
    if (*typebuf.ptr()).tb_no_abbr_cnt != 0 && offset == 0 as ::core::ffi::c_int {
        (*typebuf.ptr()).tb_no_abbr_cnt += addlen;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn ins_char_typebuf(
    mut c: ::core::ffi::c_int,
    mut modifiers: ::core::ffi::c_int,
    mut on_key_ignore: bool,
) -> ::core::ffi::c_int {
    let mut buf: [::core::ffi::c_char; 67] = [0; 67];
    let mut len: ::core::ffi::c_uint = special_to_buf(
        c,
        modifiers,
        true_0 != 0,
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    '_c2rust_label: {
        if (len as usize) < ::core::mem::size_of::<[::core::ffi::c_char; 67]>() {
        } else {
            __assert_fail(
                b"len < sizeof(buf)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1056 as ::core::ffi::c_uint,
                b"int ins_char_typebuf(int, int, _Bool)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    buf[len as usize] = NUL as ::core::ffi::c_char;
    ins_typebuf(
        &raw mut buf as *mut ::core::ffi::c_char,
        KeyNoremap.get(),
        0 as ::core::ffi::c_int,
        !KeyTyped.get(),
        cmd_silent.get(),
    );
    if KeyTyped.get() as ::core::ffi::c_int != 0 && on_key_ignore as ::core::ffi::c_int != 0 {
        on_key_ignore_len.set((*on_key_ignore_len.ptr()).wrapping_add(len as size_t));
    }
    return len as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn typebuf_changed(mut tb_change_cnt: ::core::ffi::c_int) -> bool {
    return tb_change_cnt != 0 as ::core::ffi::c_int
        && ((*typebuf.ptr()).tb_change_cnt != tb_change_cnt
            || typebuf_was_filled.get() as ::core::ffi::c_int != 0);
}
#[no_mangle]
pub unsafe extern "C" fn typebuf_typed() -> ::core::ffi::c_int {
    return ((*typebuf.ptr()).tb_maplen == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn typebuf_maplen() -> ::core::ffi::c_int {
    return (*typebuf.ptr()).tb_maplen;
}
#[no_mangle]
pub unsafe extern "C" fn del_typebuf(mut len: ::core::ffi::c_int, mut offset: ::core::ffi::c_int) {
    if len == 0 as ::core::ffi::c_int {
        return;
    }
    (*typebuf.ptr()).tb_len -= len;
    if offset == 0 as ::core::ffi::c_int
        && (*typebuf.ptr()).tb_buflen - ((*typebuf.ptr()).tb_off + len)
            >= 3 as ::core::ffi::c_int * MAXMAPLEN as ::core::ffi::c_int + 3 as ::core::ffi::c_int
    {
        (*typebuf.ptr()).tb_off += len;
    } else {
        let mut i: ::core::ffi::c_int = (*typebuf.ptr()).tb_off + offset;
        if (*typebuf.ptr()).tb_off > MAXMAPLEN as ::core::ffi::c_int {
            memmove(
                (*typebuf.ptr())
                    .tb_buf
                    .offset(MAXMAPLEN as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                (*typebuf.ptr())
                    .tb_buf
                    .offset((*typebuf.ptr()).tb_off as isize)
                    as *const ::core::ffi::c_void,
                offset as size_t,
            );
            memmove(
                (*typebuf.ptr())
                    .tb_noremap
                    .offset(MAXMAPLEN as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                (*typebuf.ptr())
                    .tb_noremap
                    .offset((*typebuf.ptr()).tb_off as isize)
                    as *const ::core::ffi::c_void,
                offset as size_t,
            );
            (*typebuf.ptr()).tb_off = MAXMAPLEN as ::core::ffi::c_int;
        }
        let mut bytes: ::core::ffi::c_int =
            (*typebuf.ptr()).tb_len - offset + 1 as ::core::ffi::c_int;
        '_c2rust_label: {
            if bytes > 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"bytes > 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1122 as ::core::ffi::c_uint,
                    b"void del_typebuf(int, int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        memmove(
            (*typebuf.ptr())
                .tb_buf
                .offset((*typebuf.ptr()).tb_off as isize)
                .offset(offset as isize) as *mut ::core::ffi::c_void,
            (*typebuf.ptr())
                .tb_buf
                .offset(i as isize)
                .offset(len as isize) as *const ::core::ffi::c_void,
            bytes as size_t,
        );
        memmove(
            (*typebuf.ptr())
                .tb_noremap
                .offset((*typebuf.ptr()).tb_off as isize)
                .offset(offset as isize) as *mut ::core::ffi::c_void,
            (*typebuf.ptr())
                .tb_noremap
                .offset(i as isize)
                .offset(len as isize) as *const ::core::ffi::c_void,
            ((*typebuf.ptr()).tb_len - offset) as size_t,
        );
    }
    if (*typebuf.ptr()).tb_maplen > offset {
        if (*typebuf.ptr()).tb_maplen < offset + len {
            (*typebuf.ptr()).tb_maplen = offset;
        } else {
            (*typebuf.ptr()).tb_maplen -= len;
        }
    }
    if (*typebuf.ptr()).tb_silent > offset {
        if (*typebuf.ptr()).tb_silent < offset + len {
            (*typebuf.ptr()).tb_silent = offset;
        } else {
            (*typebuf.ptr()).tb_silent -= len;
        }
    }
    if (*typebuf.ptr()).tb_no_abbr_cnt > offset {
        if (*typebuf.ptr()).tb_no_abbr_cnt < offset + len {
            (*typebuf.ptr()).tb_no_abbr_cnt = offset;
        } else {
            (*typebuf.ptr()).tb_no_abbr_cnt -= len;
        }
    }
    typebuf_was_filled.set(false_0 != 0);
    (*typebuf.ptr()).tb_change_cnt += 1;
    if (*typebuf.ptr()).tb_change_cnt == 0 as ::core::ffi::c_int {
        (*typebuf.ptr()).tb_change_cnt = 1 as ::core::ffi::c_int;
    }
}
unsafe extern "C" fn gotchars_add_byte(
    mut state: *mut gotchars_state_T,
    mut byte: uint8_t,
) -> bool {
    let c2rust_fresh4 = (*state).buflen;
    (*state).buflen = (*state).buflen.wrapping_add(1);
    let c2rust_lvalue_ptr = &raw mut (*state).buf[c2rust_fresh4 as usize];
    *c2rust_lvalue_ptr = byte;
    let mut c: ::core::ffi::c_int = *c2rust_lvalue_ptr as ::core::ffi::c_int;
    let mut retval: bool = false_0 != 0;
    let in_special: bool = (*state).pending_special > 0 as ::core::ffi::c_uint;
    let in_mbyte: bool = (*state).pending_mbyte > 0 as ::core::ffi::c_uint;
    if in_special {
        (*state).pending_special = (*state).pending_special.wrapping_sub(1);
    } else if c == K_SPECIAL {
        (*state).pending_special = 2 as ::core::ffi::c_uint;
    }
    '_ret_false: {
        if (*state).pending_special <= 0 as ::core::ffi::c_uint {
            if in_mbyte {
                (*state).pending_mbyte = (*state).pending_mbyte.wrapping_sub(1);
            } else {
                if in_special {
                    if (*state).prev_c == KS_MODIFIER {
                        break '_ret_false;
                    } else {
                        c = if (*state).prev_c == KS_SPECIAL {
                            K_SPECIAL
                        } else if (*state).prev_c == KS_ZERO {
                            K_ZERO
                        } else {
                            -((*state).prev_c + (c << 8 as ::core::ffi::c_int))
                        };
                    }
                }
                (*state).pending_mbyte =
                    ((if c < 0 as ::core::ffi::c_int || c > 255 as ::core::ffi::c_int {
                        1 as ::core::ffi::c_int
                    } else {
                        utf8len_tab[c as usize] as ::core::ffi::c_int
                    }) - 1 as ::core::ffi::c_int) as ::core::ffi::c_uint;
            }
            if (*state).pending_mbyte <= 0 as ::core::ffi::c_uint {
                retval = true_0 != 0;
            }
        }
    }
    (*state).prev_c = c;
    return retval;
}
unsafe extern "C" fn gotchars(mut chars: *const uint8_t, mut len: size_t) {
    let mut s: *const uint8_t = chars;
    let mut todo: size_t = len;
    static state: GlobalCell<gotchars_state_T> = GlobalCell::new(gotchars_state_T {
        buf: [0; 67],
        prev_c: 0,
        buflen: 0,
        pending_special: 0,
        pending_mbyte: 0,
    });
    loop {
        let c2rust_fresh2 = todo;
        todo = todo.wrapping_sub(1);
        if c2rust_fresh2 <= 0 as size_t {
            break;
        }
        let c2rust_fresh3 = s;
        s = s.offset(1);
        if !gotchars_add_byte(state.ptr(), *c2rust_fresh3) {
            continue;
        }
        let mut i: size_t = 0 as size_t;
        while i < (*state.ptr()).buflen {
            updatescript((*state.ptr()).buf[i as usize] as ::core::ffi::c_int);
            i = i.wrapping_add(1);
        }
        if (*state.ptr()).buflen > on_key_ignore_len.get() {
            if (*state.ptr()).buflen.wrapping_sub(on_key_ignore_len.get()) > 0 as size_t {
                if (*on_key_buf.ptr()).capacity
                    < (*on_key_buf.ptr())
                        .size
                        .wrapping_add((*state.ptr()).buflen)
                        .wrapping_sub(on_key_ignore_len.get())
                {
                    (*on_key_buf.ptr()).capacity = (*on_key_buf.ptr())
                        .size
                        .wrapping_add((*state.ptr()).buflen)
                        .wrapping_sub(on_key_ignore_len.get());
                    (*on_key_buf.ptr()).capacity = (*on_key_buf.ptr()).capacity.wrapping_sub(1);
                    (*on_key_buf.ptr()).capacity |=
                        (*on_key_buf.ptr()).capacity >> 1 as ::core::ffi::c_int;
                    (*on_key_buf.ptr()).capacity |=
                        (*on_key_buf.ptr()).capacity >> 2 as ::core::ffi::c_int;
                    (*on_key_buf.ptr()).capacity |=
                        (*on_key_buf.ptr()).capacity >> 4 as ::core::ffi::c_int;
                    (*on_key_buf.ptr()).capacity |=
                        (*on_key_buf.ptr()).capacity >> 8 as ::core::ffi::c_int;
                    (*on_key_buf.ptr()).capacity |=
                        (*on_key_buf.ptr()).capacity >> 16 as ::core::ffi::c_int;
                    (*on_key_buf.ptr()).capacity = (*on_key_buf.ptr()).capacity.wrapping_add(1);
                    (*on_key_buf.ptr()).capacity = if (*on_key_buf.ptr()).capacity
                        > ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                            .wrapping_div(
                                (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        (*on_key_buf.ptr()).capacity
                    } else {
                        ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                            .wrapping_div(
                                (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    (*on_key_buf.ptr()).items = (if (*on_key_buf.ptr()).capacity
                        == ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                            .wrapping_div(
                                (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if (*on_key_buf.ptr()).items
                            == &raw mut (*on_key_buf.ptr()).init_array as *mut ::core::ffi::c_char
                        {
                            (*on_key_buf.ptr()).items as *mut ::core::ffi::c_void
                        } else {
                            _memcpy_free(
                                &raw mut (*on_key_buf.ptr()).init_array as *mut ::core::ffi::c_char
                                    as *mut ::core::ffi::c_void,
                                (*on_key_buf.ptr()).items as *mut ::core::ffi::c_void,
                                (*on_key_buf.ptr())
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                            )
                        }
                    } else {
                        if (*on_key_buf.ptr()).items
                            == &raw mut (*on_key_buf.ptr()).init_array as *mut ::core::ffi::c_char
                        {
                            memcpy(
                                xmalloc(
                                    (*on_key_buf.ptr())
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                                ),
                                (*on_key_buf.ptr()).items as *const ::core::ffi::c_void,
                                (*on_key_buf.ptr())
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                            )
                        } else {
                            xrealloc(
                                (*on_key_buf.ptr()).items as *mut ::core::ffi::c_void,
                                (*on_key_buf.ptr())
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                            )
                        }
                    }) as *mut ::core::ffi::c_char;
                }
                '_c2rust_label: {
                    if !(*on_key_buf.ptr()).items.is_null() {
                    } else {
                        __assert_fail(
                            b"(on_key_buf).items\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            1230 as ::core::ffi::c_uint,
                            b"void gotchars(const uint8_t *, size_t)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                memcpy(
                    (*on_key_buf.ptr())
                        .items
                        .offset((*on_key_buf.ptr()).size as isize)
                        as *mut ::core::ffi::c_void,
                    (&raw mut (*state.ptr()).buf as *mut uint8_t as *mut ::core::ffi::c_char)
                        .offset(on_key_ignore_len.get() as isize)
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*state.ptr()).buflen)
                        .wrapping_sub(on_key_ignore_len.get()),
                );
                (*on_key_buf.ptr()).size = (*on_key_buf.ptr())
                    .size
                    .wrapping_add((*state.ptr()).buflen)
                    .wrapping_sub(on_key_ignore_len.get());
            }
            on_key_ignore_len.set(0 as size_t);
        } else {
            on_key_ignore_len.set((*on_key_ignore_len.ptr()).wrapping_sub((*state.ptr()).buflen));
        }
        if reg_recording.get() != 0 as ::core::ffi::c_int {
            (*state.ptr()).buf[(*state.ptr()).buflen as usize] = NUL as uint8_t;
            add_buff(
                recordbuff.ptr(),
                &raw mut (*state.ptr()).buf as *mut uint8_t as *mut ::core::ffi::c_char,
                (*state.ptr()).buflen as ptrdiff_t,
            );
            last_recorded_len.set((*last_recorded_len.ptr()).wrapping_add((*state.ptr()).buflen));
        }
        (*state.ptr()).buflen = 0 as size_t;
    }
    may_sync_undo();
    debug_did_msg.set(false_0 != 0);
    (*maptick.ptr()) += 1;
}
#[no_mangle]
pub unsafe extern "C" fn gotchars_ignore() {
    let mut nop_buf: [uint8_t; 3] = [
        K_SPECIAL as uint8_t,
        KS_EXTRA as uint8_t,
        KE_IGNORE as ::core::ffi::c_int as uint8_t,
    ];
    on_key_ignore_len.set((*on_key_ignore_len.ptr()).wrapping_add(3 as size_t));
    gotchars(&raw mut nop_buf as *mut uint8_t, 3 as size_t);
}
#[no_mangle]
pub unsafe extern "C" fn ungetchars(mut len: ::core::ffi::c_int) {
    if reg_recording.get() == 0 as ::core::ffi::c_int {
        return;
    }
    delete_buff_tail(recordbuff.ptr(), len);
    last_recorded_len.set((*last_recorded_len.ptr()).wrapping_sub(len as size_t));
}
#[no_mangle]
pub unsafe extern "C" fn may_sync_undo() {
    if (State.get() & (MODE_INSERT as ::core::ffi::c_int | MODE_CMDLINE as ::core::ffi::c_int) == 0
        || arrow_used.get() as ::core::ffi::c_int != 0)
        && curscript.get() < 0 as ::core::ffi::c_int
    {
        u_sync(false_0 != 0);
    }
}
unsafe extern "C" fn alloc_typebuf() {
    (*typebuf.ptr()).tb_buf = xmalloc(
        (5 as ::core::ffi::c_int * (MAXMAPLEN as ::core::ffi::c_int + 3 as ::core::ffi::c_int))
            as size_t,
    ) as *mut uint8_t;
    (*typebuf.ptr()).tb_noremap = xmalloc(
        (5 as ::core::ffi::c_int * (MAXMAPLEN as ::core::ffi::c_int + 3 as ::core::ffi::c_int))
            as size_t,
    ) as *mut uint8_t;
    (*typebuf.ptr()).tb_buflen =
        5 as ::core::ffi::c_int * (MAXMAPLEN as ::core::ffi::c_int + 3 as ::core::ffi::c_int);
    (*typebuf.ptr()).tb_off = MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int;
    (*typebuf.ptr()).tb_len = 0 as ::core::ffi::c_int;
    (*typebuf.ptr()).tb_maplen = 0 as ::core::ffi::c_int;
    (*typebuf.ptr()).tb_silent = 0 as ::core::ffi::c_int;
    (*typebuf.ptr()).tb_no_abbr_cnt = 0 as ::core::ffi::c_int;
    (*typebuf.ptr()).tb_change_cnt += 1;
    if (*typebuf.ptr()).tb_change_cnt == 0 as ::core::ffi::c_int {
        (*typebuf.ptr()).tb_change_cnt = 1 as ::core::ffi::c_int;
    }
    typebuf_was_filled.set(false_0 != 0);
}
unsafe extern "C" fn free_typebuf() {
    if (*typebuf.ptr()).tb_buf == typebuf_init.ptr() as *mut uint8_t {
        internal_error(b"Free typebuf 1\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*typebuf.ptr()).tb_buf as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    }
    if (*typebuf.ptr()).tb_noremap == noremapbuf_init.ptr() as *mut uint8_t {
        internal_error(b"Free typebuf 2\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*typebuf.ptr()).tb_noremap as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
    };
}
static saved_typebuf: GlobalCell<[typebuf_T; 15]> = GlobalCell::new(
    [typebuf_T {
        tb_buf: ::core::ptr::null_mut::<uint8_t>(),
        tb_noremap: ::core::ptr::null_mut::<uint8_t>(),
        tb_buflen: 0,
        tb_off: 0,
        tb_len: 0,
        tb_maplen: 0,
        tb_silent: 0,
        tb_no_abbr_cnt: 0,
        tb_change_cnt: 0,
    }; 15],
);
unsafe extern "C" fn save_typebuf() {
    '_c2rust_label: {
        if curscript.get() >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"curscript >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1330 as ::core::ffi::c_uint,
                b"void save_typebuf(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    init_typebuf();
    (*saved_typebuf.ptr())[curscript.get() as usize] = typebuf.get();
    alloc_typebuf();
}
static old_char: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
static old_mod_mask: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static old_mouse_grid: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static old_mouse_row: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static old_mouse_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static old_KeyStuffed: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
unsafe extern "C" fn can_get_old_char() -> bool {
    return old_char.get() != -1 as ::core::ffi::c_int
        && (old_KeyStuffed.get() != 0 || stuff_empty() as ::core::ffi::c_int != 0);
}
#[no_mangle]
pub unsafe extern "C" fn save_typeahead(mut tp: *mut tasave_T) {
    (*tp).save_typebuf = typebuf.get();
    alloc_typebuf();
    (*tp).typebuf_valid = true_0 != 0;
    (*tp).old_char = old_char.get();
    (*tp).old_mod_mask = old_mod_mask.get();
    old_char.set(-1 as ::core::ffi::c_int);
    (*tp).save_readbuf1 = readbuf1.get();
    (*readbuf1.ptr()).bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
    (*tp).save_readbuf2 = readbuf2.get();
    (*readbuf2.ptr()).bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
}
#[no_mangle]
pub unsafe extern "C" fn restore_typeahead(mut tp: *mut tasave_T) {
    if (*tp).typebuf_valid {
        free_typebuf();
        typebuf.set((*tp).save_typebuf);
    }
    old_char.set((*tp).old_char);
    old_mod_mask.set((*tp).old_mod_mask);
    free_buff(readbuf1.ptr());
    readbuf1.set((*tp).save_readbuf1);
    free_buff(readbuf2.ptr());
    readbuf2.set((*tp).save_readbuf2);
}
#[no_mangle]
pub unsafe extern "C" fn openscript(mut name: *mut ::core::ffi::c_char, mut directly: bool) {
    if curscript.get() + 1 as ::core::ffi::c_int == NSCRIPT as ::core::ffi::c_int {
        emsg(gettext(&raw const e_nesting as *const ::core::ffi::c_char));
        return;
    }
    if check_secure() {
        return;
    }
    if ignore_script.get() {
        return;
    }
    (*curscript.ptr()) += 1;
    expand_env(name, NameBuff.ptr() as *mut ::core::ffi::c_char, MAXPATHL);
    let mut error: ::core::ffi::c_int = file_open(
        (scriptin.ptr() as *mut FileDescriptor).offset(curscript.get() as isize),
        NameBuff.ptr() as *mut ::core::ffi::c_char,
        kFileReadOnly as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if error != 0 {
        semsg(
            gettext(&raw const e_notopen_2 as *const ::core::ffi::c_char),
            name,
            uv_strerror(error),
        );
        (*curscript.ptr()) -= 1;
        return;
    }
    save_typebuf();
    if directly {
        let mut oa: oparg_T = oparg_T {
            op_type: 0,
            regname: 0,
            motion_type: kMTCharWise,
            motion_force: 0,
            use_reg_one: false,
            inclusive: false,
            end_adjusted: false,
            start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            end: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            cursor_start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            line_count: 0,
            empty: false,
            is_VIsual: false,
            start_vcol: 0,
            end_vcol: 0,
            prev_opcount: 0,
            prev_count0: 0,
            excl_tr_ws: false,
        };
        let mut save_State: ::core::ffi::c_int = State.get();
        let mut save_restart_edit: ::core::ffi::c_int = restart_edit.get();
        let mut save_finish_op: ::core::ffi::c_int = finish_op.get() as ::core::ffi::c_int;
        let mut save_msg_scroll: ::core::ffi::c_int = msg_scroll.get();
        State.set(MODE_NORMAL as ::core::ffi::c_int);
        msg_scroll.set(false_0);
        restart_edit.set(0 as ::core::ffi::c_int);
        clear_oparg(&raw mut oa);
        finish_op.set(false_0 != 0);
        let mut oldcurscript: ::core::ffi::c_int = curscript.get();
        loop {
            update_topline_cursor();
            normal_cmd(&raw mut oa, false_0 != 0);
            vpeekc();
            if curscript.get() < oldcurscript {
                break;
            }
        }
        State.set(save_State);
        msg_scroll.set(save_msg_scroll);
        restart_edit.set(save_restart_edit);
        finish_op.set(save_finish_op != 0);
    }
}
unsafe extern "C" fn closescript() {
    '_c2rust_label: {
        if curscript.get() >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"curscript >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1450 as ::core::ffi::c_uint,
                b"void closescript(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    free_typebuf();
    typebuf.set((*saved_typebuf.ptr())[curscript.get() as usize]);
    file_close(
        (scriptin.ptr() as *mut FileDescriptor).offset(curscript.get() as isize),
        false_0 != 0,
    );
    (*curscript.ptr()) -= 1;
}
#[no_mangle]
pub unsafe extern "C" fn open_scriptin(mut scriptin_name: *mut ::core::ffi::c_char) -> bool {
    '_c2rust_label: {
        if curscript.get() == -1 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"curscript == -1\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1471 as ::core::ffi::c_uint,
                b"_Bool open_scriptin(char *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    (*curscript.ptr()) += 1;
    let mut error: ::core::ffi::c_int = 0;
    if strequal(scriptin_name, b"-\0".as_ptr() as *const ::core::ffi::c_char) {
        error = file_open_stdin(
            (scriptin.ptr() as *mut FileDescriptor).offset(0 as ::core::ffi::c_int as isize),
        );
    } else {
        error = file_open(
            (scriptin.ptr() as *mut FileDescriptor).offset(0 as ::core::ffi::c_int as isize),
            scriptin_name,
            kFileReadOnly as ::core::ffi::c_int | kFileNonBlocking as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
    if error != 0 {
        fprintf(
            stderr,
            gettext(
                b"Cannot open for reading: \"%s\": %s\n\0".as_ptr() as *const ::core::ffi::c_char
            ),
            scriptin_name,
            uv_strerror(error),
        );
        (*curscript.ptr()) -= 1;
        return false_0 != 0;
    }
    save_typebuf();
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn using_script() -> ::core::ffi::c_int {
    return (curscript.get() >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn before_blocking() {
    updatescript(0 as ::core::ffi::c_int);
    if may_garbage_collect.get() {
        garbage_collect(false_0 != 0);
    }
}
unsafe extern "C" fn updatescript(mut c: ::core::ffi::c_int) {
    static count: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    if c != 0 && !(*scriptout.ptr()).is_null() {
        putc(c, scriptout.get());
    }
    let mut idle: bool = c == 0 as ::core::ffi::c_int;
    if idle as ::core::ffi::c_int != 0
        || p_uc.get() > 0 as OptInt && {
            (*count.ptr()) += 1;
            count.get() as OptInt >= p_uc.get()
        }
    {
        ml_sync_all(
            idle as ::core::ffi::c_int,
            true_0,
            p_fs.get() != 0 || idle as ::core::ffi::c_int != 0,
        );
        count.set(0 as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn merge_modifiers(
    mut c_arg: ::core::ffi::c_int,
    mut modifiers: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = c_arg;
    if *modifiers & MOD_MASK_CTRL != 0 {
        if c >= '@' as ::core::ffi::c_int && c <= 0x7f as ::core::ffi::c_int {
            c &= 0x1f as ::core::ffi::c_int;
            if c == NUL {
                c = K_ZERO;
            }
        } else if c == '6' as ::core::ffi::c_int {
            c = 0x1e as ::core::ffi::c_int;
        }
        if c != c_arg {
            *modifiers &= !MOD_MASK_CTRL;
        }
    }
    return c;
}
unsafe extern "C" fn add_byte_to_showcmd(mut byte: uint8_t) {
    static state: GlobalCell<gotchars_state_T> = GlobalCell::new(gotchars_state_T {
        buf: [0; 67],
        prev_c: 0,
        buflen: 0,
        pending_special: 0,
        pending_mbyte: 0,
    });
    if p_sc.get() == 0 || msg_silent.get() != 0 as ::core::ffi::c_int {
        return;
    }
    if !gotchars_add_byte(state.ptr(), byte) {
        return;
    }
    (*state.ptr()).buf[(*state.ptr()).buflen as usize] = NUL as uint8_t;
    (*state.ptr()).buflen = 0 as size_t;
    let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut c: ::core::ffi::c_int = NUL;
    let mut ptr: *const uint8_t = &raw mut (*state.ptr()).buf as *mut uint8_t;
    if *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == K_SPECIAL
        && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_MODIFIER
        && *ptr.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
    {
        modifiers = *ptr.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
        ptr = ptr.offset(3 as ::core::ffi::c_int as isize);
    }
    if *ptr as ::core::ffi::c_int != NUL {
        let mut mb_ptr: *const ::core::ffi::c_char =
            mb_unescape(&raw mut ptr as *mut *const ::core::ffi::c_char);
        c = if !mb_ptr.is_null() {
            utf_ptr2char(mb_ptr)
        } else {
            let c2rust_fresh7 = ptr;
            ptr = ptr.offset(1);
            *c2rust_fresh7 as ::core::ffi::c_int
        };
        if c <= 0x7f as ::core::ffi::c_int {
            let mut modifiers_after: ::core::ffi::c_int = modifiers;
            let mut mod_c: ::core::ffi::c_int = merge_modifiers(c, &raw mut modifiers_after);
            if modifiers_after == 0 as ::core::ffi::c_int {
                modifiers = 0 as ::core::ffi::c_int;
                c = mod_c;
            }
        }
    }
    if modifiers != 0 as ::core::ffi::c_int {
        add_to_showcmd(K_SPECIAL);
        add_to_showcmd(KS_MODIFIER);
        add_to_showcmd(modifiers);
    }
    if c != NUL {
        add_to_showcmd(c);
    }
    while *ptr as ::core::ffi::c_int != NUL {
        let c2rust_fresh8 = ptr;
        ptr = ptr.offset(1);
        add_to_showcmd(*c2rust_fresh8 as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn vgetc() -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    let mut buf: [uint8_t; 22] = [0; 22];
    if may_garbage_collect.get() as ::core::ffi::c_int != 0
        && want_garbage_collect.get() as ::core::ffi::c_int != 0
    {
        garbage_collect(false_0 != 0);
    }
    if can_get_old_char() {
        c = old_char.get();
        old_char.set(-1 as ::core::ffi::c_int);
        mod_mask.set(old_mod_mask.get());
        mouse_grid.set(old_mouse_grid.get());
        mouse_row.set(old_mouse_row.get());
        mouse_col.set(old_mouse_col.get());
    } else {
        static last_vgetc_recorded_len: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
        mod_mask.set(0 as ::core::ffi::c_int);
        vgetc_mod_mask.set(0 as ::core::ffi::c_int);
        vgetc_char.set(0 as ::core::ffi::c_int);
        last_recorded_len
            .set((*last_recorded_len.ptr()).wrapping_sub(last_vgetc_recorded_len.get()));
        loop {
            let mut did_inc: bool = false_0 != 0;
            if mod_mask.get() != 0 {
                (*no_mapping.ptr()) += 1;
                (*allow_keys.ptr()) += 1;
                did_inc = true_0 != 0;
            }
            c = vgetorpeek(true_0 != 0);
            if did_inc {
                (*no_mapping.ptr()) -= 1;
                (*allow_keys.ptr()) -= 1;
            }
            if c == K_SPECIAL {
                let mut save_allow_keys: ::core::ffi::c_int = allow_keys.get();
                (*no_mapping.ptr()) += 1;
                allow_keys.set(0 as ::core::ffi::c_int);
                let mut c2: ::core::ffi::c_int = vgetorpeek(true_0 != 0);
                c = vgetorpeek(true_0 != 0);
                (*no_mapping.ptr()) -= 1;
                allow_keys.set(save_allow_keys);
                if c2 == KS_MODIFIER {
                    mod_mask.set(c);
                    continue;
                } else {
                    c = if c2 == KS_SPECIAL {
                        K_SPECIAL
                    } else if c2 == KS_ZERO {
                        K_ZERO
                    } else {
                        -(c2 + (c << 8 as ::core::ffi::c_int))
                    };
                }
            }
            let mut n: ::core::ffi::c_int = 0;
            n = if c < 0 as ::core::ffi::c_int || c > 255 as ::core::ffi::c_int {
                1 as ::core::ffi::c_int
            } else {
                utf8len_tab[c as usize] as ::core::ffi::c_int
            };
            if n > 1 as ::core::ffi::c_int {
                (*no_mapping.ptr()) += 1;
                buf[0 as ::core::ffi::c_int as usize] = c as uint8_t;
                let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                while i < n {
                    buf[i as usize] = vgetorpeek(true_0 != 0) as uint8_t;
                    if buf[i as usize] as ::core::ffi::c_int == K_SPECIAL {
                        vgetorpeek(true_0 != 0);
                        vgetorpeek(true_0 != 0);
                    }
                    i += 1;
                }
                (*no_mapping.ptr()) -= 1;
                c = utf_ptr2char(&raw mut buf as *mut uint8_t as *mut ::core::ffi::c_char);
            }
            if no_mapping.get() == 0
                && KeyTyped.get() as ::core::ffi::c_int != 0
                && mod_mask.get() == MOD_MASK_ALT
                && State.get() & MODE_TERMINAL as ::core::ffi::c_int == 0
                && !is_mouse_key(c)
            {
                mod_mask.set(0 as ::core::ffi::c_int);
                let mut len: ::core::ffi::c_int =
                    ins_char_typebuf(c, 0 as ::core::ffi::c_int, false_0 != 0);
                ins_char_typebuf(ESC, 0 as ::core::ffi::c_int, false_0 != 0);
                let mut old_len: ::core::ffi::c_int = len + 3 as ::core::ffi::c_int;
                ungetchars(old_len);
                if (*on_key_buf.ptr()).size >= old_len as size_t {
                    (*on_key_buf.ptr()).size =
                        (*on_key_buf.ptr()).size.wrapping_sub(old_len as size_t);
                }
            } else {
                if vgetc_char.get() == 0 as ::core::ffi::c_int {
                    vgetc_mod_mask.set(mod_mask.get());
                    vgetc_char.set(c);
                }
                match c {
                    K_KPLUS => {
                        c = '+' as ::core::ffi::c_int;
                    }
                    K_KMINUS => {
                        c = '-' as ::core::ffi::c_int;
                    }
                    K_KDIVIDE => {
                        c = '/' as ::core::ffi::c_int;
                    }
                    K_KMULTIPLY => {
                        c = '*' as ::core::ffi::c_int;
                    }
                    K_KENTER => {
                        c = CAR;
                    }
                    K_KPOINT => {
                        c = '.' as ::core::ffi::c_int;
                    }
                    K_KCOMMA => {
                        c = ',' as ::core::ffi::c_int;
                    }
                    K_KEQUAL => {
                        c = '=' as ::core::ffi::c_int;
                    }
                    K_K0 => {
                        c = '0' as ::core::ffi::c_int;
                    }
                    K_K1 => {
                        c = '1' as ::core::ffi::c_int;
                    }
                    K_K2 => {
                        c = '2' as ::core::ffi::c_int;
                    }
                    K_K3 => {
                        c = '3' as ::core::ffi::c_int;
                    }
                    K_K4 => {
                        c = '4' as ::core::ffi::c_int;
                    }
                    K_K5 => {
                        c = '5' as ::core::ffi::c_int;
                    }
                    K_K6 => {
                        c = '6' as ::core::ffi::c_int;
                    }
                    K_K7 => {
                        c = '7' as ::core::ffi::c_int;
                    }
                    K_K8 => {
                        c = '8' as ::core::ffi::c_int;
                    }
                    K_K9 => {
                        c = '9' as ::core::ffi::c_int;
                    }
                    K_XHOME | K_ZHOME => {
                        if mod_mask.get() == MOD_MASK_SHIFT {
                            c = K_S_HOME;
                            mod_mask.set(0 as ::core::ffi::c_int);
                        } else if mod_mask.get() == MOD_MASK_CTRL {
                            c = -(253 as ::core::ffi::c_int
                                + ((KE_C_HOME as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
                            mod_mask.set(0 as ::core::ffi::c_int);
                        } else {
                            c = K_HOME;
                        }
                    }
                    K_XEND | K_ZEND => {
                        if mod_mask.get() == MOD_MASK_SHIFT {
                            c = K_S_END;
                            mod_mask.set(0 as ::core::ffi::c_int);
                        } else if mod_mask.get() == MOD_MASK_CTRL {
                            c = -(253 as ::core::ffi::c_int
                                + ((KE_C_END as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
                            mod_mask.set(0 as ::core::ffi::c_int);
                        } else {
                            c = K_END;
                        }
                    }
                    K_KUP | K_XUP => {
                        c = K_UP;
                    }
                    K_KDOWN | K_XDOWN => {
                        c = K_DOWN;
                    }
                    K_KLEFT | K_XLEFT => {
                        c = K_LEFT;
                    }
                    K_KRIGHT | K_XRIGHT => {
                        c = K_RIGHT;
                    }
                    _ => {}
                }
                break;
            }
        }
        last_vgetc_recorded_len.set(last_recorded_len.get());
    }
    may_garbage_collect.set(false_0 != 0);
    if (*on_key_buf.ptr()).size == (*on_key_buf.ptr()).capacity {
        (*on_key_buf.ptr()).capacity = if (*on_key_buf.ptr()).capacity << 1 as ::core::ffi::c_int
            > ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            (*on_key_buf.ptr()).capacity << 1 as ::core::ffi::c_int
        } else {
            ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as size_t,
                )
        };
        (*on_key_buf.ptr()).items = (if (*on_key_buf.ptr()).capacity
            == ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            if (*on_key_buf.ptr()).items
                == &raw mut (*on_key_buf.ptr()).init_array as *mut ::core::ffi::c_char
            {
                (*on_key_buf.ptr()).items as *mut ::core::ffi::c_void
            } else {
                _memcpy_free(
                    &raw mut (*on_key_buf.ptr()).init_array as *mut ::core::ffi::c_char
                        as *mut ::core::ffi::c_void,
                    (*on_key_buf.ptr()).items as *mut ::core::ffi::c_void,
                    (*on_key_buf.ptr())
                        .size
                        .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                )
            }
        } else {
            if (*on_key_buf.ptr()).items
                == &raw mut (*on_key_buf.ptr()).init_array as *mut ::core::ffi::c_char
            {
                memcpy(
                    xmalloc(
                        (*on_key_buf.ptr())
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                    ),
                    (*on_key_buf.ptr()).items as *const ::core::ffi::c_void,
                    (*on_key_buf.ptr())
                        .size
                        .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                )
            } else {
                xrealloc(
                    (*on_key_buf.ptr()).items as *mut ::core::ffi::c_void,
                    (*on_key_buf.ptr())
                        .capacity
                        .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                )
            }
        }) as *mut ::core::ffi::c_char;
    } else {
    };
    let c2rust_fresh10 = (*on_key_buf.ptr()).size;
    (*on_key_buf.ptr()).size = (*on_key_buf.ptr()).size.wrapping_add(1);
    *(*on_key_buf.ptr()).items.offset(c2rust_fresh10 as isize) = '\0' as ::core::ffi::c_char;
    if nlua_execute_on_key(c, (*on_key_buf.ptr()).items) {
        if c == -(253 as ::core::ffi::c_int
            + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            xfree(
                getcmdkeycmd(NUL, NULL_0, 0 as ::core::ffi::c_int, false_0 != 0)
                    as *mut ::core::ffi::c_void,
            );
        } else if c
            == -(253 as ::core::ffi::c_int
                + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            map_execute_lua(false_0 != 0, true_0 != 0);
        } else if c == K_PASTE_START {
            paste_repeat(0 as ::core::ffi::c_int);
        }
        c = -(253 as ::core::ffi::c_int
            + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
    }
    if (*on_key_buf.ptr()).items
        != &raw mut (*on_key_buf.ptr()).init_array as *mut ::core::ffi::c_char
    {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*on_key_buf.ptr()).items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    }
    (*on_key_buf.ptr()).capacity = ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
        .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
        .wrapping_div(
            (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    (*on_key_buf.ptr()).size = 0 as size_t;
    (*on_key_buf.ptr()).items = &raw mut (*on_key_buf.ptr()).init_array as *mut ::core::ffi::c_char;
    if c != -(253 as ::core::ffi::c_int
        + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        state_no_longer_safe(b"key typed\0".as_ptr() as *const ::core::ffi::c_char);
    }
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn safe_vgetc() -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = vgetc();
    if c == NUL {
        c = get_keystroke(::core::ptr::null_mut::<MultiQueue>());
    }
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn plain_vgetc() -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    loop {
        c = safe_vgetc();
        if !(c
            == -(253 as ::core::ffi::c_int
                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == K_VER_SCROLLBAR
            || c == K_HOR_SCROLLBAR
            || c == -(253 as ::core::ffi::c_int
                + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)))
        {
            break;
        }
    }
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn vpeekc() -> ::core::ffi::c_int {
    if can_get_old_char() {
        return old_char.get();
    }
    return vgetorpeek(false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn vpeekc_any() -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = vpeekc();
    if c == NUL && (*typebuf.ptr()).tb_len > 0 as ::core::ffi::c_int {
        c = ESC;
    }
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn char_avail() -> bool {
    if test_disable_char_avail.get() {
        return false_0 != 0;
    }
    (*no_mapping.ptr()) += 1;
    let mut retval: ::core::ffi::c_int = vpeekc();
    (*no_mapping.ptr()) -= 1;
    return retval != NUL;
}
static no_reduce_keys: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
unsafe extern "C" fn getchar_common(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut allow_number: bool,
) {
    let mut n: varnumber_T = 0 as varnumber_T;
    let called_emsg_start: ::core::ffi::c_int = called_emsg.get();
    let mut error: bool = false_0 != 0;
    let mut simplify: bool = true_0 != 0;
    let mut cursor_flag: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_check_for_opt_dict_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut d: *mut dict_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        if allow_number {
            allow_number = tv_dict_get_bool(
                d,
                b"number\0".as_ptr() as *const ::core::ffi::c_char,
                true_0,
            ) != 0;
        } else if tv_dict_has_key(d, b"number\0".as_ptr() as *const ::core::ffi::c_char) {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                b"number\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        simplify = tv_dict_get_bool(
            d,
            b"simplify\0".as_ptr() as *const ::core::ffi::c_char,
            true_0,
        ) != 0;
        let mut cursor_str: *const ::core::ffi::c_char = tv_dict_get_string(
            d,
            b"cursor\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        if !cursor_str.is_null() {
            if strcmp(cursor_str, b"hide\0".as_ptr() as *const ::core::ffi::c_char)
                != 0 as ::core::ffi::c_int
                && strcmp(cursor_str, b"keep\0".as_ptr() as *const ::core::ffi::c_char)
                    != 0 as ::core::ffi::c_int
                && strcmp(cursor_str, b"msg\0".as_ptr() as *const ::core::ffi::c_char)
                    != 0 as ::core::ffi::c_int
            {
                semsg(
                    gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
                    b"cursor\0".as_ptr() as *const ::core::ffi::c_char,
                    cursor_str,
                );
            } else {
                cursor_flag = *cursor_str.offset(0 as ::core::ffi::c_int as isize);
            }
        }
    }
    if called_emsg.get() != called_emsg_start {
        return;
    }
    if cursor_flag as ::core::ffi::c_int == 'h' as ::core::ffi::c_int {
        ui_busy_start();
    }
    (*no_mapping.ptr()) += 1;
    (*allow_keys.ptr()) += 1;
    if !simplify {
        (*no_reduce_keys.ptr()) += 1;
    }
    loop {
        if cursor_flag as ::core::ffi::c_int == 'm' as ::core::ffi::c_int
            || cursor_flag as ::core::ffi::c_int == NUL && msg_col.get() > 0 as ::core::ffi::c_int
        {
            ui_cursor_goto(msg_row.get(), msg_col.get());
        }
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_number
                    == -1 as varnumber_T
        {
            if !char_avail() {
                ui_flush();
                input_get(
                    ::core::ptr::null_mut::<uint8_t>(),
                    0 as ::core::ffi::c_int,
                    -1 as ::core::ffi::c_int,
                    (*typebuf.ptr()).tb_change_cnt,
                    (*main_loop.ptr()).events,
                );
                if input_available() == 0 && !multiqueue_empty((*main_loop.ptr()).events) {
                    state_handle_k_event();
                    continue;
                }
            }
            n = safe_vgetc() as varnumber_T;
        } else if tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) == 1 as varnumber_T
        {
            n = vpeekc_any() as varnumber_T;
        } else if error as ::core::ffi::c_int != 0 || vpeekc_any() == NUL {
            n = 0 as varnumber_T;
        } else {
            n = safe_vgetc() as varnumber_T;
        }
        if !(n
            == -(253 as ::core::ffi::c_int
                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                as varnumber_T
            || n == -(253 as ::core::ffi::c_int
                + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                as varnumber_T
            || n == K_VER_SCROLLBAR as varnumber_T
            || n == K_HOR_SCROLLBAR as varnumber_T)
        {
            break;
        }
    }
    (*no_mapping.ptr()) -= 1;
    (*allow_keys.ptr()) -= 1;
    if !simplify {
        (*no_reduce_keys.ptr()) -= 1;
    }
    if cursor_flag as ::core::ffi::c_int == 'h' as ::core::ffi::c_int {
        ui_busy_stop();
    }
    set_vim_var_nr(VV_MOUSE_WIN, 0 as varnumber_T);
    set_vim_var_nr(VV_MOUSE_WINID, 0 as varnumber_T);
    set_vim_var_nr(VV_MOUSE_LNUM, 0 as varnumber_T);
    set_vim_var_nr(VV_MOUSE_COL, 0 as varnumber_T);
    if n != 0 as varnumber_T
        && (!allow_number || n < 0 as varnumber_T || mod_mask.get() != 0 as ::core::ffi::c_int)
    {
        let mut temp: [::core::ffi::c_char; 10] = [0; 10];
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if mod_mask.get() != 0 as ::core::ffi::c_int {
            let c2rust_fresh11 = i;
            i = i + 1;
            temp[c2rust_fresh11 as usize] = K_SPECIAL as ::core::ffi::c_char;
            let c2rust_fresh12 = i;
            i = i + 1;
            temp[c2rust_fresh12 as usize] = KS_MODIFIER as ::core::ffi::c_char;
            let c2rust_fresh13 = i;
            i = i + 1;
            temp[c2rust_fresh13 as usize] = mod_mask.get() as ::core::ffi::c_char;
        }
        if n < 0 as varnumber_T {
            let c2rust_fresh14 = i;
            i = i + 1;
            temp[c2rust_fresh14 as usize] = K_SPECIAL as ::core::ffi::c_char;
            let c2rust_fresh15 = i;
            i = i + 1;
            temp[c2rust_fresh15 as usize] = (if n == K_SPECIAL as varnumber_T {
                KS_SPECIAL as varnumber_T
            } else if n == NUL as varnumber_T {
                KS_ZERO as varnumber_T
            } else {
                -n & 0xff as varnumber_T
            }) as ::core::ffi::c_char;
            let c2rust_fresh16 = i;
            i = i + 1;
            temp[c2rust_fresh16 as usize] = (if n == K_SPECIAL as varnumber_T
                || n == NUL as varnumber_T
            {
                KE_FILLER as ::core::ffi::c_uint
            } else {
                -n as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint
            }) as ::core::ffi::c_char;
        } else {
            i += utf_char2bytes(
                n as ::core::ffi::c_int,
                (&raw mut temp as *mut ::core::ffi::c_char).offset(i as isize),
            );
        }
        '_c2rust_label: {
            if i < 10 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"i < 10\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2021 as ::core::ffi::c_uint,
                    b"void getchar_common(typval_T *, typval_T *, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        temp[i as usize] = NUL as ::core::ffi::c_char;
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = xmemdupz(
            &raw mut temp as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            i as size_t,
        ) as *mut ::core::ffi::c_char;
        if is_mouse_key(n as ::core::ffi::c_int) {
            let mut row: ::core::ffi::c_int = mouse_row.get();
            let mut col: ::core::ffi::c_int = mouse_col.get();
            let mut grid: ::core::ffi::c_int = mouse_grid.get();
            let mut lnum: linenr_T = 0;
            let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
            if row >= 0 as ::core::ffi::c_int && col >= 0 as ::core::ffi::c_int {
                let mut winnr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                let win: *mut win_T =
                    mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col);
                if win.is_null() {
                    return;
                }
                mouse_comp_pos(win, &raw mut row, &raw mut col, &raw mut lnum);
                wp = firstwin.get();
                while wp != win {
                    winnr += 1;
                    wp = (*wp).w_next;
                }
                set_vim_var_nr(VV_MOUSE_WIN, winnr as varnumber_T);
                set_vim_var_nr(VV_MOUSE_WINID, (*wp).handle as varnumber_T);
                set_vim_var_nr(VV_MOUSE_LNUM, lnum as varnumber_T);
                set_vim_var_nr(VV_MOUSE_COL, (col + 1 as ::core::ffi::c_int) as varnumber_T);
            }
        }
    } else if !allow_number {
        (*rettv).v_type = VAR_STRING;
    } else {
        (*rettv).vval.v_number = n;
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_getchar(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    getchar_common(argvars, rettv, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_getcharstr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    getchar_common(argvars, rettv, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_getcharmod(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = mod_mask.get() as varnumber_T;
}
unsafe extern "C" fn put_string_in_typebuf(
    mut offset: ::core::ffi::c_int,
    mut slen: ::core::ffi::c_int,
    mut string: *mut uint8_t,
    mut new_slen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut extra: ::core::ffi::c_int = new_slen - slen;
    *string.offset(new_slen as isize) = NUL as uint8_t;
    if extra < 0 as ::core::ffi::c_int {
        del_typebuf(-extra, offset);
    } else if extra > 0 as ::core::ffi::c_int {
        if ins_typebuf(
            (string as *mut ::core::ffi::c_char).offset(slen as isize),
            REMAP_YES as ::core::ffi::c_int,
            offset,
            false_0 != 0,
            false_0 != 0,
        ) == FAIL
        {
            return FAIL;
        }
    }
    memmove(
        (*typebuf.ptr())
            .tb_buf
            .offset((*typebuf.ptr()).tb_off as isize)
            .offset(offset as isize) as *mut ::core::ffi::c_void,
        string as *const ::core::ffi::c_void,
        new_slen as size_t,
    );
    return OK;
}
unsafe extern "C" fn at_ins_compl_key() -> bool {
    let mut p: *mut uint8_t = (*typebuf.ptr())
        .tb_buf
        .offset((*typebuf.ptr()).tb_off as isize);
    let mut c: ::core::ffi::c_int = *p as ::core::ffi::c_int;
    if (*typebuf.ptr()).tb_len > 3 as ::core::ffi::c_int
        && c == K_SPECIAL
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_MODIFIER
        && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int & MOD_MASK_CTRL != 0
    {
        c = *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int;
    }
    return ctrl_x_mode_not_default() as ::core::ffi::c_int != 0
        && vim_is_ctrl_x_key(c) as ::core::ffi::c_int != 0
        || compl_status_local() as ::core::ffi::c_int != 0 && (c == Ctrl_N || c == Ctrl_P);
}
unsafe extern "C" fn check_simplify_modifier(
    mut max_offset: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if State.get() & MODE_TERMINAL as ::core::ffi::c_int != 0
        || no_reduce_keys.get() > 0 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    let mut offset: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while offset < max_offset {
        if offset + 3 as ::core::ffi::c_int >= (*typebuf.ptr()).tb_len {
            break;
        }
        let mut tp: *mut uint8_t = (*typebuf.ptr())
            .tb_buf
            .offset((*typebuf.ptr()).tb_off as isize)
            .offset(offset as isize);
        if *tp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == K_SPECIAL
            && *tp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_MODIFIER
        {
            let mut modifier: ::core::ffi::c_int =
                *tp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
            let mut c: ::core::ffi::c_int =
                *tp.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
            let mut new_c: ::core::ffi::c_int = merge_modifiers(c, &raw mut modifier);
            if new_c != c {
                if offset == 0 as ::core::ffi::c_int {
                    vgetc_char.set(c);
                    vgetc_mod_mask
                        .set(*tp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int);
                }
                let mut new_string: [uint8_t; 21] = [0; 21];
                let mut len: ::core::ffi::c_int = 0;
                if new_c < 0 as ::core::ffi::c_int {
                    new_string[0 as ::core::ffi::c_int as usize] = K_SPECIAL as uint8_t;
                    new_string[1 as ::core::ffi::c_int as usize] = (if new_c == K_SPECIAL {
                        KS_SPECIAL
                    } else if new_c == NUL {
                        KS_ZERO
                    } else {
                        -new_c & 0xff as ::core::ffi::c_int
                    })
                        as uint8_t;
                    new_string[2 as ::core::ffi::c_int as usize] =
                        (if new_c == K_SPECIAL || new_c == NUL {
                            KE_FILLER as ::core::ffi::c_uint
                        } else {
                            -new_c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int
                                & 0xff as ::core::ffi::c_uint
                        }) as uint8_t;
                    len = 3 as ::core::ffi::c_int;
                } else {
                    len = utf_char2bytes(
                        new_c,
                        &raw mut new_string as *mut uint8_t as *mut ::core::ffi::c_char,
                    );
                }
                if modifier == 0 as ::core::ffi::c_int {
                    if put_string_in_typebuf(
                        offset,
                        4 as ::core::ffi::c_int,
                        &raw mut new_string as *mut uint8_t,
                        len,
                    ) == FAIL
                    {
                        return -1 as ::core::ffi::c_int;
                    }
                } else {
                    *tp.offset(2 as ::core::ffi::c_int as isize) = modifier as uint8_t;
                    if put_string_in_typebuf(
                        offset + 3 as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                        &raw mut new_string as *mut uint8_t,
                        len,
                    ) == FAIL
                    {
                        return -1 as ::core::ffi::c_int;
                    }
                }
                return len;
            }
        }
        offset += 1;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn handle_mapping(
    mut keylenp: *mut ::core::ffi::c_int,
    mut timedout: *const bool,
    mut mapdepth: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
    let mut mp2: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
    let mut mp_match: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
    let mut mp_match_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut max_mlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut keylen: ::core::ffi::c_int = *keylenp;
    let mut local_State: ::core::ffi::c_int = get_real_state();
    let mut is_plug_map: bool = false_0 != 0;
    if (*typebuf.ptr()).tb_len >= 3 as ::core::ffi::c_int
        && *(*typebuf.ptr())
            .tb_buf
            .offset((*typebuf.ptr()).tb_off as isize) as ::core::ffi::c_int
            == K_SPECIAL
        && *(*typebuf.ptr())
            .tb_buf
            .offset(((*typebuf.ptr()).tb_off + 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            == KS_EXTRA
        && *(*typebuf.ptr())
            .tb_buf
            .offset(((*typebuf.ptr()).tb_off + 2 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            == KE_PLUG as ::core::ffi::c_int
    {
        is_plug_map = true_0 != 0;
    }
    let mut tb_c1: ::core::ffi::c_int = *(*typebuf.ptr())
        .tb_buf
        .offset((*typebuf.ptr()).tb_off as isize)
        as ::core::ffi::c_int;
    if no_mapping.get() == 0 as ::core::ffi::c_int
        && (no_zero_mapping.get() == 0 as ::core::ffi::c_int || tb_c1 != '0' as ::core::ffi::c_int)
        && ((*typebuf.ptr()).tb_maplen == 0 as ::core::ffi::c_int
            || is_plug_map as ::core::ffi::c_int != 0
            || *(*typebuf.ptr())
                .tb_noremap
                .offset((*typebuf.ptr()).tb_off as isize) as ::core::ffi::c_int
                & (RM_NONE as ::core::ffi::c_int | RM_ABBR as ::core::ffi::c_int)
                == 0)
        && !(p_paste.get() != 0
            && State.get()
                & (MODE_INSERT as ::core::ffi::c_int | MODE_CMDLINE as ::core::ffi::c_int)
                != 0)
        && !(State.get() == MODE_HITRETURN as ::core::ffi::c_int
            && (tb_c1 == CAR || tb_c1 == ' ' as ::core::ffi::c_int))
        && State.get() != MODE_ASKMORE as ::core::ffi::c_int
        && !at_ins_compl_key()
    {
        let mut mlen: ::core::ffi::c_int = 0;
        let mut nolmaplen: ::core::ffi::c_int = 0;
        if tb_c1 == K_SPECIAL {
            nolmaplen = 2 as ::core::ffi::c_int;
        } else {
            if *p_langmap.get() as ::core::ffi::c_int != 0
                && (State.get()
                    & (MODE_CMDLINE as ::core::ffi::c_int | MODE_INSERT as ::core::ffi::c_int)
                    == 0 as ::core::ffi::c_int
                    && get_real_state() != MODE_SELECT as ::core::ffi::c_int)
                && (p_lrm.get() != 0
                    || (if vgetc_busy.get() != 0 {
                        (typebuf_maplen() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                    } else {
                        KeyTyped.get() as ::core::ffi::c_int
                    }) != 0)
                && KeyStuffed.get() == 0
                && tb_c1 >= 0 as ::core::ffi::c_int
            {
                if tb_c1 < 256 as ::core::ffi::c_int {
                    tb_c1 = (*langmap_mapchar.ptr())[tb_c1 as usize] as ::core::ffi::c_int;
                } else {
                    tb_c1 = langmap_adjust_mb(tb_c1);
                }
            }
            nolmaplen = 0 as ::core::ffi::c_int;
        }
        mp = get_buf_maphash_list(local_State, tb_c1);
        mp2 = get_maphash_list(local_State, tb_c1);
        if mp.is_null() {
            mp = mp2;
            mp2 = ::core::ptr::null_mut::<mapblock_T>();
        }
        mp_match = ::core::ptr::null_mut::<mapblock_T>();
        mp_match_len = 0 as ::core::ffi::c_int;
        while !mp.is_null() {
            if *(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                as ::core::ffi::c_int
                == tb_c1
                && (*mp).m_mode & local_State != 0
                && ((*mp).m_mode & MODE_LANGMAP as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                    || (*typebuf.ptr()).tb_maplen == 0 as ::core::ffi::c_int)
            {
                let mut nomap: ::core::ffi::c_int = nolmaplen;
                let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                mlen = 1 as ::core::ffi::c_int;
                while mlen < (*typebuf.ptr()).tb_len {
                    let mut c2: ::core::ffi::c_int = *(*typebuf.ptr())
                        .tb_buf
                        .offset(((*typebuf.ptr()).tb_off + mlen) as isize)
                        as ::core::ffi::c_int;
                    if nomap > 0 as ::core::ffi::c_int {
                        if nomap == 2 as ::core::ffi::c_int && c2 == KS_MODIFIER {
                            modifiers = 1 as ::core::ffi::c_int;
                        } else if nomap == 1 as ::core::ffi::c_int
                            && modifiers == 1 as ::core::ffi::c_int
                        {
                            modifiers = c2;
                        }
                        nomap -= 1;
                    } else {
                        if c2 == K_SPECIAL {
                            nomap = 2 as ::core::ffi::c_int;
                        } else if merge_modifiers(c2, &raw mut modifiers) == c2 {
                            if *p_langmap.get() as ::core::ffi::c_int != 0
                                && true
                                && (p_lrm.get() != 0
                                    || (if vgetc_busy.get() != 0 {
                                        (typebuf_maplen() == 0 as ::core::ffi::c_int)
                                            as ::core::ffi::c_int
                                    } else {
                                        KeyTyped.get() as ::core::ffi::c_int
                                    }) != 0)
                                && KeyStuffed.get() == 0
                                && c2 >= 0 as ::core::ffi::c_int
                            {
                                if c2 < 256 as ::core::ffi::c_int {
                                    c2 =
                                        (*langmap_mapchar.ptr())[c2 as usize] as ::core::ffi::c_int;
                                } else {
                                    c2 = langmap_adjust_mb(c2);
                                }
                            }
                        }
                        modifiers = 0 as ::core::ffi::c_int;
                    }
                    if *(*mp).m_keys.offset(mlen as isize) as uint8_t as ::core::ffi::c_int != c2 {
                        break;
                    }
                    mlen += 1;
                }
                let mut p1: *const ::core::ffi::c_char = (*mp).m_keys;
                let mut p2: *const ::core::ffi::c_char = mb_unescape(&raw mut p1);
                if !p2.is_null()
                    && utf8len_tab[tb_c1 as usize] as ::core::ffi::c_int > utfc_ptr2len(p2)
                {
                    mlen = 0 as ::core::ffi::c_int;
                }
                keylen = (*mp).m_keylen;
                if mlen == keylen
                    || mlen == (*typebuf.ptr()).tb_len && (*typebuf.ptr()).tb_len < keylen
                {
                    let mut n: ::core::ffi::c_int = 0;
                    let mut s: *mut uint8_t = (*typebuf.ptr())
                        .tb_noremap
                        .offset((*typebuf.ptr()).tb_off as isize);
                    if !(*s as ::core::ffi::c_int == RM_SCRIPT as ::core::ffi::c_int
                        && (*(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int
                            != K_SPECIAL
                            || *(*mp).m_keys.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int
                                != KS_EXTRA
                            || *(*mp).m_keys.offset(2 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                != KE_SNR as ::core::ffi::c_int))
                    {
                        n = mlen;
                        loop {
                            n -= 1;
                            if n < 0 as ::core::ffi::c_int {
                                break;
                            }
                            let c2rust_fresh9 = s;
                            s = s.offset(1);
                            if *c2rust_fresh9 as ::core::ffi::c_int
                                & (RM_NONE as ::core::ffi::c_int | RM_ABBR as ::core::ffi::c_int)
                                != 0
                            {
                                break;
                            }
                        }
                        if !(!is_plug_map && n >= 0 as ::core::ffi::c_int) {
                            if keylen > (*typebuf.ptr()).tb_len {
                                if !*timedout
                                    && !(!mp_match.is_null()
                                        && (*mp_match).m_nowait as ::core::ffi::c_int != 0)
                                {
                                    keylen = KEYLEN_PART_MAP as ::core::ffi::c_int;
                                    break;
                                }
                            } else if keylen > mp_match_len
                                || keylen == mp_match_len
                                    && !mp_match.is_null()
                                    && (*mp_match).m_mode & MODE_LANGMAP as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    && (*mp).m_mode & MODE_LANGMAP as ::core::ffi::c_int
                                        != 0 as ::core::ffi::c_int
                            {
                                mp_match = mp;
                                mp_match_len = keylen;
                            }
                        }
                    }
                } else {
                    max_mlen = if max_mlen > mlen { max_mlen } else { mlen };
                }
            }
            if (*mp).m_next.is_null() {
                mp = mp2;
                mp2 = ::core::ptr::null_mut::<mapblock_T>();
            } else {
                mp = (*mp).m_next;
            };
        }
        if keylen != KEYLEN_PART_MAP as ::core::ffi::c_int && !mp_match.is_null() {
            mp = mp_match;
            keylen = mp_match_len;
        }
    }
    if (mp.is_null() || max_mlen > mp_match_len) && keylen != KEYLEN_PART_MAP as ::core::ffi::c_int
    {
        if no_mapping.get() == 0 as ::core::ffi::c_int
            || allow_keys.get() != 0 as ::core::ffi::c_int
        {
            if tb_c1 == K_SPECIAL
                && ((*typebuf.ptr()).tb_len < 2 as ::core::ffi::c_int
                    || *(*typebuf.ptr())
                        .tb_buf
                        .offset(((*typebuf.ptr()).tb_off + 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        == KS_MODIFIER
                        && (*typebuf.ptr()).tb_len < 4 as ::core::ffi::c_int)
            {
                keylen = KEYLEN_PART_KEY as ::core::ffi::c_int;
            } else {
                keylen = check_simplify_modifier(max_mlen + 1 as ::core::ffi::c_int);
                if keylen < 0 as ::core::ffi::c_int {
                    return map_result_fail as ::core::ffi::c_int;
                }
            }
        } else {
            keylen = 0 as ::core::ffi::c_int;
        }
        if keylen == 0 as ::core::ffi::c_int {
            if mp.is_null() {
                *keylenp = keylen;
                return map_result_get as ::core::ffi::c_int;
            }
        }
        if keylen > 0 as ::core::ffi::c_int {
            *keylenp = keylen;
            return map_result_retry as ::core::ffi::c_int;
        }
        if keylen < 0 as ::core::ffi::c_int {
            '_c2rust_label: {
                if keylen == KEYLEN_PART_KEY as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"keylen == KEYLEN_PART_KEY\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2385 as ::core::ffi::c_uint,
                        b"int handle_mapping(int *, const _Bool *, int *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
        } else {
            '_c2rust_label_0: {
                if !mp.is_null() {
                } else {
                    __assert_fail(
                        b"mp != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2387 as ::core::ffi::c_uint,
                        b"int handle_mapping(int *, const _Bool *, int *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            keylen = mp_match_len;
        }
    }
    if keylen >= 0 as ::core::ffi::c_int && keylen <= (*typebuf.ptr()).tb_len {
        let mut i: ::core::ffi::c_int = 0;
        let mut map_str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if keylen > (*typebuf.ptr()).tb_maplen
            && (*mp).m_mode & MODE_LANGMAP as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        {
            gotchars(
                (*typebuf.ptr())
                    .tb_buf
                    .offset((*typebuf.ptr()).tb_off as isize)
                    .offset((*typebuf.ptr()).tb_maplen as isize),
                (keylen - (*typebuf.ptr()).tb_maplen) as size_t,
            );
        }
        cmd_silent.set((*typebuf.ptr()).tb_silent > 0 as ::core::ffi::c_int);
        del_typebuf(keylen, 0 as ::core::ffi::c_int);
        *mapdepth += 1;
        if *mapdepth as OptInt >= p_mmd.get() {
            emsg(gettext(
                (e_recursive_mapping.ptr() as *const _) as *const ::core::ffi::c_char,
            ));
            if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0 {
                redrawcmdline();
            } else {
                setcursor();
            }
            flush_buffers(FLUSH_MINIMAL);
            *mapdepth = 0 as ::core::ffi::c_int;
            *keylenp = keylen;
            return map_result_fail as ::core::ffi::c_int;
        }
        if VIsual_active.get() as ::core::ffi::c_int != 0
            && VIsual_select.get() as ::core::ffi::c_int != 0
            && (*mp).m_mode & MODE_VISUAL as ::core::ffi::c_int != 0
        {
            VIsual_select.set(false_0 != 0);
            ins_typebuf(
                K_SELECT_STRING.as_ptr() as *mut ::core::ffi::c_char,
                REMAP_NONE as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                true_0 != 0,
                false_0 != 0,
            );
        }
        let save_m_expr: bool = (*mp).m_expr != 0;
        let save_m_noremap: ::core::ffi::c_int = (*mp).m_noremap;
        let save_m_silent: bool = (*mp).m_silent != 0;
        let mut save_m_keys: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut save_alt_m_keys: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let save_alt_m_keylen: ::core::ffi::c_int = if !(*mp).m_alt.is_null() {
            (*(*mp).m_alt).m_keylen
        } else {
            0 as ::core::ffi::c_int
        };
        if (*mp).m_expr != 0 {
            let save_vgetc_busy: ::core::ffi::c_int = vgetc_busy.get();
            let save_may_garbage_collect: bool = may_garbage_collect.get();
            let prev_did_emsg: ::core::ffi::c_int = did_emsg.get();
            vgetc_busy.set(0 as ::core::ffi::c_int);
            may_garbage_collect.set(false_0 != 0);
            save_m_keys = xmemdupz(
                (*mp).m_keys as *const ::core::ffi::c_void,
                (*mp).m_keylen as size_t,
            ) as *mut ::core::ffi::c_char;
            save_alt_m_keys = (if !(*mp).m_alt.is_null() {
                xmemdupz(
                    (*(*mp).m_alt).m_keys as *const ::core::ffi::c_void,
                    save_alt_m_keylen as size_t,
                )
            } else {
                NULL_0
            }) as *mut ::core::ffi::c_char;
            map_str = eval_map_expr(mp, NUL);
            if map_str.is_null() || *map_str as ::core::ffi::c_int == NUL {
                if prev_did_emsg != did_emsg.get() {
                    let mut buf: [::core::ffi::c_char; 4] = [0; 4];
                    xfree(map_str as *mut ::core::ffi::c_void);
                    buf[0 as ::core::ffi::c_int as usize] = K_SPECIAL as ::core::ffi::c_char;
                    buf[1 as ::core::ffi::c_int as usize] = KS_EXTRA as ::core::ffi::c_char;
                    buf[2 as ::core::ffi::c_int as usize] =
                        KE_IGNORE as ::core::ffi::c_int as ::core::ffi::c_char;
                    buf[3 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                    map_str = xmemdupz(
                        &raw mut buf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                        3 as size_t,
                    ) as *mut ::core::ffi::c_char;
                    if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0 {
                        msg_didout.set(true_0 != 0);
                        msg_row.set(if msg_row.get() > cmdline_row.get() {
                            msg_row.get()
                        } else {
                            cmdline_row.get()
                        });
                        redrawcmd();
                    }
                } else if State.get()
                    & (MODE_NORMAL as ::core::ffi::c_int | MODE_INSERT as ::core::ffi::c_int)
                    != 0
                {
                    setcursor();
                }
            }
            vgetc_busy.set(save_vgetc_busy);
            may_garbage_collect.set(save_may_garbage_collect);
        } else {
            map_str = (*mp).m_str;
        }
        if map_str.is_null() {
            i = FAIL;
        } else {
            let mut noremap: ::core::ffi::c_int = 0;
            if keylen > (*typebuf.ptr()).tb_maplen
                && (*mp).m_mode & MODE_LANGMAP as ::core::ffi::c_int != 0 as ::core::ffi::c_int
            {
                gotchars(map_str as *mut uint8_t, strlen(map_str));
            }
            if save_m_noremap != REMAP_YES as ::core::ffi::c_int {
                noremap = save_m_noremap;
            } else if if save_m_expr as ::core::ffi::c_int != 0 {
                (strncmp(map_str, save_m_keys, keylen as size_t) == 0 as ::core::ffi::c_int
                    || !save_alt_m_keys.is_null()
                        && strncmp(map_str, save_alt_m_keys, save_alt_m_keylen as size_t)
                            == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            } else {
                (strncmp(map_str, (*mp).m_keys, keylen as size_t) == 0 as ::core::ffi::c_int
                    || !(*mp).m_alt.is_null()
                        && strncmp(
                            map_str,
                            (*(*mp).m_alt).m_keys,
                            (*(*mp).m_alt).m_keylen as size_t,
                        ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            } != 0
            {
                noremap = REMAP_SKIP as ::core::ffi::c_int;
            } else {
                noremap = REMAP_YES as ::core::ffi::c_int;
            }
            i = ins_typebuf(
                map_str,
                noremap,
                0 as ::core::ffi::c_int,
                true_0 != 0,
                cmd_silent.get() as ::core::ffi::c_int != 0
                    || save_m_silent as ::core::ffi::c_int != 0,
            );
            if save_m_expr {
                xfree(map_str as *mut ::core::ffi::c_void);
            }
        }
        xfree(save_m_keys as *mut ::core::ffi::c_void);
        xfree(save_alt_m_keys as *mut ::core::ffi::c_void);
        *keylenp = keylen;
        if i == FAIL {
            return map_result_fail as ::core::ffi::c_int;
        }
        return map_result_retry as ::core::ffi::c_int;
    }
    *keylenp = keylen;
    return map_result_nomatch as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn vungetc(mut c: ::core::ffi::c_int) {
    old_char.set(c);
    old_mod_mask.set(mod_mask.get());
    old_mouse_grid.set(mouse_grid.get());
    old_mouse_row.set(mouse_row.get());
    old_mouse_col.set(mouse_col.get());
    old_KeyStuffed.set(KeyStuffed.get());
}
#[no_mangle]
pub unsafe extern "C" fn check_end_reg_executing(mut advance: bool) {
    if reg_executing.get() != 0 as ::core::ffi::c_int
        && ((*typebuf.ptr()).tb_maplen == 0 as ::core::ffi::c_int
            || pending_end_reg_executing.get() as ::core::ffi::c_int != 0)
    {
        if advance {
            reg_executing.set(0 as ::core::ffi::c_int);
            pending_end_reg_executing.set(false_0 != 0);
        } else {
            pending_end_reg_executing.set(true_0 != 0);
        }
    }
}
unsafe extern "C" fn vgetorpeek(mut advance: bool) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    let mut timedout: bool = false_0 != 0;
    let mut mapdepth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut mode_deleted: bool = false_0 != 0;
    if vgetc_busy.get() > 0 as ::core::ffi::c_int && ex_normal_busy.get() == 0 as ::core::ffi::c_int
    {
        return NUL;
    }
    (*vgetc_busy.ptr()) += 1;
    if advance {
        KeyStuffed.set(false_0);
        typebuf_was_empty.set(false_0 != 0);
    }
    init_typebuf();
    start_stuff();
    check_end_reg_executing(advance);
    loop {
        if typeahead_char.get() != 0 as ::core::ffi::c_int {
            c = typeahead_char.get();
            if advance {
                typeahead_char.set(0 as ::core::ffi::c_int);
            }
        } else {
            c = read_readbuffers(advance);
        }
        if c != NUL && !got_int.get() {
            if advance {
                KeyStuffed.set(true_0);
            }
            if (*typebuf.ptr()).tb_no_abbr_cnt == 0 as ::core::ffi::c_int {
                (*typebuf.ptr()).tb_no_abbr_cnt = 1 as ::core::ffi::c_int;
            }
        } else {
            loop {
                check_end_reg_executing(advance);
                if (*typebuf.ptr()).tb_maplen != 0 {
                    line_breakcheck();
                } else {
                    if (mapped_ctrl_c.get() | (*curbuf.get()).b_mapped_ctrl_c) & get_real_state()
                        != 0
                    {
                        ctrl_c_interrupts.set(false_0 != 0);
                    }
                    os_breakcheck();
                    ctrl_c_interrupts.set(true_0 != 0);
                }
                let mut keylen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                if got_int.get() {
                    c = inchar(
                        (*typebuf.ptr()).tb_buf,
                        (*typebuf.ptr()).tb_buflen - 1 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_long,
                    );
                    if (c != 0 || (*typebuf.ptr()).tb_maplen != 0)
                        && State.get()
                            & (MODE_INSERT as ::core::ffi::c_int
                                | MODE_CMDLINE as ::core::ffi::c_int)
                            != 0
                    {
                        c = ESC;
                    } else {
                        c = Ctrl_C;
                    }
                    flush_buffers(FLUSH_INPUT);
                    if advance {
                        *(*typebuf.ptr()).tb_buf = c as uint8_t;
                        gotchars((*typebuf.ptr()).tb_buf, 1 as size_t);
                    }
                    cmd_silent.set(false_0 != 0);
                    break;
                } else {
                    if (*typebuf.ptr()).tb_len > 0 as ::core::ffi::c_int {
                        let mut result: map_result_T =
                            handle_mapping(&raw mut keylen, &raw mut timedout, &raw mut mapdepth)
                                as map_result_T;
                        if result as ::core::ffi::c_uint
                            == map_result_retry as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            continue;
                        }
                        if result as ::core::ffi::c_uint
                            == map_result_fail as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            c = -1 as ::core::ffi::c_int;
                            break;
                        } else if result as ::core::ffi::c_uint
                            == map_result_get as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            c = *(*typebuf.ptr())
                                .tb_buf
                                .offset((*typebuf.ptr()).tb_off as isize)
                                as ::core::ffi::c_int;
                            if advance {
                                cmd_silent
                                    .set((*typebuf.ptr()).tb_silent > 0 as ::core::ffi::c_int);
                                if (*typebuf.ptr()).tb_maplen > 0 as ::core::ffi::c_int {
                                    KeyTyped.set(false_0 != 0);
                                } else {
                                    KeyTyped.set(true_0 != 0);
                                    gotchars(
                                        (*typebuf.ptr())
                                            .tb_buf
                                            .offset((*typebuf.ptr()).tb_off as isize),
                                        1 as size_t,
                                    );
                                }
                                KeyNoremap.set(
                                    *(*typebuf.ptr())
                                        .tb_noremap
                                        .offset((*typebuf.ptr()).tb_off as isize)
                                        as ::core::ffi::c_uchar
                                        as ::core::ffi::c_int,
                                );
                                del_typebuf(1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
                            }
                            break;
                        }
                    }
                    c = 0 as ::core::ffi::c_int;
                    let mut new_wcol: ::core::ffi::c_int = (*curwin.get()).w_wcol;
                    let mut new_wrow: ::core::ffi::c_int = (*curwin.get()).w_wrow;
                    if advance as ::core::ffi::c_int != 0
                        && (*typebuf.ptr()).tb_len == 1 as ::core::ffi::c_int
                        && *(*typebuf.ptr())
                            .tb_buf
                            .offset((*typebuf.ptr()).tb_off as isize)
                            as ::core::ffi::c_int
                            == ESC
                        && no_mapping.get() == 0
                        && ex_normal_busy.get() == 0 as ::core::ffi::c_int
                        && (*typebuf.ptr()).tb_maplen == 0 as ::core::ffi::c_int
                        && State.get() & MODE_INSERT as ::core::ffi::c_int != 0
                        && (p_timeout.get() != 0
                            || keylen == KEYLEN_PART_KEY as ::core::ffi::c_int
                                && p_ttimeout.get() != 0)
                        && {
                            c = inchar(
                                (*typebuf.ptr())
                                    .tb_buf
                                    .offset((*typebuf.ptr()).tb_off as isize)
                                    .offset((*typebuf.ptr()).tb_len as isize),
                                3 as ::core::ffi::c_int,
                                25 as ::core::ffi::c_long,
                            );
                            c == 0 as ::core::ffi::c_int
                        }
                    {
                        if mode_displayed.get() {
                            unshowmode(true_0 != 0);
                            mode_deleted = true_0 != 0;
                        }
                        validate_cursor(curwin.get());
                        let mut old_wcol: ::core::ffi::c_int = (*curwin.get()).w_wcol;
                        let mut old_wrow: ::core::ffi::c_int = (*curwin.get()).w_wrow;
                        if (*curwin.get()).w_cursor.col != 0 as ::core::ffi::c_int {
                            let mut col: colnr_T = 0 as colnr_T;
                            let mut ptr: *mut ::core::ffi::c_char =
                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                            if (*curwin.get()).w_wcol > 0 as ::core::ffi::c_int {
                                if did_ai.get() as ::core::ffi::c_int != 0
                                    && *skipwhite(
                                        get_cursor_line_ptr()
                                            .offset((*curwin.get()).w_cursor.col as isize),
                                    ) as ::core::ffi::c_int
                                        == NUL
                                {
                                    (*curwin.get()).w_wcol = 0 as ::core::ffi::c_int;
                                    ptr = get_cursor_line_ptr();
                                    let mut endptr: *mut ::core::ffi::c_char =
                                        ptr.offset((*curwin.get()).w_cursor.col as isize);
                                    let mut csarg: CharsizeArg = CharsizeArg {
                                        win: ::core::ptr::null_mut::<win_T>(),
                                        line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                        use_tabstop: false,
                                        indent_width: 0,
                                        virt_row: 0,
                                        cur_text_width_left: 0,
                                        cur_text_width_right: 0,
                                        max_head_vcol: 0,
                                        iter: [MarkTreeIter {
                                            pos: MTPos { row: 0, col: 0 },
                                            lvl: 0,
                                            x: ::core::ptr::null_mut::<MTNode>(),
                                            i: 0,
                                            s: [C2Rust_Unnamed_28 { oldcol: 0, i: 0 }; 20],
                                            intersect_idx: 0,
                                            intersect_pos: MTPos { row: 0, col: 0 },
                                            intersect_pos_x: MTPos { row: 0, col: 0 },
                                        }; 1],
                                    };
                                    let mut cstype: CSType = init_charsize_arg(
                                        &raw mut csarg,
                                        curwin.get(),
                                        (*curwin.get()).w_cursor.lnum,
                                        ptr,
                                    );
                                    let mut ci: StrCharInfo = utf_ptr2StrCharInfo(ptr);
                                    let mut vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    while ci.ptr < endptr {
                                        if !ascii_iswhite(ci.chr.value as ::core::ffi::c_int) {
                                            (*curwin.get()).w_wcol = vcol;
                                        }
                                        vcol += win_charsize(
                                            cstype,
                                            vcol,
                                            ci.ptr,
                                            ci.chr.value,
                                            &raw mut csarg,
                                        )
                                        .width;
                                        ci = utfc_next(ci);
                                    }
                                    (*curwin.get()).w_wrow = (*curwin.get()).w_cline_row
                                        + (*curwin.get()).w_wcol / (*curwin.get()).w_view_width;
                                    (*curwin.get()).w_wcol %= (*curwin.get()).w_view_width;
                                    (*curwin.get()).w_wcol += win_col_off(curwin.get());
                                    col = 0 as ::core::ffi::c_int as colnr_T;
                                } else {
                                    (*curwin.get()).w_wcol -= 1;
                                    col = ((*curwin.get()).w_cursor.col as ::core::ffi::c_int
                                        - 1 as ::core::ffi::c_int)
                                        as colnr_T;
                                }
                            } else if (*curwin.get()).w_onebuf_opt.wo_wrap != 0
                                && (*curwin.get()).w_wrow != 0
                            {
                                (*curwin.get()).w_wrow -= 1;
                                (*curwin.get()).w_wcol =
                                    (*curwin.get()).w_view_width - 1 as ::core::ffi::c_int;
                                col = ((*curwin.get()).w_cursor.col as ::core::ffi::c_int
                                    - 1 as ::core::ffi::c_int)
                                    as colnr_T;
                            }
                            if col > 0 as ::core::ffi::c_int
                                && (*curwin.get()).w_wcol > 0 as ::core::ffi::c_int
                            {
                                ptr = get_cursor_line_ptr();
                                col -= utf_head_off(ptr, ptr.offset(col as isize));
                                if utf_ptr2cells(ptr.offset(col as isize)) > 1 as ::core::ffi::c_int
                                {
                                    (*curwin.get()).w_wcol -= 1;
                                }
                            }
                        }
                        setcursor();
                        ui_flush();
                        new_wcol = (*curwin.get()).w_wcol;
                        new_wrow = (*curwin.get()).w_wrow;
                        (*curwin.get()).w_wcol = old_wcol;
                        (*curwin.get()).w_wrow = old_wrow;
                    }
                    if c < 0 as ::core::ffi::c_int {
                        continue;
                    }
                    let mut n: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                    while n <= c {
                        *(*typebuf.ptr())
                            .tb_noremap
                            .offset(((*typebuf.ptr()).tb_off + n) as isize) =
                            RM_YES as ::core::ffi::c_int as uint8_t;
                        n += 1;
                    }
                    (*typebuf.ptr()).tb_len += c;
                    if (*typebuf.ptr()).tb_len
                        >= (*typebuf.ptr()).tb_maplen + MAXMAPLEN as ::core::ffi::c_int
                    {
                        timedout = true_0 != 0;
                    } else if ex_normal_busy.get() > 0 as ::core::ffi::c_int {
                        static tc: GlobalCell<::core::ffi::c_int> =
                            GlobalCell::new(0 as ::core::ffi::c_int);
                        if (*typebuf.ptr()).tb_len > 0 as ::core::ffi::c_int {
                            timedout = true_0 != 0;
                        } else {
                            c = if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0
                                || cmdwin_type.get() > 0 as ::core::ffi::c_int && tc.get() == ESC
                            {
                                Ctrl_C
                            } else {
                                ESC
                            };
                            tc.set(c);
                            if advance {
                                typebuf_was_empty.set(true_0 != 0);
                            }
                            if pending_exmode_active.get() {
                                exmode_active.set(true_0 != 0);
                            }
                            (*typebuf.ptr()).tb_no_abbr_cnt = 0 as ::core::ffi::c_int;
                            break;
                        }
                    } else {
                        if (State.get() & MODE_INSERT as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_int
                            || p_lz.get() != 0)
                            && State.get() & MODE_CMDLINE as ::core::ffi::c_int
                                == 0 as ::core::ffi::c_int
                            && advance as ::core::ffi::c_int != 0
                            && must_redraw.get() != 0 as ::core::ffi::c_int
                            && !need_wait_return.get()
                        {
                            update_screen();
                            setcursor();
                        }
                        let mut showcmd_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut showing_partial: bool = false_0 != 0;
                        if (*typebuf.ptr()).tb_len > 0 as ::core::ffi::c_int
                            && advance as ::core::ffi::c_int != 0
                            && !exmode_active.get()
                        {
                            if (State.get()
                                & (MODE_NORMAL as ::core::ffi::c_int
                                    | MODE_INSERT as ::core::ffi::c_int)
                                != 0
                                || State.get() == MODE_LANGMAP as ::core::ffi::c_int)
                                && State.get() != MODE_HITRETURN as ::core::ffi::c_int
                            {
                                if State.get() & MODE_INSERT as ::core::ffi::c_int != 0
                                    && ptr2cells(
                                        ((*typebuf.ptr()).tb_buf as *mut ::core::ffi::c_char)
                                            .offset((*typebuf.ptr()).tb_off as isize)
                                            .offset((*typebuf.ptr()).tb_len as isize)
                                            .offset(-(1 as ::core::ffi::c_int as isize)),
                                    ) == 1 as ::core::ffi::c_int
                                {
                                    edit_putchar(
                                        *(*typebuf.ptr()).tb_buf.offset(
                                            ((*typebuf.ptr()).tb_off + (*typebuf.ptr()).tb_len
                                                - 1 as ::core::ffi::c_int)
                                                as isize,
                                        )
                                            as ::core::ffi::c_int,
                                        false_0 != 0,
                                    );
                                    setcursor();
                                    showing_partial = true_0 != 0;
                                }
                                let mut old_wcol_0: ::core::ffi::c_int = (*curwin.get()).w_wcol;
                                let mut old_wrow_0: ::core::ffi::c_int = (*curwin.get()).w_wrow;
                                (*curwin.get()).w_wcol = new_wcol;
                                (*curwin.get()).w_wrow = new_wrow;
                                push_showcmd();
                                if (*typebuf.ptr()).tb_len > SHOWCMD_COLS as ::core::ffi::c_int {
                                    showcmd_idx = (*typebuf.ptr()).tb_len
                                        - SHOWCMD_COLS as ::core::ffi::c_int;
                                }
                                while showcmd_idx < (*typebuf.ptr()).tb_len {
                                    let c2rust_fresh5 = showcmd_idx;
                                    showcmd_idx = showcmd_idx + 1;
                                    add_byte_to_showcmd(*(*typebuf.ptr()).tb_buf.offset(
                                        ((*typebuf.ptr()).tb_off + c2rust_fresh5) as isize,
                                    ));
                                }
                                (*curwin.get()).w_wcol = old_wcol_0;
                                (*curwin.get()).w_wrow = old_wrow_0;
                            }
                            if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0
                                && !(*get_cmdline_info()).cmdbuff.is_null()
                                && cmdline_star.get() == 0 as ::core::ffi::c_int
                            {
                                let mut p: *mut ::core::ffi::c_char = ((*typebuf.ptr()).tb_buf
                                    as *mut ::core::ffi::c_char)
                                    .offset((*typebuf.ptr()).tb_off as isize)
                                    .offset((*typebuf.ptr()).tb_len as isize)
                                    .offset(-(1 as ::core::ffi::c_int as isize));
                                if ptr2cells(p) == 1 as ::core::ffi::c_int
                                    && (*p as uint8_t as ::core::ffi::c_int)
                                        < 128 as ::core::ffi::c_int
                                {
                                    putcmdline(*p, false_0 != 0);
                                    showing_partial = true_0 != 0;
                                }
                            }
                        }
                        if (*typebuf.ptr()).tb_len == 0 as ::core::ffi::c_int {
                            timedout = false_0 != 0;
                        }
                        let mut wait_time: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if advance {
                            if (*typebuf.ptr()).tb_len == 0 as ::core::ffi::c_int
                                || !(p_timeout.get() != 0
                                    || p_ttimeout.get() != 0
                                        && keylen == KEYLEN_PART_KEY as ::core::ffi::c_int)
                            {
                                wait_time = -1 as ::core::ffi::c_int;
                            } else if keylen == KEYLEN_PART_KEY as ::core::ffi::c_int
                                && p_ttm.get() >= 0 as OptInt
                            {
                                wait_time = p_ttm.get() as ::core::ffi::c_int;
                            } else {
                                wait_time = p_tm.get() as ::core::ffi::c_int;
                            }
                        }
                        let mut wait_tb_len: ::core::ffi::c_int = (*typebuf.ptr()).tb_len;
                        c = inchar(
                            (*typebuf.ptr())
                                .tb_buf
                                .offset((*typebuf.ptr()).tb_off as isize)
                                .offset((*typebuf.ptr()).tb_len as isize),
                            (*typebuf.ptr()).tb_buflen
                                - (*typebuf.ptr()).tb_off
                                - (*typebuf.ptr()).tb_len
                                - 1 as ::core::ffi::c_int,
                            wait_time as ::core::ffi::c_long,
                        );
                        if showcmd_idx != 0 as ::core::ffi::c_int {
                            pop_showcmd();
                        }
                        if showing_partial as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
                            if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 {
                                edit_unputchar();
                            }
                            if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0
                                && !(*get_cmdline_info()).cmdbuff.is_null()
                            {
                                unputcmdline();
                            } else {
                                setcursor();
                            }
                        }
                        if c < 0 as ::core::ffi::c_int {
                            continue;
                        }
                        if c == NUL {
                            if !advance {
                                break;
                            }
                            if wait_tb_len <= 0 as ::core::ffi::c_int {
                                continue;
                            }
                            timedout = true_0 != 0;
                        } else {
                            while *(*typebuf.ptr()).tb_buf.offset(
                                ((*typebuf.ptr()).tb_off + (*typebuf.ptr()).tb_len) as isize,
                            ) as ::core::ffi::c_int
                                != NUL
                            {
                                let c2rust_fresh6 = (*typebuf.ptr()).tb_len;
                                (*typebuf.ptr()).tb_len = (*typebuf.ptr()).tb_len + 1;
                                *(*typebuf.ptr())
                                    .tb_noremap
                                    .offset(((*typebuf.ptr()).tb_off + c2rust_fresh6) as isize) =
                                    RM_YES as ::core::ffi::c_int as uint8_t;
                            }
                        }
                    }
                }
            }
        }
        if !(c < 0 as ::core::ffi::c_int || advance as ::core::ffi::c_int != 0 && c == NUL) {
            break;
        }
    }
    if advance as ::core::ffi::c_int != 0
        && p_smd.get() != 0
        && msg_silent.get() == 0 as ::core::ffi::c_int
        && State.get() & MODE_INSERT as ::core::ffi::c_int != 0
    {
        if c == ESC
            && !mode_deleted
            && no_mapping.get() == 0
            && mode_displayed.get() as ::core::ffi::c_int != 0
        {
            if (*typebuf.ptr()).tb_len != 0 && !KeyTyped.get() {
                redraw_cmdline.set(true_0 != 0);
            } else {
                unshowmode(false_0 != 0);
            }
        } else if c != ESC && mode_deleted as ::core::ffi::c_int != 0 {
            if (*typebuf.ptr()).tb_len != 0 && !KeyTyped.get() {
                redraw_cmdline.set(true_0 != 0);
            } else {
                showmode();
            }
        }
    }
    if timedout as ::core::ffi::c_int != 0 && c == ESC {
        gotchars_ignore();
    }
    (*vgetc_busy.ptr()) -= 1;
    return c;
}
unsafe extern "C" fn inchar(
    mut buf: *mut uint8_t,
    mut maxlen: ::core::ffi::c_int,
    mut wait_time: ::core::ffi::c_long,
) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut retesc: ::core::ffi::c_int = false_0;
    let tb_change_cnt: ::core::ffi::c_int = (*typebuf.ptr()).tb_change_cnt;
    if wait_time == -1 as ::core::ffi::c_long || wait_time > 100 as ::core::ffi::c_long {
        ui_flush();
    }
    if State.get() != MODE_HITRETURN as ::core::ffi::c_int {
        did_outofmem_msg.set(false_0 != 0);
        did_swapwrite_msg.set(false_0 != 0);
    }
    let mut read_size: ptrdiff_t = -1 as ptrdiff_t;
    while curscript.get() >= 0 as ::core::ffi::c_int
        && read_size <= 0 as ptrdiff_t
        && !ignore_script.get()
    {
        let mut script_char: ::core::ffi::c_char = 0;
        if got_int.get() as ::core::ffi::c_int != 0 || {
            read_size = file_read(
                (scriptin.ptr() as *mut FileDescriptor).offset(curscript.get() as isize),
                &raw mut script_char,
                1 as size_t,
            );
            read_size != 1 as ptrdiff_t
        } {
            closescript();
            if got_int.get() {
                retesc = true_0;
            } else {
                return -1 as ::core::ffi::c_int;
            }
        } else {
            *buf.offset(0 as ::core::ffi::c_int as isize) = script_char as uint8_t;
            len = 1 as ::core::ffi::c_int;
        }
    }
    if read_size <= 0 as ptrdiff_t {
        if got_int.get() {
            let mut dum: [uint8_t; 154] = [0; 154];
            loop {
                len = input_get(
                    &raw mut dum as *mut uint8_t,
                    MAXMAPLEN as ::core::ffi::c_int * 3 as ::core::ffi::c_int
                        + 3 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<MultiQueue>(),
                );
                if len == 0 as ::core::ffi::c_int
                    || len == 1 as ::core::ffi::c_int
                        && dum[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == Ctrl_C
                {
                    break;
                }
            }
            return retesc;
        }
        if wait_time == -1 as ::core::ffi::c_long || wait_time > 10 as ::core::ffi::c_long {
            ui_flush();
        }
        len = input_get(
            buf,
            maxlen / 3 as ::core::ffi::c_int,
            wait_time as ::core::ffi::c_int,
            tb_change_cnt,
            ::core::ptr::null_mut::<MultiQueue>(),
        );
    }
    if typebuf_changed(tb_change_cnt) {
        return 0 as ::core::ffi::c_int;
    }
    if len > 0 as ::core::ffi::c_int && {
        (*typebuf.ptr()).tb_change_cnt += 1;
        (*typebuf.ptr()).tb_change_cnt == 0 as ::core::ffi::c_int
    } {
        (*typebuf.ptr()).tb_change_cnt = 1 as ::core::ffi::c_int;
    }
    return fix_input_buffer(buf, len);
}
#[no_mangle]
pub unsafe extern "C" fn fix_input_buffer(
    mut buf: *mut uint8_t,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if using_script() == 0 {
        *buf.offset(len as isize) = NUL as uint8_t;
        return len;
    }
    let mut p: *mut uint8_t = buf;
    let mut i: ::core::ffi::c_int = len;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == K_SPECIAL
                && (i < 2 as ::core::ffi::c_int
                    || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != KS_EXTRA)
        {
            memmove(
                p.offset(3 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                i as size_t,
            );
            *p.offset(2 as ::core::ffi::c_int as isize) =
                (if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == K_SPECIAL
                    || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                {
                    KE_FILLER as ::core::ffi::c_uint
                } else {
                    -(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                        as ::core::ffi::c_uint
                        >> 8 as ::core::ffi::c_int
                        & 0xff as ::core::ffi::c_uint
                }) as uint8_t;
            *p.offset(1 as ::core::ffi::c_int as isize) = (if *p
                .offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == K_SPECIAL
            {
                KS_SPECIAL
            } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                KS_ZERO
            } else {
                -(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    & 0xff as ::core::ffi::c_int
            }) as uint8_t;
            *p.offset(0 as ::core::ffi::c_int as isize) = K_SPECIAL as uint8_t;
            p = p.offset(2 as ::core::ffi::c_int as isize);
            len += 2 as ::core::ffi::c_int;
        }
        p = p.offset(1);
    }
    *p = NUL as uint8_t;
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn getcmdkeycmd(
    mut _promptc: ::core::ffi::c_int,
    mut _cookie: *mut ::core::ffi::c_void,
    mut _indent: ::core::ffi::c_int,
    mut _do_concat: bool,
) -> *mut ::core::ffi::c_char {
    let mut line_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut c1: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut cmod: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut aborted: bool = false_0 != 0;
    ga_init(
        &raw mut line_ga,
        1 as ::core::ffi::c_int,
        32 as ::core::ffi::c_int,
    );
    (*no_mapping.ptr()) += 1;
    got_int.set(false_0 != 0);
    while c1 != NUL && !aborted {
        ga_grow(&raw mut line_ga, 32 as ::core::ffi::c_int);
        if vgetorpeek(false_0 != 0) == NUL {
            emsg(gettext(
                (e_cmd_mapping_must_end_with_cr.ptr() as *const _) as *const ::core::ffi::c_char,
            ));
            aborted = true_0 != 0;
            break;
        } else {
            c1 = vgetorpeek(true_0 != 0);
            if c1 == K_SPECIAL {
                c1 = vgetorpeek(true_0 != 0);
                let mut c2: ::core::ffi::c_int = vgetorpeek(true_0 != 0);
                if c1 == KS_MODIFIER {
                    cmod = c2;
                    continue;
                } else {
                    c1 = if c1 == KS_SPECIAL {
                        K_SPECIAL
                    } else if c1 == KS_ZERO {
                        K_ZERO
                    } else {
                        -(c1 + (c2 << 8 as ::core::ffi::c_int))
                    };
                }
            }
            if got_int.get() {
                aborted = true_0 != 0;
            } else if c1 == '\r' as ::core::ffi::c_int || c1 == '\n' as ::core::ffi::c_int {
                c1 = NUL;
            } else if c1 == ESC {
                aborted = true_0 != 0;
            } else if c1
                == -(253 as ::core::ffi::c_int
                    + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                emsg(gettext(
                    (e_cmd_mapping_must_end_with_cr_before_second_cmd.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ));
                aborted = true_0 != 0;
            } else if c1
                == -(253 as ::core::ffi::c_int
                    + ((KE_SNR as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                ga_concat_len(
                    &raw mut line_ga,
                    b"<SNR>\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                );
            } else {
                if cmod != 0 as ::core::ffi::c_int {
                    ga_append(&raw mut line_ga, K_SPECIAL as uint8_t);
                    ga_append(&raw mut line_ga, KS_MODIFIER as uint8_t);
                    ga_append(&raw mut line_ga, cmod as uint8_t);
                }
                if c1 < 0 as ::core::ffi::c_int {
                    ga_append(&raw mut line_ga, K_SPECIAL as uint8_t);
                    ga_append(
                        &raw mut line_ga,
                        (if c1 == K_SPECIAL {
                            KS_SPECIAL
                        } else if c1 == NUL {
                            KS_ZERO
                        } else {
                            -c1 & 0xff as ::core::ffi::c_int
                        }) as uint8_t,
                    );
                    ga_append(
                        &raw mut line_ga,
                        (if c1 == K_SPECIAL || c1 == NUL {
                            KE_FILLER as ::core::ffi::c_uint
                        } else {
                            -c1 as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int
                                & 0xff as ::core::ffi::c_uint
                        }) as uint8_t,
                    );
                } else {
                    ga_append(&raw mut line_ga, c1 as uint8_t);
                }
            }
            cmod = 0 as ::core::ffi::c_int;
        }
    }
    (*no_mapping.ptr()) -= 1;
    if aborted {
        ga_clear(&raw mut line_ga);
    }
    return line_ga.ga_data as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn map_execute_lua(mut may_repeat: bool, mut discard: bool) -> bool {
    let mut line_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut c1: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut aborted: bool = false_0 != 0;
    ga_init(
        &raw mut line_ga,
        1 as ::core::ffi::c_int,
        32 as ::core::ffi::c_int,
    );
    (*no_mapping.ptr()) += 1;
    got_int.set(false_0 != 0);
    while c1 != NUL && !aborted {
        ga_grow(&raw mut line_ga, 32 as ::core::ffi::c_int);
        c1 = vgetorpeek(true_0 != 0);
        if got_int.get() {
            aborted = true_0 != 0;
        } else if c1 == '\r' as ::core::ffi::c_int || c1 == '\n' as ::core::ffi::c_int {
            c1 = NUL;
        } else {
            ga_append(&raw mut line_ga, c1 as uint8_t);
        }
    }
    (*no_mapping.ptr()) -= 1;
    if aborted as ::core::ffi::c_int != 0 || discard as ::core::ffi::c_int != 0 {
        ga_clear(&raw mut line_ga);
        return !aborted;
    }
    let mut ref_0: LuaRef = atoi(line_ga.ga_data as *const ::core::ffi::c_char);
    if may_repeat {
        repeat_luaref.set(ref_0);
    }
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut args: Array = ARRAY_DICT_INIT;
    nlua_call_ref(
        ref_0,
        ::core::ptr::null::<::core::ffi::c_char>(),
        args,
        kRetNilBool,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        semsg_multiline(
            b"emsg\0".as_ptr() as *const ::core::ffi::c_char,
            b"E5108: %s\0".as_ptr() as *const ::core::ffi::c_char,
            err.msg,
        );
        api_clear_error(&raw mut err);
    }
    ga_clear(&raw mut line_ga);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn paste_store(
    channel_id: uint64_t,
    state: TriState,
    str: String_0,
    crlf: bool,
) {
    if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0 {
        return;
    }
    let need_redo: bool = !block_redo.get();
    let need_record: bool =
        reg_recording.get() != 0 as ::core::ffi::c_int && !is_internal_call(channel_id);
    if !need_redo && !need_record {
        return;
    }
    if state as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
        let c: ::core::ffi::c_int = if state as ::core::ffi::c_int == kFalse as ::core::ffi::c_int {
            K_PASTE_START
        } else {
            K_PASTE_END
        };
        if need_redo {
            if state as ::core::ffi::c_int == kFalse as ::core::ffi::c_int
                && State.get() & MODE_INSERT as ::core::ffi::c_int == 0
            {
                ResetRedobuff();
            }
            add_char_buff(redobuff.ptr(), c);
        }
        if need_record {
            add_char_buff(recordbuff.ptr(), c);
        }
        return;
    }
    let mut s: *const ::core::ffi::c_char = str.data;
    let str_end: *const ::core::ffi::c_char = str.data.offset(str.size as isize);
    while s < str_end {
        let mut start: *const ::core::ffi::c_char = s;
        while s < str_end
            && *s as uint8_t as ::core::ffi::c_int != K_SPECIAL
            && *s as ::core::ffi::c_int != NUL
            && *s as ::core::ffi::c_int != NL
            && !(crlf as ::core::ffi::c_int != 0 && *s as ::core::ffi::c_int == CAR)
        {
            s = s.offset(1);
        }
        if s > start {
            if need_redo {
                add_buff(redobuff.ptr(), start, s.offset_from(start));
            }
            if need_record {
                add_buff(recordbuff.ptr(), start, s.offset_from(start));
            }
        }
        if s < str_end {
            let c2rust_fresh17 = s;
            s = s.offset(1);
            let mut c_0: ::core::ffi::c_int = *c2rust_fresh17 as uint8_t as ::core::ffi::c_int;
            if crlf as ::core::ffi::c_int != 0 && c_0 == CAR {
                if s < str_end && *s as ::core::ffi::c_int == NL {
                    s = s.offset(1);
                }
                c_0 = NL;
            }
            if need_redo {
                add_byte_buff(redobuff.ptr(), c_0);
            }
            if need_record {
                add_byte_buff(recordbuff.ptr(), c_0);
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn paste_repeat(mut count: ::core::ffi::c_int) {
    let mut ga: garray_T = garray_T {
        ga_len: 0 as ::core::ffi::c_int,
        ga_maxlen: 0 as ::core::ffi::c_int,
        ga_itemsize: 1 as ::core::ffi::c_int,
        ga_growsize: 32 as ::core::ffi::c_int,
        ga_data: NULL_0,
    };
    let mut aborted: bool = false_0 != 0;
    (*no_mapping.ptr()) += 1;
    got_int.set(false_0 != 0);
    while !aborted {
        ga_grow(&raw mut ga, 32 as ::core::ffi::c_int);
        let mut c1: uint8_t = vgetorpeek(true_0 != 0) as uint8_t;
        if c1 as ::core::ffi::c_int == K_SPECIAL {
            c1 = vgetorpeek(true_0 != 0) as uint8_t;
            let mut c2: uint8_t = vgetorpeek(true_0 != 0) as uint8_t;
            let mut c: ::core::ffi::c_int = if c1 as ::core::ffi::c_int == KS_SPECIAL {
                K_SPECIAL
            } else if c1 as ::core::ffi::c_int == KS_ZERO {
                K_ZERO
            } else {
                -(c1 as ::core::ffi::c_int
                    + ((c2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            };
            if c == K_PASTE_END {
                break;
            }
            if c == K_ZERO {
                ga_append(&raw mut ga, NUL as uint8_t);
            } else if c == K_SPECIAL {
                ga_append(&raw mut ga, K_SPECIAL as uint8_t);
            } else {
                ga_append(&raw mut ga, K_SPECIAL as uint8_t);
                ga_append(&raw mut ga, c1);
                ga_append(&raw mut ga, c2);
            }
        } else {
            ga_append(&raw mut ga, c1);
        }
        aborted = got_int.get();
    }
    (*no_mapping.ptr()) -= 1;
    let mut str: String_0 = String_0 {
        data: ga.ga_data as *mut ::core::ffi::c_char,
        size: ga.ga_len as size_t,
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !aborted && i < count {
        nvim_paste(
            LUA_INTERNAL_CALL,
            str,
            false_0 != 0,
            -1 as Integer,
            &raw mut arena,
            &raw mut err,
        );
        aborted = err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int;
        i += 1;
    }
    api_clear_error(&raw mut err);
    arena_mem_free(arena_finish(&raw mut arena));
    ga_clear(&raw mut ga);
}
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const KS_ZERO: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
pub const K_SELECT_STRING: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"\x80\xF5X\0") };
pub const KE_FILLER: ::core::ffi::c_int = 'X' as ::core::ffi::c_int;
pub const K_ZERO: ::core::ffi::c_int =
    -(255 as ::core::ffi::c_int + (('X' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_UP: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('u' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KUP: ::core::ffi::c_int = -30027;
pub const K_DOWN: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('d' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KDOWN: ::core::ffi::c_int = -25675;
pub const K_LEFT: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('l' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KLEFT: ::core::ffi::c_int = -27723;
pub const K_RIGHT: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('r' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KRIGHT: ::core::ffi::c_int = -29259;
pub const K_S_HOME: ::core::ffi::c_int =
    -('#' as ::core::ffi::c_int + (('2' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_S_END: ::core::ffi::c_int =
    -('*' as ::core::ffi::c_int + (('7' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_XUP: ::core::ffi::c_int = -16893;
pub const K_XDOWN: ::core::ffi::c_int = -17149;
pub const K_XLEFT: ::core::ffi::c_int = -17405;
pub const K_XRIGHT: ::core::ffi::c_int = -17661;
pub const K_HOME: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('h' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_XHOME: ::core::ffi::c_int = -16381;
pub const K_ZHOME: ::core::ffi::c_int = -16637;
pub const K_END: ::core::ffi::c_int =
    -('@' as ::core::ffi::c_int + (('7' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_XEND: ::core::ffi::c_int = -15869;
pub const K_ZEND: ::core::ffi::c_int = -16125;
pub const K_KPLUS: ::core::ffi::c_int = -13899;
pub const K_KMINUS: ::core::ffi::c_int = -14155;
pub const K_KDIVIDE: ::core::ffi::c_int = -14411;
pub const K_KMULTIPLY: ::core::ffi::c_int = -14667;
pub const K_KENTER: ::core::ffi::c_int = -16715;
pub const K_KPOINT: ::core::ffi::c_int = -16971;
pub const K_PASTE_START: ::core::ffi::c_int =
    -('P' as ::core::ffi::c_int + (('S' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_PASTE_END: ::core::ffi::c_int =
    -('P' as ::core::ffi::c_int + (('E' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_K0: ::core::ffi::c_int = -17227;
pub const K_K1: ::core::ffi::c_int = -17483;
pub const K_K2: ::core::ffi::c_int = -17739;
pub const K_K3: ::core::ffi::c_int = -17995;
pub const K_K4: ::core::ffi::c_int = -18251;
pub const K_K5: ::core::ffi::c_int = -18507;
pub const K_K6: ::core::ffi::c_int = -18763;
pub const K_K7: ::core::ffi::c_int = -19019;
pub const K_K8: ::core::ffi::c_int = -19275;
pub const K_K9: ::core::ffi::c_int = -19531;
pub const K_KCOMMA: ::core::ffi::c_int = -19787;
pub const K_KEQUAL: ::core::ffi::c_int = -20043;
pub const K_VER_SCROLLBAR: ::core::ffi::c_int =
    -(249 as ::core::ffi::c_int + (('X' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_HOR_SCROLLBAR: ::core::ffi::c_int =
    -(248 as ::core::ffi::c_int + (('X' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const MOD_MASK_SHIFT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOD_MASK_CTRL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MOD_MASK_ALT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn utf_ptr2CharInfo(p_in: *const ::core::ffi::c_char) -> CharInfo {
    let p: *const uint8_t = p_in as *const uint8_t;
    let first: uint8_t = *p;
    if (first as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        return CharInfo {
            value: first as int32_t,
            len: 1 as ::core::ffi::c_int,
        };
    } else {
        let mut len: ::core::ffi::c_int = utf8len_tab[first as usize] as ::core::ffi::c_int;
        let code_point: int32_t = utf_ptr2CharInfo_impl(p, len as uintptr_t);
        if code_point < 0 as int32_t {
            len = 1 as ::core::ffi::c_int;
        }
        return CharInfo {
            value: code_point,
            len: len,
        };
    };
}
#[inline(always)]
unsafe extern "C" fn utfc_next(mut cur: StrCharInfo) -> StrCharInfo {
    let mut next: *mut uint8_t = cur.ptr.offset(cur.chr.len as isize) as *mut uint8_t;
    if ((*next as ::core::ffi::c_uint) < 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
        as ::core::ffi::c_long
        != 0
    {
        return StrCharInfo {
            ptr: next as *mut ::core::ffi::c_char,
            chr: CharInfo {
                value: *next as int32_t,
                len: 1 as ::core::ffi::c_int,
            },
        };
    }
    return utfc_next_impl(cur);
}
#[inline(always)]
unsafe extern "C" fn utf_ptr2StrCharInfo(mut ptr: *mut ::core::ffi::c_char) -> StrCharInfo {
    return StrCharInfo {
        ptr: ptr,
        chr: utf_ptr2CharInfo(ptr),
    };
}
#[inline(always)]
unsafe extern "C" fn win_charsize(
    mut cstype: CSType,
    mut vcol: ::core::ffi::c_int,
    mut ptr: *mut ::core::ffi::c_char,
    mut chr: int32_t,
    mut csarg: *mut CharsizeArg,
) -> CharSize {
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        return charsize_fast(csarg, ptr, vcol as colnr_T, chr);
    } else {
        return charsize_regular(csarg, ptr, vcol as colnr_T, chr);
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
unsafe extern "C" fn c2rust_run_static_initializers() {
    on_key_buf.set(C2Rust_Unnamed_31 {
        size: 0 as size_t,
        capacity: ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as size_t,
            ),
        items: &raw mut (*on_key_buf.ptr()).init_array as *mut ::core::ffi::c_char,
        init_array: [0; 51],
    });
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [c2rust_run_static_initializers];
