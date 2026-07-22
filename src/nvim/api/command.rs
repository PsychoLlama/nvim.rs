use crate::src::nvim::api::private::dispatch::{
    KeyDict_cmd_magic_get_field, KeyDict_cmd_mods_filter_get_field, KeyDict_cmd_mods_get_field,
};
use crate::src::nvim::api::private::helpers::{
    api_dict_to_keydict, api_set_error, api_set_sctx, api_typename, arena_array, arena_dict,
    arena_string, cstr_as_string, cstrn_as_string, find_buffer_by_handle, string_to_cstr,
    try_enter, try_leave,
};
use crate::src::nvim::api::private::validate::{api_err_exp, api_err_invalid, api_err_required};
use crate::src::nvim::autocmd::{apply_autocmds, has_event};
use crate::src::nvim::charset::{skiptowhite, skipwhite};
use crate::src::nvim::ex_docmd::{
    excmd_get_argt, execute_cmd, find_ex_command, get_cmd_default_range, get_command_name,
    getargcmd, getargopt, invalid_range, is_cmd_ni, is_map_cmd, parse_cmdline, replace_makeprg,
    set_cmd_addr_type, set_cmd_count, set_cmd_dflall_range, undo_cmdmod,
};
use crate::src::nvim::ex_eval::aborting;

use crate::src::nvim::lua::executor::{api_free_luaref, api_new_luaref};
use crate::src::nvim::main::{capture_ga, curbuf, current_sctx, msg_col, msg_silent, redir_off};
use crate::src::nvim::mbyte::mb_islower;
use crate::src::nvim::memory::{xcalloc, xfree, xrealloc};
use crate::src::nvim::os::libc::{
    __assert_fail, memcpy, memmove, memset, snprintf, strcmp, strlen, strncmp, strtol,
};
use crate::src::nvim::register::valid_yank_reg;
use crate::src::nvim::strings::kv_do_printf;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, Arena, Array, BoolVarValue, Boolean, BufUpdateCallbacks, Buffer,
    CMD_index, Callback, CallbackType, Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem,
    CmdParseInfo, CmdParseInfo_magic as C2Rust_Unnamed_13, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, Dict,
    Direction, Error, ErrorType, ExtmarkUndoObject, FieldHashfn, FileID, Float, FloatAnchor,
    FloatRelative, GridView, Integer, Intersection, KeyDict_cmd, KeyDict_cmd_opts, KeyDict_empty,
    KeyDict_get_commands, KeyDict_user_command, KeySetLink, KeyValuePair, LineGetter, LuaRef,
    MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, Object, ObjectType, OptInt, OptionalKeys, ScopeDictDictItem,
    ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue,
    StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_12, StringBuilder, String_0,
    Terminal, Timestamp, TryState, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk,
    VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle, Window, __time_t, alist_T, auto_event,
    bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, cmd_addr_T, cmdidx_T,
    cmdmod_T, colnr_T, cstack_T, cstack_T_cs_pend as C2Rust_Unnamed_14, dict_T, dictvar_S,
    disptick_T, eslist_T, eslist_elem, event_T, exarg, exarg_T, except_T, except_type_T, expand_T,
    extmark_undo_vec_t, fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_11, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, key_value_pair, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T,
    msglist, msglist_T, mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed, partial_S,
    partial_T, pos_T, pos_save_T, proftime_T, ptr_t, qf_info_S, qf_info_T, queue, reg_extmatch_T,
    regmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t,
    syn_state, syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T,
    taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header,
    u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8, u_header_uh_alt_prev as C2Rust_Unnamed_7,
    u_header_uh_next as C2Rust_Unnamed_10, u_header_uh_prev as C2Rust_Unnamed_9, ucmd_T, ufunc_S,
    ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t, undo_object, varnumber_T, vim_exception,
    virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, xp_prefix_T,
    QUEUE,
};
use crate::src::nvim::usercmd::{
    commands_array, free_ucmd, get_user_command_name, parse_addr_type_arg, parse_compl_arg,
    uc_add_command, uc_nargs_upper_bound, uc_split_args_iter, uc_validate_name, ucmds,
};
extern "C" {
    fn arena_alloc(arena: *mut Arena, size: size_t, align: bool) -> *mut ::core::ffi::c_void;
    fn arena_memdupz(
        arena: *mut Arena,
        buf: *const ::core::ffi::c_char,
        size: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_cmd_magic {
    pub is_set__cmd_magic_: OptionalKeys,
    pub file: Boolean,
    pub bar: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_cmd_mods {
    pub is_set__cmd_mods_: OptionalKeys,
    pub silent: Boolean,
    pub emsg_silent: Boolean,
    pub unsilent: Boolean,
    pub filter: Dict,
    pub sandbox: Boolean,
    pub noautocmd: Boolean,
    pub browse: Boolean,
    pub confirm: Boolean,
    pub hide: Boolean,
    pub horizontal: Boolean,
    pub keepalt: Boolean,
    pub keepjumps: Boolean,
    pub keepmarks: Boolean,
    pub keeppatterns: Boolean,
    pub lockmarks: Boolean,
    pub noswapfile: Boolean,
    pub tab: Integer,
    pub verbose: Integer,
    pub vertical: Boolean,
    pub split: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_cmd_mods_filter {
    pub is_set__cmd_mods_filter_: OptionalKeys,
    pub pattern: String_0,
    pub force: Boolean,
}
pub const WSP_ABOVE: C2Rust_Unnamed_19 = 128;
pub const WSP_BELOW: C2Rust_Unnamed_19 = 64;
pub const WSP_TOP: C2Rust_Unnamed_19 = 8;
pub const WSP_BOT: C2Rust_Unnamed_19 = 16;
pub const WSP_HOR: C2Rust_Unnamed_19 = 4;
pub const WSP_VERT: C2Rust_Unnamed_19 = 2;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_17 = 8192;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_17 = 2048;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_17 = 4096;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_17 = 512;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_17 = 1024;
pub const CMOD_KEEPALT: C2Rust_Unnamed_17 = 256;
pub const CMOD_HIDE: C2Rust_Unnamed_17 = 32;
pub const CMOD_CONFIRM: C2Rust_Unnamed_17 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_17 = 64;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_17 = 16;
pub const CMOD_SANDBOX: C2Rust_Unnamed_17 = 1;
pub const CMOD_UNSILENT: C2Rust_Unnamed_17 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_17 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_17 = 2;
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
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
pub const NUMBUFLEN: C2Rust_Unnamed_15 = 65;
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
pub const EXPAND_NOTHING: C2Rust_Unnamed_16 = 0;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_16 = 32;
pub const UC_BUFFER: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_16 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_16 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_16 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_16 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_16 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_16 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_16 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_16 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_16 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_16 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_16 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_16 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_16 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_16 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_16 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_16 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_16 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_16 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_16 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_16 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_16 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_16 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_16 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_16 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_16 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_16 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_16 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_16 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_16 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_16 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_16 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_16 = 33;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_16 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_16 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_16 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_16 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_16 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_16 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_16 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_16 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_16 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_16 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_16 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_16 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_16 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_16 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_16 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_16 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_16 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_16 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_16 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_16 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_16 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_16 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_16 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_16 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_16 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_16 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_16 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_16 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_16 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_16 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_16 = 1;
pub const EXPAND_OK: C2Rust_Unnamed_16 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_16 = -2;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const WSP_QUICKFIX: C2Rust_Unnamed_19 = 1024;
pub const WSP_NOENTER: C2Rust_Unnamed_19 = 512;
pub const WSP_NEWLOC: C2Rust_Unnamed_19 = 256;
pub const WSP_HELP: C2Rust_Unnamed_19 = 32;
pub const WSP_ROOM: C2Rust_Unnamed_19 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__addr: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__count: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__force: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__nargs: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__range: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__preview: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__complete: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__cmd: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__reg: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__bang: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__addr: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__mods: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__args: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__count: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__magic: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__nargs: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__range: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__nextcmd: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_magic__bar: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_magic__file: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods__tab: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods__split: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods__filter: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods__verbose: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods_filter__pattern: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
pub const EX_RANGE: ::core::ffi::c_uint = 0x1 as ::core::ffi::c_uint;
pub const EX_BANG: ::core::ffi::c_uint = 0x2 as ::core::ffi::c_uint;
pub const EX_EXTRA: ::core::ffi::c_uint = 0x4 as ::core::ffi::c_uint;
pub const EX_XFILE: ::core::ffi::c_uint = 0x8 as ::core::ffi::c_uint;
pub const EX_NOSPC: ::core::ffi::c_uint = 0x10 as ::core::ffi::c_uint;
pub const EX_DFLALL: ::core::ffi::c_uint = 0x20 as ::core::ffi::c_uint;
pub const EX_NEEDARG: ::core::ffi::c_uint = 0x80 as ::core::ffi::c_uint;
pub const EX_TRLBAR: ::core::ffi::c_uint = 0x100 as ::core::ffi::c_uint;
pub const EX_REGSTR: ::core::ffi::c_uint = 0x200 as ::core::ffi::c_uint;
pub const EX_COUNT: ::core::ffi::c_uint = 0x400 as ::core::ffi::c_uint;
pub const EX_ZEROR: ::core::ffi::c_uint = 0x1000 as ::core::ffi::c_uint;
pub const EX_CMDARG: ::core::ffi::c_uint = 0x4000 as ::core::ffi::c_uint;
pub const EX_ARGOPT: ::core::ffi::c_uint = 0x20000 as ::core::ffi::c_uint;
pub const EX_SBOXOK: ::core::ffi::c_uint = 0x40000 as ::core::ffi::c_uint;
pub const EX_KEEPSCRIPT: ::core::ffi::c_uint = 0x4000000 as ::core::ffi::c_uint;
pub const EX_PREVIEW: ::core::ffi::c_uint = 0x8000000 as ::core::ffi::c_uint;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 66] = unsafe {
    ::core::mem::transmute::<[u8; 66], [::core::ffi::c_char; 66]>(
        *b"void build_cmdline_str(char **, exarg_T *, CmdParseInfo *, Array)\0",
    )
};
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
unsafe extern "C" fn parse_map_cmd(
    mut arg_str: *const ::core::ffi::c_char,
    mut arena: *mut Arena,
) -> Array {
    let mut args: Array = arena_array(arena, 2 as size_t);
    let mut lhs_start: *mut ::core::ffi::c_char = arg_str as *mut ::core::ffi::c_char;
    let mut lhs_end: *mut ::core::ffi::c_char = skiptowhite(lhs_start);
    let mut lhs_len: size_t = lhs_end.offset_from(lhs_start) as size_t;
    let c2rust_fresh28 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh28 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstrn_as_string(lhs_start, lhs_len),
        },
    };
    let mut rhs_start: *mut ::core::ffi::c_char = skipwhite(lhs_end);
    if *rhs_start as ::core::ffi::c_int != NUL {
        let mut rhs_len: size_t = strlen(rhs_start);
        let c2rust_fresh29 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh29 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstrn_as_string(rhs_start, rhs_len),
            },
        };
    }
    return args;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_parse_cmd(
    mut str: String_0,
    mut _opts: *mut KeyDict_empty,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> KeyDict_cmd {
    let mut args: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut length: size_t = 0;
    let mut cmd: *mut ucmd_T = ::core::ptr::null_mut::<ucmd_T>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut nargs: [::core::ffi::c_char; 2] = [0; 2];
    let mut addr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mods: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut filter: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut split: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut magic: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut result: KeyDict_cmd = KeyDict_cmd {
        is_set__cmd_: 0 as OptionalKeys,
        cmd: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        range: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        count: 0,
        reg: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        bang: false,
        args: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        magic: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
        mods: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
        nargs: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        addr: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        nextcmd: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut ea: exarg_T = exarg_T {
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
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
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
    let mut cmdinfo: CmdParseInfo = CmdParseInfo {
        cmdmod: cmdmod_T {
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
        },
        magic: C2Rust_Unnamed_13 {
            file: false,
            bar: false,
        },
    };
    let mut cmdline: *mut ::core::ffi::c_char = arena_memdupz(arena, str.data, str.size);
    let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !parse_cmdline(
        &raw mut cmdline,
        &raw mut ea,
        &raw mut cmdinfo,
        &raw mut errormsg,
    ) {
        if !errormsg.is_null() {
            api_set_error(
                err,
                kErrorTypeException,
                b"Parsing command-line: %s\0".as_ptr() as *const ::core::ffi::c_char,
                errormsg,
            );
        } else {
            api_set_error(
                err,
                kErrorTypeException,
                b"Parsing command-line\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    } else {
        args = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        length = strlen(ea.arg);
        if ea.cmdidx as ::core::ffi::c_int != CMD_SIZE as ::core::ffi::c_int
            && is_map_cmd(ea.cmdidx) as ::core::ffi::c_int != 0
            && *ea.arg as ::core::ffi::c_int != NUL
        {
            args = parse_map_cmd(ea.arg, arena);
        } else if ea.argt & EX_NOSPC as uint32_t != 0 {
            if *ea.arg as ::core::ffi::c_int != NUL {
                args = arena_array(arena, 1 as size_t);
                let c2rust_fresh0 = args.size;
                args.size = args.size.wrapping_add(1);
                *args.items.offset(c2rust_fresh0 as isize) = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstrn_as_string(ea.arg, length),
                    },
                };
            }
        } else {
            let mut end: size_t = 0 as size_t;
            let mut len: size_t = 0 as size_t;
            let mut buf: *mut ::core::ffi::c_char =
                arena_alloc(arena, length.wrapping_add(1 as size_t), false_0 != 0)
                    as *mut ::core::ffi::c_char;
            let mut done: bool = false_0 != 0;
            args = arena_array(arena, uc_nargs_upper_bound(ea.arg, length));
            while !done {
                done = uc_split_args_iter(ea.arg, length, &raw mut end, buf, &raw mut len);
                if len > 0 as size_t {
                    let c2rust_fresh1 = args.size;
                    args.size = args.size.wrapping_add(1);
                    *args.items.offset(c2rust_fresh1 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed {
                            string: cstrn_as_string(buf, len),
                        },
                    };
                    buf = buf.offset(len.wrapping_add(1 as size_t) as isize);
                }
            }
        }
        cmd = ::core::ptr::null_mut::<ucmd_T>();
        if ea.cmdidx as ::core::ffi::c_int == CMD_USER as ::core::ffi::c_int {
            cmd = ((*ucmds.ptr()).ga_data as *mut ucmd_T).offset(ea.useridx as isize);
        } else if ea.cmdidx as ::core::ffi::c_int == CMD_USER_BUF as ::core::ffi::c_int {
            cmd = ((*curbuf.get()).b_ucmds.ga_data as *mut ucmd_T).offset(ea.useridx as isize);
        }
        name = (if ea.cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            (if !cmd.is_null() {
                (*cmd).uc_name
            } else {
                get_command_name(
                    ::core::ptr::null_mut::<expand_T>(),
                    ea.cmdidx as ::core::ffi::c_int,
                )
            }) as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char;
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__cmd)
            as OptionalKeys;
        result.cmd = cstr_as_string(name);
        if ea.argt & EX_RANGE as uint32_t != 0 && ea.addr_count > 0 as ::core::ffi::c_int {
            let mut range: Array = arena_array(arena, 2 as size_t);
            if ea.addr_count > 1 as ::core::ffi::c_int {
                let c2rust_fresh2 = range.size;
                range.size = range.size.wrapping_add(1);
                *range.items.offset(c2rust_fresh2 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: ea.line1 as Integer,
                    },
                };
            }
            let c2rust_fresh3 = range.size;
            range.size = range.size.wrapping_add(1);
            *range.items.offset(c2rust_fresh3 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: ea.line2 as Integer,
                },
            };
            result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__range)
                as OptionalKeys;
            result.range = range;
        }
        if ea.argt & EX_COUNT as uint32_t != 0 {
            let mut count: Integer = if ea.addr_count > 0 as ::core::ffi::c_int {
                ea.line2 as Integer
            } else if !cmd.is_null() {
                (*cmd).uc_def as Integer
            } else {
                0 as Integer
            };
            if ea.addr_count > 0 as ::core::ffi::c_int
                || !cmd.is_null() && (*cmd).uc_def != 0 as int64_t
                || count != 0 as Integer
            {
                result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
                    | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__count)
                    as OptionalKeys;
                result.count = count;
            }
        }
        if ea.argt & EX_REGSTR as uint32_t != 0 {
            let mut reg: [::core::ffi::c_char; 2] = [
                ea.regname as ::core::ffi::c_char,
                NUL as ::core::ffi::c_char,
            ];
            result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__reg)
                as OptionalKeys;
            result.reg = arena_string(
                arena,
                cstr_as_string(&raw mut reg as *mut ::core::ffi::c_char),
            );
        }
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__bang)
            as OptionalKeys;
        result.bang = ea.forceit != 0;
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__args)
            as OptionalKeys;
        result.args = args;
        nargs = [0; 2];
        if ea.argt & EX_EXTRA as uint32_t != 0 {
            if ea.argt & EX_NOSPC as uint32_t != 0 {
                if ea.argt & EX_NEEDARG as uint32_t != 0 {
                    nargs[0 as ::core::ffi::c_int as usize] = '1' as ::core::ffi::c_char;
                } else {
                    nargs[0 as ::core::ffi::c_int as usize] = '?' as ::core::ffi::c_char;
                }
            } else if ea.argt & EX_NEEDARG as uint32_t != 0 {
                nargs[0 as ::core::ffi::c_int as usize] = '+' as ::core::ffi::c_char;
            } else {
                nargs[0 as ::core::ffi::c_int as usize] = '*' as ::core::ffi::c_char;
            }
        } else {
            nargs[0 as ::core::ffi::c_int as usize] = '0' as ::core::ffi::c_char;
        }
        nargs[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__nargs)
            as OptionalKeys;
        result.nargs = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: arena_string(
                    arena,
                    cstr_as_string(&raw mut nargs as *mut ::core::ffi::c_char),
                ),
            },
        };
        addr = ::core::ptr::null_mut::<::core::ffi::c_char>();
        match ea.addr_type as ::core::ffi::c_uint {
            0 => {
                addr = b"line\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            2 => {
                addr = b"arg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            4 => {
                addr = b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            3 => {
                addr = b"load\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            1 => {
                addr = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            5 => {
                addr = b"tab\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            8 => {
                addr = b"qf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            11 => {
                addr = b"none\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            _ => {
                addr = b"?\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
        }
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__addr)
            as OptionalKeys;
        result.addr = cstr_as_string(addr);
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__nextcmd)
            as OptionalKeys;
        result.nextcmd = cstr_as_string(ea.nextcmd);
        mods = arena_dict(arena, 20 as size_t);
        filter = arena_dict(arena, 2 as size_t);
        let c2rust_fresh4 = filter.size;
        filter.size = filter.size.wrapping_add(1);
        *filter.items.offset(c2rust_fresh4 as isize) = key_value_pair {
            key: cstr_as_string(b"pattern\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: arena_string(arena, cstr_as_string(cmdinfo.cmdmod.cmod_filter_pat)),
                },
            },
        };
        let c2rust_fresh5 = filter.size;
        filter.size = filter.size.wrapping_add(1);
        *filter.items.offset(c2rust_fresh5 as isize) = key_value_pair {
            key: cstr_as_string(b"force\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_filter_force,
                },
            },
        };
        let c2rust_fresh6 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh6 as isize) = key_value_pair {
            key: cstr_as_string(b"filter\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: filter },
            },
        };
        let c2rust_fresh7 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh7 as isize) = key_value_pair {
            key: cstr_as_string(b"silent\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_SILENT as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh8 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh8 as isize) = key_value_pair {
            key: cstr_as_string(b"emsg_silent\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_ERRSILENT as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh9 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh9 as isize) = key_value_pair {
            key: cstr_as_string(b"unsilent\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_UNSILENT as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh10 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh10 as isize) = key_value_pair {
            key: cstr_as_string(b"sandbox\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_SANDBOX as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh11 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh11 as isize) = key_value_pair {
            key: cstr_as_string(b"noautocmd\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_NOAUTOCMD as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh12 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh12 as isize) = key_value_pair {
            key: cstr_as_string(b"tab\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (cmdinfo.cmdmod.cmod_tab - 1 as ::core::ffi::c_int) as Integer,
                },
            },
        };
        let c2rust_fresh13 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh13 as isize) = key_value_pair {
            key: cstr_as_string(b"verbose\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (cmdinfo.cmdmod.cmod_verbose - 1 as ::core::ffi::c_int) as Integer,
                },
            },
        };
        let c2rust_fresh14 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh14 as isize) = key_value_pair {
            key: cstr_as_string(b"browse\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_BROWSE as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh15 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh15 as isize) = key_value_pair {
            key: cstr_as_string(b"confirm\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh16 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh16 as isize) = key_value_pair {
            key: cstr_as_string(b"hide\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_HIDE as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh17 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh17 as isize) = key_value_pair {
            key: cstr_as_string(b"keepalt\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh18 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh18 as isize) = key_value_pair {
            key: cstr_as_string(b"keepjumps\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_KEEPJUMPS as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh19 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh19 as isize) = key_value_pair {
            key: cstr_as_string(b"keepmarks\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_KEEPMARKS as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh20 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh20 as isize) = key_value_pair {
            key: cstr_as_string(b"keeppatterns\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_KEEPPATTERNS as ::core::ffi::c_int
                        != 0,
                },
            },
        };
        let c2rust_fresh21 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh21 as isize) = key_value_pair {
            key: cstr_as_string(b"lockmarks\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh22 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh22 as isize) = key_value_pair {
            key: cstr_as_string(b"noswapfile\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_NOSWAPFILE as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh23 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh23 as isize) = key_value_pair {
            key: cstr_as_string(b"vertical\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_split & WSP_VERT as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh24 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh24 as isize) = key_value_pair {
            key: cstr_as_string(b"horizontal\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_split & WSP_HOR as ::core::ffi::c_int != 0,
                },
            },
        };
        split = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if cmdinfo.cmdmod.cmod_split & WSP_BOT as ::core::ffi::c_int != 0 {
            split =
                b"botright\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if cmdinfo.cmdmod.cmod_split & WSP_TOP as ::core::ffi::c_int != 0 {
            split = b"topleft\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if cmdinfo.cmdmod.cmod_split & WSP_BELOW as ::core::ffi::c_int != 0 {
            split =
                b"belowright\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if cmdinfo.cmdmod.cmod_split & WSP_ABOVE as ::core::ffi::c_int != 0 {
            split =
                b"aboveleft\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            split = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        let c2rust_fresh25 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh25 as isize) = key_value_pair {
            key: cstr_as_string(b"split\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(split),
                },
            },
        };
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__mods)
            as OptionalKeys;
        result.mods = mods;
        magic = arena_dict(arena, 2 as size_t);
        let c2rust_fresh26 = magic.size;
        magic.size = magic.size.wrapping_add(1);
        *magic.items.offset(c2rust_fresh26 as isize) = key_value_pair {
            key: cstr_as_string(b"file\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.magic.file,
                },
            },
        };
        let c2rust_fresh27 = magic.size;
        magic.size = magic.size.wrapping_add(1);
        *magic.items.offset(c2rust_fresh27 as isize) = key_value_pair {
            key: cstr_as_string(b"bar\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.magic.bar,
                },
            },
        };
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__magic)
            as OptionalKeys;
        result.magic = magic;
        undo_cmdmod(&raw mut cmdinfo.cmdmod);
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_cmd(
    mut channel_id: uint64_t,
    mut cmd: *mut KeyDict_cmd,
    mut opts: *mut KeyDict_cmd_opts,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> String_0 {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut range_only: bool = false;
    let mut count_from_first_arg: bool = false;
    let mut capture_local: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut save_msg_silent: ::core::ffi::c_int = 0;
    let mut save_redir_off: bool = false;
    let mut save_capture_ga: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
    let mut save_msg_col: ::core::ffi::c_int = 0;
    let mut ea: exarg_T = exarg_T {
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
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
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
    memset(
        &raw mut ea as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<exarg_T>(),
    );
    let mut cmdinfo: CmdParseInfo = CmdParseInfo {
        cmdmod: cmdmod_T {
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
        },
        magic: C2Rust_Unnamed_13 {
            file: false,
            bar: false,
        },
    };
    memset(
        &raw mut cmdinfo as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<CmdParseInfo>(),
    );
    let mut cmdline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cmdname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut retv: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0 as size_t,
    };
    '_end: {
        if !((*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong)
        {
            api_err_required(err, b"cmd\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            if *(*cmd).cmd.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == NUL
            {
                if !((*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 10 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_ulonglong
                    && (*cmd).range.size > 0 as size_t
                    || (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_ulonglong)
                {
                    api_err_exp(
                        err,
                        b"cmd\0".as_ptr() as *const ::core::ffi::c_char,
                        b"non-empty String\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    break '_end;
                }
            }
            cmdname = arena_string(arena, (*cmd).cmd).data;
            ea.cmd = cmdname;
            p = find_ex_command(&raw mut ea, ::core::ptr::null_mut::<::core::ffi::c_int>());
            if !p.is_null()
                && ea.cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int
                && (*ea.cmd as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                    && *ea.cmd as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint)
                && has_event(EVENT_CMDUNDEFINED) as ::core::ffi::c_int != 0
            {
                p = arena_string(arena, (*cmd).cmd).data;
                let mut ret: ::core::ffi::c_int = apply_autocmds(
                    EVENT_CMDUNDEFINED,
                    p,
                    p,
                    true_0 != 0,
                    ::core::ptr::null_mut::<buf_T>(),
                ) as ::core::ffi::c_int;
                p = if ret != 0 && !aborting() {
                    find_ex_command(&raw mut ea, ::core::ptr::null_mut::<::core::ffi::c_int>())
                } else {
                    ea.cmd
                };
            }
            range_only = ea.cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int
                && *(*cmd).cmd.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == NUL
                && (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__range
                    != 0 as ::core::ffi::c_ulonglong
                && (*cmd).range.size > 0 as size_t;
            if !(ea.cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int
                && *(*cmd).cmd.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == NUL
                && (!((*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__range
                    != 0 as ::core::ffi::c_ulonglong)
                    || (*cmd).range.size == 0 as size_t)
                && (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__mods
                    != 0 as ::core::ffi::c_ulonglong)
            {
                if !(!p.is_null()
                    && ea.cmdidx as ::core::ffi::c_int != CMD_SIZE as ::core::ffi::c_int
                    || range_only as ::core::ffi::c_int != 0)
                {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"Command not found: %s\0".as_ptr() as *const ::core::ffi::c_char,
                        cmdname,
                    );
                } else if !(range_only as ::core::ffi::c_int != 0 || !is_cmd_ni(ea.cmdidx)) {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"Command not implemented: %s\0".as_ptr() as *const ::core::ffi::c_char,
                        cmdname,
                    );
                } else {
                    if !range_only {
                        let mut fullname: *const ::core::ffi::c_char =
                            if (ea.cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                                get_user_command_name(ea.useridx, ea.cmdidx as ::core::ffi::c_int)
                            } else {
                                get_command_name(
                                    ::core::ptr::null_mut::<expand_T>(),
                                    ea.cmdidx as ::core::ffi::c_int,
                                )
                            };
                        if !(strncmp(fullname, cmdname, strlen(cmdname)) == 0 as ::core::ffi::c_int)
                        {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Invalid command: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
                                cmdname,
                            );
                            break '_end;
                        }
                    }
                    if range_only {
                        ea.argt = (EX_RANGE | EX_SBOXOK) as uint32_t;
                    } else if !((ea.cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int) {
                        ea.argt = excmd_get_argt(ea.cmdidx);
                    }
                    count_from_first_arg = false_0 != 0;
                    if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__args
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if (*cmd).args.size == 1 as size_t
                            && ea.argt & EX_COUNT as uint32_t != 0
                            && ea.argt & EX_EXTRA as uint32_t == 0
                        {
                            let mut first_arg: Object =
                                *(*cmd).args.items.offset(0 as ::core::ffi::c_int as isize);
                            let mut is_numeric: bool = false_0 != 0;
                            let mut count_value: int64_t = 0 as int64_t;
                            if first_arg.type_0 as ::core::ffi::c_uint
                                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                is_numeric = true_0 != 0;
                                count_value = first_arg.data.integer as int64_t;
                            } else if first_arg.type_0 as ::core::ffi::c_uint
                                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                let mut endptr: *mut ::core::ffi::c_char =
                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                let mut val: ::core::ffi::c_long = strtol(
                                    first_arg.data.string.data,
                                    &raw mut endptr,
                                    10 as ::core::ffi::c_int,
                                );
                                if *endptr as ::core::ffi::c_int == '\0' as ::core::ffi::c_int
                                    && first_arg.data.string.size > 0 as size_t
                                {
                                    is_numeric = true_0 != 0;
                                    count_value = val as int64_t;
                                }
                            }
                            if is_numeric as ::core::ffi::c_int != 0 && count_value >= 0 as int64_t
                            {
                                count_from_first_arg = true_0 != 0;
                                ea.addr_count = 1 as ::core::ffi::c_int;
                                ea.line2 = count_value as linenr_T;
                                ea.line1 = ea.line2;
                                args = arena_array(arena, 0 as size_t);
                            }
                        }
                        if !count_from_first_arg {
                            args = arena_array(arena, (*cmd).args.size);
                            let mut i: size_t = 0 as size_t;
                            while i < (*cmd).args.size {
                                let mut elem: Object = *(*cmd).args.items.offset(i as isize);
                                let mut data_str: *mut ::core::ffi::c_char =
                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                match elem.type_0 as ::core::ffi::c_uint {
                                    1 => {
                                        data_str = arena_alloc(arena, 2 as size_t, false_0 != 0)
                                            as *mut ::core::ffi::c_char;
                                        *data_str.offset(0 as ::core::ffi::c_int as isize) =
                                            (if elem.data.boolean as ::core::ffi::c_int != 0 {
                                                '1' as ::core::ffi::c_int
                                            } else {
                                                '0' as ::core::ffi::c_int
                                            })
                                                as ::core::ffi::c_char;
                                        *data_str.offset(1 as ::core::ffi::c_int as isize) =
                                            NUL as ::core::ffi::c_char;
                                        let c2rust_fresh30 = args.size;
                                        args.size = args.size.wrapping_add(1);
                                        *args.items.offset(c2rust_fresh30 as isize) = object {
                                            type_0: kObjectTypeString,
                                            data: C2Rust_Unnamed {
                                                string: cstr_as_string(data_str),
                                            },
                                        };
                                    }
                                    8 | 9 | 10 | 2 => {
                                        data_str = arena_alloc(
                                            arena,
                                            NUMBUFLEN as ::core::ffi::c_int as size_t,
                                            false_0 != 0,
                                        )
                                            as *mut ::core::ffi::c_char;
                                        snprintf(
                                            data_str,
                                            NUMBUFLEN as ::core::ffi::c_int as size_t,
                                            b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                                            elem.data.integer,
                                        );
                                        let c2rust_fresh31 = args.size;
                                        args.size = args.size.wrapping_add(1);
                                        *args.items.offset(c2rust_fresh31 as isize) = object {
                                            type_0: kObjectTypeString,
                                            data: C2Rust_Unnamed {
                                                string: cstr_as_string(data_str),
                                            },
                                        };
                                    }
                                    4 => {
                                        if string_iswhite(elem.data.string) {
                                            api_err_exp(
                                                err,
                                                b"command arg\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                b"non-whitespace\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                            );
                                            break '_end;
                                        } else {
                                            let c2rust_fresh32 = args.size;
                                            args.size = args.size.wrapping_add(1);
                                            *args.items.offset(c2rust_fresh32 as isize) = elem;
                                        }
                                    }
                                    _ => {
                                        if true {
                                            api_err_exp(
                                                err,
                                                b"command arg\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                b"valid type\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                api_typename(elem.type_0),
                                            );
                                            break '_end;
                                        }
                                    }
                                }
                                i = i.wrapping_add(1);
                            }
                            let mut argc_valid: bool = false;
                            match ea.argt
                                & (EX_EXTRA as uint32_t
                                    | EX_NOSPC as uint32_t
                                    | EX_NEEDARG as uint32_t)
                            {
                                148 => {
                                    argc_valid = args.size == 1 as size_t;
                                }
                                20 => {
                                    argc_valid = args.size <= 1 as size_t;
                                }
                                132 => {
                                    argc_valid = args.size >= 1 as size_t;
                                }
                                EX_EXTRA => {
                                    argc_valid = true_0 != 0;
                                }
                                _ => {
                                    argc_valid = args.size == 0 as size_t;
                                }
                            }
                            if !argc_valid {
                                api_set_error(
                                    err,
                                    kErrorTypeValidation,
                                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"Wrong number of arguments\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                                break '_end;
                            }
                        }
                    }
                    if !range_only {
                        set_cmd_addr_type(
                            &raw mut ea,
                            if args.size > 0 as size_t {
                                (*args.items.offset(0 as ::core::ffi::c_int as isize))
                                    .data
                                    .string
                                    .data
                            } else {
                                ::core::ptr::null_mut::<::core::ffi::c_char>()
                            },
                        );
                    }
                    if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__range
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if ea.argt & 0x1 as uint32_t == 0 {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Command cannot accept %s: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"range\0".as_ptr() as *const ::core::ffi::c_char,
                                (*cmd).cmd.data,
                            );
                            break '_end;
                        } else if !((*cmd).range.size <= 2 as size_t) {
                            api_err_exp(
                                err,
                                b"range\0".as_ptr() as *const ::core::ffi::c_char,
                                b"<=2 elements\0".as_ptr() as *const ::core::ffi::c_char,
                                ::core::ptr::null::<::core::ffi::c_char>(),
                            );
                            break '_end;
                        } else {
                            let mut range: Array = (*cmd).range;
                            ea.addr_count = range.size as ::core::ffi::c_int;
                            let mut i_0: size_t = 0 as size_t;
                            while i_0 < range.size {
                                let mut elem_0: Object = *range.items.offset(i_0 as isize);
                                if !(elem_0.type_0 as ::core::ffi::c_uint
                                    == kObjectTypeInteger as ::core::ffi::c_int
                                        as ::core::ffi::c_uint
                                    && elem_0.data.integer >= 0 as Integer)
                                {
                                    api_err_exp(
                                        err,
                                        b"range element\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"non-negative Integer\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                    );
                                    break '_end;
                                } else {
                                    i_0 = i_0.wrapping_add(1);
                                }
                            }
                            if range.size > 0 as size_t {
                                ea.line1 = (*range.items.offset(0 as ::core::ffi::c_int as isize))
                                    .data
                                    .integer as linenr_T;
                                ea.line2 = (*range
                                    .items
                                    .offset(range.size.wrapping_sub(1 as size_t) as isize))
                                .data
                                .integer as linenr_T;
                            }
                            if !invalid_range(&raw mut ea).is_null() {
                                api_err_invalid(
                                    err,
                                    b"range\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"\0".as_ptr() as *const ::core::ffi::c_char,
                                    0 as int64_t,
                                    true_0 != 0,
                                );
                                break '_end;
                            }
                        }
                    }
                    if ea.addr_count == 0 as ::core::ffi::c_int {
                        if ea.argt & EX_DFLALL as uint32_t != 0 {
                            set_cmd_dflall_range(&raw mut ea);
                        } else {
                            ea.line2 = get_cmd_default_range(&raw mut ea);
                            ea.line1 = ea.line2;
                            if ea.addr_type as ::core::ffi::c_uint
                                == ADDR_OTHER as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                ea.line2 = 1 as ::core::ffi::c_int as linenr_T;
                            }
                        }
                    }
                    if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__count
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if count_from_first_arg {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                b"Cannot specify both 'count' and numeric argument\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                            break '_end;
                        } else if ea.argt & 0x400 as uint32_t == 0 {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Command cannot accept %s: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"count\0".as_ptr() as *const ::core::ffi::c_char,
                                (*cmd).cmd.data,
                            );
                            break '_end;
                        } else if !((*cmd).count >= 0 as Integer) {
                            api_err_exp(
                                err,
                                b"count\0".as_ptr() as *const ::core::ffi::c_char,
                                b"non-negative Integer\0".as_ptr() as *const ::core::ffi::c_char,
                                ::core::ptr::null::<::core::ffi::c_char>(),
                            );
                            break '_end;
                        } else {
                            set_cmd_count(&raw mut ea, (*cmd).count as linenr_T, true_0 != 0);
                        }
                    }
                    if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__reg
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if ea.argt & 0x200 as uint32_t == 0 {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Command cannot accept %s: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"register\0".as_ptr() as *const ::core::ffi::c_char,
                                (*cmd).cmd.data,
                            );
                            break '_end;
                        } else if !((*cmd).reg.size == 1 as size_t) {
                            api_err_exp(
                                err,
                                b"reg\0".as_ptr() as *const ::core::ffi::c_char,
                                b"single character\0".as_ptr() as *const ::core::ffi::c_char,
                                (*cmd).reg.data,
                            );
                            break '_end;
                        } else {
                            let mut regname: ::core::ffi::c_char =
                                *(*cmd).reg.data.offset(0 as ::core::ffi::c_int as isize);
                            if !(regname as ::core::ffi::c_int != '=' as ::core::ffi::c_int) {
                                api_set_error(
                                    err,
                                    kErrorTypeValidation,
                                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"Cannot use register \"=\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                                break '_end;
                            } else if !valid_yank_reg(
                                regname as ::core::ffi::c_int,
                                !((ea.cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                                    && ea.cmdidx as ::core::ffi::c_int
                                        != CMD_put as ::core::ffi::c_int
                                    && ea.cmdidx as ::core::ffi::c_int
                                        != CMD_iput as ::core::ffi::c_int,
                            ) {
                                api_set_error(
                                    err,
                                    kErrorTypeValidation,
                                    b"Invalid register: \"%c\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    regname as ::core::ffi::c_int,
                                );
                                break '_end;
                            } else {
                                ea.regname = regname as uint8_t as ::core::ffi::c_int;
                            }
                        }
                    }
                    ea.forceit = (*cmd).bang as ::core::ffi::c_int;
                    if !(ea.forceit == 0 || ea.argt & 0x2 as uint32_t != 0) {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            b"Command cannot accept %s: %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"bang\0".as_ptr() as *const ::core::ffi::c_char,
                            (*cmd).cmd.data,
                        );
                    } else {
                        if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__magic
                            != 0 as ::core::ffi::c_ulonglong
                        {
                            let mut magic: [KeyDict_cmd_magic; 1] = [KeyDict_cmd_magic {
                                is_set__cmd_magic_: 0 as OptionalKeys,
                                file: false,
                                bar: false,
                            }];
                            if !api_dict_to_keydict(
                                &raw mut magic as *mut KeyDict_cmd_magic
                                    as *mut ::core::ffi::c_void,
                                Some(
                                    KeyDict_cmd_magic_get_field
                                        as unsafe extern "C" fn(
                                            *const ::core::ffi::c_char,
                                            size_t,
                                        )
                                            -> *mut KeySetLink,
                                ),
                                (*cmd).magic,
                                err,
                            ) {
                                break '_end;
                            } else {
                                cmdinfo.magic.file = if (*(&raw mut magic
                                    as *mut KeyDict_cmd_magic))
                                    .is_set__cmd_magic_
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_cmd_magic__file
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*(&raw mut magic as *mut KeyDict_cmd_magic)).file as uint32_t
                                } else {
                                    ea.argt & EX_XFILE as uint32_t
                                } != 0;
                                cmdinfo.magic.bar = if (*(&raw mut magic as *mut KeyDict_cmd_magic))
                                    .is_set__cmd_magic_
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_cmd_magic__bar
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*(&raw mut magic as *mut KeyDict_cmd_magic)).bar as uint32_t
                                } else {
                                    ea.argt & EX_TRLBAR as uint32_t
                                } != 0;
                                if cmdinfo.magic.file {
                                    ea.argt =
                                        (ea.argt as ::core::ffi::c_uint | EX_XFILE) as uint32_t;
                                } else {
                                    ea.argt =
                                        (ea.argt as ::core::ffi::c_uint & !EX_XFILE) as uint32_t;
                                }
                            }
                        } else {
                            cmdinfo.magic.file = ea.argt & EX_XFILE as uint32_t != 0;
                            cmdinfo.magic.bar = ea.argt & EX_TRLBAR as uint32_t != 0;
                        }
                        if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__mods
                            != 0 as ::core::ffi::c_ulonglong
                        {
                            let mut mods: [KeyDict_cmd_mods; 1] = [KeyDict_cmd_mods {
                                is_set__cmd_mods_: 0 as OptionalKeys,
                                silent: false,
                                emsg_silent: false,
                                unsilent: false,
                                filter: Dict {
                                    size: 0,
                                    capacity: 0,
                                    items: ::core::ptr::null_mut::<KeyValuePair>(),
                                },
                                sandbox: false,
                                noautocmd: false,
                                browse: false,
                                confirm: false,
                                hide: false,
                                horizontal: false,
                                keepalt: false,
                                keepjumps: false,
                                keepmarks: false,
                                keeppatterns: false,
                                lockmarks: false,
                                noswapfile: false,
                                tab: 0,
                                verbose: 0,
                                vertical: false,
                                split: String_0 {
                                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                    size: 0,
                                },
                            }];
                            if !api_dict_to_keydict(
                                &raw mut mods as *mut KeyDict_cmd_mods as *mut ::core::ffi::c_void,
                                Some(
                                    KeyDict_cmd_mods_get_field
                                        as unsafe extern "C" fn(
                                            *const ::core::ffi::c_char,
                                            size_t,
                                        )
                                            -> *mut KeySetLink,
                                ),
                                (*cmd).mods,
                                err,
                            ) {
                                break '_end;
                            } else {
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).is_set__cmd_mods_
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_cmd_mods__filter
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    let mut filter: [KeyDict_cmd_mods_filter; 1] =
                                        [KeyDict_cmd_mods_filter {
                                            is_set__cmd_mods_filter_: 0 as OptionalKeys,
                                            pattern: String_0 {
                                                data: ::core::ptr::null_mut::<::core::ffi::c_char>(
                                                ),
                                                size: 0,
                                            },
                                            force: false,
                                        }];
                                    if !api_dict_to_keydict(
                                        &raw mut filter as *mut ::core::ffi::c_void,
                                        Some(
                                            KeyDict_cmd_mods_filter_get_field
                                                as unsafe extern "C" fn(
                                                    *const ::core::ffi::c_char,
                                                    size_t,
                                                )
                                                    -> *mut KeySetLink,
                                        ),
                                        (*(&raw mut mods as *mut KeyDict_cmd_mods)).filter,
                                        err,
                                    ) {
                                        break '_end;
                                    } else if (*(&raw mut filter as *mut KeyDict_cmd_mods_filter))
                                        .is_set__cmd_mods_filter_
                                        as ::core::ffi::c_ulonglong
                                        & (1 as ::core::ffi::c_ulonglong)
                                            << KEYSET_OPTIDX_cmd_mods_filter__pattern
                                        != 0 as ::core::ffi::c_ulonglong
                                    {
                                        cmdinfo.cmdmod.cmod_filter_force = (*(&raw mut filter
                                            as *mut KeyDict_cmd_mods_filter))
                                            .force
                                            as bool;
                                        if *(*(&raw mut filter as *mut KeyDict_cmd_mods_filter))
                                            .pattern
                                            .data
                                            as ::core::ffi::c_int
                                            != NUL
                                            || cmdinfo.cmdmod.cmod_filter_force
                                                as ::core::ffi::c_int
                                                != 0
                                        {
                                            cmdinfo.cmdmod.cmod_filter_pat = string_to_cstr(
                                                (*(&raw mut filter
                                                    as *mut KeyDict_cmd_mods_filter))
                                                    .pattern,
                                            );
                                            cmdinfo.cmdmod.cmod_filter_regmatch.regprog =
                                                vim_regcomp(
                                                    cmdinfo.cmdmod.cmod_filter_pat,
                                                    RE_MAGIC,
                                                );
                                        }
                                    }
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).is_set__cmd_mods_
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd_mods__tab
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    if (*(&raw mut mods as *mut KeyDict_cmd_mods)).tab
                                        as ::core::ffi::c_int
                                        >= 0 as ::core::ffi::c_int
                                    {
                                        cmdinfo.cmdmod.cmod_tab =
                                            (*(&raw mut mods as *mut KeyDict_cmd_mods)).tab
                                                as ::core::ffi::c_int
                                                + 1 as ::core::ffi::c_int;
                                    }
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).is_set__cmd_mods_
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_cmd_mods__verbose
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    if (*(&raw mut mods as *mut KeyDict_cmd_mods)).verbose
                                        as ::core::ffi::c_int
                                        >= 0 as ::core::ffi::c_int
                                    {
                                        cmdinfo.cmdmod.cmod_verbose =
                                            (*(&raw mut mods as *mut KeyDict_cmd_mods)).verbose
                                                as ::core::ffi::c_int
                                                + 1 as ::core::ffi::c_int;
                                    }
                                }
                                cmdinfo.cmdmod.cmod_split |=
                                    if (*(&raw mut mods as *mut KeyDict_cmd_mods)).vertical
                                        as ::core::ffi::c_int
                                        != 0
                                    {
                                        WSP_VERT as ::core::ffi::c_int
                                    } else {
                                        0 as ::core::ffi::c_int
                                    };
                                cmdinfo.cmdmod.cmod_split |=
                                    if (*(&raw mut mods as *mut KeyDict_cmd_mods)).horizontal
                                        as ::core::ffi::c_int
                                        != 0
                                    {
                                        WSP_HOR as ::core::ffi::c_int
                                    } else {
                                        0 as ::core::ffi::c_int
                                    };
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).is_set__cmd_mods_
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_cmd_mods__split
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    if *(*(&raw mut mods as *mut KeyDict_cmd_mods)).split.data
                                        as ::core::ffi::c_int
                                        != NUL
                                    {
                                        if strcmp(
                                            (*(&raw mut mods as *mut KeyDict_cmd_mods)).split.data,
                                            b"aboveleft\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                            || strcmp(
                                                (*(&raw mut mods as *mut KeyDict_cmd_mods))
                                                    .split
                                                    .data,
                                                b"leftabove\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ) == 0 as ::core::ffi::c_int
                                        {
                                            cmdinfo.cmdmod.cmod_split |=
                                                WSP_ABOVE as ::core::ffi::c_int;
                                        } else if strcmp(
                                            (*(&raw mut mods as *mut KeyDict_cmd_mods)).split.data,
                                            b"belowright\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                            || strcmp(
                                                (*(&raw mut mods as *mut KeyDict_cmd_mods))
                                                    .split
                                                    .data,
                                                b"rightbelow\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ) == 0 as ::core::ffi::c_int
                                        {
                                            cmdinfo.cmdmod.cmod_split |=
                                                WSP_BELOW as ::core::ffi::c_int;
                                        } else if strcmp(
                                            (*(&raw mut mods as *mut KeyDict_cmd_mods)).split.data,
                                            b"topleft\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                        {
                                            cmdinfo.cmdmod.cmod_split |=
                                                WSP_TOP as ::core::ffi::c_int;
                                        } else if strcmp(
                                            (*(&raw mut mods as *mut KeyDict_cmd_mods)).split.data,
                                            b"botright\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                        {
                                            cmdinfo.cmdmod.cmod_split |=
                                                WSP_BOT as ::core::ffi::c_int;
                                        } else if true {
                                            api_err_invalid(
                                                err,
                                                b"mods.split\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                b"\0".as_ptr() as *const ::core::ffi::c_char,
                                                0 as int64_t,
                                                true_0 != 0,
                                            );
                                            break '_end;
                                        }
                                    }
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).silent {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_SILENT as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).emsg_silent {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_ERRSILENT as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).unsilent {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_UNSILENT as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).sandbox {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_SANDBOX as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).noautocmd {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_NOAUTOCMD as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).browse {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_BROWSE as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).confirm {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_CONFIRM as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).hide {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_HIDE as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).keepalt {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_KEEPALT as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).keepjumps {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_KEEPJUMPS as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).keepmarks {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_KEEPMARKS as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).keeppatterns {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_KEEPPATTERNS as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).lockmarks {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_LOCKMARKS as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).noswapfile {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_NOSWAPFILE as ::core::ffi::c_int;
                                }
                                if cmdinfo.cmdmod.cmod_flags & CMOD_ERRSILENT as ::core::ffi::c_int
                                    != 0
                                {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_SILENT as ::core::ffi::c_int;
                                }
                                if cmdinfo.cmdmod.cmod_flags & CMOD_SANDBOX as ::core::ffi::c_int
                                    != 0
                                    && ea.argt & 0x40000 as uint32_t == 0
                                {
                                    api_set_error(
                                        err,
                                        kErrorTypeValidation,
                                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"Command cannot be run in sandbox\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                    );
                                    break '_end;
                                }
                            }
                        }
                        build_cmdline_str(&raw mut cmdline, &raw mut ea, &raw mut cmdinfo, args);
                        ea.cmdlinep = &raw mut cmdline;
                        's_1442: {
                            if ea.argt & EX_ARGOPT as uint32_t != 0 {
                                loop {
                                    if !(*ea.arg.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == '+' as ::core::ffi::c_int
                                        && *ea.arg.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '+' as ::core::ffi::c_int)
                                    {
                                        break 's_1442;
                                    }
                                    let mut orig_arg: *mut ::core::ffi::c_char = ea.arg;
                                    let mut result: ::core::ffi::c_int = getargopt(&raw mut ea);
                                    if result != 0 as ::core::ffi::c_int
                                        || is_cmd_ni(ea.cmdidx) as ::core::ffi::c_int != 0
                                    {
                                        continue;
                                    }
                                    api_err_invalid(
                                        err,
                                        b"argument \0".as_ptr() as *const ::core::ffi::c_char,
                                        orig_arg,
                                        0 as int64_t,
                                        true_0 != 0,
                                    );
                                    break '_end;
                                }
                            }
                        }
                        if ea.argt & EX_CMDARG as uint32_t != 0 && ea.usefilter == 0 {
                            ea.do_ecmd_cmd = getargcmd(&raw mut ea.arg);
                        }
                        capture_local = garray_T {
                            ga_len: 0,
                            ga_maxlen: 0,
                            ga_itemsize: 0,
                            ga_growsize: 0,
                            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        };
                        save_msg_silent = msg_silent.get();
                        save_redir_off = redir_off.get();
                        save_capture_ga = capture_ga.get();
                        save_msg_col = msg_col.get();
                        if (*opts).output {
                            ga_init(
                                &raw mut capture_local,
                                1 as ::core::ffi::c_int,
                                80 as ::core::ffi::c_int,
                            );
                            capture_ga.set(&raw mut capture_local);
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
                        if (*opts).output {
                            (*msg_silent.ptr()) += 1;
                            redir_off.set(false);
                            msg_col.set(0 as ::core::ffi::c_int);
                        }
                        let save_current_sctx: sctx_T = api_set_sctx(channel_id);
                        execute_cmd(&raw mut ea, &raw mut cmdinfo, false);
                        current_sctx.set(save_current_sctx);
                        if (*opts).output {
                            capture_ga.set(save_capture_ga);
                            msg_silent.set(save_msg_silent);
                            redir_off.set(save_redir_off);
                            msg_col.set(save_msg_col);
                        }
                        try_leave(&raw mut tstate, err);
                        if (*err).type_0 as ::core::ffi::c_int
                            == kErrorTypeNone as ::core::ffi::c_int
                        {
                            if (*opts).output as ::core::ffi::c_int != 0
                                && capture_local.ga_len > 1 as ::core::ffi::c_int
                            {
                                retv = arena_string(
                                    arena,
                                    String_0 {
                                        data: capture_local.ga_data as *mut ::core::ffi::c_char,
                                        size: capture_local.ga_len as size_t,
                                    },
                                );
                                if *retv.data.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == '\n' as ::core::ffi::c_int
                                {
                                    retv.data = retv.data.offset(1);
                                    retv.size = retv.size.wrapping_sub(1);
                                }
                            }
                        }
                        if (*opts).output {
                            ga_clear(&raw mut capture_local);
                        }
                    }
                }
            }
        }
    }
    xfree(cmdline as *mut ::core::ffi::c_void);
    xfree(ea.args as *mut ::core::ffi::c_void);
    xfree(ea.arglens as *mut ::core::ffi::c_void);
    return retv;
}
unsafe extern "C" fn string_iswhite(mut str: String_0) -> bool {
    let mut i: size_t = 0 as size_t;
    while i < str.size {
        if !ascii_iswhite(*str.data.offset(i as isize) as ::core::ffi::c_int) {
            return false_0 != 0;
        } else {
            if *str.data.offset(i as isize) as ::core::ffi::c_int == NUL {
                break;
            }
            i = i.wrapping_add(1);
        }
    }
    return true_0 != 0;
}
unsafe extern "C" fn build_cmdline_str(
    mut cmdlinep: *mut *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    mut cmdinfo: *mut CmdParseInfo,
    mut args: Array,
) {
    let mut argc: size_t = args.size;
    let mut cmdline: StringBuilder = StringBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    cmdline.capacity = 32 as size_t;
    cmdline.items = xrealloc(
        cmdline.items as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
    ) as *mut ::core::ffi::c_char;
    if (*cmdinfo).cmdmod.cmod_tab != 0 as ::core::ffi::c_int {
        kv_do_printf(
            &raw mut cmdline,
            b"%dtab \0".as_ptr() as *const ::core::ffi::c_char,
            (*cmdinfo).cmdmod.cmod_tab - 1 as ::core::ffi::c_int,
        );
    }
    if (*cmdinfo).cmdmod.cmod_verbose > 0 as ::core::ffi::c_int {
        kv_do_printf(
            &raw mut cmdline,
            b"%dverbose \0".as_ptr() as *const ::core::ffi::c_char,
            (*cmdinfo).cmdmod.cmod_verbose - 1 as ::core::ffi::c_int,
        );
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_ERRSILENT as ::core::ffi::c_int != 0 {
        if strlen(b"silent! \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(strlen(b"silent! \0".as_ptr() as *const ::core::ffi::c_char))
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(strlen(b"silent! \0".as_ptr() as *const ::core::ffi::c_char));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        852 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"silent! \0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(strlen(b"silent! \0".as_ptr() as *const ::core::ffi::c_char)),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(strlen(b"silent! \0".as_ptr() as *const ::core::ffi::c_char));
        }
    } else if (*cmdinfo).cmdmod.cmod_flags & CMOD_SILENT as ::core::ffi::c_int != 0 {
        if strlen(b"silent \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(strlen(b"silent \0".as_ptr() as *const ::core::ffi::c_char))
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(strlen(b"silent \0".as_ptr() as *const ::core::ffi::c_char));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_0: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        854 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"silent \0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(strlen(b"silent \0".as_ptr() as *const ::core::ffi::c_char)),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(strlen(b"silent \0".as_ptr() as *const ::core::ffi::c_char));
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_UNSILENT as ::core::ffi::c_int != 0 {
        if strlen(b"unsilent \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(strlen(b"unsilent \0".as_ptr() as *const ::core::ffi::c_char))
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(strlen(b"unsilent \0".as_ptr() as *const ::core::ffi::c_char));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_1: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        858 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"unsilent \0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(strlen(b"unsilent \0".as_ptr() as *const ::core::ffi::c_char)),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(strlen(b"unsilent \0".as_ptr() as *const ::core::ffi::c_char));
        }
    }
    match (*cmdinfo).cmdmod.cmod_split
        & (WSP_ABOVE as ::core::ffi::c_int
            | WSP_BELOW as ::core::ffi::c_int
            | WSP_TOP as ::core::ffi::c_int
            | WSP_BOT as ::core::ffi::c_int)
    {
        128 => {
            if strlen(b"aboveleft \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
                if cmdline.capacity
                    < cmdline.size.wrapping_add(strlen(
                        b"aboveleft \0".as_ptr() as *const ::core::ffi::c_char
                    ))
                {
                    cmdline.capacity = cmdline.size.wrapping_add(strlen(
                        b"aboveleft \0".as_ptr() as *const ::core::ffi::c_char
                    ));
                    cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                    cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                    cmdline.capacity = cmdline.capacity.wrapping_add(1);
                    cmdline.capacity = cmdline.capacity;
                    cmdline.items = xrealloc(
                        cmdline.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_char>()
                            .wrapping_mul(cmdline.capacity),
                    ) as *mut ::core::ffi::c_char;
                }
                '_c2rust_label_2: {
                    if !cmdline.items.is_null() {
                    } else {
                        __assert_fail(
                            b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            863 as ::core::ffi::c_uint,
                            __ASSERT_FUNCTION.as_ptr(),
                        );
                    }
                };
                memcpy(
                    cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                    b"aboveleft \0".as_ptr() as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(strlen(
                        b"aboveleft \0".as_ptr() as *const ::core::ffi::c_char,
                    )),
                );
                cmdline.size = cmdline.size.wrapping_add(strlen(
                    b"aboveleft \0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
        }
        64 => {
            if strlen(b"belowright \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
                if cmdline.capacity
                    < cmdline.size.wrapping_add(strlen(
                        b"belowright \0".as_ptr() as *const ::core::ffi::c_char
                    ))
                {
                    cmdline.capacity = cmdline.size.wrapping_add(strlen(
                        b"belowright \0".as_ptr() as *const ::core::ffi::c_char
                    ));
                    cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                    cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                    cmdline.capacity = cmdline.capacity.wrapping_add(1);
                    cmdline.capacity = cmdline.capacity;
                    cmdline.items = xrealloc(
                        cmdline.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_char>()
                            .wrapping_mul(cmdline.capacity),
                    ) as *mut ::core::ffi::c_char;
                }
                '_c2rust_label_3: {
                    if !cmdline.items.is_null() {
                    } else {
                        __assert_fail(
                            b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            866 as ::core::ffi::c_uint,
                            __ASSERT_FUNCTION.as_ptr(),
                        );
                    }
                };
                memcpy(
                    cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                    b"belowright \0".as_ptr() as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(strlen(
                        b"belowright \0".as_ptr() as *const ::core::ffi::c_char,
                    )),
                );
                cmdline.size = cmdline.size.wrapping_add(strlen(
                    b"belowright \0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
        }
        8 => {
            if strlen(b"topleft \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
                if cmdline.capacity
                    < cmdline
                        .size
                        .wrapping_add(strlen(b"topleft \0".as_ptr() as *const ::core::ffi::c_char))
                {
                    cmdline.capacity = cmdline
                        .size
                        .wrapping_add(strlen(b"topleft \0".as_ptr() as *const ::core::ffi::c_char));
                    cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                    cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                    cmdline.capacity = cmdline.capacity.wrapping_add(1);
                    cmdline.capacity = cmdline.capacity;
                    cmdline.items = xrealloc(
                        cmdline.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_char>()
                            .wrapping_mul(cmdline.capacity),
                    ) as *mut ::core::ffi::c_char;
                }
                '_c2rust_label_4: {
                    if !cmdline.items.is_null() {
                    } else {
                        __assert_fail(
                            b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            869 as ::core::ffi::c_uint,
                            __ASSERT_FUNCTION.as_ptr(),
                        );
                    }
                };
                memcpy(
                    cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                    b"topleft \0".as_ptr() as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(strlen(b"topleft \0".as_ptr() as *const ::core::ffi::c_char)),
                );
                cmdline.size = cmdline
                    .size
                    .wrapping_add(strlen(b"topleft \0".as_ptr() as *const ::core::ffi::c_char));
            }
        }
        16 => {
            if strlen(b"botright \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
                if cmdline.capacity
                    < cmdline
                        .size
                        .wrapping_add(strlen(b"botright \0".as_ptr() as *const ::core::ffi::c_char))
                {
                    cmdline.capacity = cmdline.size.wrapping_add(strlen(
                        b"botright \0".as_ptr() as *const ::core::ffi::c_char
                    ));
                    cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                    cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                    cmdline.capacity = cmdline.capacity.wrapping_add(1);
                    cmdline.capacity = cmdline.capacity;
                    cmdline.items = xrealloc(
                        cmdline.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_char>()
                            .wrapping_mul(cmdline.capacity),
                    ) as *mut ::core::ffi::c_char;
                }
                '_c2rust_label_5: {
                    if !cmdline.items.is_null() {
                    } else {
                        __assert_fail(
                            b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            872 as ::core::ffi::c_uint,
                            __ASSERT_FUNCTION.as_ptr(),
                        );
                    }
                };
                memcpy(
                    cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                    b"botright \0".as_ptr() as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(strlen(
                        b"botright \0".as_ptr() as *const ::core::ffi::c_char,
                    )),
                );
                cmdline.size = cmdline
                    .size
                    .wrapping_add(strlen(b"botright \0".as_ptr() as *const ::core::ffi::c_char));
            }
        }
        _ => {}
    }
    if (*cmdinfo).cmdmod.cmod_split & WSP_VERT as ::core::ffi::c_int != 0 {
        if strlen(b"vertical \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(strlen(b"vertical \0".as_ptr() as *const ::core::ffi::c_char))
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(strlen(b"vertical \0".as_ptr() as *const ::core::ffi::c_char));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_6: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        885 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"vertical \0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(strlen(b"vertical \0".as_ptr() as *const ::core::ffi::c_char)),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(strlen(b"vertical \0".as_ptr() as *const ::core::ffi::c_char));
        }
    }
    if (*cmdinfo).cmdmod.cmod_split & WSP_HOR as ::core::ffi::c_int != 0 {
        if strlen(b"horizontal \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline.size.wrapping_add(strlen(
                    b"horizontal \0".as_ptr() as *const ::core::ffi::c_char
                ))
            {
                cmdline.capacity = cmdline.size.wrapping_add(strlen(
                    b"horizontal \0".as_ptr() as *const ::core::ffi::c_char
                ));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_7: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        886 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"horizontal \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(strlen(
                    b"horizontal \0".as_ptr() as *const ::core::ffi::c_char,
                )),
            );
            cmdline.size = cmdline.size.wrapping_add(strlen(
                b"horizontal \0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_SANDBOX as ::core::ffi::c_int != 0 {
        if strlen(b"sandbox \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(strlen(b"sandbox \0".as_ptr() as *const ::core::ffi::c_char))
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(strlen(b"sandbox \0".as_ptr() as *const ::core::ffi::c_char));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_8: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        887 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"sandbox \0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(strlen(b"sandbox \0".as_ptr() as *const ::core::ffi::c_char)),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(strlen(b"sandbox \0".as_ptr() as *const ::core::ffi::c_char));
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_NOAUTOCMD as ::core::ffi::c_int != 0 {
        if strlen(b"noautocmd \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline.size.wrapping_add(strlen(
                    b"noautocmd \0".as_ptr() as *const ::core::ffi::c_char
                ))
            {
                cmdline.capacity = cmdline.size.wrapping_add(strlen(
                    b"noautocmd \0".as_ptr() as *const ::core::ffi::c_char
                ));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_9: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        888 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"noautocmd \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(strlen(
                    b"noautocmd \0".as_ptr() as *const ::core::ffi::c_char,
                )),
            );
            cmdline.size = cmdline.size.wrapping_add(strlen(
                b"noautocmd \0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_BROWSE as ::core::ffi::c_int != 0 {
        if strlen(b"browse \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(strlen(b"browse \0".as_ptr() as *const ::core::ffi::c_char))
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(strlen(b"browse \0".as_ptr() as *const ::core::ffi::c_char));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_10: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        889 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"browse \0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(strlen(b"browse \0".as_ptr() as *const ::core::ffi::c_char)),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(strlen(b"browse \0".as_ptr() as *const ::core::ffi::c_char));
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0 {
        if strlen(b"confirm \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(strlen(b"confirm \0".as_ptr() as *const ::core::ffi::c_char))
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(strlen(b"confirm \0".as_ptr() as *const ::core::ffi::c_char));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_11: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        890 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"confirm \0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(strlen(b"confirm \0".as_ptr() as *const ::core::ffi::c_char)),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(strlen(b"confirm \0".as_ptr() as *const ::core::ffi::c_char));
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_HIDE as ::core::ffi::c_int != 0 {
        if strlen(b"hide \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(strlen(b"hide \0".as_ptr() as *const ::core::ffi::c_char))
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(strlen(b"hide \0".as_ptr() as *const ::core::ffi::c_char));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_12: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        891 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"hide \0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(strlen(b"hide \0".as_ptr() as *const ::core::ffi::c_char)),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(strlen(b"hide \0".as_ptr() as *const ::core::ffi::c_char));
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int != 0 {
        if strlen(b"keepalt \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(strlen(b"keepalt \0".as_ptr() as *const ::core::ffi::c_char))
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(strlen(b"keepalt \0".as_ptr() as *const ::core::ffi::c_char));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_13: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        892 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"keepalt \0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(strlen(b"keepalt \0".as_ptr() as *const ::core::ffi::c_char)),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(strlen(b"keepalt \0".as_ptr() as *const ::core::ffi::c_char));
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_KEEPJUMPS as ::core::ffi::c_int != 0 {
        if strlen(b"keepjumps \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline.size.wrapping_add(strlen(
                    b"keepjumps \0".as_ptr() as *const ::core::ffi::c_char
                ))
            {
                cmdline.capacity = cmdline.size.wrapping_add(strlen(
                    b"keepjumps \0".as_ptr() as *const ::core::ffi::c_char
                ));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_14: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        893 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"keepjumps \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(strlen(
                    b"keepjumps \0".as_ptr() as *const ::core::ffi::c_char,
                )),
            );
            cmdline.size = cmdline.size.wrapping_add(strlen(
                b"keepjumps \0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_KEEPMARKS as ::core::ffi::c_int != 0 {
        if strlen(b"keepmarks \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline.size.wrapping_add(strlen(
                    b"keepmarks \0".as_ptr() as *const ::core::ffi::c_char
                ))
            {
                cmdline.capacity = cmdline.size.wrapping_add(strlen(
                    b"keepmarks \0".as_ptr() as *const ::core::ffi::c_char
                ));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_15: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        894 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"keepmarks \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(strlen(
                    b"keepmarks \0".as_ptr() as *const ::core::ffi::c_char,
                )),
            );
            cmdline.size = cmdline.size.wrapping_add(strlen(
                b"keepmarks \0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_KEEPPATTERNS as ::core::ffi::c_int != 0 {
        if strlen(b"keeppatterns \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline.size.wrapping_add(strlen(
                    b"keeppatterns \0".as_ptr() as *const ::core::ffi::c_char
                ))
            {
                cmdline.capacity = cmdline.size.wrapping_add(strlen(
                    b"keeppatterns \0".as_ptr() as *const ::core::ffi::c_char
                ));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_16: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        895 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"keeppatterns \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(strlen(
                    b"keeppatterns \0".as_ptr() as *const ::core::ffi::c_char,
                )),
            );
            cmdline.size = cmdline.size.wrapping_add(strlen(
                b"keeppatterns \0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0 {
        if strlen(b"lockmarks \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline.size.wrapping_add(strlen(
                    b"lockmarks \0".as_ptr() as *const ::core::ffi::c_char
                ))
            {
                cmdline.capacity = cmdline.size.wrapping_add(strlen(
                    b"lockmarks \0".as_ptr() as *const ::core::ffi::c_char
                ));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_17: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        896 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"lockmarks \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(strlen(
                    b"lockmarks \0".as_ptr() as *const ::core::ffi::c_char,
                )),
            );
            cmdline.size = cmdline.size.wrapping_add(strlen(
                b"lockmarks \0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_NOSWAPFILE as ::core::ffi::c_int != 0 {
        if strlen(b"noswapfile \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline.size.wrapping_add(strlen(
                    b"noswapfile \0".as_ptr() as *const ::core::ffi::c_char
                ))
            {
                cmdline.capacity = cmdline.size.wrapping_add(strlen(
                    b"noswapfile \0".as_ptr() as *const ::core::ffi::c_char
                ));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_18: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        897 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"noswapfile \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(strlen(
                    b"noswapfile \0".as_ptr() as *const ::core::ffi::c_char,
                )),
            );
            cmdline.size = cmdline.size.wrapping_add(strlen(
                b"noswapfile \0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
    }
    if (*eap).argt & EX_RANGE as uint32_t != 0 {
        if (*eap).addr_count == 1 as ::core::ffi::c_int {
            kv_do_printf(
                &raw mut cmdline,
                b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                (*eap).line2,
            );
        } else if (*eap).addr_count > 1 as ::core::ffi::c_int {
            kv_do_printf(
                &raw mut cmdline,
                b"%d,%d\0".as_ptr() as *const ::core::ffi::c_char,
                (*eap).line1,
                (*eap).line2,
            );
            (*eap).addr_count = 2 as ::core::ffi::c_int;
        }
    }
    let mut cmdname_idx: size_t = cmdline.size;
    if strlen((*eap).cmd) > 0 as size_t {
        if cmdline.capacity < cmdline.size.wrapping_add(strlen((*eap).cmd)) {
            cmdline.capacity = cmdline.size.wrapping_add(strlen((*eap).cmd));
            cmdline.capacity = cmdline.capacity.wrapping_sub(1);
            cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
            cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
            cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
            cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
            cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
            cmdline.capacity = cmdline.capacity.wrapping_add(1);
            cmdline.capacity = cmdline.capacity;
            cmdline.items = xrealloc(
                cmdline.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
            ) as *mut ::core::ffi::c_char;
        }
        '_c2rust_label_19: {
            if !cmdline.items.is_null() {
            } else {
                __assert_fail(
                    b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    912 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            }
        };
        memcpy(
            cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
            (*eap).cmd as *const ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(strlen((*eap).cmd)),
        );
        cmdline.size = cmdline.size.wrapping_add(strlen((*eap).cmd));
    }
    if (*eap).argt & EX_BANG as uint32_t != 0 && (*eap).forceit != 0 {
        if strlen(b"!\0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(strlen(b"!\0".as_ptr() as *const ::core::ffi::c_char))
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(strlen(b"!\0".as_ptr() as *const ::core::ffi::c_char));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_20: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        916 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"!\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(strlen(b"!\0".as_ptr() as *const ::core::ffi::c_char)),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(strlen(b"!\0".as_ptr() as *const ::core::ffi::c_char));
        }
    }
    if (*eap).argt & EX_REGSTR as uint32_t != 0 && (*eap).regname != 0 {
        kv_do_printf(
            &raw mut cmdline,
            b" %c\0".as_ptr() as *const ::core::ffi::c_char,
            (*eap).regname,
        );
    }
    (*eap).argc = argc;
    (*eap).arglens = (if (*eap).argc > 0 as size_t {
        xcalloc(argc, ::core::mem::size_of::<size_t>())
    } else {
        NULL
    }) as *mut size_t;
    let mut argstart_idx: size_t = cmdline.size;
    let mut i: size_t = 0 as size_t;
    while i < argc {
        let mut s: String_0 = (*args.items.offset(i as isize)).data.string;
        *(*eap).arglens.offset(i as isize) = s.size;
        if strlen(b" \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(strlen(b" \0".as_ptr() as *const ::core::ffi::c_char))
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(strlen(b" \0".as_ptr() as *const ::core::ffi::c_char));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_21: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        930 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b" \0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(strlen(b" \0".as_ptr() as *const ::core::ffi::c_char)),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(strlen(b" \0".as_ptr() as *const ::core::ffi::c_char));
        }
        if s.size > 0 as size_t {
            if cmdline.capacity < cmdline.size.wrapping_add(s.size) {
                cmdline.capacity = cmdline.size.wrapping_add(s.size);
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_22: {
                if !cmdline.items.is_null() {
                } else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/command.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        931 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                s.data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(s.size),
            );
            cmdline.size = cmdline.size.wrapping_add(s.size);
        }
        i = i.wrapping_add(1);
    }
    if cmdline.size == cmdline.capacity {
        cmdline.capacity = if cmdline.capacity != 0 {
            cmdline.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        cmdline.items = xrealloc(
            cmdline.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
        ) as *mut ::core::ffi::c_char;
    } else {
    };
    let c2rust_fresh33 = cmdline.size;
    cmdline.size = cmdline.size.wrapping_add(1);
    *cmdline.items.offset(c2rust_fresh33 as isize) = '\0' as ::core::ffi::c_char;
    (*eap).cmd = cmdline.items.offset(cmdname_idx as isize);
    (*eap).args = (if (*eap).argc > 0 as size_t {
        xcalloc(argc, ::core::mem::size_of::<*mut ::core::ffi::c_char>())
    } else {
        NULL
    }) as *mut *mut ::core::ffi::c_char;
    let mut offset: size_t = argstart_idx;
    let mut i_0: size_t = 0 as size_t;
    while i_0 < argc {
        offset = offset.wrapping_add(1);
        *(*eap).args.offset(i_0 as isize) = cmdline.items.offset(offset as isize);
        offset = offset.wrapping_add(*(*eap).arglens.offset(i_0 as isize));
        i_0 = i_0.wrapping_add(1);
    }
    (*eap).arg = if argc > 0 as size_t {
        *(*eap).args.offset(0 as ::core::ffi::c_int as isize)
    } else {
        cmdline
            .items
            .offset(cmdline.size as isize)
            .offset(-(1 as ::core::ffi::c_int as isize))
    };
    *cmdlinep = cmdline.items;
    let mut p: *mut ::core::ffi::c_char = replace_makeprg(eap, (*eap).arg, cmdlinep);
    if p != (*eap).arg {
        (*eap).arg = p;
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*eap).args as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*eap).arglens as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL;
        let _ = *ptr__0;
        (*eap).argc = 0 as size_t;
    }
}
#[no_mangle]
pub unsafe extern "C" fn nvim_create_user_command(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut cmd: Object,
    mut opts: *mut KeyDict_user_command,
    mut err: *mut Error,
) {
    create_user_command(channel_id, name, cmd, opts, 0 as ::core::ffi::c_int, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_del_user_command(mut name: String_0, mut err: *mut Error) {
    nvim_buf_del_user_command(-1 as Buffer, name, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_create_user_command(
    mut channel_id: uint64_t,
    mut buf: Buffer,
    mut name: String_0,
    mut cmd: Object,
    mut opts: *mut KeyDict_user_command,
    mut err: *mut Error,
) {
    let mut target_buf: *mut buf_T = find_buffer_by_handle(buf, err);
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return;
    }
    let mut save_curbuf: *mut buf_T = curbuf.get();
    curbuf.set(target_buf);
    create_user_command(
        channel_id,
        name,
        cmd,
        opts,
        UC_BUFFER as ::core::ffi::c_int,
        err,
    );
    curbuf.set(save_curbuf);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_del_user_command(
    mut buf: Buffer,
    mut name: String_0,
    mut err: *mut Error,
) {
    let mut gap: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
    if buf == -1 as ::core::ffi::c_int {
        gap = ucmds.ptr();
    } else {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return;
        }
        gap = &raw mut (*b).b_ucmds;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        let mut cmd: *mut ucmd_T = ((*gap).ga_data as *mut ucmd_T).offset(i as isize);
        if strcmp(name.data, (*cmd).uc_name) == 0 {
            free_ucmd(cmd);
            (*gap).ga_len -= 1 as ::core::ffi::c_int;
            if i < (*gap).ga_len {
                memmove(
                    cmd as *mut ::core::ffi::c_void,
                    cmd.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    (((*gap).ga_len - i) as size_t).wrapping_mul(::core::mem::size_of::<ucmd_T>()),
                );
            }
            return;
        }
        i += 1;
    }
    api_set_error(
        err,
        kErrorTypeException,
        b"Invalid command (not found): %s\0".as_ptr() as *const ::core::ffi::c_char,
        name.data,
    );
}
#[no_mangle]
pub unsafe extern "C" fn create_user_command(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut cmd: Object,
    mut opts: *mut KeyDict_user_command,
    mut flags: ::core::ffi::c_int,
    mut err: *mut Error,
) {
    let mut force: bool = false;
    let mut argt: uint32_t = 0 as uint32_t;
    let mut def: int64_t = -1 as int64_t;
    let mut addr_type_arg: cmd_addr_T = ADDR_NONE;
    let mut context: ::core::ffi::c_int = EXPAND_NOTHING as ::core::ffi::c_int;
    let mut compl_arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut rep: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut luaref: LuaRef = LUA_NOREF;
    let mut compl_luaref: LuaRef = LUA_NOREF;
    let mut preview_luaref: LuaRef = LUA_NOREF;
    '_err: {
        if uc_validate_name(name.data).is_null() {
            api_err_invalid(
                err,
                b"command name\0".as_ptr() as *const ::core::ffi::c_char,
                name.data,
                0 as int64_t,
                true_0 != 0,
            );
        } else if mb_islower(
            *name.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        ) {
            api_err_invalid(
                err,
                b"command name (must start with uppercase)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                name.data,
                0 as int64_t,
                true_0 != 0,
            );
        } else if !(!((*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 8 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong)
            || !((*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong))
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"Cannot use both 'range' and 'count'\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            if (*opts).nargs.type_0 as ::core::ffi::c_uint
                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                match (*opts).nargs.data.integer {
                    0 => {}
                    1 => {
                        argt = (argt as ::core::ffi::c_uint | (EX_EXTRA | EX_NOSPC | EX_NEEDARG))
                            as uint32_t;
                    }
                    _ => {
                        if true {
                            api_err_invalid(
                                err,
                                b"nargs\0".as_ptr() as *const ::core::ffi::c_char,
                                ::core::ptr::null::<::core::ffi::c_char>(),
                                (*opts).nargs.data.integer,
                                false_0 != 0,
                            );
                            break '_err;
                        }
                    }
                }
            } else if (*opts).nargs.type_0 as ::core::ffi::c_uint
                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if !((*opts).nargs.data.string.size <= 1 as size_t) {
                    api_err_invalid(
                        err,
                        b"nargs\0".as_ptr() as *const ::core::ffi::c_char,
                        (*opts).nargs.data.string.data,
                        0 as int64_t,
                        true_0 != 0,
                    );
                    break '_err;
                } else {
                    match *(*opts)
                        .nargs
                        .data
                        .string
                        .data
                        .offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                    {
                        42 => {
                            argt = (argt as ::core::ffi::c_uint | EX_EXTRA) as uint32_t;
                        }
                        63 => {
                            argt =
                                (argt as ::core::ffi::c_uint | (EX_EXTRA | EX_NOSPC)) as uint32_t;
                        }
                        43 => {
                            argt =
                                (argt as ::core::ffi::c_uint | (EX_EXTRA | EX_NEEDARG)) as uint32_t;
                        }
                        _ => {
                            if true {
                                api_err_invalid(
                                    err,
                                    b"nargs\0".as_ptr() as *const ::core::ffi::c_char,
                                    (*opts).nargs.data.string.data,
                                    0 as int64_t,
                                    true_0 != 0,
                                );
                                break '_err;
                            }
                        }
                    }
                }
            } else if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__nargs
                != 0 as ::core::ffi::c_ulonglong
            {
                if true {
                    api_err_invalid(
                        err,
                        b"nargs\0".as_ptr() as *const ::core::ffi::c_char,
                        b"\0".as_ptr() as *const ::core::ffi::c_char,
                        0 as int64_t,
                        true_0 != 0,
                    );
                    break '_err;
                }
            }
            if !(!((*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 10 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong)
                || argt != 0)
            {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    b"'complete' used without 'nargs'\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                if (*opts).range.type_0 as ::core::ffi::c_uint
                    == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if (*opts).range.data.boolean {
                        argt = (argt as ::core::ffi::c_uint | EX_RANGE) as uint32_t;
                        addr_type_arg = ADDR_LINES;
                    }
                } else if (*opts).range.type_0 as ::core::ffi::c_uint
                    == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if !(*(*opts)
                        .range
                        .data
                        .string
                        .data
                        .offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == '%' as ::core::ffi::c_int
                        && (*opts).range.data.string.size == 1 as size_t)
                    {
                        api_err_invalid(
                            err,
                            b"range\0".as_ptr() as *const ::core::ffi::c_char,
                            b"\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_err;
                    } else {
                        argt = (argt as ::core::ffi::c_uint | (EX_RANGE | EX_DFLALL)) as uint32_t;
                        addr_type_arg = ADDR_LINES;
                    }
                } else if (*opts).range.type_0 as ::core::ffi::c_uint
                    == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    argt = (argt as ::core::ffi::c_uint | (EX_RANGE | EX_ZEROR)) as uint32_t;
                    def = (*opts).range.data.integer as int64_t;
                    addr_type_arg = ADDR_LINES;
                } else if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__range
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if true {
                        api_err_invalid(
                            err,
                            b"range\0".as_ptr() as *const ::core::ffi::c_char,
                            b"\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_err;
                    }
                }
                if (*opts).count.type_0 as ::core::ffi::c_uint
                    == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if (*opts).count.data.boolean {
                        argt = (argt as ::core::ffi::c_uint | (EX_COUNT | EX_ZEROR | EX_RANGE))
                            as uint32_t;
                        addr_type_arg = ADDR_OTHER;
                        def = 0 as int64_t;
                    }
                } else if (*opts).count.type_0 as ::core::ffi::c_uint
                    == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    argt = (argt as ::core::ffi::c_uint | (EX_COUNT | EX_ZEROR | EX_RANGE))
                        as uint32_t;
                    addr_type_arg = ADDR_OTHER;
                    def = (*opts).count.data.integer as int64_t;
                } else if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__count
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if true {
                        api_err_invalid(
                            err,
                            b"count\0".as_ptr() as *const ::core::ffi::c_char,
                            b"\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_err;
                    }
                }
                if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__addr
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                        != (*opts).addr.type_0 as ::core::ffi::c_uint
                    {
                        api_err_exp(
                            err,
                            b"addr\0".as_ptr() as *const ::core::ffi::c_char,
                            api_typename(kObjectTypeString),
                            api_typename((*opts).addr.type_0),
                        );
                        break '_err;
                    } else if !(1 as ::core::ffi::c_int
                        == parse_addr_type_arg(
                            (*opts).addr.data.string.data,
                            (*opts).addr.data.string.size as ::core::ffi::c_int,
                            &raw mut addr_type_arg,
                        ))
                    {
                        api_err_invalid(
                            err,
                            b"addr\0".as_ptr() as *const ::core::ffi::c_char,
                            (*opts).addr.data.string.data,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_err;
                    } else {
                        argt = (argt as ::core::ffi::c_uint | EX_RANGE) as uint32_t;
                        if addr_type_arg as ::core::ffi::c_uint
                            != ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            argt = (argt as ::core::ffi::c_uint | EX_ZEROR) as uint32_t;
                        }
                    }
                }
                if (*opts).bang {
                    argt = (argt as ::core::ffi::c_uint | EX_BANG) as uint32_t;
                }
                if (*opts).bar {
                    argt = (argt as ::core::ffi::c_uint | EX_TRLBAR) as uint32_t;
                }
                if (*opts).register_ {
                    argt = (argt as ::core::ffi::c_uint | EX_REGSTR) as uint32_t;
                }
                if (*opts).keepscript {
                    argt = (argt as ::core::ffi::c_uint | EX_KEEPSCRIPT) as uint32_t;
                }
                force = if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__force
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*opts).force as ::core::ffi::c_int
                } else {
                    true_0
                } != 0;
                if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                    if (*opts).complete.type_0 as ::core::ffi::c_uint
                        == kObjectTypeLuaRef as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        context = EXPAND_USER_LUA as ::core::ffi::c_int;
                        compl_luaref = (*opts).complete.data.luaref;
                        (*opts).complete.data.luaref = LUA_NOREF as LuaRef;
                    } else if (*opts).complete.type_0 as ::core::ffi::c_uint
                        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if !(1 as ::core::ffi::c_int
                            == parse_compl_arg(
                                (*opts).complete.data.string.data,
                                (*opts).complete.data.string.size as ::core::ffi::c_int,
                                &raw mut context,
                                &raw mut argt,
                                &raw mut compl_arg,
                            ))
                        {
                            api_err_invalid(
                                err,
                                b"complete\0".as_ptr() as *const ::core::ffi::c_char,
                                (*opts).complete.data.string.data,
                                0 as int64_t,
                                true_0 != 0,
                            );
                            break '_err;
                        }
                    } else if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__complete
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if true {
                            api_err_exp(
                                err,
                                b"complete\0".as_ptr() as *const ::core::ffi::c_char,
                                b"Function or String\0".as_ptr() as *const ::core::ffi::c_char,
                                ::core::ptr::null::<::core::ffi::c_char>(),
                            );
                            break '_err;
                        }
                    }
                    if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__preview
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if kObjectTypeLuaRef as ::core::ffi::c_int as ::core::ffi::c_uint
                            != (*opts).preview.type_0 as ::core::ffi::c_uint
                        {
                            api_err_exp(
                                err,
                                b"preview\0".as_ptr() as *const ::core::ffi::c_char,
                                api_typename(kObjectTypeLuaRef),
                                api_typename((*opts).preview.type_0),
                            );
                            break '_err;
                        } else {
                            argt = (argt as ::core::ffi::c_uint | EX_PREVIEW) as uint32_t;
                            preview_luaref = (*opts).preview.data.luaref;
                            (*opts).preview.data.luaref = LUA_NOREF as LuaRef;
                        }
                    }
                    match cmd.type_0 as ::core::ffi::c_uint {
                        7 => {
                            luaref = api_new_luaref(cmd.data.luaref);
                            if (*opts).desc.type_0 as ::core::ffi::c_uint
                                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                rep = (*opts).desc.data.string.data;
                            } else {
                                rep = b"\0".as_ptr() as *const ::core::ffi::c_char;
                            }
                        }
                        4 => {
                            rep = cmd.data.string.data;
                        }
                        _ => {
                            if true {
                                api_err_exp(
                                    err,
                                    b"command\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"Function or String\0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                );
                                break '_err;
                            }
                        }
                    }
                    let save_current_sctx: sctx_T = api_set_sctx(channel_id);
                    if uc_add_command(
                        name.data,
                        name.size,
                        rep,
                        argt,
                        def,
                        flags,
                        context,
                        compl_arg,
                        compl_luaref,
                        preview_luaref,
                        addr_type_arg,
                        luaref,
                        force,
                    ) != 1 as ::core::ffi::c_int
                    {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            b"Failed to create user command\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                    current_sctx.set(save_current_sctx);
                    return;
                }
            }
        }
    }
    if luaref != LUA_NOREF {
        api_free_luaref(luaref);
        luaref = LUA_NOREF as LuaRef;
    }
    if compl_luaref != LUA_NOREF {
        api_free_luaref(compl_luaref);
        compl_luaref = LUA_NOREF as LuaRef;
    }
    if preview_luaref != LUA_NOREF {
        api_free_luaref(preview_luaref);
        preview_luaref = LUA_NOREF as LuaRef;
    }
    xfree(compl_arg as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_commands(
    mut opts: *mut KeyDict_get_commands,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    return nvim_buf_get_commands(-1 as Buffer, opts, arena, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_get_commands(
    mut buf: Buffer,
    mut opts: *mut KeyDict_get_commands,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut global: bool = buf == -1 as ::core::ffi::c_int;
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
    }
    if global {
        if (*opts).builtin {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"builtin=true not implemented\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return Dict {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            };
        }
        return commands_array(::core::ptr::null_mut::<buf_T>(), arena);
    }
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if (*opts).builtin as ::core::ffi::c_int != 0 || b.is_null() {
        return Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
    }
    return commands_array(b, arena);
}
