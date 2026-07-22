use crate::src::nvim::api::private::converter::{object_to_vim, vim_to_object};
use crate::src::nvim::api::private::helpers::{
    api_set_error, api_set_sctx, arena_array, arena_dict, arena_string, cstr_as_string,
    cstr_to_string, try_enter, try_leave,
};
use crate::src::nvim::api::private::validate::api_err_exp;
use crate::src::nvim::eval::typval::{tv_clear, tv_dict_find};
use crate::src::nvim::eval::userfunc::call_func;
use crate::src::nvim::eval_1::{clear_evalarg, eval0};
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    capture_ga, current_sctx, curwin, did_emsg, did_throw, force_abort, msg_col, msg_silent,
    redir_off, suppress_errthrow, EVALARG_EVALUATE,
};
use crate::src::nvim::memory::{xfree, xmalloc, xrealloc};
use crate::src::nvim::os::libc::{__assert_fail, abort, memcpy, memmove, strlen};
use crate::src::nvim::runtime::do_source_str;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, Arena, ArgvFunc, Array, BoolVarValue, Boolean,
    BufUpdateCallbacks, CMD_index, Callback, CallbackType, Callback_data as C2Rust_Unnamed_5,
    ChangedtickDictItem, DecorExt, DecorHighlightInline, DecorInlineData, DecorPriority,
    DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, Dict, Error, ErrorType, ExprAST,
    ExprASTError, ExprASTNode, ExprASTNodeType, ExprAssignmentType, ExprCaseCompareStrategy,
    ExprComparisonType, ExprOptScope, ExprParserFlags, ExprVarScope, ExtmarkUndoObject, FileID,
    Float, FloatAnchor, FloatRelative, GridView, Integer, Intersection, KeyDict_exec_opts,
    KeyValuePair, LineGetter, LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t,
    Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, Object, ObjectType,
    OptInt, ParserHighlight, ParserHighlightChunk, ParserInputReader,
    ParserInputReader_lines as C2Rust_Unnamed_33, ParserLine, ParserLineGetter, ParserPosition,
    ParserState, ParserStateItem, ParserStateItem_data as C2Rust_Unnamed_29,
    ParserStateItem_data_expr as C2Rust_Unnamed_30,
    ParserStateItem_data_expr_type_0 as C2Rust_Unnamed_31,
    ParserStateItem_type_0 as C2Rust_Unnamed_32, ParserState_stack as C2Rust_Unnamed_28,
    ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_12, String_0,
    Terminal, Timestamp, TryState, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk,
    VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle, Window, __time_t, alist_T, bhdr_T, blob_T,
    blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, cmd_addr_T, cmdidx_T, colnr_T, cstack_T,
    cstack_T_cs_pend as C2Rust_Unnamed_14, dict_T, dictitem_T, dictvar_S, disptick_T, eslist_T,
    eslist_elem, evalarg_T, exarg, exarg_T, except_T, except_type_T, expr_ast_node,
    expr_ast_node_data as C2Rust_Unnamed_15, expr_ast_node_data_ass as C2Rust_Unnamed_16,
    expr_ast_node_data_cmp as C2Rust_Unnamed_22, expr_ast_node_data_env as C2Rust_Unnamed_17,
    expr_ast_node_data_fig as C2Rust_Unnamed_25,
    expr_ast_node_data_fig_type_guesses as C2Rust_Unnamed_26,
    expr_ast_node_data_flt as C2Rust_Unnamed_20, expr_ast_node_data_num as C2Rust_Unnamed_21,
    expr_ast_node_data_opt as C2Rust_Unnamed_18, expr_ast_node_data_reg as C2Rust_Unnamed_27,
    expr_ast_node_data_str as C2Rust_Unnamed_19, expr_ast_node_data_ter as C2Rust_Unnamed_23,
    expr_ast_node_data_var as C2Rust_Unnamed_24, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_3, file_buffer_b_wininfo as C2Rust_Unnamed_11,
    file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, funcexe_T, garray_T,
    handle_T, hash_T, hashitem_T, hashtab_T, iconv_t, infoptr_T, int16_t, int32_t, int64_t,
    key_value_pair, lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S,
    listwatch_T, llpos_T, lpos_T, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T,
    memline_T, mfdirty_T, msglist, msglist_T, mtnode_inner_s, mtnode_s, object,
    object_data as C2Rust_Unnamed, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t,
    ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T,
    sattr_T, schar_T, scid_T, sctx_T, size_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_4,
    syn_time_T, synblock_T, synstate_T, taggy_T, terminal, time_t, typval_T, typval_vval_union,
    u_entry, u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8,
    u_header_uh_alt_prev as C2Rust_Unnamed_7, u_header_uh_next as C2Rust_Unnamed_10,
    u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, uvarnumber_T, varnumber_T, vim_exception, vimconv_T, virt_line, visualinfo_T,
    win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
};
use crate::src::nvim::viml::parser::expressions::{
    ccs_tab, east_node_type_tab, eltkn_cmp_type_tab, expr_asgn_type_tab, viml_pexpr_free_ast,
    viml_pexpr_parse,
};
use crate::src::nvim::viml::parser::parser::{parser_simple_get_line, viml_parser_destroy};
extern "C" {
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MAX_FUNC_ARGS: C2Rust_Unnamed_13 = 20;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
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
pub const kExprAsgnConcat: ExprAssignmentType = 3;
pub const kExprAsgnSubtract: ExprAssignmentType = 2;
pub const kExprAsgnAdd: ExprAssignmentType = 1;
pub const kExprAsgnPlain: ExprAssignmentType = 0;
pub const kExprOptScopeLocal: ExprOptScope = 108;
pub const kExprOptScopeGlobal: ExprOptScope = 103;
pub const kExprOptScopeUnspecified: ExprOptScope = 0;
pub const kCCStrategyIgnoreCase: ExprCaseCompareStrategy = 63;
pub const kCCStrategyMatchCase: ExprCaseCompareStrategy = 35;
pub const kCCStrategyUseOption: ExprCaseCompareStrategy = 0;
pub const kExprCmpIdentical: ExprComparisonType = 4;
pub const kExprCmpGreaterOrEqual: ExprComparisonType = 3;
pub const kExprCmpGreater: ExprComparisonType = 2;
pub const kExprCmpMatches: ExprComparisonType = 1;
pub const kExprCmpEqual: ExprComparisonType = 0;
pub const kExprVarScopeArguments: ExprVarScope = 97;
pub const kExprVarScopeLocal: ExprVarScope = 108;
pub const kExprVarScopeTabpage: ExprVarScope = 116;
pub const kExprVarScopeWindow: ExprVarScope = 119;
pub const kExprVarScopeBuffer: ExprVarScope = 98;
pub const kExprVarScopeVim: ExprVarScope = 118;
pub const kExprVarScopeGlobal: ExprVarScope = 103;
pub const kExprVarScopeScript: ExprVarScope = 115;
pub const kExprVarScopeMissing: ExprVarScope = 0;
pub const kExprNodeAssignment: ExprASTNodeType = 38;
pub const kExprNodeEnvironment: ExprASTNodeType = 37;
pub const kExprNodeOption: ExprASTNodeType = 36;
pub const kExprNodeMod: ExprASTNodeType = 35;
pub const kExprNodeDivision: ExprASTNodeType = 34;
pub const kExprNodeMultiplication: ExprASTNodeType = 33;
pub const kExprNodeNot: ExprASTNodeType = 32;
pub const kExprNodeBinaryMinus: ExprASTNodeType = 31;
pub const kExprNodeUnaryMinus: ExprASTNodeType = 30;
pub const kExprNodeAnd: ExprASTNodeType = 29;
pub const kExprNodeOr: ExprASTNodeType = 28;
pub const kExprNodeDoubleQuotedString: ExprASTNodeType = 27;
pub const kExprNodeSingleQuotedString: ExprASTNodeType = 26;
pub const kExprNodeFloat: ExprASTNodeType = 25;
pub const kExprNodeInteger: ExprASTNodeType = 24;
pub const kExprNodeConcatOrSubscript: ExprASTNodeType = 23;
pub const kExprNodeConcat: ExprASTNodeType = 22;
pub const kExprNodeComparison: ExprASTNodeType = 21;
pub const kExprNodeArrow: ExprASTNodeType = 20;
pub const kExprNodeColon: ExprASTNodeType = 19;
pub const kExprNodeComma: ExprASTNodeType = 18;
pub const kExprNodeCurlyBracesIdentifier: ExprASTNodeType = 17;
pub const kExprNodeDictLiteral: ExprASTNodeType = 16;
pub const kExprNodeLambda: ExprASTNodeType = 15;
pub const kExprNodeUnknownFigure: ExprASTNodeType = 14;
pub const kExprNodeComplexIdentifier: ExprASTNodeType = 13;
pub const kExprNodePlainKey: ExprASTNodeType = 12;
pub const kExprNodePlainIdentifier: ExprASTNodeType = 11;
pub const kExprNodeCall: ExprASTNodeType = 10;
pub const kExprNodeNested: ExprASTNodeType = 9;
pub const kExprNodeBinaryPlus: ExprASTNodeType = 8;
pub const kExprNodeUnaryPlus: ExprASTNodeType = 7;
pub const kExprNodeListLiteral: ExprASTNodeType = 6;
pub const kExprNodeSubscript: ExprASTNodeType = 5;
pub const kExprNodeRegister: ExprASTNodeType = 4;
pub const kExprNodeTernaryValue: ExprASTNodeType = 3;
pub const kExprNodeTernary: ExprASTNodeType = 2;
pub const kExprNodeOpMissing: ExprASTNodeType = 1;
pub const kExprNodeMissing: ExprASTNodeType = 0;
pub const kExprUnknown: C2Rust_Unnamed_31 = 0;
pub const kPTopStateParsingExpression: C2Rust_Unnamed_32 = 1;
pub const kPTopStateParsingCommand: C2Rust_Unnamed_32 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExprASTConvStackItem {
    pub node_p: *mut *mut ExprASTNode,
    pub ret_node_p: *mut Object,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExprASTConvStack {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ExprASTConvStackItem,
    pub init_array: [ExprASTConvStackItem; 16],
}
pub const CONV_NONE: C2Rust_Unnamed_34 = 0;
pub const kExprFlagsParseLet: ExprParserFlags = 4;
pub const kExprFlagsDisallowEOC: ExprParserFlags = 2;
pub const kExprFlagsMulti: ExprParserFlags = 1;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const CONV_ICONV: C2Rust_Unnamed_34 = 5;
pub const CONV_TO_LATIN9: C2Rust_Unnamed_34 = 4;
pub const CONV_TO_LATIN1: C2Rust_Unnamed_34 = 3;
pub const CONV_9_TO_UTF8: C2Rust_Unnamed_34 = 2;
pub const CONV_TO_UTF8: C2Rust_Unnamed_34 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Dict = Dict {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<KeyValuePair>(),
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
pub const ARRAY_DICT_INIT: Dict = KV_INITIAL_VALUE;
pub const STRING_INIT: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub unsafe extern "C" fn nvim_exec2(
    mut channel_id: uint64_t,
    mut src: String_0,
    mut opts: *mut KeyDict_exec_opts,
    mut err: *mut Error,
) -> Dict {
    let mut result: Dict = ARRAY_DICT_INIT;
    let mut output: String_0 = exec_impl(channel_id, src, opts, err);
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return result;
    }
    if (*opts).output {
        if result.size == result.capacity {
            result.capacity = if result.capacity != 0 {
                result.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            result.items = xrealloc(
                result.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<KeyValuePair>().wrapping_mul(result.capacity),
            ) as *mut KeyValuePair;
        } else {
        };
        let c2rust_fresh0 = result.size;
        result.size = result.size.wrapping_add(1);
        *result.items.offset(c2rust_fresh0 as isize) = key_value_pair {
            key: cstr_to_string(b"output\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed { string: output },
            },
        };
    }
    return result;
}
pub unsafe extern "C" fn exec_impl(
    mut channel_id: uint64_t,
    mut src: String_0,
    mut opts: *mut KeyDict_exec_opts,
    mut err: *mut Error,
) -> String_0 {
    let save_msg_silent: ::core::ffi::c_int = msg_silent.get();
    let save_redir_off: bool = redir_off.get();
    let save_capture_ga: *mut garray_T = capture_ga.get();
    let save_msg_col: ::core::ffi::c_int = msg_col.get();
    let mut capture_local: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
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
    do_source_str(
        src.data,
        b"nvim_exec2()\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
    if (*opts).output {
        capture_ga.set(save_capture_ga);
        msg_silent.set(save_msg_silent);
        redir_off.set(save_redir_off);
        msg_col.set(save_msg_col);
    }
    current_sctx.set(save_current_sctx);
    try_leave(&raw mut tstate, err);
    if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
        if (*opts).output as ::core::ffi::c_int != 0
            && capture_local.ga_len > 1 as ::core::ffi::c_int
        {
            let mut s: String_0 = String_0 {
                data: capture_local.ga_data as *mut ::core::ffi::c_char,
                size: capture_local.ga_len as size_t,
            };
            if *s.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\n' as ::core::ffi::c_int
            {
                memmove(
                    s.data as *mut ::core::ffi::c_void,
                    s.data.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    s.size.wrapping_sub(1 as size_t),
                );
                *s.data.offset(s.size.wrapping_sub(1 as size_t) as isize) =
                    NUL as ::core::ffi::c_char;
                s.size = s.size.wrapping_sub(1 as size_t);
            }
            return s;
        }
    }
    if (*opts).output {
        ga_clear(&raw mut capture_local);
    }
    return STRING_INIT;
}
pub unsafe extern "C" fn nvim_command(mut cmd: String_0, mut err: *mut Error) {
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
    do_cmdline_cmd(cmd.data);
    try_leave(&raw mut tstate, err);
}
pub unsafe extern "C" fn nvim_eval(
    mut expr: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    static recursive: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    let mut rv: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    if recursive.get() == 0 {
        force_abort.set(false_0 != 0);
        suppress_errthrow.set(false_0 != 0);
        did_throw.set(false_0 != 0);
        did_emsg.set(false_0);
    }
    (*recursive.ptr()) += 1;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut ok: ::core::ffi::c_int = 0;
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
    ok = eval0(
        expr.data,
        &raw mut rettv,
        ::core::ptr::null_mut::<exarg_T>(),
        EVALARG_EVALUATE.ptr(),
    );
    clear_evalarg(EVALARG_EVALUATE.ptr(), ::core::ptr::null_mut::<exarg_T>());
    try_leave(&raw mut tstate, err);
    if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        if ok == FAIL {
            api_set_error(
                err,
                kErrorTypeException,
                b"Failed to evaluate expression: '%.*s'\0".as_ptr() as *const ::core::ffi::c_char,
                256 as ::core::ffi::c_int,
                expr.data,
            );
        } else {
            rv = vim_to_object(&raw mut rettv, arena, false_0 != 0);
        }
    }
    tv_clear(&raw mut rettv);
    (*recursive.ptr()) -= 1;
    return rv;
}
unsafe extern "C" fn _call_function(
    mut fn_0: String_0,
    mut args: Array,
    mut self_0: *mut dict_T,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    static recursive: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    let mut rv: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    if args.size > MAX_FUNC_ARGS as ::core::ffi::c_int as size_t {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Function called with too many arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return rv;
    }
    let mut vim_args: [typval_T; 21] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 21];
    let mut i: size_t = 0 as size_t;
    while i < args.size {
        object_to_vim(
            *args.items.offset(i as isize),
            (&raw mut vim_args as *mut typval_T).offset(i as isize),
            err,
        );
        i = i.wrapping_add(1);
    }
    if recursive.get() == 0 {
        force_abort.set(false_0 != 0);
        suppress_errthrow.set(false_0 != 0);
        did_throw.set(false_0 != 0);
        did_emsg.set(false_0);
    }
    (*recursive.ptr()) += 1;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
    funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
    funcexe.fe_evaluate = true_0 != 0;
    funcexe.fe_selfdict = self_0;
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
    call_func(
        fn_0.data,
        fn_0.size as ::core::ffi::c_int,
        &raw mut rettv,
        args.size as ::core::ffi::c_int,
        &raw mut vim_args as *mut typval_T,
        &raw mut funcexe,
    );
    try_leave(&raw mut tstate, err);
    if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        rv = vim_to_object(&raw mut rettv, arena, false_0 != 0);
    }
    tv_clear(&raw mut rettv);
    (*recursive.ptr()) -= 1;
    while i > 0 as size_t {
        i = i.wrapping_sub(1);
        tv_clear((&raw mut vim_args as *mut typval_T).offset(i as isize));
    }
    return rv;
}
pub unsafe extern "C" fn nvim_call_function(
    mut fn_0: String_0,
    mut args: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    return _call_function(fn_0, args, ::core::ptr::null_mut::<dict_T>(), arena, err);
}
pub unsafe extern "C" fn nvim_call_dict_function(
    mut dict: Object,
    mut fn_0: String_0,
    mut args: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut rv: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut mustfree: bool = false_0 != 0;
    match dict.type_0 as ::core::ffi::c_uint {
        4 => {
            let mut eval_ret: ::core::ffi::c_int = 0;
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
            eval_ret = eval0(
                dict.data.string.data,
                &raw mut rettv,
                ::core::ptr::null_mut::<exarg_T>(),
                EVALARG_EVALUATE.ptr(),
            );
            clear_evalarg(EVALARG_EVALUATE.ptr(), ::core::ptr::null_mut::<exarg_T>());
            try_leave(&raw mut tstate, err);
            if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                return rv;
            }
            if eval_ret != OK {
                abort();
            }
            mustfree = true_0 != 0;
        }
        6 => {
            object_to_vim(dict, &raw mut rettv, err);
        }
        _ => {
            if true {
                api_err_exp(
                    err,
                    b"dict argument\0".as_ptr() as *const ::core::ffi::c_char,
                    b"String or Dict\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                );
                return rv;
            }
        }
    }
    let mut self_dict: *mut dict_T = rettv.vval.v_dict;
    '_end: {
        if rettv.v_type as ::core::ffi::c_uint
            != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            || self_dict.is_null()
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"dict not found\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            if !fn_0.data.is_null()
                && fn_0.size > 0 as size_t
                && dict.type_0 as ::core::ffi::c_uint
                    != kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let di: *mut dictitem_T =
                    tv_dict_find(self_dict, fn_0.data, fn_0.size as ptrdiff_t);
                if di.is_null() {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"Not found: %s\0".as_ptr() as *const ::core::ffi::c_char,
                        fn_0.data,
                    );
                    break '_end;
                } else if (*di).di_tv.v_type as ::core::ffi::c_uint
                    == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"partial function not supported\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break '_end;
                } else if !((*di).di_tv.v_type as ::core::ffi::c_uint
                    == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"Not a function: %s\0".as_ptr() as *const ::core::ffi::c_char,
                        fn_0.data,
                    );
                    break '_end;
                } else {
                    fn_0 = String_0 {
                        data: (*di).di_tv.vval.v_string,
                        size: strlen((*di).di_tv.vval.v_string),
                    };
                }
            }
            if !(!fn_0.data.is_null() && fn_0.size >= 1 as size_t) {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"Invalid function name: %s\0".as_ptr() as *const ::core::ffi::c_char,
                    b"(empty)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                rv = _call_function(fn_0, args, self_dict, arena, err);
            }
        }
    }
    if mustfree {
        tv_clear(&raw mut rettv);
    }
    return rv;
}
pub unsafe extern "C" fn nvim_parse_expression(
    mut expr: String_0,
    mut flags: String_0,
    mut hl: Boolean,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut pflags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: size_t = 0 as size_t;
    while i < flags.size {
        match *flags.data.offset(i as isize) as ::core::ffi::c_int {
            109 => {
                pflags |= kExprFlagsMulti as ::core::ffi::c_int;
            }
            69 => {
                pflags |= kExprFlagsDisallowEOC as ::core::ffi::c_int;
            }
            108 => {
                pflags |= kExprFlagsParseLet as ::core::ffi::c_int;
            }
            NUL => {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"Invalid flag: '\\0' (%u)\0".as_ptr() as *const ::core::ffi::c_char,
                    *flags.data.offset(i as isize) as ::core::ffi::c_uint,
                );
                return ARRAY_DICT_INIT;
            }
            _ => {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"Invalid flag: '%c' (%u)\0".as_ptr() as *const ::core::ffi::c_char,
                    *flags.data.offset(i as isize) as ::core::ffi::c_int,
                    *flags.data.offset(i as isize) as ::core::ffi::c_uint,
                );
                return ARRAY_DICT_INIT;
            }
        }
        i = i.wrapping_add(1);
    }
    let mut parser_lines: [ParserLine; 2] = [
        ParserLine {
            data: expr.data,
            size: expr.size,
            allocated: false_0 != 0,
        },
        ParserLine {
            data: ::core::ptr::null::<::core::ffi::c_char>(),
            size: 0 as size_t,
            allocated: false_0 != 0,
        },
    ];
    let mut plines_p: *mut ParserLine = &raw mut parser_lines as *mut ParserLine;
    let mut colors: ParserHighlight = ParserHighlight {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<ParserHighlightChunk>(),
        init_array: [ParserHighlightChunk {
            start: ParserPosition { line: 0, col: 0 },
            end_col: 0,
            group: ::core::ptr::null::<::core::ffi::c_char>(),
        }; 16],
    };
    colors.capacity = ::core::mem::size_of::<[ParserHighlightChunk; 16]>()
        .wrapping_div(::core::mem::size_of::<ParserHighlightChunk>())
        .wrapping_div(
            (::core::mem::size_of::<[ParserHighlightChunk; 16]>()
                .wrapping_rem(::core::mem::size_of::<ParserHighlightChunk>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    colors.size = 0 as size_t;
    colors.items = &raw mut colors.init_array as *mut ParserHighlightChunk;
    let colors_p: *mut ParserHighlight = if hl as ::core::ffi::c_int != 0 {
        &raw mut colors
    } else {
        ::core::ptr::null_mut::<ParserHighlight>()
    };
    let mut pstate: ParserState = ParserState {
        reader: ParserInputReader {
            get_line: None,
            cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            lines: C2Rust_Unnamed_33 {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<ParserLine>(),
                init_array: [ParserLine {
                    data: ::core::ptr::null::<::core::ffi::c_char>(),
                    size: 0,
                    allocated: false,
                }; 4],
            },
            conv: vimconv_T {
                vc_type: 0,
                vc_factor: 0,
                vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                vc_fail: false,
            },
        },
        pos: ParserPosition { line: 0, col: 0 },
        stack: C2Rust_Unnamed_28 {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<ParserStateItem>(),
            init_array: [ParserStateItem {
                type_0: kPTopStateParsingCommand,
                data: C2Rust_Unnamed_29 {
                    expr: C2Rust_Unnamed_30 {
                        type_0: kExprUnknown,
                    },
                },
            }; 16],
        },
        colors: ::core::ptr::null_mut::<ParserHighlight>(),
        can_continuate: false,
    };
    viml_parser_init(
        &raw mut pstate,
        Some(
            parser_simple_get_line
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut ParserLine) -> (),
        ),
        &raw mut plines_p as *mut ::core::ffi::c_void,
        colors_p,
    );
    let mut east: ExprAST = viml_pexpr_parse(&raw mut pstate, pflags);
    let ret_size: size_t = (2 as size_t)
        .wrapping_add(!east.err.msg.is_null() as ::core::ffi::c_int as size_t)
        .wrapping_add(hl as size_t)
        .wrapping_add(0 as size_t);
    let mut ret: Dict = arena_dict(arena, ret_size);
    let c2rust_fresh1 = ret.size;
    ret.size = ret.size.wrapping_add(1);
    *ret.items.offset(c2rust_fresh1 as isize) = key_value_pair {
        key: cstr_as_string(b"len\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (if pstate.pos.line == 1 as size_t {
                    parser_lines[0 as ::core::ffi::c_int as usize].size
                } else {
                    pstate.pos.col
                }) as Integer,
            },
        },
    };
    if !east.err.msg.is_null() {
        let mut err_dict: Dict = arena_dict(arena, 2 as size_t);
        let c2rust_fresh2 = err_dict.size;
        err_dict.size = err_dict.size.wrapping_add(1);
        *err_dict.items.offset(c2rust_fresh2 as isize) = key_value_pair {
            key: cstr_as_string(b"message\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: arena_string(arena, cstr_as_string(east.err.msg)),
                },
            },
        };
        let c2rust_fresh3 = err_dict.size;
        err_dict.size = err_dict.size.wrapping_add(1);
        *err_dict.items.offset(c2rust_fresh3 as isize) = key_value_pair {
            key: cstr_as_string(b"arg\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: arena_string(
                        arena,
                        String_0 {
                            data: east.err.arg as *mut ::core::ffi::c_char,
                            size: east.err.arg_len as size_t,
                        },
                    ),
                },
            },
        };
        let c2rust_fresh4 = ret.size;
        ret.size = ret.size.wrapping_add(1);
        *ret.items.offset(c2rust_fresh4 as isize) = key_value_pair {
            key: cstr_as_string(b"error\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: err_dict },
            },
        };
    }
    if hl {
        let mut hl_arr: Array = arena_array(arena, colors.size);
        let mut i_0: size_t = 0 as size_t;
        while i_0 < colors.size {
            let chunk: ParserHighlightChunk = *colors.items.offset(i_0 as isize);
            let mut chunk_arr: Array = arena_array(arena, 4 as size_t);
            let c2rust_fresh5 = chunk_arr.size;
            chunk_arr.size = chunk_arr.size.wrapping_add(1);
            *chunk_arr.items.offset(c2rust_fresh5 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: chunk.start.line as Integer,
                },
            };
            let c2rust_fresh6 = chunk_arr.size;
            chunk_arr.size = chunk_arr.size.wrapping_add(1);
            *chunk_arr.items.offset(c2rust_fresh6 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: chunk.start.col as Integer,
                },
            };
            let c2rust_fresh7 = chunk_arr.size;
            chunk_arr.size = chunk_arr.size.wrapping_add(1);
            *chunk_arr.items.offset(c2rust_fresh7 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: chunk.end_col as Integer,
                },
            };
            let c2rust_fresh8 = chunk_arr.size;
            chunk_arr.size = chunk_arr.size.wrapping_add(1);
            *chunk_arr.items.offset(c2rust_fresh8 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(chunk.group),
                },
            };
            let c2rust_fresh9 = hl_arr.size;
            hl_arr.size = hl_arr.size.wrapping_add(1);
            *hl_arr.items.offset(c2rust_fresh9 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: chunk_arr },
            };
            i_0 = i_0.wrapping_add(1);
        }
        let c2rust_fresh10 = ret.size;
        ret.size = ret.size.wrapping_add(1);
        *ret.items.offset(c2rust_fresh10 as isize) = key_value_pair {
            key: cstr_as_string(b"highlight\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: hl_arr },
            },
        };
    }
    if colors.items != &raw mut colors.init_array as *mut ParserHighlightChunk {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut colors.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    }
    let mut ast_conv_stack: ExprASTConvStack = ExprASTConvStack {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<ExprASTConvStackItem>(),
        init_array: [ExprASTConvStackItem {
            node_p: ::core::ptr::null_mut::<*mut ExprASTNode>(),
            ret_node_p: ::core::ptr::null_mut::<Object>(),
        }; 16],
    };
    ast_conv_stack.capacity = ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
        .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
        .wrapping_div(
            (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    ast_conv_stack.size = 0 as size_t;
    ast_conv_stack.items = &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem;
    let mut ast: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    if ast_conv_stack.size == ast_conv_stack.capacity {
        ast_conv_stack.capacity = if ast_conv_stack.capacity << 1 as ::core::ffi::c_int
            > ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                .wrapping_div(
                    (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                        .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            ast_conv_stack.capacity << 1 as ::core::ffi::c_int
        } else {
            ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                .wrapping_div(
                    (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                        .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                        == 0) as ::core::ffi::c_int as size_t,
                )
        };
        ast_conv_stack.items = (if ast_conv_stack.capacity
            == ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                .wrapping_div(
                    (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                        .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            if ast_conv_stack.items
                == &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
            {
                ast_conv_stack.items as *mut ::core::ffi::c_void
            } else {
                _memcpy_free(
                    &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
                        as *mut ::core::ffi::c_void,
                    ast_conv_stack.items as *mut ::core::ffi::c_void,
                    ast_conv_stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                )
            }
        } else {
            if ast_conv_stack.items
                == &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
            {
                memcpy(
                    xmalloc(
                        ast_conv_stack
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                    ),
                    ast_conv_stack.items as *const ::core::ffi::c_void,
                    ast_conv_stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                )
            } else {
                xrealloc(
                    ast_conv_stack.items as *mut ::core::ffi::c_void,
                    ast_conv_stack
                        .capacity
                        .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                )
            }
        }) as *mut ExprASTConvStackItem;
    } else {
    };
    let c2rust_fresh11 = ast_conv_stack.size;
    ast_conv_stack.size = ast_conv_stack.size.wrapping_add(1);
    *ast_conv_stack.items.offset(c2rust_fresh11 as isize) = ExprASTConvStackItem {
        node_p: &raw mut east.root,
        ret_node_p: &raw mut ast,
    };
    while ast_conv_stack.size != 0 {
        let mut cur_item: ExprASTConvStackItem = *ast_conv_stack.items.offset(
            ast_conv_stack
                .size
                .wrapping_sub(0 as size_t)
                .wrapping_sub(1 as size_t) as isize,
        );
        let node: *mut ExprASTNode = *cur_item.node_p;
        if node.is_null() {
            '_c2rust_label: {
                if ast_conv_stack.size == 1 as size_t {
                } else {
                    __assert_fail(
                        b"kv_size(ast_conv_stack) == 1\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/vimscript.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        511 as ::core::ffi::c_uint,
                        b"Dict nvim_parse_expression(String, String, Boolean, Arena *, Error *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            ast_conv_stack.size = ast_conv_stack.size.wrapping_sub(1 as size_t);
        } else {
            if (*cur_item.ret_node_p).type_0 as ::core::ffi::c_uint
                == kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut items_size: size_t = (3 as ::core::ffi::c_int
                    + !(*node).children.is_null() as ::core::ffi::c_int
                    + ((*node).type_0 as ::core::ffi::c_uint
                        == kExprNodeOption as ::core::ffi::c_int as ::core::ffi::c_uint
                        || (*node).type_0 as ::core::ffi::c_uint
                            == kExprNodePlainIdentifier as ::core::ffi::c_int
                                as ::core::ffi::c_uint) as ::core::ffi::c_int
                    + ((*node).type_0 as ::core::ffi::c_uint
                        == kExprNodeOption as ::core::ffi::c_int as ::core::ffi::c_uint
                        || (*node).type_0 as ::core::ffi::c_uint
                            == kExprNodePlainIdentifier as ::core::ffi::c_int
                                as ::core::ffi::c_uint
                        || (*node).type_0 as ::core::ffi::c_uint
                            == kExprNodePlainKey as ::core::ffi::c_int as ::core::ffi::c_uint
                        || (*node).type_0 as ::core::ffi::c_uint
                            == kExprNodeEnvironment as ::core::ffi::c_int as ::core::ffi::c_uint)
                        as ::core::ffi::c_int
                    + ((*node).type_0 as ::core::ffi::c_uint
                        == kExprNodeRegister as ::core::ffi::c_int as ::core::ffi::c_uint)
                        as ::core::ffi::c_int
                    + 3 as ::core::ffi::c_int
                        * ((*node).type_0 as ::core::ffi::c_uint
                            == kExprNodeComparison as ::core::ffi::c_int as ::core::ffi::c_uint)
                            as ::core::ffi::c_int
                    + ((*node).type_0 as ::core::ffi::c_uint
                        == kExprNodeInteger as ::core::ffi::c_int as ::core::ffi::c_uint)
                        as ::core::ffi::c_int
                    + ((*node).type_0 as ::core::ffi::c_uint
                        == kExprNodeFloat as ::core::ffi::c_int as ::core::ffi::c_uint)
                        as ::core::ffi::c_int
                    + ((*node).type_0 as ::core::ffi::c_uint
                        == kExprNodeDoubleQuotedString as ::core::ffi::c_int as ::core::ffi::c_uint
                        || (*node).type_0 as ::core::ffi::c_uint
                            == kExprNodeSingleQuotedString as ::core::ffi::c_int
                                as ::core::ffi::c_uint) as ::core::ffi::c_int
                    + ((*node).type_0 as ::core::ffi::c_uint
                        == kExprNodeAssignment as ::core::ffi::c_int as ::core::ffi::c_uint)
                        as ::core::ffi::c_int
                    + 0 as ::core::ffi::c_int)
                    as size_t;
                let mut ret_node: Dict = arena_dict(arena, items_size);
                *cur_item.ret_node_p = object {
                    type_0: kObjectTypeDict,
                    data: C2Rust_Unnamed { dict: ret_node },
                };
            }
            let mut ret_node_0: *mut Dict = &raw mut (*cur_item.ret_node_p).data.dict;
            if !(*node).children.is_null() {
                let num_children: size_t = (1 as ::core::ffi::c_int
                    + !(*(*node).children).next.is_null() as ::core::ffi::c_int)
                    as size_t;
                let mut children_array: Array = arena_array(arena, num_children);
                let mut i_1: size_t = 0 as size_t;
                while i_1 < num_children {
                    let c2rust_fresh12 = children_array.size;
                    children_array.size = children_array.size.wrapping_add(1);
                    *children_array.items.offset(c2rust_fresh12 as isize) = object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed { boolean: false },
                    };
                    i_1 = i_1.wrapping_add(1);
                }
                let c2rust_fresh13 = (*ret_node_0).size;
                (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                *(*ret_node_0).items.offset(c2rust_fresh13 as isize) = key_value_pair {
                    key: cstr_as_string(b"children\0".as_ptr() as *const ::core::ffi::c_char),
                    value: object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed {
                            array: children_array,
                        },
                    },
                };
                if ast_conv_stack.size == ast_conv_stack.capacity {
                    ast_conv_stack.capacity = if ast_conv_stack.capacity << 1 as ::core::ffi::c_int
                        > ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                            .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                            .wrapping_div(
                                (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        ast_conv_stack.capacity << 1 as ::core::ffi::c_int
                    } else {
                        ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                            .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                            .wrapping_div(
                                (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    ast_conv_stack.items = (if ast_conv_stack.capacity
                        == ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                            .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                            .wrapping_div(
                                (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if ast_conv_stack.items
                            == &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
                        {
                            ast_conv_stack.items as *mut ::core::ffi::c_void
                        } else {
                            _memcpy_free(
                                &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
                                    as *mut ::core::ffi::c_void,
                                ast_conv_stack.items as *mut ::core::ffi::c_void,
                                ast_conv_stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                            )
                        }
                    } else {
                        if ast_conv_stack.items
                            == &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
                        {
                            memcpy(
                                xmalloc(
                                    ast_conv_stack
                                        .capacity
                                        .wrapping_mul(
                                            ::core::mem::size_of::<ExprASTConvStackItem>(),
                                        ),
                                ),
                                ast_conv_stack.items as *const ::core::ffi::c_void,
                                ast_conv_stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                            )
                        } else {
                            xrealloc(
                                ast_conv_stack.items as *mut ::core::ffi::c_void,
                                ast_conv_stack
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                            )
                        }
                    }) as *mut ExprASTConvStackItem;
                } else {
                };
                let c2rust_fresh14 = ast_conv_stack.size;
                ast_conv_stack.size = ast_conv_stack.size.wrapping_add(1);
                *ast_conv_stack.items.offset(c2rust_fresh14 as isize) = ExprASTConvStackItem {
                    node_p: &raw mut (*node).children,
                    ret_node_p: children_array
                        .items
                        .offset(0 as ::core::ffi::c_int as isize),
                };
            } else if !(*node).next.is_null() {
                if ast_conv_stack.size == ast_conv_stack.capacity {
                    ast_conv_stack.capacity = if ast_conv_stack.capacity << 1 as ::core::ffi::c_int
                        > ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                            .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                            .wrapping_div(
                                (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        ast_conv_stack.capacity << 1 as ::core::ffi::c_int
                    } else {
                        ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                            .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                            .wrapping_div(
                                (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    ast_conv_stack.items = (if ast_conv_stack.capacity
                        == ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                            .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                            .wrapping_div(
                                (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if ast_conv_stack.items
                            == &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
                        {
                            ast_conv_stack.items as *mut ::core::ffi::c_void
                        } else {
                            _memcpy_free(
                                &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
                                    as *mut ::core::ffi::c_void,
                                ast_conv_stack.items as *mut ::core::ffi::c_void,
                                ast_conv_stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                            )
                        }
                    } else {
                        if ast_conv_stack.items
                            == &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem
                        {
                            memcpy(
                                xmalloc(
                                    ast_conv_stack
                                        .capacity
                                        .wrapping_mul(
                                            ::core::mem::size_of::<ExprASTConvStackItem>(),
                                        ),
                                ),
                                ast_conv_stack.items as *const ::core::ffi::c_void,
                                ast_conv_stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                            )
                        } else {
                            xrealloc(
                                ast_conv_stack.items as *mut ::core::ffi::c_void,
                                ast_conv_stack
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                            )
                        }
                    }) as *mut ExprASTConvStackItem;
                } else {
                };
                let c2rust_fresh15 = ast_conv_stack.size;
                ast_conv_stack.size = ast_conv_stack.size.wrapping_add(1);
                *ast_conv_stack.items.offset(c2rust_fresh15 as isize) = ExprASTConvStackItem {
                    node_p: &raw mut (*node).next,
                    ret_node_p: cur_item.ret_node_p.offset(1 as ::core::ffi::c_int as isize),
                };
            } else {
                ast_conv_stack.size = ast_conv_stack.size.wrapping_sub(1 as size_t);
                let c2rust_fresh16 = (*ret_node_0).size;
                (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                *(*ret_node_0).items.offset(c2rust_fresh16 as isize) = key_value_pair {
                    key: cstr_as_string(b"type\0".as_ptr() as *const ::core::ffi::c_char),
                    value: object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed {
                            string: cstr_as_string(
                                *(&raw const east_node_type_tab
                                    as *const *const ::core::ffi::c_char)
                                    .offset((*node).type_0 as isize),
                            ),
                        },
                    },
                };
                let mut start_array: Array = arena_array(arena, 2 as size_t);
                let c2rust_fresh17 = start_array.size;
                start_array.size = start_array.size.wrapping_add(1);
                *start_array.items.offset(c2rust_fresh17 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (*node).start.line as Integer,
                    },
                };
                let c2rust_fresh18 = start_array.size;
                start_array.size = start_array.size.wrapping_add(1);
                *start_array.items.offset(c2rust_fresh18 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (*node).start.col as Integer,
                    },
                };
                let c2rust_fresh19 = (*ret_node_0).size;
                (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                *(*ret_node_0).items.offset(c2rust_fresh19 as isize) = key_value_pair {
                    key: cstr_as_string(b"start\0".as_ptr() as *const ::core::ffi::c_char),
                    value: object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed { array: start_array },
                    },
                };
                let c2rust_fresh20 = (*ret_node_0).size;
                (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                *(*ret_node_0).items.offset(c2rust_fresh20 as isize) = key_value_pair {
                    key: cstr_as_string(b"len\0".as_ptr() as *const ::core::ffi::c_char),
                    value: object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed {
                            integer: (*node).len as Integer,
                        },
                    },
                };
                match (*node).type_0 as ::core::ffi::c_uint {
                    27 | 26 => {
                        let mut str: Object = object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed {
                                string: arena_string(
                                    arena,
                                    String_0 {
                                        data: (*node).data.str.value,
                                        size: (*node).data.str.size,
                                    },
                                ),
                            },
                        };
                        let c2rust_fresh21 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh21 as isize) = key_value_pair {
                            key: cstr_as_string(b"svalue\0".as_ptr() as *const ::core::ffi::c_char),
                            value: str,
                        };
                        xfree((*node).data.str.value as *mut ::core::ffi::c_void);
                    }
                    36 => {
                        let c2rust_fresh22 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh22 as isize) = key_value_pair {
                            key: cstr_as_string(b"scope\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeInteger,
                                data: C2Rust_Unnamed {
                                    integer: (*node).data.opt.scope as Integer,
                                },
                            },
                        };
                        let c2rust_fresh23 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh23 as isize) = key_value_pair {
                            key: cstr_as_string(b"ident\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeString,
                                data: C2Rust_Unnamed {
                                    string: arena_string(
                                        arena,
                                        String_0 {
                                            data: (*node).data.opt.ident
                                                as *mut ::core::ffi::c_char,
                                            size: (*node).data.opt.ident_len,
                                        },
                                    ),
                                },
                            },
                        };
                    }
                    11 => {
                        let c2rust_fresh24 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh24 as isize) = key_value_pair {
                            key: cstr_as_string(b"scope\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeInteger,
                                data: C2Rust_Unnamed {
                                    integer: (*node).data.var.scope as Integer,
                                },
                            },
                        };
                        let c2rust_fresh25 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh25 as isize) = key_value_pair {
                            key: cstr_as_string(b"ident\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeString,
                                data: C2Rust_Unnamed {
                                    string: arena_string(
                                        arena,
                                        String_0 {
                                            data: (*node).data.var.ident
                                                as *mut ::core::ffi::c_char,
                                            size: (*node).data.var.ident_len,
                                        },
                                    ),
                                },
                            },
                        };
                    }
                    12 => {
                        let c2rust_fresh26 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh26 as isize) = key_value_pair {
                            key: cstr_as_string(b"ident\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeString,
                                data: C2Rust_Unnamed {
                                    string: arena_string(
                                        arena,
                                        String_0 {
                                            data: (*node).data.var.ident
                                                as *mut ::core::ffi::c_char,
                                            size: (*node).data.var.ident_len,
                                        },
                                    ),
                                },
                            },
                        };
                    }
                    37 => {
                        let c2rust_fresh27 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh27 as isize) = key_value_pair {
                            key: cstr_as_string(b"ident\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeString,
                                data: C2Rust_Unnamed {
                                    string: arena_string(
                                        arena,
                                        String_0 {
                                            data: (*node).data.env.ident
                                                as *mut ::core::ffi::c_char,
                                            size: (*node).data.env.ident_len,
                                        },
                                    ),
                                },
                            },
                        };
                    }
                    4 => {
                        let c2rust_fresh28 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh28 as isize) = key_value_pair {
                            key: cstr_as_string(b"name\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeInteger,
                                data: C2Rust_Unnamed {
                                    integer: (*node).data.reg.name as Integer,
                                },
                            },
                        };
                    }
                    21 => {
                        let c2rust_fresh29 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh29 as isize) = key_value_pair {
                            key: cstr_as_string(
                                b"cmp_type\0".as_ptr() as *const ::core::ffi::c_char
                            ),
                            value: object {
                                type_0: kObjectTypeString,
                                data: C2Rust_Unnamed {
                                    string: cstr_as_string(
                                        *(&raw const eltkn_cmp_type_tab
                                            as *const *const ::core::ffi::c_char)
                                            .offset((*node).data.cmp.type_0 as isize),
                                    ),
                                },
                            },
                        };
                        let c2rust_fresh30 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh30 as isize) = key_value_pair {
                            key: cstr_as_string(
                                b"ccs_strategy\0".as_ptr() as *const ::core::ffi::c_char
                            ),
                            value: object {
                                type_0: kObjectTypeString,
                                data: C2Rust_Unnamed {
                                    string: cstr_as_string(
                                        *(&raw const ccs_tab as *const *const ::core::ffi::c_char)
                                            .offset((*node).data.cmp.ccs as isize),
                                    ),
                                },
                            },
                        };
                        let c2rust_fresh31 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh31 as isize) = key_value_pair {
                            key: cstr_as_string(b"invert\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeBoolean,
                                data: C2Rust_Unnamed {
                                    boolean: (*node).data.cmp.inv,
                                },
                            },
                        };
                    }
                    25 => {
                        let c2rust_fresh32 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh32 as isize) = key_value_pair {
                            key: cstr_as_string(b"fvalue\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeFloat,
                                data: C2Rust_Unnamed {
                                    floating: (*node).data.flt.value,
                                },
                            },
                        };
                    }
                    24 => {
                        let c2rust_fresh33 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh33 as isize) = key_value_pair {
                            key: cstr_as_string(b"ivalue\0".as_ptr() as *const ::core::ffi::c_char),
                            value: object {
                                type_0: kObjectTypeInteger,
                                data: C2Rust_Unnamed {
                                    integer: if (*node).data.num.value
                                        > 9223372036854775807 as uvarnumber_T
                                    {
                                        9223372036854775807 as Integer
                                    } else {
                                        (*node).data.num.value as Integer
                                    },
                                },
                            },
                        };
                    }
                    38 => {
                        let asgn_type: ExprAssignmentType = (*node).data.ass.type_0;
                        let mut str_0: String_0 = if asgn_type as ::core::ffi::c_uint
                            == kExprAsgnPlain as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            STRING_INIT
                        } else {
                            cstr_as_string(
                                *(&raw const expr_asgn_type_tab
                                    as *const *const ::core::ffi::c_char)
                                    .offset(asgn_type as isize),
                            )
                        };
                        let c2rust_fresh34 = (*ret_node_0).size;
                        (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                        *(*ret_node_0).items.offset(c2rust_fresh34 as isize) = key_value_pair {
                            key: cstr_as_string(
                                b"augmentation\0".as_ptr() as *const ::core::ffi::c_char
                            ),
                            value: object {
                                type_0: kObjectTypeString,
                                data: C2Rust_Unnamed { string: str_0 },
                            },
                        };
                    }
                    0 | 1 | 2 | 3 | 5 | 6 | 7 | 8 | 9 | 10 | 13 | 14 | 15 | 16 | 17 | 18 | 19
                    | 20 | 22 | 23 | 28 | 29 | 30 | 31 | 32 | 33 | 34 | 35 | _ => {}
                }
                '_c2rust_label_0: {
                    if (*cur_item.ret_node_p).data.dict.size
                        == (*cur_item.ret_node_p).data.dict.capacity
                    {
                    } else {
                        __assert_fail(
                            b"cur_item.ret_node_p->data.dict.size == cur_item.ret_node_p->data.dict.capacity\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/api/vimscript.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            640 as ::core::ffi::c_uint,
                            b"Dict nvim_parse_expression(String, String, Boolean, Arena *, Error *)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                xfree(*cur_item.node_p as *mut ::core::ffi::c_void);
                *cur_item.node_p = ::core::ptr::null_mut::<ExprASTNode>();
            }
        }
    }
    if ast_conv_stack.items != &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem {
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut ast_conv_stack.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
    }
    let c2rust_fresh35 = ret.size;
    ret.size = ret.size.wrapping_add(1);
    *ret.items.offset(c2rust_fresh35 as isize) = key_value_pair {
        key: cstr_as_string(b"ast\0".as_ptr() as *const ::core::ffi::c_char),
        value: ast,
    };
    '_c2rust_label_1: {
        if ret.size == ret.capacity {
        } else {
            __assert_fail(
                b"ret.size == ret.capacity\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/api/vimscript.rs\0".as_ptr() as *const ::core::ffi::c_char,
                649 as ::core::ffi::c_uint,
                b"Dict nvim_parse_expression(String, String, Boolean, Arena *, Error *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    viml_pexpr_free_ast(east);
    viml_parser_destroy(&raw mut pstate);
    return ret;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const FUNCEXE_INIT: funcexe_T = funcexe_T {
    fe_argv_func: None,
    fe_firstline: 0 as linenr_T,
    fe_lastline: 0 as linenr_T,
    fe_doesrange: ::core::ptr::null_mut::<bool>(),
    fe_evaluate: false_0 != 0,
    fe_partial: ::core::ptr::null_mut::<partial_T>(),
    fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
    fe_basetv: ::core::ptr::null_mut::<typval_T>(),
    fe_found_var: false_0 != 0,
};
#[inline(always)]
unsafe extern "C" fn viml_parser_init(
    ret_pstate: *mut ParserState,
    get_line: ParserLineGetter,
    cookie: *mut ::core::ffi::c_void,
    colors: *mut ParserHighlight,
) {
    *ret_pstate = ParserState {
        reader: ParserInputReader {
            get_line: get_line,
            cookie: cookie,
            lines: C2Rust_Unnamed_33 {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<ParserLine>(),
                init_array: [ParserLine {
                    data: ::core::ptr::null::<::core::ffi::c_char>(),
                    size: 0,
                    allocated: false,
                }; 4],
            },
            conv: vimconv_T {
                vc_type: CONV_NONE as ::core::ffi::c_int,
                vc_factor: 1 as ::core::ffi::c_int,
                vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                vc_fail: false_0 != 0,
            },
        },
        pos: ParserPosition {
            line: 0 as size_t,
            col: 0 as size_t,
        },
        stack: C2Rust_Unnamed_28 {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<ParserStateItem>(),
            init_array: [ParserStateItem {
                type_0: kPTopStateParsingCommand,
                data: C2Rust_Unnamed_29 {
                    expr: C2Rust_Unnamed_30 {
                        type_0: kExprUnknown,
                    },
                },
            }; 16],
        },
        colors: colors,
        can_continuate: false_0 != 0,
    };
    (*ret_pstate).reader.lines.capacity = ::core::mem::size_of::<[ParserLine; 4]>()
        .wrapping_div(::core::mem::size_of::<ParserLine>())
        .wrapping_div(
            (::core::mem::size_of::<[ParserLine; 4]>()
                .wrapping_rem(::core::mem::size_of::<ParserLine>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    (*ret_pstate).reader.lines.size = 0 as size_t;
    (*ret_pstate).reader.lines.items =
        &raw mut (*ret_pstate).reader.lines.init_array as *mut ParserLine;
    (*ret_pstate).stack.capacity = ::core::mem::size_of::<[ParserStateItem; 16]>()
        .wrapping_div(::core::mem::size_of::<ParserStateItem>())
        .wrapping_div(
            (::core::mem::size_of::<[ParserStateItem; 16]>()
                .wrapping_rem(::core::mem::size_of::<ParserStateItem>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    (*ret_pstate).stack.size = 0 as size_t;
    (*ret_pstate).stack.items = &raw mut (*ret_pstate).stack.init_array as *mut ParserStateItem;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
