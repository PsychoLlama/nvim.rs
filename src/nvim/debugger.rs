use crate::src::nvim::global_cell::GlobalCell;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, Array, AutoPat, AutoPatCmd, AutoPatCmd_S, BoolVarValue, Boolean,
    BufUpdateCallbacks, CMD_index, Callback, CallbackType, Callback_data as C2Rust_Unnamed_5,
    ChangedtickDictItem, DecorExt, DecorHighlightInline, DecorInlineData, DecorPriority,
    DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, Dict, ExtmarkUndoObject, FileID, Float,
    FloatAnchor, FloatRelative, GridView, Integer, Intersection, KeyValuePair, LineGetter, LuaRef,
    MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, Object, ObjectType, OptInt, ScopeDictDictItem, ScopeType,
    ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, String_0, Terminal, Timestamp, VarLockStatus,
    VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit,
    WinStyle, Window, __time_t, alist_T, auto_event, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T,
    buffblock, buffblock_T, buffheader_T, bufstate_T, chunksize_T, cmd_addr_T, cmdidx_T, colnr_T,
    cstack_T, cstack_T_cs_pend as C2Rust_Unnamed_14, dict_T, dictvar_S, disptick_T, eslist_T,
    eslist_elem, estack_T, estack_T_es_info as C2Rust_Unnamed_15, estack_arg_T, etype_T, event_T,
    exarg, exarg_T, except_T, except_type_T, exprtype_T, extmark_undo_vec_t, fcs_chars_T,
    file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_11, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, key_extra, key_value_pair,
    lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T,
    llpos_T, lpos_T, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T,
    mfdirty_T, msglist, msglist_T, mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed,
    partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, qf_info_S, qf_info_T, queue,
    reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t,
    syn_state, syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T,
    taggy_T, tasave_T, terminal, time_t, typebuf_T, typval_T, typval_vval_union, u_entry,
    u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8,
    u_header_uh_alt_prev as C2Rust_Unnamed_7, u_header_uh_next as C2Rust_Unnamed_10,
    u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, varnumber_T, vim_exception, virt_line, visualinfo_T, win_T, window_S, wininfo_S,
    winopt_T, wline_T, xfmark_T, QUEUE,
};
extern "C" {
    fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn getdigits_int32(pp: *mut *mut ::core::ffi::c_char, strict: bool, def: int32_t) -> int32_t;
    fn redraw_all_later(type_0: ::core::ffi::c_int);
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static e_invarg2: [::core::ffi::c_char; 0];
    static e_noname: [::core::ffi::c_char; 0];
    fn eval_expr(arg: *mut ::core::ffi::c_char, eap: *mut exarg_T) -> *mut typval_T;
    fn typval_compare(
        typ1: *mut typval_T,
        typ2: *mut typval_T,
        type_0: exprtype_T,
        ic: bool,
    ) -> ::core::ffi::c_int;
    fn typval_tostring(arg: *mut typval_T, quotes: bool) -> *mut ::core::ffi::c_char;
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn smsg(hl_id: ::core::ffi::c_int, s: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn msg_starthere();
    fn tv_free(tv: *mut typval_T);
    fn do_cmdline_cmd(cmd: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn do_cmdline(
        cmdline: *mut ::core::ffi::c_char,
        fgetline: LineGetter,
        cookie: *mut ::core::ffi::c_void,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn getcmdline_prompt(
        firstc: ::core::ffi::c_int,
        prompt: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        xp_context: ::core::ffi::c_int,
        xp_arg: *const ::core::ffi::c_char,
        highlight_callback: Callback,
        one_key: bool,
        mouse_used: *mut bool,
    ) -> *mut ::core::ffi::c_char;
    fn getexline(
        c: ::core::ffi::c_int,
        cookie: *mut ::core::ffi::c_void,
        indent: ::core::ffi::c_int,
        do_concat: bool,
    ) -> *mut ::core::ffi::c_char;
    fn ga_clear(gap: *mut garray_T);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn file_pat_to_reg_pat(
        pat: *const ::core::ffi::c_char,
        pat_end: *const ::core::ffi::c_char,
        allow_dirs: *mut ::core::ffi::c_char,
        no_bslash: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn save_typeahead(tp: *mut tasave_T);
    fn restore_typeahead(tp: *mut tasave_T);
    static Rows: GlobalCell<::core::ffi::c_int>;
    static cmdline_row: GlobalCell<::core::ffi::c_int>;
    static msg_row: GlobalCell<::core::ffi::c_int>;
    static msg_scroll: GlobalCell<::core::ffi::c_int>;
    static emsg_off: GlobalCell<::core::ffi::c_int>;
    static did_emsg: GlobalCell<::core::ffi::c_int>;
    static no_wait_return: GlobalCell<::core::ffi::c_int>;
    static need_wait_return: GlobalCell<bool>;
    static lines_left: GlobalCell<::core::ffi::c_int>;
    static ex_nesting_level: GlobalCell<::core::ffi::c_int>;
    static debug_break_level: GlobalCell<::core::ffi::c_int>;
    static debug_did_msg: GlobalCell<bool>;
    static debug_tick: GlobalCell<::core::ffi::c_int>;
    static debug_backtrace_level: GlobalCell<::core::ffi::c_int>;
    static curwin: GlobalCell<*mut win_T>;
    static curbuf: GlobalCell<*mut buf_T>;
    static State: GlobalCell<::core::ffi::c_int>;
    static debug_mode: GlobalCell<bool>;
    static msg_silent: GlobalCell<::core::ffi::c_int>;
    static emsg_silent: GlobalCell<::core::ffi::c_int>;
    static cmd_silent: GlobalCell<bool>;
    static NameBuff: GlobalCell<[::core::ffi::c_char; 4096]>;
    static RedrawingDisabled: GlobalCell<::core::ffi::c_int>;
    static ex_normal_busy: GlobalCell<::core::ffi::c_int>;
    static ignore_script: GlobalCell<bool>;
    static got_int: GlobalCell<bool>;
    static redir_off: GlobalCell<bool>;
    fn expand_env_save(src: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn home_replace(
        buf: *const buf_T,
        src: *const ::core::ffi::c_char,
        dst: *mut ::core::ffi::c_char,
        dstlen: size_t,
        one: bool,
    ) -> size_t;
    fn fix_fname(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static exestack: GlobalCell<garray_T>;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec_prog(
        prog: *mut *mut regprog_T,
        ignore_case: bool,
        line: *const ::core::ffi::c_char,
        col: colnr_T,
    ) -> bool;
    fn estack_sfile(which: estack_arg_T) -> *mut ::core::ffi::c_char;
}
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_13 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_13 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_13 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_13 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_13 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_13 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_13 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_13 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_13 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_13 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_13 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_13 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_13 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_13 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_13 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_13 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_13 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_13 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_13 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_13 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_13 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_13 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_13 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_13 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_13 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_13 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_13 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_13 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_13 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_13 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_13 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_13 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_13 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_13 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_13 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_13 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_13 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_13 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_13 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_13 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_13 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_13 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_13 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_13 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_13 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_13 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_13 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_13 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_13 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_13 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_13 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_13 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_13 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_13 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_13 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_13 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_13 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_13 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_13 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_13 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_13 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_13 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_13 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_13 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_13 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_13 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_13 = -2;
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
pub const UPD_NOT_VALID: C2Rust_Unnamed_16 = 40;
pub const DOCMD_EXCRESET: C2Rust_Unnamed_17 = 16;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_17 = 1;
pub const ESTACK_SCRIPT: estack_arg_T = 3;
pub const ESTACK_STACK: estack_arg_T = 2;
pub const ESTACK_SFILE: estack_arg_T = 1;
pub const ESTACK_NONE: estack_arg_T = 0;
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
pub const MODE_NORMAL: C2Rust_Unnamed_18 = 1;
pub const KE_SNR: key_extra = 82;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct debuggy {
    pub dbg_nr: ::core::ffi::c_int,
    pub dbg_type: ::core::ffi::c_int,
    pub dbg_name: *mut ::core::ffi::c_char,
    pub dbg_prog: *mut regprog_T,
    pub dbg_lnum: linenr_T,
    pub dbg_forceit: ::core::ffi::c_int,
    pub dbg_val: *mut typval_T,
    pub dbg_level: ::core::ffi::c_int,
}
pub const EXPR_ISNOT: exprtype_T = 10;
pub const EXPR_IS: exprtype_T = 9;
pub const EXPR_NOMATCH: exprtype_T = 8;
pub const EXPR_MATCH: exprtype_T = 7;
pub const EXPR_SEQUAL: exprtype_T = 6;
pub const EXPR_SMALLER: exprtype_T = 5;
pub const EXPR_GEQUAL: exprtype_T = 4;
pub const EXPR_GREATER: exprtype_T = 3;
pub const EXPR_NEQUAL: exprtype_T = 2;
pub const EXPR_EQUAL: exprtype_T = 1;
pub const EXPR_UNKNOWN: exprtype_T = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_16 = 50;
pub const UPD_SOME_VALID: C2Rust_Unnamed_16 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_16 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_16 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_16 = 20;
pub const UPD_VALID: C2Rust_Unnamed_16 = 10;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const DOCMD_KEEPLINE: C2Rust_Unnamed_17 = 32;
pub const DOCMD_KEYTYPED: C2Rust_Unnamed_17 = 8;
pub const DOCMD_REPEAT: C2Rust_Unnamed_17 = 4;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_17 = 2;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_18 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_18 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_18 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_18 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_18 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_18 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_18 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_18 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_18 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_18 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_18 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_18 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_18 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_18 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_18 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_18 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_18 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_18 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_18 = 2;
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
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static debug_greedy: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static debug_oldval: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static debug_newval: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
#[no_mangle]
pub unsafe extern "C" fn do_debug(mut cmd: *mut ::core::ffi::c_char) {
    let mut save_msg_scroll: ::core::ffi::c_int = msg_scroll.get();
    let mut save_State: ::core::ffi::c_int = State.get();
    let mut save_did_emsg: ::core::ffi::c_int = did_emsg.get();
    let save_cmd_silent: bool = cmd_silent.get();
    let mut save_msg_silent: ::core::ffi::c_int = msg_silent.get();
    let mut save_emsg_silent: ::core::ffi::c_int = emsg_silent.get();
    let mut save_redir_off: bool = redir_off.get();
    let mut typeaheadbuf: tasave_T = tasave_T {
        save_typebuf: typebuf_T {
            tb_buf: ::core::ptr::null_mut::<uint8_t>(),
            tb_noremap: ::core::ptr::null_mut::<uint8_t>(),
            tb_buflen: 0,
            tb_off: 0,
            tb_len: 0,
            tb_maplen: 0,
            tb_silent: 0,
            tb_no_abbr_cnt: 0,
            tb_change_cnt: 0,
        },
        typebuf_valid: false,
        old_char: 0,
        old_mod_mask: 0,
        save_readbuf1: buffheader_T {
            bh_first: buffblock_T {
                b_next: ::core::ptr::null_mut::<buffblock>(),
                b_strlen: 0,
                b_str: [0; 1],
            },
            bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
            bh_index: 0,
            bh_space: 0,
            bh_create_newblock: false,
        },
        save_readbuf2: buffheader_T {
            bh_first: buffblock_T {
                b_next: ::core::ptr::null_mut::<buffblock>(),
                b_strlen: 0,
                b_str: [0; 1],
            },
            bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
            bh_index: 0,
            bh_space: 0,
            bh_create_newblock: false,
        },
        save_inputbuf: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut typeahead_saved: bool = false_0 != 0;
    let mut save_ignore_script: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cmdline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static last_cmd: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    (*RedrawingDisabled.ptr()) += 1;
    (*no_wait_return.ptr()) += 1;
    did_emsg.set(false_0);
    cmd_silent.set(false_0 != 0);
    msg_silent.set(false_0);
    emsg_silent.set(false_0);
    redir_off.set(true_0 != 0);
    State.set(MODE_NORMAL as ::core::ffi::c_int);
    debug_mode.set(true_0 != 0);
    if !debug_did_msg.get() {
        msg(
            gettext(
                b"Entering Debug mode.  Type \"cont\" to continue.\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            0 as ::core::ffi::c_int,
        );
    }
    if !(*debug_oldval.ptr()).is_null() {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"Oldval = \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
            debug_oldval.get(),
        );
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            debug_oldval.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
    }
    if !(*debug_newval.ptr()).is_null() {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"Newval = \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
            debug_newval.get(),
        );
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            debug_newval.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL;
        let _ = *ptr__0;
    }
    let mut sname: *mut ::core::ffi::c_char = estack_sfile(ESTACK_NONE);
    if !sname.is_null() {
        msg(sname, 0 as ::core::ffi::c_int);
    }
    xfree(sname as *mut ::core::ffi::c_void);
    if (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_lnum
        != 0 as linenr_T
    {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"line %ld: %s\0".as_ptr() as *const ::core::ffi::c_char),
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum as int64_t,
            cmd,
        );
    } else {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"cmd: %s\0".as_ptr() as *const ::core::ffi::c_char),
            cmd,
        );
    }
    loop {
        msg_scroll.set(true_0);
        need_wait_return.set(false_0 != 0);
        let mut save_ex_normal_busy: ::core::ffi::c_int = ex_normal_busy.get();
        ex_normal_busy.set(0 as ::core::ffi::c_int);
        if !debug_greedy.get() {
            save_typeahead(&raw mut typeaheadbuf);
            typeahead_saved = true_0 != 0;
            save_ignore_script = ignore_script.get() as ::core::ffi::c_int;
            ignore_script.set(true_0 != 0);
        }
        let mut n: ::core::ffi::c_int = debug_break_level.get();
        debug_break_level.set(-1 as ::core::ffi::c_int);
        xfree(cmdline as *mut ::core::ffi::c_void);
        cmdline = getcmdline_prompt(
            '>' as ::core::ffi::c_int,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            EXPAND_NOTHING as ::core::ffi::c_int,
            ::core::ptr::null::<::core::ffi::c_char>(),
            Callback {
                data: C2Rust_Unnamed_5 {
                    funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                type_0: kCallbackNone,
            },
            false_0 != 0,
            ::core::ptr::null_mut::<bool>(),
        );
        debug_break_level.set(n);
        if typeahead_saved {
            restore_typeahead(&raw mut typeaheadbuf);
            ignore_script.set(save_ignore_script != 0);
        }
        ex_normal_busy.set(save_ex_normal_busy);
        cmdline_row.set(msg_row.get());
        msg_starthere();
        if !cmdline.is_null() {
            p = skipwhite(cmdline);
            if *p as ::core::ffi::c_int != NUL {
                match *p as ::core::ffi::c_int {
                    99 => {
                        last_cmd.set(CMD_CONT);
                        tail = b"ont\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    110 => {
                        last_cmd.set(CMD_NEXT);
                        tail = b"ext\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    115 => {
                        last_cmd.set(CMD_STEP);
                        tail = b"tep\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    102 => {
                        last_cmd.set(0 as ::core::ffi::c_int);
                        if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'r' as ::core::ffi::c_int
                        {
                            last_cmd.set(CMD_FRAME);
                            tail = b"rame\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            last_cmd.set(CMD_FINISH);
                            tail = b"inish\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                    }
                    113 => {
                        last_cmd.set(CMD_QUIT);
                        tail = b"uit\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    105 => {
                        last_cmd.set(CMD_INTERRUPT);
                        tail = b"nterrupt\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    98 => {
                        last_cmd.set(CMD_BACKTRACE);
                        if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 't' as ::core::ffi::c_int
                        {
                            tail = b"t\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        } else {
                            tail = b"acktrace\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                    }
                    119 => {
                        last_cmd.set(CMD_BACKTRACE);
                        tail = b"here\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    117 => {
                        last_cmd.set(CMD_UP);
                        tail = b"p\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    100 => {
                        last_cmd.set(CMD_DOWN);
                        tail = b"own\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    _ => {
                        last_cmd.set(0 as ::core::ffi::c_int);
                    }
                }
                if last_cmd.get() != 0 as ::core::ffi::c_int {
                    p = p.offset(1);
                    while *p as ::core::ffi::c_int != NUL
                        && *p as ::core::ffi::c_int == *tail as ::core::ffi::c_int
                    {
                        p = p.offset(1);
                        tail = tail.offset(1);
                    }
                    if (*p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                        && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                        || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                            && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
                        && last_cmd.get() != CMD_FRAME
                    {
                        last_cmd.set(0 as ::core::ffi::c_int);
                    }
                }
            }
            if last_cmd.get() != 0 as ::core::ffi::c_int {
                match last_cmd.get() {
                    CMD_CONT => {
                        debug_break_level.set(-1 as ::core::ffi::c_int);
                    }
                    CMD_NEXT => {
                        debug_break_level.set(ex_nesting_level.get());
                    }
                    CMD_STEP => {
                        debug_break_level.set(9999 as ::core::ffi::c_int);
                    }
                    CMD_FINISH => {
                        debug_break_level.set(ex_nesting_level.get() - 1 as ::core::ffi::c_int);
                    }
                    CMD_QUIT => {
                        got_int.set(true_0 != 0);
                        debug_break_level.set(-1 as ::core::ffi::c_int);
                    }
                    CMD_INTERRUPT => {
                        got_int.set(true_0 != 0);
                        debug_break_level.set(9999 as ::core::ffi::c_int);
                        last_cmd.set(CMD_STEP);
                    }
                    CMD_BACKTRACE => {
                        do_showbacktrace(cmd);
                        continue;
                    }
                    CMD_FRAME => {
                        if *p as ::core::ffi::c_int == NUL {
                            do_showbacktrace(cmd);
                        } else {
                            p = skipwhite(p);
                            do_setdebugtracelevel(p);
                        }
                        continue;
                    }
                    CMD_UP => {
                        (*debug_backtrace_level.ptr()) += 1;
                        do_checkbacktracelevel();
                        continue;
                    }
                    CMD_DOWN => {
                        (*debug_backtrace_level.ptr()) -= 1;
                        do_checkbacktracelevel();
                        continue;
                    }
                    _ => {}
                }
                debug_backtrace_level.set(0 as ::core::ffi::c_int);
                break;
            } else {
                n = debug_break_level.get();
                debug_break_level.set(-1 as ::core::ffi::c_int);
                do_cmdline(
                    cmdline,
                    Some(
                        getexline
                            as unsafe extern "C" fn(
                                ::core::ffi::c_int,
                                *mut ::core::ffi::c_void,
                                ::core::ffi::c_int,
                                bool,
                            )
                                -> *mut ::core::ffi::c_char,
                    ),
                    NULL,
                    DOCMD_VERBOSE as ::core::ffi::c_int | DOCMD_EXCRESET as ::core::ffi::c_int,
                );
                debug_break_level.set(n);
            }
        }
        lines_left.set(Rows.get() - 1 as ::core::ffi::c_int);
    }
    xfree(cmdline as *mut ::core::ffi::c_void);
    (*RedrawingDisabled.ptr()) -= 1;
    (*no_wait_return.ptr()) -= 1;
    redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
    need_wait_return.set(false_0 != 0);
    msg_scroll.set(save_msg_scroll);
    lines_left.set(Rows.get() - 1 as ::core::ffi::c_int);
    State.set(save_State);
    debug_mode.set(false_0 != 0);
    did_emsg.set(save_did_emsg);
    cmd_silent.set(save_cmd_silent);
    msg_silent.set(save_msg_silent);
    emsg_silent.set(save_emsg_silent);
    redir_off.set(save_redir_off);
    debug_did_msg.set(true_0 != 0);
}
pub const CMD_CONT: ::core::ffi::c_int = 1;
pub const CMD_NEXT: ::core::ffi::c_int = 2;
pub const CMD_STEP: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const CMD_FINISH: ::core::ffi::c_int = 4;
pub const CMD_QUIT: ::core::ffi::c_int = 5;
pub const CMD_INTERRUPT: ::core::ffi::c_int = 6;
pub const CMD_BACKTRACE: ::core::ffi::c_int = 7;
pub const CMD_FRAME: ::core::ffi::c_int = 8;
pub const CMD_UP: ::core::ffi::c_int = 9;
pub const CMD_DOWN: ::core::ffi::c_int = 10;
unsafe extern "C" fn get_maxbacktrace_level(
    mut sname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut maxbacktrace: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if sname.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let mut p: *mut ::core::ffi::c_char = sname;
    let mut q: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    loop {
        q = strstr(p, b"..\0".as_ptr() as *const ::core::ffi::c_char);
        if q.is_null() {
            break;
        }
        p = q.offset(2 as ::core::ffi::c_int as isize);
        maxbacktrace += 1;
    }
    return maxbacktrace;
}
unsafe extern "C" fn do_setdebugtracelevel(mut arg: *mut ::core::ffi::c_char) {
    let mut level: ::core::ffi::c_int = atoi(arg);
    if *arg as ::core::ffi::c_int == '+' as ::core::ffi::c_int || level < 0 as ::core::ffi::c_int {
        (*debug_backtrace_level.ptr()) += level;
    } else {
        debug_backtrace_level.set(level);
    }
    do_checkbacktracelevel();
}
unsafe extern "C" fn do_checkbacktracelevel() {
    if debug_backtrace_level.get() < 0 as ::core::ffi::c_int {
        debug_backtrace_level.set(0 as ::core::ffi::c_int);
        msg(
            gettext(b"frame is zero\0".as_ptr() as *const ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
    } else {
        let mut sname: *mut ::core::ffi::c_char = estack_sfile(ESTACK_NONE);
        let mut max: ::core::ffi::c_int = get_maxbacktrace_level(sname);
        if debug_backtrace_level.get() > max {
            debug_backtrace_level.set(max);
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"frame at highest level: %d\0".as_ptr() as *const ::core::ffi::c_char),
                max,
            );
        }
        xfree(sname as *mut ::core::ffi::c_void);
    };
}
unsafe extern "C" fn do_showbacktrace(mut cmd: *mut ::core::ffi::c_char) {
    let mut sname: *mut ::core::ffi::c_char = estack_sfile(ESTACK_NONE);
    let mut max: ::core::ffi::c_int = get_maxbacktrace_level(sname);
    if !sname.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut cur: *mut ::core::ffi::c_char = sname;
        while !got_int.get() {
            let mut next: *mut ::core::ffi::c_char =
                strstr(cur, b"..\0".as_ptr() as *const ::core::ffi::c_char);
            if !next.is_null() {
                *next = NUL as ::core::ffi::c_char;
            }
            if i == max - debug_backtrace_level.get() {
                smsg(
                    0 as ::core::ffi::c_int,
                    b"->%d %s\0".as_ptr() as *const ::core::ffi::c_char,
                    max - i,
                    cur,
                );
            } else {
                smsg(
                    0 as ::core::ffi::c_int,
                    b"  %d %s\0".as_ptr() as *const ::core::ffi::c_char,
                    max - i,
                    cur,
                );
            }
            i += 1;
            if next.is_null() {
                break;
            }
            *next = '.' as ::core::ffi::c_char;
            cur = next.offset(2 as ::core::ffi::c_int as isize);
        }
        xfree(sname as *mut ::core::ffi::c_void);
    }
    if (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_lnum
        != 0 as linenr_T
    {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"line %ld: %s\0".as_ptr() as *const ::core::ffi::c_char),
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum as int64_t,
            cmd,
        );
    } else {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"cmd: %s\0".as_ptr() as *const ::core::ffi::c_char),
            cmd,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn ex_debug(mut eap: *mut exarg_T) {
    let mut debug_break_level_save: ::core::ffi::c_int = debug_break_level.get();
    debug_break_level.set(9999 as ::core::ffi::c_int);
    do_cmdline_cmd((*eap).arg);
    debug_break_level.set(debug_break_level_save);
}
static debug_breakpoint_name: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static debug_breakpoint_lnum: GlobalCell<linenr_T> = GlobalCell::new(0);
static debug_skipped: GlobalCell<bool> = GlobalCell::new(false);
static debug_skipped_name: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
#[no_mangle]
pub unsafe extern "C" fn dbg_check_breakpoint(mut eap: *mut exarg_T) {
    debug_skipped.set(false_0 != 0);
    if !(*debug_breakpoint_name.ptr()).is_null() {
        if (*eap).skip == 0 {
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if *(*debug_breakpoint_name.ptr()).offset(0 as ::core::ffi::c_int as isize) as uint8_t
                as ::core::ffi::c_int
                == K_SPECIAL
                && *(*debug_breakpoint_name.ptr()).offset(1 as ::core::ffi::c_int as isize)
                    as uint8_t as ::core::ffi::c_int
                    == KS_EXTRA
                && *(*debug_breakpoint_name.ptr()).offset(2 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == KE_SNR as ::core::ffi::c_int
            {
                p = b"<SNR>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                p = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Breakpoint in \"%s%s\" line %ld\0".as_ptr() as *const ::core::ffi::c_char),
                p,
                (*debug_breakpoint_name.ptr()).offset(
                    (if *p as ::core::ffi::c_int == NUL {
                        0 as ::core::ffi::c_int
                    } else {
                        3 as ::core::ffi::c_int
                    }) as isize,
                ),
                debug_breakpoint_lnum.get() as int64_t,
            );
            debug_breakpoint_name.set(::core::ptr::null_mut::<::core::ffi::c_char>());
            do_debug((*eap).cmd);
        } else {
            debug_skipped.set(true_0 != 0);
            debug_skipped_name.set(debug_breakpoint_name.get());
            debug_breakpoint_name.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        }
    } else if ex_nesting_level.get() <= debug_break_level.get() {
        if (*eap).skip == 0 {
            do_debug((*eap).cmd);
        } else {
            debug_skipped.set(true_0 != 0);
            debug_skipped_name.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn dbg_check_skipped(mut eap: *mut exarg_T) -> bool {
    if !debug_skipped.get() {
        return false_0 != 0;
    }
    let mut prev_got_int: bool = got_int.get();
    got_int.set(false_0 != 0);
    debug_breakpoint_name.set(debug_skipped_name.get());
    (*eap).skip = false_0;
    dbg_check_breakpoint(eap);
    (*eap).skip = true_0;
    got_int.set(got_int.get() as ::core::ffi::c_int | prev_got_int as ::core::ffi::c_int != 0);
    return true_0 != 0;
}
static dbg_breakp: GlobalCell<garray_T> = GlobalCell::new(garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: ::core::mem::size_of::<debuggy>() as ::core::ffi::c_int,
    ga_growsize: 4 as ::core::ffi::c_int,
    ga_data: NULL,
});
static last_breakp: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static has_expr_breakpoint: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static prof_ga: GlobalCell<garray_T> = GlobalCell::new(garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: ::core::mem::size_of::<debuggy>() as ::core::ffi::c_int,
    ga_growsize: 4 as ::core::ffi::c_int,
    ga_data: NULL,
});
pub const DBG_FUNC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DBG_FILE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DBG_EXPR: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
unsafe extern "C" fn eval_expr_no_emsg(bp: *mut debuggy) -> *mut typval_T {
    (*emsg_off.ptr()) += 1;
    let tv: *mut typval_T = eval_expr((*bp).dbg_name, ::core::ptr::null_mut::<exarg_T>());
    (*emsg_off.ptr()) -= 1;
    return tv;
}
unsafe extern "C" fn dbg_parsearg(
    mut arg: *mut ::core::ffi::c_char,
    mut gap: *mut garray_T,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = arg;
    let mut here: bool = false_0 != 0;
    ga_grow(gap, 1 as ::core::ffi::c_int);
    let mut bp: *mut debuggy = ((*gap).ga_data as *mut debuggy).offset((*gap).ga_len as isize);
    if strncmp(
        p,
        b"func\0".as_ptr() as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        (*bp).dbg_type = DBG_FUNC;
    } else if strncmp(
        p,
        b"file\0".as_ptr() as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        (*bp).dbg_type = DBG_FILE;
    } else if gap != prof_ga.ptr()
        && strncmp(
            p,
            b"here\0".as_ptr() as *const ::core::ffi::c_char,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        if (*curbuf.get()).b_ffname.is_null() {
            emsg(gettext(&raw const e_noname as *const ::core::ffi::c_char));
            return FAIL;
        }
        (*bp).dbg_type = DBG_FILE;
        here = true_0 != 0;
    } else if gap != prof_ga.ptr()
        && strncmp(
            p,
            b"expr\0".as_ptr() as *const ::core::ffi::c_char,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        (*bp).dbg_type = DBG_EXPR;
    } else {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            p,
        );
        return FAIL;
    }
    p = skipwhite(p.offset(4 as ::core::ffi::c_int as isize));
    if here {
        (*bp).dbg_lnum = (*curwin.get()).w_cursor.lnum;
    } else if gap != prof_ga.ptr()
        && ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
    {
        (*bp).dbg_lnum = getdigits_int32(&raw mut p, true_0 != 0, 0 as int32_t) as linenr_T;
        p = skipwhite(p);
    } else {
        (*bp).dbg_lnum = 0 as ::core::ffi::c_int as linenr_T;
    }
    if !here && *p as ::core::ffi::c_int == NUL
        || here as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != NUL
        || (*bp).dbg_type == DBG_FUNC
            && !strstr(p, b"()\0".as_ptr() as *const ::core::ffi::c_char).is_null()
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            arg,
        );
        return FAIL;
    }
    if (*bp).dbg_type == DBG_FUNC {
        (*bp).dbg_name = xstrdup(
            if strncmp(
                p,
                b"g:\0".as_ptr() as *const ::core::ffi::c_char,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p.offset(2 as ::core::ffi::c_int as isize)
            } else {
                p
            },
        );
    } else if here {
        (*bp).dbg_name = xstrdup((*curbuf.get()).b_ffname);
    } else if (*bp).dbg_type == DBG_EXPR {
        (*bp).dbg_name = xstrdup(p);
        (*bp).dbg_val = eval_expr_no_emsg(bp);
    } else {
        let mut q: *mut ::core::ffi::c_char = expand_env_save(p);
        if q.is_null() {
            return FAIL;
        }
        p = expand_env_save(q);
        xfree(q as *mut ::core::ffi::c_void);
        if p.is_null() {
            return FAIL;
        }
        if *p as ::core::ffi::c_int != '*' as ::core::ffi::c_int {
            (*bp).dbg_name = fix_fname(p);
            xfree(p as *mut ::core::ffi::c_void);
        } else {
            (*bp).dbg_name = p;
        }
    }
    if (*bp).dbg_name.is_null() {
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn ex_breakadd(mut eap: *mut exarg_T) {
    let mut gap: *mut garray_T = dbg_breakp.ptr();
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_profile as ::core::ffi::c_int {
        gap = prof_ga.ptr();
    }
    if dbg_parsearg((*eap).arg, gap) != OK {
        return;
    }
    let mut bp: *mut debuggy = ((*gap).ga_data as *mut debuggy).offset((*gap).ga_len as isize);
    (*bp).dbg_forceit = (*eap).forceit;
    if (*bp).dbg_type != DBG_EXPR {
        let mut pat: *mut ::core::ffi::c_char = file_pat_to_reg_pat(
            (*bp).dbg_name,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0,
        );
        if !pat.is_null() {
            (*bp).dbg_prog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
            xfree(pat as *mut ::core::ffi::c_void);
        }
        if pat.is_null() || (*bp).dbg_prog.is_null() {
            xfree((*bp).dbg_name as *mut ::core::ffi::c_void);
        } else {
            if (*bp).dbg_lnum == 0 as linenr_T {
                (*bp).dbg_lnum = 1 as ::core::ffi::c_int as linenr_T;
            }
            if (*eap).cmdidx as ::core::ffi::c_int != CMD_profile as ::core::ffi::c_int {
                (*last_breakp.ptr()) += 1;
                (*((*gap).ga_data as *mut debuggy).offset((*gap).ga_len as isize)).dbg_nr =
                    last_breakp.get();
                (*debug_tick.ptr()) += 1;
            }
            (*gap).ga_len += 1;
        }
    } else {
        (*last_breakp.ptr()) += 1;
        let c2rust_fresh0 = (*gap).ga_len;
        (*gap).ga_len = (*gap).ga_len + 1;
        (*((*gap).ga_data as *mut debuggy).offset(c2rust_fresh0 as isize)).dbg_nr =
            last_breakp.get();
        (*debug_tick.ptr()) += 1;
        if gap == dbg_breakp.ptr() {
            has_expr_breakpoint.set(true_0 != 0);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn ex_debuggreedy(mut eap: *mut exarg_T) {
    if (*eap).addr_count == 0 as ::core::ffi::c_int || (*eap).line2 != 0 as linenr_T {
        debug_greedy.set(true_0 != 0);
    } else {
        debug_greedy.set(false_0 != 0);
    };
}
unsafe extern "C" fn update_has_expr_breakpoint() {
    has_expr_breakpoint.set(false_0 != 0);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*dbg_breakp.ptr()).ga_len {
        if (*((*dbg_breakp.ptr()).ga_data as *mut debuggy).offset(i as isize)).dbg_type == DBG_EXPR
        {
            has_expr_breakpoint.set(true_0 != 0);
            break;
        } else {
            i += 1;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_breakdel(mut eap: *mut exarg_T) {
    let mut todel: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut del_all: bool = false_0 != 0;
    let mut best_lnum: linenr_T = 0 as linenr_T;
    let mut gap: *mut garray_T = dbg_breakp.ptr();
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_profdel as ::core::ffi::c_int {
        gap = prof_ga.ptr();
    }
    if ascii_isdigit(*(*eap).arg as ::core::ffi::c_int) {
        let mut nr: ::core::ffi::c_int = atoi((*eap).arg);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*gap).ga_len {
            if (*((*gap).ga_data as *mut debuggy).offset(i as isize)).dbg_nr == nr {
                todel = i;
                break;
            } else {
                i += 1;
            }
        }
    } else if *(*eap).arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
        todel = 0 as ::core::ffi::c_int;
        del_all = true_0 != 0;
    } else {
        if dbg_parsearg((*eap).arg, gap) == FAIL {
            return;
        }
        let mut bp: *mut debuggy = ((*gap).ga_data as *mut debuggy).offset((*gap).ga_len as isize);
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*gap).ga_len {
            let mut bpi: *mut debuggy = ((*gap).ga_data as *mut debuggy).offset(i_0 as isize);
            if (*bp).dbg_type == (*bpi).dbg_type
                && strcmp((*bp).dbg_name, (*bpi).dbg_name) == 0 as ::core::ffi::c_int
                && ((*bp).dbg_lnum == (*bpi).dbg_lnum
                    || (*bp).dbg_lnum == 0 as linenr_T
                        && (best_lnum == 0 as linenr_T || (*bpi).dbg_lnum < best_lnum))
            {
                todel = i_0;
                best_lnum = (*bpi).dbg_lnum;
            }
            i_0 += 1;
        }
        xfree((*bp).dbg_name as *mut ::core::ffi::c_void);
    }
    if todel < 0 as ::core::ffi::c_int {
        semsg(
            gettext(b"E161: Breakpoint not found: %s\0".as_ptr() as *const ::core::ffi::c_char),
            (*eap).arg,
        );
        return;
    }
    while (*gap).ga_len > 0 as ::core::ffi::c_int {
        xfree(
            (*((*gap).ga_data as *mut debuggy).offset(todel as isize)).dbg_name
                as *mut ::core::ffi::c_void,
        );
        if (*((*gap).ga_data as *mut debuggy).offset(todel as isize)).dbg_type == DBG_EXPR
            && !(*((*gap).ga_data as *mut debuggy).offset(todel as isize))
                .dbg_val
                .is_null()
        {
            tv_free((*((*gap).ga_data as *mut debuggy).offset(todel as isize)).dbg_val);
        }
        vim_regfree((*((*gap).ga_data as *mut debuggy).offset(todel as isize)).dbg_prog);
        (*gap).ga_len -= 1;
        if todel < (*gap).ga_len {
            memmove(
                ((*gap).ga_data as *mut debuggy).offset(todel as isize) as *mut ::core::ffi::c_void,
                ((*gap).ga_data as *mut debuggy).offset((todel + 1 as ::core::ffi::c_int) as isize)
                    as *const ::core::ffi::c_void,
                (((*gap).ga_len - todel) as size_t).wrapping_mul(::core::mem::size_of::<debuggy>()),
            );
        }
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_breakdel as ::core::ffi::c_int {
            (*debug_tick.ptr()) += 1;
        }
        if !del_all {
            break;
        }
    }
    if (*gap).ga_len <= 0 as ::core::ffi::c_int {
        ga_clear(gap);
    }
    if gap == dbg_breakp.ptr() {
        update_has_expr_breakpoint();
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_breaklist(mut _eap: *mut exarg_T) {
    if (*dbg_breakp.ptr()).ga_len <= 0 as ::core::ffi::c_int {
        msg(
            gettext(b"No breakpoints defined\0".as_ptr() as *const ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*dbg_breakp.ptr()).ga_len {
        let mut bp: *mut debuggy = ((*dbg_breakp.ptr()).ga_data as *mut debuggy).offset(i as isize);
        if (*bp).dbg_type == DBG_FILE {
            home_replace(
                ::core::ptr::null::<buf_T>(),
                (*bp).dbg_name,
                NameBuff.ptr() as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                true_0 != 0,
            );
        }
        if (*bp).dbg_type != DBG_EXPR {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"%3d  %s %s  line %ld\0".as_ptr() as *const ::core::ffi::c_char),
                (*bp).dbg_nr,
                if (*bp).dbg_type == DBG_FUNC {
                    b"func\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"file\0".as_ptr() as *const ::core::ffi::c_char
                },
                if (*bp).dbg_type == DBG_FUNC {
                    (*bp).dbg_name
                } else {
                    NameBuff.ptr() as *mut ::core::ffi::c_char
                },
                (*bp).dbg_lnum as int64_t,
            );
        } else {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"%3d  expr %s\0".as_ptr() as *const ::core::ffi::c_char),
                (*bp).dbg_nr,
                (*bp).dbg_name,
            );
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn dbg_find_breakpoint(
    mut file: bool,
    mut fname: *mut ::core::ffi::c_char,
    mut after: linenr_T,
) -> linenr_T {
    return debuggy_find(
        file,
        fname,
        after,
        dbg_breakp.ptr(),
        ::core::ptr::null_mut::<bool>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn has_profiling(
    mut file: bool,
    mut fname: *mut ::core::ffi::c_char,
    mut fp: *mut bool,
) -> bool {
    return debuggy_find(file, fname, 0 as linenr_T, prof_ga.ptr(), fp) != 0 as linenr_T;
}
unsafe extern "C" fn debuggy_find(
    mut file: bool,
    mut fname: *mut ::core::ffi::c_char,
    mut after: linenr_T,
    mut gap: *mut garray_T,
    mut fp: *mut bool,
) -> linenr_T {
    let mut bp: *mut debuggy = ::core::ptr::null_mut::<debuggy>();
    let mut lnum: linenr_T = 0 as linenr_T;
    let mut name: *mut ::core::ffi::c_char = fname;
    if (*gap).ga_len <= 0 as ::core::ffi::c_int {
        return 0 as linenr_T;
    }
    if !file
        && *fname.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            == K_SPECIAL
    {
        name = xmalloc(strlen(fname).wrapping_add(3 as size_t)) as *mut ::core::ffi::c_char;
        strcpy(
            name,
            b"<SNR>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        strcpy(
            name.offset(5 as ::core::ffi::c_int as isize),
            fname.offset(3 as ::core::ffi::c_int as isize),
        );
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        bp = ((*gap).ga_data as *mut debuggy).offset(i as isize);
        if ((*bp).dbg_type == DBG_FILE) as ::core::ffi::c_int == file as ::core::ffi::c_int
            && (*bp).dbg_type != DBG_EXPR
            && (gap == prof_ga.ptr()
                || (*bp).dbg_lnum > after && (lnum == 0 as linenr_T || (*bp).dbg_lnum < lnum))
        {
            let mut prev_got_int: bool = got_int.get();
            got_int.set(false_0 != 0);
            if vim_regexec_prog(&raw mut (*bp).dbg_prog, false_0 != 0, name, 0 as colnr_T) {
                lnum = (*bp).dbg_lnum;
                if !fp.is_null() {
                    *fp = (*bp).dbg_forceit != 0;
                }
            }
            got_int
                .set(got_int.get() as ::core::ffi::c_int | prev_got_int as ::core::ffi::c_int != 0);
        } else if (*bp).dbg_type == DBG_EXPR {
            let mut line: bool = false_0 != 0;
            let tv: *mut typval_T = eval_expr_no_emsg(bp);
            if !tv.is_null() {
                if (*bp).dbg_val.is_null() {
                    xfree(debug_oldval.get() as *mut ::core::ffi::c_void);
                    debug_oldval.set(typval_tostring(
                        ::core::ptr::null_mut::<typval_T>(),
                        true_0 != 0,
                    ));
                    (*bp).dbg_val = tv;
                    xfree(debug_newval.get() as *mut ::core::ffi::c_void);
                    debug_newval.set(typval_tostring((*bp).dbg_val, true_0 != 0));
                    line = true_0 != 0;
                } else {
                    if typval_compare(tv, (*bp).dbg_val, EXPR_IS, false_0 != 0) == OK
                        && (*tv).vval.v_number == false_0 as varnumber_T
                    {
                        line = true_0 != 0;
                        xfree(debug_oldval.get() as *mut ::core::ffi::c_void);
                        debug_oldval.set(typval_tostring((*bp).dbg_val, true_0 != 0));
                        let v: *mut typval_T = eval_expr_no_emsg(bp);
                        xfree(debug_newval.get() as *mut ::core::ffi::c_void);
                        debug_newval.set(typval_tostring(v, true_0 != 0));
                        tv_free((*bp).dbg_val);
                        (*bp).dbg_val = v;
                    }
                    tv_free(tv);
                }
            } else if !(*bp).dbg_val.is_null() {
                xfree(debug_oldval.get() as *mut ::core::ffi::c_void);
                debug_oldval.set(typval_tostring((*bp).dbg_val, true_0 != 0));
                xfree(debug_newval.get() as *mut ::core::ffi::c_void);
                debug_newval.set(typval_tostring(
                    ::core::ptr::null_mut::<typval_T>(),
                    true_0 != 0,
                ));
                tv_free((*bp).dbg_val);
                (*bp).dbg_val = ::core::ptr::null_mut::<typval_T>();
                line = true_0 != 0;
            }
            if line {
                lnum = if after > 0 as linenr_T {
                    after
                } else {
                    1 as linenr_T
                };
                break;
            }
        }
        i += 1;
    }
    if name != fname {
        xfree(name as *mut ::core::ffi::c_void);
    }
    return lnum;
}
#[no_mangle]
pub unsafe extern "C" fn dbg_breakpoint(mut name: *mut ::core::ffi::c_char, mut lnum: linenr_T) {
    debug_breakpoint_name.set(name);
    debug_breakpoint_lnum.set(lnum);
}
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RE_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
