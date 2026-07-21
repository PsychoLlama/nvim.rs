use crate::src::nvim::global_cell::GlobalCell;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, CMD_index, Callback,
    CallbackType, Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_2, ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative,
    GridView, Intersection, LineGetter, LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t,
    Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, OptInt,
    ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_12, Terminal,
    Timestamp, VarLockStatus, VarType, VimVarIndex, VirtLines, VirtText, VirtTextChunk,
    VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle, Window, __time_t, aco_save_T, aentry_T,
    alist_T, auto_event, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufref_T, bufstate_T,
    chunksize_T, cmd_addr_T, cmdidx_T, cmdmod_T, colnr_T, cstack_T,
    cstack_T_cs_pend as C2Rust_Unnamed_14, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T,
    dobuf_action_values, dobuf_start_values, eslist_T, eslist_elem, event_T, exarg, exarg_T,
    extmark_undo_vec_t, fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_11, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T, linenr_T, list_T,
    listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s,
    mtnode_s, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S,
    qf_info_T, queue, reg_extmatch_T, regmatch_T, regmmatch_T, regprog, regprog_T, sattr_T,
    schar_T, scid_T, sctx_T, size_t, ssize_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_4,
    syn_time_T, synblock_T, synstate_T, tabpage_S, tabpage_T, taggy_T, terminal, time_t, typval_T,
    typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_8, u_header_uh_alt_prev as C2Rust_Unnamed_7,
    u_header_uh_next as C2Rust_Unnamed_10, u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, varnumber_T, virt_line, visualinfo_T,
    win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
};
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn set_arglist(str: *mut ::core::ffi::c_char);
    fn editing_arg_idx(win: *mut win_T) -> bool;
    fn ex_rewind(eap: *mut exarg_T);
    fn do_argfile(eap: *mut exarg_T, argn: ::core::ffi::c_int);
    fn ex_all(eap: *mut exarg_T);
    fn au_event_disable(what: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn au_event_restore(old_ei: *mut ::core::ffi::c_char);
    fn aucmd_prepbuf(aco: *mut aco_save_T, buf: *mut buf_T);
    fn aucmd_restbuf(aco: *mut aco_save_T);
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn set_bufref(bufref: *mut bufref_T, buf: *mut buf_T);
    fn bufref_valid(bufref: *mut bufref_T) -> bool;
    fn goto_buffer(
        eap: *mut exarg_T,
        start: ::core::ffi::c_int,
        dir: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
    );
    fn set_curbuf(buf: *mut buf_T, action: ::core::ffi::c_int, update_jumplist: bool);
    fn no_write_message();
    fn no_write_message_nobang(buf: *const buf_T);
    fn buflist_findnr(nr: ::core::ffi::c_int) -> *mut buf_T;
    fn buf_set_name(fnum: ::core::ffi::c_int, name: *mut ::core::ffi::c_char);
    fn bt_dontwrite(buf: *const buf_T) -> bool;
    fn buf_hide(buf: *const buf_T) -> bool;
    fn buf_spname(buf: *mut buf_T) -> *mut ::core::ffi::c_char;
    fn buf_write(
        buf: *mut buf_T,
        fname: *mut ::core::ffi::c_char,
        sfname: *mut ::core::ffi::c_char,
        start: linenr_T,
        end: linenr_T,
        eap: *mut exarg_T,
        append: bool,
        forceit: bool,
        reset_changed: bool,
        filtering: bool,
    ) -> ::core::ffi::c_int;
    fn unchanged(buf: *mut buf_T, ff: bool, always_inc_changedtick: bool);
    fn channel_job_running(id: uint64_t) -> bool;
    static e_noname: [::core::ffi::c_char; 0];
    static e_winfixbuf_cannot_go_to_buffer: [::core::ffi::c_char; 0];
    fn eval_call_provider(
        provider: *mut ::core::ffi::c_char,
        method: *mut ::core::ffi::c_char,
        arguments: *mut list_T,
        discard: bool,
    ) -> typval_T;
    static msg_listdo_overwrite: GlobalCell<::core::ffi::c_int>;
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn msg_source(hl_id: ::core::ffi::c_int);
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn wait_return(redraw: ::core::ffi::c_int);
    fn vim_dialog_yesnocancel(
        type_0: ::core::ffi::c_int,
        title: *mut ::core::ffi::c_char,
        message: *mut ::core::ffi::c_char,
        dflt: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn vim_dialog_yesnoallcancel(
        type_0: ::core::ffi::c_int,
        title: *mut ::core::ffi::c_char,
        message: *mut ::core::ffi::c_char,
        dflt: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_append_string(l: *mut list_T, str: *const ::core::ffi::c_char, len: ssize_t);
    fn tv_list_append_allocated_string(l: *mut list_T, str: *mut ::core::ffi::c_char);
    fn tv_list_append_number(l: *mut list_T, n: varnumber_T);
    fn set_internal_string_var(name: *const ::core::ffi::c_char, value: *mut ::core::ffi::c_char);
    fn do_unlet(
        name: *const ::core::ffi::c_char,
        name_len: size_t,
        forceit: bool,
    ) -> ::core::ffi::c_int;
    fn set_vim_var_string(idx: VimVarIndex, val: *const ::core::ffi::c_char, len: ptrdiff_t);
    fn get_var_value(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn check_overwrite(
        eap: *mut exarg_T,
        buf: *mut buf_T,
        fname: *mut ::core::ffi::c_char,
        ffname: *mut ::core::ffi::c_char,
        other: bool,
    ) -> ::core::ffi::c_int;
    fn set_swapcommand(command: *mut ::core::ffi::c_char, newlnum: linenr_T) -> bool;
    fn do_cmdline_cmd(cmd: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn do_cmdline(
        cmdline: *mut ::core::ffi::c_char,
        fgetline: LineGetter,
        cookie: *mut ::core::ffi::c_void,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn dialog_msg(
        buff: *mut ::core::ffi::c_char,
        format: *mut ::core::ffi::c_char,
        fname: *mut ::core::ffi::c_char,
    );
    fn script_get(eap: *mut exarg_T, lenp: *mut size_t) -> *mut ::core::ffi::c_char;
    fn check_timestamps(focus: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn buf_check_timestamp(buf: *mut buf_T) -> ::core::ffi::c_int;
    static cmdline_row: GlobalCell<::core::ffi::c_int>;
    static msg_col: GlobalCell<::core::ffi::c_int>;
    static msg_row: GlobalCell<::core::ffi::c_int>;
    static msg_didout: GlobalCell<bool>;
    static msg_didany: GlobalCell<bool>;
    static emsg_off: GlobalCell<::core::ffi::c_int>;
    static no_wait_return: GlobalCell<::core::ffi::c_int>;
    static vgetc_busy: GlobalCell<::core::ffi::c_int>;
    static no_check_timestamps: GlobalCell<::core::ffi::c_int>;
    static firstwin: GlobalCell<*mut win_T>;
    static prevwin: GlobalCell<*mut win_T>;
    static curwin: GlobalCell<*mut win_T>;
    static first_tabpage: GlobalCell<*mut tabpage_T>;
    static curtab: GlobalCell<*mut tabpage_T>;
    static firstbuf: GlobalCell<*mut buf_T>;
    static curbuf: GlobalCell<*mut buf_T>;
    static exiting: GlobalCell<bool>;
    static cmdmod: GlobalCell<cmdmod_T>;
    static got_int: GlobalCell<bool>;
    static listcmd_busy: GlobalCell<bool>;
    fn setpcmark();
    fn validate_cursor(wp: *mut win_T);
    fn do_check_scrollbind(check: bool);
    static p_aw: GlobalCell<::core::ffi::c_int>;
    static p_awa: GlobalCell<::core::ffi::c_int>;
    static p_confirm: GlobalCell<::core::ffi::c_int>;
    static p_write: GlobalCell<::core::ffi::c_int>;
    fn vim_FullName(
        fname: *const ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        len: size_t,
        force: bool,
    ) -> ::core::ffi::c_int;
    fn qf_get_valid_size(eap: *mut exarg_T) -> size_t;
    fn qf_get_cur_idx(eap: *mut exarg_T) -> size_t;
    fn ex_cc(eap: *mut exarg_T);
    fn ex_cnext(eap: *mut exarg_T);
    fn source_runtime_vim_lua(
        name: *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn bufIsChanged(buf: *mut buf_T) -> bool;
    fn win_split(size: ::core::ffi::c_int, flags: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn win_valid(win: *const win_T) -> bool;
    fn valid_tabpage(tpc: *mut tabpage_T) -> bool;
    fn goto_tabpage_tp(
        tp: *mut tabpage_T,
        trigger_enter_autocmds: bool,
        trigger_leave_autocmds: bool,
    );
    fn goto_tabpage_win(tp: *mut tabpage_T, wp: *mut win_T);
    fn win_goto(wp: *mut win_T);
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed = 2147483647;
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_int;
pub const BACKWARD_FILE: C2Rust_Unnamed_13 = -3;
pub const FORWARD_FILE: C2Rust_Unnamed_13 = 3;
pub const BACKWARD: C2Rust_Unnamed_13 = -1;
pub const FORWARD: C2Rust_Unnamed_13 = 1;
pub const kDirectionNotSet: C2Rust_Unnamed_13 = 0;
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
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_15 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_15 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_15 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_15 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_15 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_15 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_15 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_15 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_15 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_15 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_15 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_15 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_15 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_16 = 76;
pub const HLF_PRE: C2Rust_Unnamed_16 = 75;
pub const HLF_OK: C2Rust_Unnamed_16 = 74;
pub const HLF_SO: C2Rust_Unnamed_16 = 73;
pub const HLF_SE: C2Rust_Unnamed_16 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_16 = 71;
pub const HLF_TS: C2Rust_Unnamed_16 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_16 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_16 = 68;
pub const HLF_CU: C2Rust_Unnamed_16 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_16 = 66;
pub const HLF_WBR: C2Rust_Unnamed_16 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_16 = 64;
pub const HLF_MSG: C2Rust_Unnamed_16 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_16 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_16 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_16 = 60;
pub const HLF_0: C2Rust_Unnamed_16 = 59;
pub const HLF_QFL: C2Rust_Unnamed_16 = 58;
pub const HLF_MC: C2Rust_Unnamed_16 = 57;
pub const HLF_CUL: C2Rust_Unnamed_16 = 56;
pub const HLF_CUC: C2Rust_Unnamed_16 = 55;
pub const HLF_TPF: C2Rust_Unnamed_16 = 54;
pub const HLF_TPS: C2Rust_Unnamed_16 = 53;
pub const HLF_TP: C2Rust_Unnamed_16 = 52;
pub const HLF_PBR: C2Rust_Unnamed_16 = 51;
pub const HLF_PST: C2Rust_Unnamed_16 = 50;
pub const HLF_PSB: C2Rust_Unnamed_16 = 49;
pub const HLF_PSX: C2Rust_Unnamed_16 = 48;
pub const HLF_PNX: C2Rust_Unnamed_16 = 47;
pub const HLF_PSK: C2Rust_Unnamed_16 = 46;
pub const HLF_PNK: C2Rust_Unnamed_16 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_16 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_16 = 43;
pub const HLF_PSI: C2Rust_Unnamed_16 = 42;
pub const HLF_PNI: C2Rust_Unnamed_16 = 41;
pub const HLF_SPL: C2Rust_Unnamed_16 = 40;
pub const HLF_SPR: C2Rust_Unnamed_16 = 39;
pub const HLF_SPC: C2Rust_Unnamed_16 = 38;
pub const HLF_SPB: C2Rust_Unnamed_16 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_16 = 36;
pub const HLF_SC: C2Rust_Unnamed_16 = 35;
pub const HLF_TXA: C2Rust_Unnamed_16 = 34;
pub const HLF_TXD: C2Rust_Unnamed_16 = 33;
pub const HLF_DED: C2Rust_Unnamed_16 = 32;
pub const HLF_CHD: C2Rust_Unnamed_16 = 31;
pub const HLF_ADD: C2Rust_Unnamed_16 = 30;
pub const HLF_FC: C2Rust_Unnamed_16 = 29;
pub const HLF_FL: C2Rust_Unnamed_16 = 28;
pub const HLF_WM: C2Rust_Unnamed_16 = 27;
pub const HLF_W: C2Rust_Unnamed_16 = 26;
pub const HLF_VNC: C2Rust_Unnamed_16 = 25;
pub const HLF_V: C2Rust_Unnamed_16 = 24;
pub const HLF_T: C2Rust_Unnamed_16 = 23;
pub const HLF_VSP: C2Rust_Unnamed_16 = 22;
pub const HLF_C: C2Rust_Unnamed_16 = 21;
pub const HLF_SNC: C2Rust_Unnamed_16 = 20;
pub const HLF_S: C2Rust_Unnamed_16 = 19;
pub const HLF_R: C2Rust_Unnamed_16 = 18;
pub const HLF_CLF: C2Rust_Unnamed_16 = 17;
pub const HLF_CLS: C2Rust_Unnamed_16 = 16;
pub const HLF_CLN: C2Rust_Unnamed_16 = 15;
pub const HLF_LNB: C2Rust_Unnamed_16 = 14;
pub const HLF_LNA: C2Rust_Unnamed_16 = 13;
pub const HLF_N: C2Rust_Unnamed_16 = 12;
pub const HLF_CM: C2Rust_Unnamed_16 = 11;
pub const HLF_M: C2Rust_Unnamed_16 = 10;
pub const HLF_LC: C2Rust_Unnamed_16 = 9;
pub const HLF_L: C2Rust_Unnamed_16 = 8;
pub const HLF_I: C2Rust_Unnamed_16 = 7;
pub const HLF_E: C2Rust_Unnamed_16 = 6;
pub const HLF_D: C2Rust_Unnamed_16 = 5;
pub const HLF_AT: C2Rust_Unnamed_16 = 4;
pub const HLF_TERM: C2Rust_Unnamed_16 = 3;
pub const HLF_EOB: C2Rust_Unnamed_16 = 2;
pub const HLF_8: C2Rust_Unnamed_16 = 1;
pub const HLF_NONE: C2Rust_Unnamed_16 = 0;
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
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_SPLIT: dobuf_action_values = 1;
pub const DOBUF_GOTO: dobuf_action_values = 0;
pub const DOBUF_MOD: dobuf_start_values = 3;
pub const DOBUF_LAST: dobuf_start_values = 2;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const DOBUF_CURRENT: dobuf_start_values = 0;
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
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const VIM_LAST_TYPE: C2Rust_Unnamed_17 = 4;
pub const VIM_QUESTION: C2Rust_Unnamed_17 = 4;
pub const VIM_INFO: C2Rust_Unnamed_17 = 3;
pub const VIM_WARNING: C2Rust_Unnamed_17 = 2;
pub const VIM_ERROR: C2Rust_Unnamed_17 = 1;
pub const VIM_GENERIC: C2Rust_Unnamed_17 = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const VIM_DISCARDALL: C2Rust_Unnamed_18 = 6;
pub const VIM_ALL: C2Rust_Unnamed_18 = 5;
pub const VIM_CANCEL: C2Rust_Unnamed_18 = 4;
pub const VIM_NO: C2Rust_Unnamed_18 = 3;
pub const VIM_YES: C2Rust_Unnamed_18 = 2;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const CCGD_EXCMD: C2Rust_Unnamed_19 = 16;
pub const CCGD_ALLBUF: C2Rust_Unnamed_19 = 8;
pub const CCGD_FORCEIT: C2Rust_Unnamed_19 = 4;
pub const CCGD_MULTWIN: C2Rust_Unnamed_19 = 2;
pub const CCGD_AW: C2Rust_Unnamed_19 = 1;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_20 = 2;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_20 = 1;
pub const DIP_ALL: C2Rust_Unnamed_21 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const DOCMD_KEEPLINE: C2Rust_Unnamed_20 = 32;
pub const DOCMD_EXCRESET: C2Rust_Unnamed_20 = 16;
pub const DOCMD_KEYTYPED: C2Rust_Unnamed_20 = 8;
pub const DOCMD_REPEAT: C2Rust_Unnamed_20 = 4;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const DIP_DIRFILE: C2Rust_Unnamed_21 = 512;
pub const DIP_AFTER: C2Rust_Unnamed_21 = 128;
pub const DIP_NOAFTER: C2Rust_Unnamed_21 = 64;
pub const DIP_NORTP: C2Rust_Unnamed_21 = 32;
pub const DIP_OPT: C2Rust_Unnamed_21 = 16;
pub const DIP_START: C2Rust_Unnamed_21 = 8;
pub const DIP_ERR: C2Rust_Unnamed_21 = 4;
pub const DIP_DIR: C2Rust_Unnamed_21 = 2;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const BF_SYN_SET: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static e_compiler_not_supported_str: GlobalCell<[::core::ffi::c_char; 33]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
            *b"E666: Compiler not supported: %s\0",
        )
    });
#[no_mangle]
pub unsafe extern "C" fn ex_ruby(mut eap: *mut exarg_T) {
    script_host_execute(
        b"ruby\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ex_rubyfile(mut eap: *mut exarg_T) {
    script_host_execute_file(
        b"ruby\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ex_rubydo(mut eap: *mut exarg_T) {
    script_host_do_range(
        b"ruby\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ex_python3(mut eap: *mut exarg_T) {
    script_host_execute(
        b"python3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ex_py3file(mut eap: *mut exarg_T) {
    script_host_execute_file(
        b"python3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ex_pydo3(mut eap: *mut exarg_T) {
    script_host_do_range(
        b"python3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ex_perl(mut eap: *mut exarg_T) {
    script_host_execute(
        b"perl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ex_perlfile(mut eap: *mut exarg_T) {
    script_host_execute_file(
        b"perl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ex_perldo(mut eap: *mut exarg_T) {
    script_host_do_range(
        b"perl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
#[no_mangle]
pub unsafe extern "C" fn autowrite(mut buf: *mut buf_T, mut forceit: bool) -> ::core::ffi::c_int {
    let mut bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    if !(p_aw.get() != 0 || p_awa.get() != 0)
        || p_write.get() == 0
        || bt_dontwrite(buf) as ::core::ffi::c_int != 0
        || !forceit && (*buf).b_p_ro != 0
        || (*buf).b_ffname.is_null()
    {
        return FAIL;
    }
    set_bufref(&raw mut bufref, buf);
    let mut r: ::core::ffi::c_int = buf_write_all(buf, forceit);
    if bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
        && bufIsChanged(buf) as ::core::ffi::c_int != 0
    {
        r = FAIL;
    }
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn autowrite_all() {
    if !(p_aw.get() != 0 || p_awa.get() != 0) || p_write.get() == 0 {
        return;
    }
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if bufIsChanged(buf) as ::core::ffi::c_int != 0 && (*buf).b_p_ro == 0 && !bt_dontwrite(buf)
        {
            let mut bufref: bufref_T = bufref_T {
                br_buf: ::core::ptr::null_mut::<buf_T>(),
                br_fnum: 0,
                br_buf_free_count: 0,
            };
            set_bufref(&raw mut bufref, buf);
            buf_write_all(buf, false_0 != 0);
            if !bufref_valid(&raw mut bufref) {
                buf = firstbuf.get();
            }
        }
        buf = (*buf).b_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn check_changed(mut buf: *mut buf_T, mut flags: ::core::ffi::c_int) -> bool {
    let mut forceit: bool = flags & CCGD_FORCEIT as ::core::ffi::c_int != 0;
    let mut bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    set_bufref(&raw mut bufref, buf);
    if !forceit
        && bufIsChanged(buf) as ::core::ffi::c_int != 0
        && (flags & CCGD_MULTWIN as ::core::ffi::c_int != 0
            || (*buf).b_nwindows <= 1 as ::core::ffi::c_int)
        && (flags & CCGD_AW as ::core::ffi::c_int == 0 || autowrite(buf, forceit) == FAIL)
    {
        if (p_confirm.get() != 0
            || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0)
            && p_write.get() != 0
        {
            let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if flags & CCGD_ALLBUF as ::core::ffi::c_int != 0 {
                let mut buf2: *mut buf_T = firstbuf.get();
                while !buf2.is_null() {
                    if bufIsChanged(buf2) as ::core::ffi::c_int != 0 && !(*buf2).b_ffname.is_null()
                    {
                        count += 1;
                    }
                    buf2 = (*buf2).b_next;
                }
            }
            if !bufref_valid(&raw mut bufref) {
                return false_0 != 0;
            }
            dialog_changed(buf, count > 1 as ::core::ffi::c_int);
            if !bufref_valid(&raw mut bufref) {
                return false_0 != 0;
            }
            return bufIsChanged(buf);
        }
        if flags & CCGD_EXCMD as ::core::ffi::c_int != 0 {
            no_write_message();
        } else {
            no_write_message_nobang(curbuf.get());
        }
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn dialog_changed(mut buf: *mut buf_T, mut checkall: bool) {
    let mut buff: [::core::ffi::c_char; 1000] = [0; 1000];
    let mut ret: ::core::ffi::c_int = 0;
    let mut ea: exarg_T = exarg {
        arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdidx: CMD_append,
        argt: 0,
        skip: 0,
        forceit: false_0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        do_ecmd_lnum: 0,
        append: false_0,
        usefilter: 0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    dialog_msg(
        &raw mut buff as *mut ::core::ffi::c_char,
        gettext(b"Save changes to \"%s\"?\0".as_ptr() as *const ::core::ffi::c_char),
        (*buf).b_fname,
    );
    if checkall {
        ret = vim_dialog_yesnoallcancel(
            VIM_QUESTION as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            &raw mut buff as *mut ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
    } else {
        ret = vim_dialog_yesnocancel(
            VIM_QUESTION as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            &raw mut buff as *mut ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
    }
    if ret == VIM_YES as ::core::ffi::c_int {
        let mut empty_bufname: bool = (*buf).b_fname.is_null();
        if empty_bufname {
            buf_set_name(
                (*buf).handle as ::core::ffi::c_int,
                b"Untitled\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
        }
        if check_overwrite(
            &raw mut ea,
            buf,
            (*buf).b_fname,
            (*buf).b_ffname,
            false_0 != 0,
        ) == OK
        {
            if buf_write_all(buf, false_0 != 0) == OK {
                return;
            }
        }
        if empty_bufname {
            (*buf).b_fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*buf).b_ffname as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut (*buf).b_sfname as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL;
            let _ = *ptr__0;
        }
    } else if ret == VIM_NO as ::core::ffi::c_int {
        unchanged(buf, true_0 != 0, false_0 != 0);
    } else if ret == VIM_ALL as ::core::ffi::c_int {
        let mut buf2: *mut buf_T = firstbuf.get();
        while !buf2.is_null() {
            if bufIsChanged(buf2) as ::core::ffi::c_int != 0
                && !(*buf2).b_ffname.is_null()
                && (*buf2).b_p_ro == 0
            {
                let mut bufref: bufref_T = bufref_T {
                    br_buf: ::core::ptr::null_mut::<buf_T>(),
                    br_fnum: 0,
                    br_buf_free_count: 0,
                };
                set_bufref(&raw mut bufref, buf2);
                if !(*buf2).b_fname.is_null()
                    && check_overwrite(
                        &raw mut ea,
                        buf2,
                        (*buf2).b_fname,
                        (*buf2).b_ffname,
                        false_0 != 0,
                    ) == OK
                {
                    buf_write_all(buf2, false_0 != 0);
                }
                if !bufref_valid(&raw mut bufref) {
                    buf2 = firstbuf.get();
                }
            }
            buf2 = (*buf2).b_next;
        }
    } else if ret == VIM_DISCARDALL as ::core::ffi::c_int {
        let mut buf2_0: *mut buf_T = firstbuf.get();
        while !buf2_0.is_null() {
            unchanged(buf2_0, true_0 != 0, false_0 != 0);
            buf2_0 = (*buf2_0).b_next;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn dialog_close_terminal(mut buf: *mut buf_T) -> bool {
    let mut buff: [::core::ffi::c_char; 1000] = [0; 1000];
    dialog_msg(
        &raw mut buff as *mut ::core::ffi::c_char,
        gettext(b"Close \"%s\"?\0".as_ptr() as *const ::core::ffi::c_char),
        (if !(*buf).b_fname.is_null() {
            (*buf).b_fname as *const ::core::ffi::c_char
        } else {
            b"?\0".as_ptr() as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char,
    );
    let mut ret: ::core::ffi::c_int = vim_dialog_yesnocancel(
        VIM_QUESTION as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        &raw mut buff as *mut ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    return ret == VIM_YES as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn can_abandon(mut buf: *mut buf_T, mut forceit: bool) -> bool {
    return buf_hide(buf) as ::core::ffi::c_int != 0
        || !bufIsChanged(buf)
        || (*buf).b_nwindows > 1 as ::core::ffi::c_int
        || autowrite(buf, forceit) == OK
        || forceit as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn add_bufnum(
    mut bufnrs: *mut ::core::ffi::c_int,
    mut bufnump: *mut ::core::ffi::c_int,
    mut nr: ::core::ffi::c_int,
) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < *bufnump {
        if *bufnrs.offset(i as isize) == nr {
            return;
        }
        i += 1;
    }
    *bufnrs.offset(*bufnump as isize) = nr;
    *bufnump = *bufnump + 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn check_changed_any(mut hidden: bool, mut unload: bool) -> bool {
    let mut ret: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut bufnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut bufcount: size_t = 0 as size_t;
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        bufcount = bufcount.wrapping_add(1);
        buf = (*buf).b_next;
    }
    if bufcount == 0 as size_t {
        return false_0 != 0;
    }
    let mut bufnrs: *mut ::core::ffi::c_int =
        xmalloc(::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(bufcount))
            as *mut ::core::ffi::c_int;
    let c2rust_fresh0 = bufnum;
    bufnum = bufnum + 1;
    *bufnrs.offset(c2rust_fresh0 as isize) = (*curbuf.get()).handle as ::core::ffi::c_int;
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer != curbuf.get() {
            add_bufnum(
                bufnrs,
                &raw mut bufnum,
                (*(*wp).w_buffer).handle as ::core::ffi::c_int,
            );
        }
        wp = (*wp).w_next;
    }
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        if tp != curtab.get() {
            let mut wp_0: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp_0.is_null() {
                add_bufnum(
                    bufnrs,
                    &raw mut bufnum,
                    (*(*wp_0).w_buffer).handle as ::core::ffi::c_int,
                );
                wp_0 = (*wp_0).w_next;
            }
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    let mut buf_0: *mut buf_T = firstbuf.get();
    while !buf_0.is_null() {
        add_bufnum(
            bufnrs,
            &raw mut bufnum,
            (*buf_0).handle as ::core::ffi::c_int,
        );
        buf_0 = (*buf_0).b_next;
    }
    let mut buf_1: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    i = 0 as ::core::ffi::c_int;
    while i < bufnum {
        buf_1 = buflist_findnr(*bufnrs.offset(i as isize));
        if !buf_1.is_null() {
            if (!hidden || (*buf_1).b_nwindows == 0 as ::core::ffi::c_int)
                && bufIsChanged(buf_1) as ::core::ffi::c_int != 0
            {
                let mut bufref: bufref_T = bufref_T {
                    br_buf: ::core::ptr::null_mut::<buf_T>(),
                    br_fnum: 0,
                    br_buf_free_count: 0,
                };
                set_bufref(&raw mut bufref, buf_1);
                if check_changed(
                    buf_1,
                    (if p_awa.get() != 0 {
                        CCGD_AW as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) | CCGD_MULTWIN as ::core::ffi::c_int
                        | CCGD_ALLBUF as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0
                    && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
                {
                    break;
                }
            }
        }
        i += 1;
    }
    '_theend: {
        if i < bufnum {
            ret = true_0 != 0;
            exiting.set(false_0 != 0);
            if !(p_confirm.get() != 0
                || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0)
            {
                if vgetc_busy.get() > 0 as ::core::ffi::c_int {
                    msg_row.set(cmdline_row.get());
                    msg_col.set(0 as ::core::ffi::c_int);
                    msg_didout.set(false_0 != 0);
                }
                if (if !(*buf_1).terminal.is_null()
                    && channel_job_running((*buf_1).b_p_channel as uint64_t) as ::core::ffi::c_int
                        != 0
                {
                    semsg(
                        gettext(b"E947: Job still running in buffer \"%s\"\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        (*buf_1).b_fname,
                    ) as ::core::ffi::c_int
                } else {
                    semsg(
                        gettext(
                            b"E162: No write since last change for buffer \"%s\"\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        if !buf_spname(buf_1).is_null() {
                            buf_spname(buf_1)
                        } else {
                            (*buf_1).b_fname
                        },
                    ) as ::core::ffi::c_int
                }) != 0
                    && msg_didany.get() as ::core::ffi::c_int != 0
                {
                    let mut save: ::core::ffi::c_int = no_wait_return.get();
                    no_wait_return.set(false_0);
                    wait_return(false_0);
                    no_wait_return.set(save);
                }
            }
            '_buf_found: {
                if buf_1 != curbuf.get() {
                    let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                    loop {
                        if tp_0.is_null() {
                            break '_buf_found;
                        }
                        let mut wp_1: *mut win_T = if tp_0 == curtab.get() {
                            firstwin.get()
                        } else {
                            (*tp_0).tp_firstwin
                        };
                        while !wp_1.is_null() {
                            if (*wp_1).w_buffer == buf_1 {
                                let mut bufref_0: bufref_T = bufref_T {
                                    br_buf: ::core::ptr::null_mut::<buf_T>(),
                                    br_fnum: 0,
                                    br_buf_free_count: 0,
                                };
                                set_bufref(&raw mut bufref_0, buf_1);
                                goto_tabpage_win(tp_0 as *mut tabpage_T, wp_1);
                                if !bufref_valid(&raw mut bufref_0) {
                                    break '_theend;
                                } else {
                                    break '_buf_found;
                                }
                            } else {
                                wp_1 = (*wp_1).w_next;
                            }
                        }
                        tp_0 = (*tp_0).tp_next as *mut tabpage_T;
                    }
                }
            }
            if buf_1 != curbuf.get() {
                set_curbuf(
                    buf_1,
                    if unload as ::core::ffi::c_int != 0 {
                        DOBUF_UNLOAD as ::core::ffi::c_int
                    } else {
                        DOBUF_GOTO as ::core::ffi::c_int
                    },
                    true_0 != 0,
                );
            }
        }
    }
    xfree(bufnrs as *mut ::core::ffi::c_void);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn check_fname() -> ::core::ffi::c_int {
    if (*curbuf.get()).b_ffname.is_null() {
        emsg(gettext(&raw const e_noname as *const ::core::ffi::c_char));
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn buf_write_all(
    mut buf: *mut buf_T,
    mut forceit: bool,
) -> ::core::ffi::c_int {
    let mut old_curbuf: *mut buf_T = curbuf.get();
    let mut retval: ::core::ffi::c_int = buf_write(
        buf,
        (*buf).b_ffname,
        (*buf).b_fname,
        1 as linenr_T,
        (*buf).b_ml.ml_line_count,
        ::core::ptr::null_mut::<exarg_T>(),
        false_0 != 0,
        forceit,
        true_0 != 0,
        false_0 != 0,
    );
    if curbuf.get() != old_curbuf {
        msg_source(HLF_W as ::core::ffi::c_int);
        msg(
            gettext(
                b"Warning: Entered other buffer unexpectedly (check autocommands)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            0 as ::core::ffi::c_int,
        );
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn ex_listdo(mut eap: *mut exarg_T) {
    if (*curwin.get()).w_onebuf_opt.wo_wfb != 0 {
        if ((*eap).cmdidx as ::core::ffi::c_int == CMD_ldo as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_lfdo as ::core::ffi::c_int)
            && (*eap).forceit == 0
        {
            emsg(gettext(
                &raw const e_winfixbuf_cannot_go_to_buffer as *const ::core::ffi::c_char,
            ));
            return;
        }
        if win_valid(prevwin.get()) as ::core::ffi::c_int != 0
            && (*prevwin.get()).w_onebuf_opt.wo_wfb == 0
        {
            win_goto(prevwin.get());
        }
        if (*curwin.get()).w_onebuf_opt.wo_wfb != 0 {
            win_split(0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
            if (*curwin.get()).w_onebuf_opt.wo_wfb != 0 {
                emsg(gettext(
                    &raw const e_winfixbuf_cannot_go_to_buffer as *const ::core::ffi::c_char,
                ));
                return;
            }
        }
    }
    let mut save_ei: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*msg_listdo_overwrite.ptr()) += 1;
    if (*eap).cmdidx as ::core::ffi::c_int != CMD_windo as ::core::ffi::c_int
        && (*eap).cmdidx as ::core::ffi::c_int != CMD_tabdo as ::core::ffi::c_int
    {
        save_ei = au_event_disable(
            b",Syntax\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char
        );
        let mut buf: *mut buf_T = firstbuf.get();
        while !buf.is_null() {
            (*buf).b_flags &= !BF_SYN_SET;
            buf = (*buf).b_next;
        }
    }
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_windo as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_tabdo as ::core::ffi::c_int
        || buf_hide(curbuf.get()) as ::core::ffi::c_int != 0
        || !check_changed(
            curbuf.get(),
            CCGD_AW as ::core::ffi::c_int
                | (if (*eap).forceit != 0 {
                    CCGD_FORCEIT as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
                | CCGD_EXCMD as ::core::ffi::c_int,
        )
    {
        let mut next_fnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut wp: *mut win_T = firstwin.get();
        let mut tp: *mut tabpage_T = first_tabpage.get();
        match (*eap).cmdidx as ::core::ffi::c_int {
            528 => {
                while !wp.is_null() && (i as linenr_T + 1 as linenr_T) < (*eap).line1 {
                    i += 1;
                    wp = (*wp).w_next;
                }
            }
            455 => {
                while !tp.is_null() && (i as linenr_T + 1 as linenr_T) < (*eap).line1 {
                    i += 1;
                    tp = (*tp).tp_next;
                }
            }
            10 => {
                i = (*eap).line1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
            }
            _ => {}
        }
        let mut buf_0: *mut buf_T = curbuf.get();
        let mut qf_size: size_t = 0 as size_t;
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_bufdo as ::core::ffi::c_int {
            buf_0 = firstbuf.get();
            while !buf_0.is_null()
                && (((*buf_0).handle as linenr_T) < (*eap).line1 || (*buf_0).b_p_bl == 0)
            {
                if (*buf_0).handle as linenr_T > (*eap).line2 {
                    buf_0 = ::core::ptr::null_mut::<buf_T>();
                    break;
                } else {
                    buf_0 = (*buf_0).b_next;
                }
            }
            if !buf_0.is_null() {
                goto_buffer(
                    eap,
                    DOBUF_FIRST as ::core::ffi::c_int,
                    FORWARD as ::core::ffi::c_int,
                    (*buf_0).handle as ::core::ffi::c_int,
                );
            }
        } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_cdo as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_ldo as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_cfdo as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_lfdo as ::core::ffi::c_int
        {
            qf_size = qf_get_valid_size(eap);
            '_c2rust_label: {
                if (*eap).line1 >= 0 as linenr_T {
                } else {
                    __assert_fail(
                        b"eap->line1 >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/ex_cmds2.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        550 as ::core::ffi::c_uint,
                        b"void ex_listdo(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            if qf_size == 0 as size_t || (*eap).line1 as size_t > qf_size {
                buf_0 = ::core::ptr::null_mut::<buf_T>();
            } else {
                ex_cc(eap);
                buf_0 = curbuf.get();
                i = (*eap).line1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                if (*eap).addr_count <= 0 as ::core::ffi::c_int {
                    '_c2rust_label_0: {
                        if qf_size < MAXLNUM as ::core::ffi::c_int as size_t {
                        } else {
                            __assert_fail(
                                b"qf_size < MAXLNUM\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/ex_cmds2.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                560 as ::core::ffi::c_uint,
                                b"void ex_listdo(exarg_T *)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    (*eap).line2 = qf_size as linenr_T;
                }
            }
        } else {
            setpcmark();
        }
        listcmd_busy.set(true_0 != 0);
        while !got_int.get() && !buf_0.is_null() {
            let mut execute: bool = true_0 != 0;
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_argdo as ::core::ffi::c_int {
                if i == (*(*curwin.get()).w_alist).al_ga.ga_len {
                    break;
                }
                if (*curwin.get()).w_arg_idx != i || !editing_arg_idx(curwin.get()) {
                    do_argfile(eap, i);
                }
                if (*curwin.get()).w_arg_idx != i {
                    break;
                }
            } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_windo as ::core::ffi::c_int {
                if !win_valid(wp) {
                    break;
                }
                '_c2rust_label_1: {
                    if !wp.is_null() {
                    } else {
                        __assert_fail(
                            b"wp\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/ex_cmds2.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            591 as ::core::ffi::c_uint,
                            b"void ex_listdo(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                execute = !(*wp).w_floating
                    || !(*wp).w_config.hide && (*wp).w_config.focusable as ::core::ffi::c_int != 0;
                if execute {
                    win_goto(wp);
                    if curwin.get() != wp {
                        break;
                    }
                }
                wp = (*wp).w_next;
            } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_tabdo as ::core::ffi::c_int {
                if !valid_tabpage(tp) {
                    break;
                }
                '_c2rust_label_2: {
                    if !tp.is_null() {
                    } else {
                        __assert_fail(
                            b"tp\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/ex_cmds2.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            605 as ::core::ffi::c_uint,
                            b"void ex_listdo(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                goto_tabpage_tp(tp, true_0 != 0, true_0 != 0);
                tp = (*tp).tp_next;
            } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_bufdo as ::core::ffi::c_int {
                next_fnum = -1 as ::core::ffi::c_int;
                let mut bp: *mut buf_T = (*curbuf.get()).b_next;
                while !bp.is_null() {
                    if (*bp).b_p_bl != 0 {
                        next_fnum = (*bp).handle as ::core::ffi::c_int;
                        break;
                    } else {
                        bp = (*bp).b_next;
                    }
                }
            }
            i += 1;
            if execute {
                do_cmdline(
                    (*eap).arg,
                    (*eap).ea_getline,
                    (*eap).cookie,
                    DOCMD_VERBOSE as ::core::ffi::c_int + DOCMD_NOWAIT as ::core::ffi::c_int,
                );
            }
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_bufdo as ::core::ffi::c_int {
                if next_fnum < 0 as ::core::ffi::c_int || next_fnum as linenr_T > (*eap).line2 {
                    break;
                }
                let mut buf_still_exists: bool = false_0 != 0;
                let mut bp_0: *mut buf_T = firstbuf.get();
                while !bp_0.is_null() {
                    if (*bp_0).handle == next_fnum {
                        buf_still_exists = true_0 != 0;
                        break;
                    } else {
                        bp_0 = (*bp_0).b_next;
                    }
                }
                if !buf_still_exists {
                    break;
                }
                goto_buffer(
                    eap,
                    DOBUF_FIRST as ::core::ffi::c_int,
                    FORWARD as ::core::ffi::c_int,
                    next_fnum,
                );
                if (*curbuf.get()).handle != next_fnum {
                    break;
                }
            }
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_cdo as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_ldo as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_cfdo as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_lfdo as ::core::ffi::c_int
            {
                '_c2rust_label_3: {
                    if i >= 0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                            b"i >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/ex_cmds2.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            655 as ::core::ffi::c_uint,
                            b"void ex_listdo(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if i as size_t >= qf_size || i as linenr_T >= (*eap).line2 {
                    break;
                }
                let mut qf_idx: size_t = qf_get_cur_idx(eap);
                ex_cnext(eap);
                if qf_get_cur_idx(eap) == qf_idx {
                    break;
                }
            }
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_windo as ::core::ffi::c_int
                && execute as ::core::ffi::c_int != 0
            {
                validate_cursor(curwin.get());
                if (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
                    do_check_scrollbind(true_0 != 0);
                }
            }
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_windo as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_tabdo as ::core::ffi::c_int
            {
                if i as linenr_T + 1 as linenr_T > (*eap).line2 {
                    break;
                }
            }
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_argdo as ::core::ffi::c_int
                && i as linenr_T >= (*eap).line2
            {
                break;
            }
        }
        listcmd_busy.set(false_0 != 0);
    }
    (*msg_listdo_overwrite.ptr()) -= 1;
    if !save_ei.is_null() {
        let mut bnext: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut aco: aco_save_T = aco_save_T {
            use_aucmd_win_idx: 0,
            save_curwin_handle: 0,
            new_curwin_handle: 0,
            save_prevwin_handle: 0,
            new_curbuf: bufref_T {
                br_buf: ::core::ptr::null_mut::<buf_T>(),
                br_fnum: 0,
                br_buf_free_count: 0,
            },
            tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            save_VIsual_active: false,
            save_prompt_insert: 0,
        };
        au_event_restore(save_ei);
        let mut buf_1: *mut buf_T = firstbuf.get();
        while !buf_1.is_null() {
            bnext = (*buf_1).b_next;
            if (*buf_1).b_nwindows > 0 as ::core::ffi::c_int && (*buf_1).b_flags & BF_SYN_SET != 0 {
                (*buf_1).b_flags &= !BF_SYN_SET;
                if buf_1 == curbuf.get() {
                    apply_autocmds(
                        EVENT_SYNTAX,
                        (*curbuf.get()).b_p_syn,
                        (*curbuf.get()).b_fname,
                        true_0 != 0,
                        curbuf.get(),
                    );
                } else {
                    aucmd_prepbuf(&raw mut aco, buf_1);
                    apply_autocmds(
                        EVENT_SYNTAX,
                        (*buf_1).b_p_syn,
                        (*buf_1).b_fname,
                        true_0 != 0,
                        buf_1,
                    );
                    aucmd_restbuf(&raw mut aco);
                }
                bnext = firstbuf.get();
            }
            buf_1 = bnext;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_compiler(mut eap: *mut exarg_T) {
    let mut old_cur_comp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *(*eap).arg as ::core::ffi::c_int == NUL {
        do_cmdline_cmd(
            b"echo globpath(&rtp, 'compiler/*.vim')\0".as_ptr() as *const ::core::ffi::c_char
        );
        do_cmdline_cmd(
            b"echo globpath(&rtp, 'compiler/*.lua')\0".as_ptr() as *const ::core::ffi::c_char
        );
        return;
    }
    let mut bufsize: size_t = strlen((*eap).arg).wrapping_add(14 as size_t);
    let mut buf: *mut ::core::ffi::c_char = xmalloc(bufsize) as *mut ::core::ffi::c_char;
    if (*eap).forceit != 0 {
        do_cmdline_cmd(
            b"command -nargs=* -keepscript CompilerSet set <args>\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
    } else {
        old_cur_comp =
            get_var_value(b"g:current_compiler\0".as_ptr() as *const ::core::ffi::c_char);
        if !old_cur_comp.is_null() {
            old_cur_comp = xstrdup(old_cur_comp);
        }
        do_cmdline_cmd(
            b"command -nargs=* -keepscript CompilerSet setlocal <args>\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
    }
    do_unlet(
        b"g:current_compiler\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 19]>().wrapping_sub(1 as size_t),
        true_0 != 0,
    );
    do_unlet(
        b"b:current_compiler\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 19]>().wrapping_sub(1 as size_t),
        true_0 != 0,
    );
    snprintf(
        buf,
        bufsize,
        b"compiler/%s.*\0".as_ptr() as *const ::core::ffi::c_char,
        (*eap).arg,
    );
    if source_runtime_vim_lua(buf, DIP_ALL as ::core::ffi::c_int) == FAIL {
        semsg(
            gettext((e_compiler_not_supported_str.ptr() as *const _) as *const ::core::ffi::c_char),
            (*eap).arg,
        );
    }
    xfree(buf as *mut ::core::ffi::c_void);
    do_cmdline_cmd(b":delcommand CompilerSet\0".as_ptr() as *const ::core::ffi::c_char);
    let mut p: *mut ::core::ffi::c_char =
        get_var_value(b"g:current_compiler\0".as_ptr() as *const ::core::ffi::c_char);
    if !p.is_null() {
        set_internal_string_var(
            b"b:current_compiler\0".as_ptr() as *const ::core::ffi::c_char,
            p,
        );
    }
    if (*eap).forceit == 0 {
        if !old_cur_comp.is_null() {
            set_internal_string_var(
                b"g:current_compiler\0".as_ptr() as *const ::core::ffi::c_char,
                old_cur_comp,
            );
            xfree(old_cur_comp as *mut ::core::ffi::c_void);
        } else {
            do_unlet(
                b"g:current_compiler\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 19]>().wrapping_sub(1 as size_t),
                true_0 != 0,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_checktime(mut eap: *mut exarg_T) {
    let mut save_no_check_timestamps: ::core::ffi::c_int = no_check_timestamps.get();
    no_check_timestamps.set(0 as ::core::ffi::c_int);
    if (*eap).addr_count == 0 as ::core::ffi::c_int {
        check_timestamps(false_0);
    } else {
        let mut buf: *mut buf_T = buflist_findnr((*eap).line2 as ::core::ffi::c_int);
        if !buf.is_null() {
            buf_check_timestamp(buf);
        }
    }
    no_check_timestamps.set(save_no_check_timestamps);
}
unsafe extern "C" fn script_host_execute(
    mut name: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
) {
    let mut len: size_t = 0;
    let script: *mut ::core::ffi::c_char = script_get(eap, &raw mut len);
    if !script.is_null() {
        let args: *mut list_T = tv_list_alloc(3 as ptrdiff_t);
        tv_list_append_allocated_string(args, script);
        tv_list_append_number(args, (*eap).line1 as ::core::ffi::c_int as varnumber_T);
        tv_list_append_number(args, (*eap).line2 as ::core::ffi::c_int as varnumber_T);
        eval_call_provider(
            name,
            b"execute\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            args,
            true_0 != 0,
        );
    }
}
unsafe extern "C" fn script_host_execute_file(
    mut name: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
) {
    if (*eap).skip == 0 {
        let mut buffer: [uint8_t; 4096] = [0; 4096];
        vim_FullName(
            (*eap).arg,
            &raw mut buffer as *mut uint8_t as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[uint8_t; 4096]>(),
            false_0 != 0,
        );
        let mut args: *mut list_T = tv_list_alloc(3 as ptrdiff_t);
        tv_list_append_string(
            args,
            &raw mut buffer as *mut uint8_t as *const ::core::ffi::c_char,
            -1 as ssize_t,
        );
        tv_list_append_number(args, (*eap).line1 as ::core::ffi::c_int as varnumber_T);
        tv_list_append_number(args, (*eap).line2 as ::core::ffi::c_int as varnumber_T);
        eval_call_provider(
            name,
            b"execute_file\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            args,
            true_0 != 0,
        );
    }
}
unsafe extern "C" fn script_host_do_range(
    mut name: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
) {
    if (*eap).skip == 0 {
        let mut args: *mut list_T = tv_list_alloc(3 as ptrdiff_t);
        tv_list_append_number(args, (*eap).line1 as ::core::ffi::c_int as varnumber_T);
        tv_list_append_number(args, (*eap).line2 as ::core::ffi::c_int as varnumber_T);
        tv_list_append_string(args, (*eap).arg, -1 as ssize_t);
        eval_call_provider(
            name,
            b"do_range\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            args,
            true_0 != 0,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_drop(mut eap: *mut exarg_T) {
    let mut split: bool = false_0 != 0;
    set_arglist((*eap).arg);
    if (*(*curwin.get()).w_alist).al_ga.ga_len == 0 as ::core::ffi::c_int {
        return;
    }
    if (*cmdmod.ptr()).cmod_tab != 0 {
        ex_all(eap);
        (*cmdmod.ptr()).cmod_tab = 0 as ::core::ffi::c_int;
        ex_rewind(eap);
        return;
    }
    let mut buf: *mut buf_T = buflist_findnr(
        (*((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)
            .offset(0 as ::core::ffi::c_int as isize))
        .ae_fnum,
    );
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf {
                goto_tabpage_win(tp as *mut tabpage_T, wp);
                (*curwin.get()).w_arg_idx = 0 as ::core::ffi::c_int;
                if !bufIsChanged(curbuf.get()) {
                    let save_ar: ::core::ffi::c_int = (*curbuf.get()).b_p_ar;
                    (*curbuf.get()).b_p_ar = true_0;
                    buf_check_timestamp(curbuf.get());
                    (*curbuf.get()).b_p_ar = save_ar;
                }
                if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
                    ex_rewind(eap);
                }
                if !(*eap).do_ecmd_cmd.is_null() {
                    let mut did_set_swapcommand: bool =
                        set_swapcommand((*eap).do_ecmd_cmd, 0 as linenr_T);
                    do_cmdline(
                        (*eap).do_ecmd_cmd,
                        None,
                        NULL,
                        DOCMD_VERBOSE as ::core::ffi::c_int,
                    );
                    if did_set_swapcommand {
                        set_vim_var_string(
                            VV_SWAPCOMMAND,
                            ::core::ptr::null::<::core::ffi::c_char>(),
                            -1 as ptrdiff_t,
                        );
                    }
                }
                return;
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    if !buf_hide(curbuf.get()) {
        (*emsg_off.ptr()) += 1;
        split = check_changed(
            curbuf.get(),
            CCGD_AW as ::core::ffi::c_int | CCGD_EXCMD as ::core::ffi::c_int,
        );
        (*emsg_off.ptr()) -= 1;
    }
    if split {
        (*eap).cmdidx = CMD_sfirst;
        *(*eap).cmd.offset(0 as ::core::ffi::c_int as isize) = 's' as ::core::ffi::c_char;
    } else {
        (*eap).cmdidx = CMD_first;
    }
    ex_rewind(eap);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
