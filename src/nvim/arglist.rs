use crate::src::nvim::autocmd::is_aucmd_win;
use crate::src::nvim::buffer::{
    buf_hide, buf_is_empty, buf_set_name, buflist_add, buflist_findnr, bufref_valid,
    curbuf_reusable, maketitle, otherfile, set_bufref,
};
use crate::src::nvim::charset::{rem_backslash, skipwhite};
use crate::src::nvim::eval::typval::{
    tv_get_number, tv_get_number_chk, tv_list_alloc_ret, tv_list_append_string,
};
use crate::src::nvim::eval::window::{find_tabwin, find_win_by_nr_or_id};
use crate::src::nvim::ex_cmds::do_ecmd;
use crate::src::nvim::ex_cmds2::{autowrite, check_changed};
use crate::src::nvim::ex_getln::gotocmdline;
use crate::src::nvim::fileio::file_pat_to_reg_pat;
use crate::src::nvim::garray::{ga_clear, ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    arg_had_last, autocmd_no_enter, autocmd_no_leave, cmdmod, cmdwin_type, curbuf, curtab, curwin,
    e_cannot_change_arglist_recursively, e_cmdwin, e_invarg, e_invrange, e_nomatch, e_nomatch2,
    first_tabpage, firstwin, global_alist, got_int, lastused_tabpage, lastwin, max_alist_id, p_ea,
    p_fic, p_tpm, tabpage_move_disallowed, Columns,
};
use crate::src::nvim::mark::{setmark, setpcmark};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xstrdup};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::normal::reset_VIsual_and_resel;
use crate::src::nvim::option::magic_isset;
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::libc::{__assert_fail, gettext, memmove};
use crate::src::nvim::path::{
    expand_wildcards, fix_fname, gen_expand_wildcards, path_fnamecmp, path_full_compare,
    FullName_save,
};
pub use crate::src::nvim::types::{
    __time_t, aentry_T, alist_T, bhdr_T, bln_values, blob_T, blobvar_S, blocknr_T, buf_T, bufref_T,
    bufstate_T, chunksize_T, cmd_addr_T, cmdidx_T, cmdmod_T, colnr_T, cstack_T,
    cstack_T_cs_pend as C2Rust_Unnamed_12, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T,
    eslist_T, eslist_elem, exarg, exarg_T, expand_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_2, file_buffer_b_wininfo as C2Rust_Unnamed_10,
    file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, file_comparison, float_T, fmark_T, fmarkv_T,
    frame_S, frame_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T,
    handle_T, hash_T, hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T,
    mtnode_inner_s, mtnode_s, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t,
    ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T, regmatch_T, regmmatch_T, regprog,
    regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t, ssize_t, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T, synblock_T, synstate_T, tabpage_S,
    tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T,
    u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_7,
    u_header_uh_alt_prev as C2Rust_Unnamed_6, u_header_uh_next as C2Rust_Unnamed_9,
    u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T,
    wline_T, xfmark_T, xp_prefix_T, AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks,
    CMD_index, Callback, CallbackType, Callback_data as C2Rust_Unnamed_4, ChangedtickDictItem,
    DecorExt, DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_1, Direction, EvalFuncData, ExtmarkUndoObject,
    FileComparison, FileID, FloatAnchor, FloatRelative, GridView, Intersection, LineGetter, LuaRef,
    MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, MsgpackRpcRequestHandler, OptInt, ScopeDictDictItem, ScopeType,
    ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_11, Terminal, Timestamp, VarLockStatus, VarType,
    VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle,
    Window, QUEUE,
};
use crate::src::nvim::undo::bufIsChanged;
use crate::src::nvim::version::list_in_columns;
use crate::src::nvim::window::{
    check_can_set_curbuf_forceit, goto_tabpage_tp, lastwin_nofloating, tabpage_index,
    valid_tabpage, win_close, win_enter, win_move_after, win_split, win_valid,
};
extern "C" {
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
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
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
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
pub const BLN_CURBUF: bln_values = 1;
pub const BLN_LISTED: bln_values = 2;
pub const EW_NOTWILD: C2Rust_Unnamed_16 = 1024;
pub const EW_NOTFOUND: C2Rust_Unnamed_16 = 4;
pub const EW_FILE: C2Rust_Unnamed_16 = 2;
pub const AL_SET: C2Rust_Unnamed_18 = 1;
pub const kEqualFiles: file_comparison = 1;
pub const kEqualFileNames: file_comparison = 7;
pub const kOneFileMissing: file_comparison = 6;
pub const kBothFilesMissing: file_comparison = 4;
pub const kDifferentFiles: file_comparison = 2;
pub const AL_ADD: C2Rust_Unnamed_18 = 2;
pub const EW_ADDSLASH: C2Rust_Unnamed_16 = 8;
pub const EW_DIR: C2Rust_Unnamed_16 = 1;
pub const AL_DEL: C2Rust_Unnamed_18 = 3;
pub const ECMD_FORCEIT: C2Rust_Unnamed_13 = 8;
pub const ECMD_HIDE: C2Rust_Unnamed_13 = 1;
pub const ECMD_LAST: C2Rust_Unnamed_14 = -1;
pub const CCGD_EXCMD: C2Rust_Unnamed_15 = 16;
pub const CCGD_FORCEIT: C2Rust_Unnamed_15 = 4;
pub const CCGD_MULTWIN: C2Rust_Unnamed_15 = 2;
pub const CCGD_AW: C2Rust_Unnamed_15 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct arg_all_state_T {
    pub alist: *mut alist_T,
    pub had_tab: ::core::ffi::c_int,
    pub keep_tabs: bool,
    pub forceit: bool,
    pub use_firstwin: bool,
    pub opened: *mut uint8_t,
    pub opened_len: ::core::ffi::c_int,
    pub new_curwin: *mut win_T,
    pub new_curtab: *mut tabpage_T,
}
pub const ECMD_OLDBUF: C2Rust_Unnamed_13 = 4;
pub const ECMD_ONE: C2Rust_Unnamed_14 = 1;
pub const WSP_BELOW: C2Rust_Unnamed_17 = 64;
pub const WSP_ROOM: C2Rust_Unnamed_17 = 1;
pub const BLN_NOCURWIN: bln_values = 128;
pub const BLN_NOOPT: bln_values = 16;
pub const BLN_NEW: bln_values = 8;
pub const BLN_DUMMY: bln_values = 4;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const ECMD_NOWINENTER: C2Rust_Unnamed_13 = 64;
pub const ECMD_ALTBUF: C2Rust_Unnamed_13 = 32;
pub const ECMD_ADDBUF: C2Rust_Unnamed_13 = 16;
pub const ECMD_SET_HELP: C2Rust_Unnamed_13 = 2;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const ECMD_LASTL: C2Rust_Unnamed_14 = 0;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const CCGD_ALLBUF: C2Rust_Unnamed_15 = 8;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const EW_NOBREAK: C2Rust_Unnamed_16 = 262144;
pub const EW_CDPATH: C2Rust_Unnamed_16 = 131072;
pub const EW_NOTENV: C2Rust_Unnamed_16 = 65536;
pub const EW_EMPTYOK: C2Rust_Unnamed_16 = 32768;
pub const EW_DODOT: C2Rust_Unnamed_16 = 16384;
pub const EW_SHELLCMD: C2Rust_Unnamed_16 = 8192;
pub const EW_ALLLINKS: C2Rust_Unnamed_16 = 4096;
pub const EW_KEEPDOLLAR: C2Rust_Unnamed_16 = 2048;
pub const EW_NOERROR: C2Rust_Unnamed_16 = 512;
pub const EW_ICASE: C2Rust_Unnamed_16 = 256;
pub const EW_PATH: C2Rust_Unnamed_16 = 128;
pub const EW_EXEC: C2Rust_Unnamed_16 = 64;
pub const EW_SILENT: C2Rust_Unnamed_16 = 32;
pub const EW_KEEPALL: C2Rust_Unnamed_16 = 16;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const WSP_QUICKFIX: C2Rust_Unnamed_17 = 1024;
pub const WSP_NOENTER: C2Rust_Unnamed_17 = 512;
pub const WSP_NEWLOC: C2Rust_Unnamed_17 = 256;
pub const WSP_ABOVE: C2Rust_Unnamed_17 = 128;
pub const WSP_HELP: C2Rust_Unnamed_17 = 32;
pub const WSP_BOT: C2Rust_Unnamed_17 = 16;
pub const WSP_TOP: C2Rust_Unnamed_17 = 8;
pub const WSP_HOR: C2Rust_Unnamed_17 = 4;
pub const WSP_VERT: C2Rust_Unnamed_17 = 2;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
static e_window_layout_changed_unexpectedly: GlobalCell<[::core::ffi::c_char; 41]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
            *b"E249: Window layout changed unexpectedly\0",
        )
    });
static arglist_locked: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
unsafe extern "C" fn check_arglist_locked() -> ::core::ffi::c_int {
    if arglist_locked.get() {
        emsg(gettext(
            &raw const e_cannot_change_arglist_recursively as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn alist_clear(mut al: *mut alist_T) {
    if check_arglist_locked() == FAIL {
        return;
    }
    let mut _gap: *mut garray_T = &raw mut (*al).al_ga;
    if !(*_gap).ga_data.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*_gap).ga_len {
            let mut _item: *mut aentry_T = ((*_gap).ga_data as *mut aentry_T).offset(i as isize);
            xfree((*_item).ae_fname as *mut ::core::ffi::c_void);
            i += 1;
        }
    }
    ga_clear(_gap);
}
pub unsafe extern "C" fn alist_init(mut al: *mut alist_T) {
    ga_init(
        &raw mut (*al).al_ga,
        ::core::mem::size_of::<aentry_T>() as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    );
}
pub unsafe extern "C" fn alist_unlink(mut al: *mut alist_T) {
    if al != global_alist.ptr() && {
        (*al).al_refcount -= 1;
        (*al).al_refcount <= 0 as ::core::ffi::c_int
    } {
        alist_clear(al);
        xfree(al as *mut ::core::ffi::c_void);
    }
}
pub unsafe extern "C" fn alist_new() {
    (*curwin.get()).w_alist = xmalloc(::core::mem::size_of::<alist_T>()) as *mut alist_T;
    (*(*curwin.get()).w_alist).al_refcount = 1 as ::core::ffi::c_int;
    (*max_alist_id.ptr()) += 1;
    (*(*curwin.get()).w_alist).id = max_alist_id.get();
    alist_init((*curwin.get()).w_alist);
}
pub unsafe extern "C" fn alist_set(
    mut al: *mut alist_T,
    mut count: ::core::ffi::c_int,
    mut files: *mut *mut ::core::ffi::c_char,
    mut use_curbuf: ::core::ffi::c_int,
    mut fnum_list: *mut ::core::ffi::c_int,
    mut fnum_len: ::core::ffi::c_int,
) {
    if check_arglist_locked() == FAIL {
        return;
    }
    alist_clear(al);
    ga_grow(&raw mut (*al).al_ga, count);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < count {
        if got_int.get() {
            while i < count {
                let c2rust_fresh0 = i;
                i = i + 1;
                xfree(*files.offset(c2rust_fresh0 as isize) as *mut ::core::ffi::c_void);
            }
            break;
        } else {
            if !fnum_list.is_null() && i < fnum_len {
                arglist_locked.set(true_0 != 0);
                buf_set_name(*fnum_list.offset(i as isize), *files.offset(i as isize));
                arglist_locked.set(false_0 != 0);
            }
            alist_add(
                al,
                *files.offset(i as isize),
                if use_curbuf != 0 {
                    2 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                },
            );
            os_breakcheck();
            i += 1;
        }
    }
    xfree(files as *mut ::core::ffi::c_void);
    if al == global_alist.ptr() {
        arg_had_last.set(false_0 != 0);
    }
}
pub unsafe extern "C" fn alist_add(
    mut al: *mut alist_T,
    mut fname: *mut ::core::ffi::c_char,
    mut set_fnum: ::core::ffi::c_int,
) {
    let mut wp: *mut win_T = curwin.get();
    if fname.is_null() {
        return;
    }
    if check_arglist_locked() == FAIL {
        return;
    }
    arglist_locked.set(true_0 != 0);
    (*wp).w_locked = true_0 != 0;
    (*((*al).al_ga.ga_data as *mut aentry_T).offset((*al).al_ga.ga_len as isize)).ae_fname = fname;
    if set_fnum > 0 as ::core::ffi::c_int {
        (*((*al).al_ga.ga_data as *mut aentry_T).offset((*al).al_ga.ga_len as isize)).ae_fnum =
            buflist_add(
                fname,
                BLN_LISTED as ::core::ffi::c_int
                    | (if set_fnum == 2 as ::core::ffi::c_int {
                        BLN_CURBUF as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }),
            );
    }
    (*al).al_ga.ga_len += 1;
    arglist_locked.set(false_0 != 0);
    (*wp).w_locked = false_0 != 0;
}
unsafe extern "C" fn do_one_arg(mut str: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut inbacktick: bool = false_0 != 0;
    p = str;
    while *str != 0 {
        if rem_backslash(str) {
            let c2rust_fresh1 = str;
            str = str.offset(1);
            let c2rust_fresh2 = p;
            p = p.offset(1);
            *c2rust_fresh2 = *c2rust_fresh1;
            let c2rust_fresh3 = p;
            p = p.offset(1);
            *c2rust_fresh3 = *str;
        } else {
            if !inbacktick && ascii_isspace(*str as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
                break;
            }
            if *str as ::core::ffi::c_int == '`' as ::core::ffi::c_int {
                inbacktick = inbacktick as ::core::ffi::c_int ^ true_0 != 0;
            }
            let c2rust_fresh4 = p;
            p = p.offset(1);
            *c2rust_fresh4 = *str;
        }
        str = str.offset(1);
    }
    str = skipwhite(str);
    *p = NUL as ::core::ffi::c_char;
    return str;
}
unsafe extern "C" fn get_arglist(
    mut gap: *mut garray_T,
    mut str: *mut ::core::ffi::c_char,
    mut escaped: bool,
) {
    ga_init(
        gap,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
    );
    while *str as ::core::ffi::c_int != NUL {
        ga_grow(gap, 1 as ::core::ffi::c_int);
        *((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset((*gap).ga_len as isize) = str;
        (*gap).ga_len += 1;
        if !escaped {
            return;
        }
        str = do_one_arg(str);
    }
}
pub unsafe extern "C" fn get_arglist_exp(
    mut str: *mut ::core::ffi::c_char,
    mut fcountp: *mut ::core::ffi::c_int,
    mut fnamesp: *mut *mut *mut ::core::ffi::c_char,
    mut wig: bool,
) -> ::core::ffi::c_int {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut i: ::core::ffi::c_int = 0;
    get_arglist(&raw mut ga, str, true_0 != 0);
    if wig {
        i = expand_wildcards(
            ga.ga_len,
            ga.ga_data as *mut *mut ::core::ffi::c_char,
            fcountp,
            fnamesp,
            EW_FILE as ::core::ffi::c_int
                | EW_NOTFOUND as ::core::ffi::c_int
                | EW_NOTWILD as ::core::ffi::c_int,
        );
    } else {
        i = gen_expand_wildcards(
            ga.ga_len,
            ga.ga_data as *mut *mut ::core::ffi::c_char,
            fcountp,
            fnamesp,
            EW_FILE as ::core::ffi::c_int
                | EW_NOTFOUND as ::core::ffi::c_int
                | EW_NOTWILD as ::core::ffi::c_int,
        );
    }
    ga_clear(&raw mut ga);
    return i;
}
unsafe extern "C" fn alist_check_arg_idx() {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut win: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !win.is_null() {
            if (*win).w_alist == (*curwin.get()).w_alist {
                check_arg_idx(win);
            }
            win = (*win).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
unsafe extern "C" fn alist_add_list(
    mut count: ::core::ffi::c_int,
    mut files: *mut *mut ::core::ffi::c_char,
    mut after: ::core::ffi::c_int,
    mut will_edit: bool,
) {
    let mut old_argcount: ::core::ffi::c_int = (*(*curwin.get()).w_alist).al_ga.ga_len;
    if check_arglist_locked() != FAIL {
        let mut wp: *mut win_T = curwin.get();
        ga_grow(&raw mut (*(*wp).w_alist).al_ga, count);
        after = if (if after > 0 as ::core::ffi::c_int {
            after
        } else {
            0 as ::core::ffi::c_int
        }) < (*(*curwin.get()).w_alist).al_ga.ga_len
        {
            if after > 0 as ::core::ffi::c_int {
                after
            } else {
                0 as ::core::ffi::c_int
            }
        } else {
            (*(*curwin.get()).w_alist).al_ga.ga_len
        };
        if after < (*(*curwin.get()).w_alist).al_ga.ga_len {
            memmove(
                ((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)
                    .offset((after + count) as isize) as *mut ::core::ffi::c_void,
                ((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T).offset(after as isize)
                    as *const ::core::ffi::c_void,
                (((*(*curwin.get()).w_alist).al_ga.ga_len - after) as size_t)
                    .wrapping_mul(::core::mem::size_of::<aentry_T>()),
            );
        }
        arglist_locked.set(true_0 != 0);
        (*wp).w_locked = true_0 != 0;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < count {
            let flags: ::core::ffi::c_int = BLN_LISTED as ::core::ffi::c_int
                | (if will_edit as ::core::ffi::c_int != 0 {
                    BLN_CURBUF as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                });
            (*((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)
                .offset((after + i) as isize))
            .ae_fname = *files.offset(i as isize);
            (*((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)
                .offset((after + i) as isize))
            .ae_fnum = buflist_add(*files.offset(i as isize), flags);
            i += 1;
        }
        arglist_locked.set(false_0 != 0);
        (*wp).w_locked = false_0 != 0;
        (*(*wp).w_alist).al_ga.ga_len += count;
        if old_argcount > 0 as ::core::ffi::c_int && (*wp).w_arg_idx >= after {
            (*wp).w_arg_idx += count;
        }
        return;
    }
}
unsafe extern "C" fn arglist_del_files(mut alist_ga: *mut garray_T) {
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    regmatch.rm_ic = p_fic.get() != 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*alist_ga).ga_len && !got_int.get() {
        let mut p: *mut ::core::ffi::c_char =
            *((*alist_ga).ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize);
        p = file_pat_to_reg_pat(
            p,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0,
        );
        if p.is_null() {
            break;
        }
        regmatch.regprog = vim_regcomp(
            p,
            if magic_isset() as ::core::ffi::c_int != 0 {
                RE_MAGIC
            } else {
                0 as ::core::ffi::c_int
            },
        );
        if regmatch.regprog.is_null() {
            xfree(p as *mut ::core::ffi::c_void);
            break;
        } else {
            let mut didone: bool = false_0 != 0;
            let mut match_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while match_0 < (*(*curwin.get()).w_alist).al_ga.ga_len {
                if vim_regexec(
                    &raw mut regmatch,
                    alist_name(
                        ((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)
                            .offset(match_0 as isize),
                    ),
                    0 as colnr_T,
                ) {
                    didone = true_0 != 0;
                    xfree(
                        (*((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)
                            .offset(match_0 as isize))
                        .ae_fname as *mut ::core::ffi::c_void,
                    );
                    memmove(
                        ((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)
                            .offset(match_0 as isize)
                            as *mut ::core::ffi::c_void,
                        ((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)
                            .offset(match_0 as isize)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as *const ::core::ffi::c_void,
                        (((*(*curwin.get()).w_alist).al_ga.ga_len
                            - match_0
                            - 1 as ::core::ffi::c_int) as size_t)
                            .wrapping_mul(::core::mem::size_of::<aentry_T>()),
                    );
                    (*(*curwin.get()).w_alist).al_ga.ga_len -= 1;
                    if (*curwin.get()).w_arg_idx > match_0 {
                        (*curwin.get()).w_arg_idx -= 1;
                    }
                    match_0 -= 1;
                }
                match_0 += 1;
            }
            vim_regfree(regmatch.regprog);
            xfree(p as *mut ::core::ffi::c_void);
            if !didone {
                semsg(
                    gettext(&raw const e_nomatch2 as *const ::core::ffi::c_char),
                    *((*alist_ga).ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize),
                );
            }
            i += 1;
        }
    }
    ga_clear(alist_ga);
}
unsafe extern "C" fn do_arglist(
    mut str: *mut ::core::ffi::c_char,
    mut what: ::core::ffi::c_int,
    mut after: ::core::ffi::c_int,
    mut will_edit: bool,
) -> ::core::ffi::c_int {
    let mut new_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut exp_count: ::core::ffi::c_int = 0;
    let mut exp_files: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut arg_escaped: bool = true_0 != 0;
    if check_arglist_locked() == FAIL {
        return FAIL;
    }
    if what == AL_ADD as ::core::ffi::c_int && *str as ::core::ffi::c_int == NUL {
        if (*curbuf.get()).b_ffname.is_null() {
            return FAIL;
        }
        str = (*curbuf.get()).b_fname;
        arg_escaped = false_0 != 0;
    }
    get_arglist(&raw mut new_ga, str, arg_escaped);
    if what == AL_DEL as ::core::ffi::c_int {
        arglist_del_files(&raw mut new_ga);
    } else {
        let mut i: ::core::ffi::c_int = expand_wildcards(
            new_ga.ga_len,
            new_ga.ga_data as *mut *mut ::core::ffi::c_char,
            &raw mut exp_count,
            &raw mut exp_files,
            EW_DIR as ::core::ffi::c_int
                | EW_FILE as ::core::ffi::c_int
                | EW_ADDSLASH as ::core::ffi::c_int
                | EW_NOTFOUND as ::core::ffi::c_int,
        );
        ga_clear(&raw mut new_ga);
        if i == FAIL || exp_count == 0 as ::core::ffi::c_int {
            emsg(gettext(&raw const e_nomatch as *const ::core::ffi::c_char));
            return FAIL;
        }
        if what == AL_ADD as ::core::ffi::c_int {
            alist_add_list(exp_count, exp_files, after, will_edit);
            xfree(exp_files as *mut ::core::ffi::c_void);
        } else {
            '_c2rust_label: {
                if what == AL_SET as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"what == AL_SET\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/arglist.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        471 as ::core::ffi::c_uint,
                        b"int do_arglist(char *, int, int, _Bool)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            alist_set(
                (*curwin.get()).w_alist,
                exp_count,
                exp_files,
                will_edit as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                0 as ::core::ffi::c_int,
            );
        }
    }
    alist_check_arg_idx();
    return OK;
}
pub unsafe extern "C" fn set_arglist(mut str: *mut ::core::ffi::c_char) {
    do_arglist(
        str,
        AL_SET as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        true_0 != 0,
    );
}
pub unsafe extern "C" fn editing_arg_idx(mut win: *mut win_T) -> bool {
    return !((*win).w_arg_idx >= (*(*win).w_alist).al_ga.ga_len
        || (*(*win).w_buffer).handle
            != (*((*(*win).w_alist).al_ga.ga_data as *mut aentry_T)
                .offset((*win).w_arg_idx as isize))
            .ae_fnum
            && ((*(*win).w_buffer).b_ffname.is_null()
                || path_full_compare(
                    alist_name(
                        ((*(*win).w_alist).al_ga.ga_data as *mut aentry_T)
                            .offset((*win).w_arg_idx as isize),
                    ),
                    (*(*win).w_buffer).b_ffname,
                    true_0 != 0,
                    true_0 != 0,
                ) as ::core::ffi::c_uint
                    & kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
                    == 0));
}
pub unsafe extern "C" fn check_arg_idx(mut win: *mut win_T) {
    if (*(*win).w_alist).al_ga.ga_len > 1 as ::core::ffi::c_int && !editing_arg_idx(win) {
        (*win).w_arg_idx_invalid = true_0;
        if (*win).w_arg_idx != (*(*win).w_alist).al_ga.ga_len - 1 as ::core::ffi::c_int
            && arg_had_last.get() as ::core::ffi::c_int == false_0
            && (*win).w_alist == global_alist.ptr()
            && (*global_alist.ptr()).al_ga.ga_len > 0 as ::core::ffi::c_int
            && (*win).w_arg_idx < (*global_alist.ptr()).al_ga.ga_len
            && ((*(*win).w_buffer).handle
                == (*((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T).offset(
                    ((*global_alist.ptr()).al_ga.ga_len - 1 as ::core::ffi::c_int) as isize,
                ))
                .ae_fnum
                || !(*(*win).w_buffer).b_ffname.is_null()
                    && path_full_compare(
                        alist_name(
                            ((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T).offset(
                                ((*global_alist.ptr()).al_ga.ga_len - 1 as ::core::ffi::c_int)
                                    as isize,
                            ),
                        ),
                        (*(*win).w_buffer).b_ffname,
                        true_0 != 0,
                        true_0 != 0,
                    ) as ::core::ffi::c_uint
                        & kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0)
        {
            arg_had_last.set(true_0 != 0);
        }
    } else {
        (*win).w_arg_idx_invalid = false_0;
        if (*win).w_arg_idx == (*(*win).w_alist).al_ga.ga_len - 1 as ::core::ffi::c_int
            && (*win).w_alist == global_alist.ptr()
        {
            arg_had_last.set(true_0 != 0);
        }
    };
}
pub unsafe extern "C" fn ex_args(mut eap: *mut exarg_T) {
    if (*eap).cmdidx as ::core::ffi::c_int != CMD_args as ::core::ffi::c_int {
        if check_arglist_locked() == FAIL {
            return;
        }
        alist_unlink((*curwin.get()).w_alist);
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_argglobal as ::core::ffi::c_int {
            (*curwin.get()).w_alist = global_alist.ptr();
        } else {
            alist_new();
        }
    }
    if *(*eap).arg as ::core::ffi::c_int != NUL {
        if check_arglist_locked() == FAIL {
            return;
        }
        ex_next(eap);
        return;
    }
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_args as ::core::ffi::c_int {
        if (*(*curwin.get()).w_alist).al_ga.ga_len <= 0 as ::core::ffi::c_int {
            return;
        }
        let mut items: *mut *mut ::core::ffi::c_char = xmalloc(
            ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                .wrapping_mul((*(*curwin.get()).w_alist).al_ga.ga_len as size_t),
        ) as *mut *mut ::core::ffi::c_char;
        gotocmdline(true_0 != 0);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*(*curwin.get()).w_alist).al_ga.ga_len {
            *items.offset(i as isize) = alist_name(
                ((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T).offset(i as isize),
            );
            i += 1;
        }
        list_in_columns(
            items,
            (*(*curwin.get()).w_alist).al_ga.ga_len,
            (*curwin.get()).w_arg_idx,
        );
        xfree(items as *mut ::core::ffi::c_void);
        return;
    }
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_arglocal as ::core::ffi::c_int {
        let mut gap: *mut garray_T = &raw mut (*(*curwin.get()).w_alist).al_ga;
        ga_grow(gap, (*global_alist.ptr()).al_ga.ga_len);
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*global_alist.ptr()).al_ga.ga_len {
            if !(*((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T).offset(i_0 as isize))
                .ae_fname
                .is_null()
            {
                (*((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)
                    .offset((*gap).ga_len as isize))
                .ae_fname = xstrdup(
                    (*((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T).offset(i_0 as isize))
                        .ae_fname,
                );
                (*((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)
                    .offset((*gap).ga_len as isize))
                .ae_fnum = (*((*global_alist.ptr()).al_ga.ga_data as *mut aentry_T)
                    .offset(i_0 as isize))
                .ae_fnum;
                (*gap).ga_len += 1;
            }
            i_0 += 1;
        }
    }
}
pub unsafe extern "C" fn ex_previous(mut eap: *mut exarg_T) {
    if (*curwin.get()).w_arg_idx - (*eap).line2 as ::core::ffi::c_int
        >= (*(*curwin.get()).w_alist).al_ga.ga_len
    {
        do_argfile(
            eap,
            (*(*curwin.get()).w_alist).al_ga.ga_len - 1 as ::core::ffi::c_int,
        );
    } else {
        do_argfile(
            eap,
            (*curwin.get()).w_arg_idx - (*eap).line2 as ::core::ffi::c_int,
        );
    };
}
pub unsafe extern "C" fn ex_rewind(mut eap: *mut exarg_T) {
    do_argfile(eap, 0 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn ex_last(mut eap: *mut exarg_T) {
    do_argfile(
        eap,
        (*(*curwin.get()).w_alist).al_ga.ga_len - 1 as ::core::ffi::c_int,
    );
}
pub unsafe extern "C" fn ex_argument(mut eap: *mut exarg_T) {
    let mut i: ::core::ffi::c_int = 0;
    if (*eap).addr_count > 0 as ::core::ffi::c_int {
        i = (*eap).line2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    } else {
        i = (*curwin.get()).w_arg_idx;
    }
    do_argfile(eap, i);
}
pub unsafe extern "C" fn do_argfile(mut eap: *mut exarg_T, mut argn: ::core::ffi::c_int) {
    let mut is_split_cmd: bool = *(*eap).cmd as ::core::ffi::c_int == 's' as ::core::ffi::c_int;
    let mut old_arg_idx: ::core::ffi::c_int = (*curwin.get()).w_arg_idx;
    if argn < 0 as ::core::ffi::c_int || argn >= (*(*curwin.get()).w_alist).al_ga.ga_len {
        if (*(*curwin.get()).w_alist).al_ga.ga_len <= 1 as ::core::ffi::c_int {
            emsg(gettext(
                b"E163: There is only one file to edit\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else if argn < 0 as ::core::ffi::c_int {
            emsg(gettext(
                b"E164: Cannot go before first file\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else {
            emsg(gettext(
                b"E165: Cannot go beyond last file\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        return;
    }
    if !is_split_cmd
        && (*((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T).offset(argn as isize))
            .ae_fnum
            != (*curbuf.get()).handle
        && !check_can_set_curbuf_forceit((*eap).forceit)
    {
        return;
    }
    setpcmark();
    if is_split_cmd as ::core::ffi::c_int != 0
        || (*cmdmod.ptr()).cmod_tab != 0 as ::core::ffi::c_int
    {
        if win_split(0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) == FAIL {
            return;
        }
        (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
        (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
    } else {
        let mut other: ::core::ffi::c_int = true_0;
        if buf_hide(curbuf.get()) {
            let mut p: *mut ::core::ffi::c_char = fix_fname(alist_name(
                ((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T).offset(argn as isize),
            ));
            other = otherfile(p) as ::core::ffi::c_int;
            xfree(p as *mut ::core::ffi::c_void);
        }
        if (!buf_hide(curbuf.get()) || other == 0)
            && check_changed(
                curbuf.get(),
                CCGD_AW as ::core::ffi::c_int
                    | (if other != 0 {
                        0 as ::core::ffi::c_int
                    } else {
                        CCGD_MULTWIN as ::core::ffi::c_int
                    })
                    | (if (*eap).forceit != 0 {
                        CCGD_FORCEIT as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    })
                    | CCGD_EXCMD as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
        {
            return;
        }
    }
    (*curwin.get()).w_arg_idx = argn;
    if argn == (*(*curwin.get()).w_alist).al_ga.ga_len - 1 as ::core::ffi::c_int
        && (*curwin.get()).w_alist == global_alist.ptr()
    {
        arg_had_last.set(true_0 != 0);
    }
    if do_ecmd(
        0 as ::core::ffi::c_int,
        alist_name(
            ((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)
                .offset((*curwin.get()).w_arg_idx as isize),
        ),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        eap,
        ECMD_LAST as ::core::ffi::c_int as linenr_T,
        (if buf_hide((*curwin.get()).w_buffer) as ::core::ffi::c_int != 0 {
            ECMD_HIDE as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) + (if (*eap).forceit != 0 {
            ECMD_FORCEIT as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }),
        curwin.get(),
    ) == FAIL
    {
        (*curwin.get()).w_arg_idx = old_arg_idx;
    } else if (*eap).cmdidx as ::core::ffi::c_int != CMD_argdo as ::core::ffi::c_int {
        setmark('\'' as ::core::ffi::c_int);
    }
}
pub unsafe extern "C" fn ex_next(mut eap: *mut exarg_T) {
    if buf_hide(curbuf.get()) as ::core::ffi::c_int != 0
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_snext as ::core::ffi::c_int
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
        let mut i: ::core::ffi::c_int = 0;
        if *(*eap).arg as ::core::ffi::c_int != NUL {
            if do_arglist(
                (*eap).arg,
                AL_SET as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                true_0 != 0,
            ) == FAIL
            {
                return;
            }
            i = 0 as ::core::ffi::c_int;
        } else {
            i = (*curwin.get()).w_arg_idx + (*eap).line2 as ::core::ffi::c_int;
        }
        do_argfile(eap, i);
    }
}
pub unsafe extern "C" fn ex_argdedupe(mut _eap: *mut exarg_T) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*(*curwin.get()).w_alist).al_ga.ga_len {
        let mut firstFullname: *mut ::core::ffi::c_char = FullName_save(
            (*((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T).offset(i as isize))
                .ae_fname,
            false_0 != 0,
        );
        let mut j: ::core::ffi::c_int = i + 1 as ::core::ffi::c_int;
        while j < (*(*curwin.get()).w_alist).al_ga.ga_len {
            let mut secondFullname: *mut ::core::ffi::c_char = FullName_save(
                (*((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T).offset(j as isize))
                    .ae_fname,
                false_0 != 0,
            );
            let mut areNamesDuplicate: bool =
                path_fnamecmp(firstFullname, secondFullname) == 0 as ::core::ffi::c_int;
            xfree(secondFullname as *mut ::core::ffi::c_void);
            if areNamesDuplicate {
                xfree(
                    (*((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)
                        .offset(j as isize))
                    .ae_fname as *mut ::core::ffi::c_void,
                );
                memmove(
                    ((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T).offset(j as isize)
                        as *mut ::core::ffi::c_void,
                    ((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)
                        .offset(j as isize)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *const ::core::ffi::c_void,
                    (((*(*curwin.get()).w_alist).al_ga.ga_len - j - 1 as ::core::ffi::c_int)
                        as size_t)
                        .wrapping_mul(::core::mem::size_of::<aentry_T>()),
                );
                (*(*curwin.get()).w_alist).al_ga.ga_len -= 1;
                if (*curwin.get()).w_arg_idx == j {
                    (*curwin.get()).w_arg_idx = i;
                } else if (*curwin.get()).w_arg_idx > j {
                    (*curwin.get()).w_arg_idx -= 1;
                }
                j -= 1;
            }
            j += 1;
        }
        xfree(firstFullname as *mut ::core::ffi::c_void);
        i += 1;
    }
}
pub unsafe extern "C" fn ex_argedit(mut eap: *mut exarg_T) {
    let mut i: ::core::ffi::c_int = if (*eap).addr_count != 0 {
        (*eap).line2 as ::core::ffi::c_int
    } else {
        (*curwin.get()).w_arg_idx + 1 as ::core::ffi::c_int
    };
    let mut curbuf_is_reusable: bool = curbuf_reusable();
    if do_arglist((*eap).arg, AL_ADD as ::core::ffi::c_int, i, true_0 != 0) == FAIL {
        return;
    }
    maketitle();
    if (*curwin.get()).w_arg_idx == 0 as ::core::ffi::c_int
        && (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0
        && ((*curbuf.get()).b_ffname.is_null() || curbuf_is_reusable as ::core::ffi::c_int != 0)
    {
        i = 0 as ::core::ffi::c_int;
    }
    if i < (*(*curwin.get()).w_alist).al_ga.ga_len {
        do_argfile(eap, i);
    }
}
pub unsafe extern "C" fn ex_argadd(mut eap: *mut exarg_T) {
    do_arglist(
        (*eap).arg,
        AL_ADD as ::core::ffi::c_int,
        if (*eap).addr_count > 0 as ::core::ffi::c_int {
            (*eap).line2 as ::core::ffi::c_int
        } else {
            (*curwin.get()).w_arg_idx + 1 as ::core::ffi::c_int
        },
        false_0 != 0,
    );
    maketitle();
}
pub unsafe extern "C" fn ex_argdelete(mut eap: *mut exarg_T) {
    if check_arglist_locked() == FAIL {
        return;
    }
    if (*eap).addr_count > 0 as ::core::ffi::c_int || *(*eap).arg as ::core::ffi::c_int == NUL {
        if (*eap).addr_count == 0 as ::core::ffi::c_int {
            if (*curwin.get()).w_arg_idx >= (*(*curwin.get()).w_alist).al_ga.ga_len {
                emsg(gettext(
                    b"E610: No argument to delete\0".as_ptr() as *const ::core::ffi::c_char
                ));
                return;
            }
            (*eap).line2 = ((*curwin.get()).w_arg_idx + 1 as ::core::ffi::c_int) as linenr_T;
            (*eap).line1 = (*eap).line2;
        } else if (*eap).line2 > (*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T {
            (*eap).line2 = (*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T;
        }
        let mut n: linenr_T = (*eap).line2 - (*eap).line1 + 1 as linenr_T;
        if *(*eap).arg as ::core::ffi::c_int != NUL {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        } else if n <= 0 as linenr_T {
            if (*eap).line1 != 1 as linenr_T || (*eap).line2 != 0 as linenr_T {
                emsg(gettext(&raw const e_invrange as *const ::core::ffi::c_char));
            }
        } else {
            let mut i: linenr_T = (*eap).line1;
            while i <= (*eap).line2 {
                xfree(
                    (*((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)
                        .offset((i - 1 as linenr_T) as isize))
                    .ae_fname as *mut ::core::ffi::c_void,
                );
                i += 1;
            }
            memmove(
                ((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)
                    .offset((*eap).line1 as isize)
                    .offset(-(1 as ::core::ffi::c_int as isize))
                    as *mut ::core::ffi::c_void,
                ((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)
                    .offset((*eap).line2 as isize) as *const ::core::ffi::c_void,
                (((*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T - (*eap).line2) as size_t)
                    .wrapping_mul(::core::mem::size_of::<aentry_T>()),
            );
            (*(*curwin.get()).w_alist).al_ga.ga_len -= n as ::core::ffi::c_int;
            if (*curwin.get()).w_arg_idx as linenr_T >= (*eap).line2 {
                (*curwin.get()).w_arg_idx -= n as ::core::ffi::c_int;
            } else if (*curwin.get()).w_arg_idx as linenr_T > (*eap).line1 {
                (*curwin.get()).w_arg_idx = (*eap).line1 as ::core::ffi::c_int;
            }
            if (*(*curwin.get()).w_alist).al_ga.ga_len == 0 as ::core::ffi::c_int {
                (*curwin.get()).w_arg_idx = 0 as ::core::ffi::c_int;
            } else if (*curwin.get()).w_arg_idx >= (*(*curwin.get()).w_alist).al_ga.ga_len {
                (*curwin.get()).w_arg_idx =
                    (*(*curwin.get()).w_alist).al_ga.ga_len - 1 as ::core::ffi::c_int;
            }
        }
    } else {
        do_arglist(
            (*eap).arg,
            AL_DEL as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            false_0 != 0,
        );
    }
    maketitle();
}
pub unsafe extern "C" fn get_arglist_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx >= (*(*curwin.get()).w_alist).al_ga.ga_len {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return alist_name(
        ((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T).offset(idx as isize),
    );
}
pub unsafe extern "C" fn alist_name(mut aep: *mut aentry_T) -> *mut ::core::ffi::c_char {
    let mut bp: *mut buf_T = buflist_findnr((*aep).ae_fnum);
    if bp.is_null() || (*bp).b_fname.is_null() {
        return (*aep).ae_fname;
    }
    return (*bp).b_fname;
}
unsafe extern "C" fn arg_all_close_unused_windows(mut aall: *mut arg_all_state_T) {
    let mut old_curwin: *mut win_T = curwin.get();
    let mut old_curtab: *mut tabpage_T = curtab.get();
    if (*aall).had_tab > 0 as ::core::ffi::c_int {
        goto_tabpage_tp(first_tabpage.get(), true_0 != 0, true_0 != 0);
    }
    (*tabpage_move_disallowed.ptr()) += 1;
    loop {
        let mut wpnext: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut tpnext: *mut tabpage_T = (*curtab.get()).tp_next;
        let mut wp: *mut win_T = if (*lastwin.get()).w_floating as ::core::ffi::c_int != 0 {
            lastwin.get()
        } else {
            firstwin.get()
        };
        while !wp.is_null() {
            let mut i: ::core::ffi::c_int = 0;
            wpnext = if (*wp).w_floating as ::core::ffi::c_int != 0 {
                if (*(*wp).w_prev).w_floating as ::core::ffi::c_int != 0 {
                    (*wp).w_prev
                } else {
                    firstwin.get()
                }
            } else if (*wp).w_next.is_null()
                || (*(*wp).w_next).w_floating as ::core::ffi::c_int != 0
            {
                ::core::ptr::null_mut::<win_T>()
            } else {
                (*wp).w_next
            };
            let mut buf: *mut buf_T = (*wp).w_buffer;
            if (*buf).b_ffname.is_null()
                || !(*aall).keep_tabs
                    && ((*buf).b_nwindows > 1 as ::core::ffi::c_int
                        || (*wp).w_width != Columns.get()
                        || (*wp).w_floating as ::core::ffi::c_int != 0 && !is_aucmd_win(wp))
            {
                i = (*aall).opened_len;
            } else {
                i = 0 as ::core::ffi::c_int;
                while i < (*aall).opened_len {
                    if i < (*(*aall).alist).al_ga.ga_len
                        && ((*((*(*aall).alist).al_ga.ga_data as *mut aentry_T).offset(i as isize))
                            .ae_fnum
                            == (*buf).handle
                            || path_full_compare(
                                alist_name(
                                    ((*(*aall).alist).al_ga.ga_data as *mut aentry_T)
                                        .offset(i as isize),
                                ),
                                (*buf).b_ffname,
                                true_0 != 0,
                                true_0 != 0,
                            ) as ::core::ffi::c_uint
                                & kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
                                != 0)
                    {
                        let mut weight: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                        if old_curtab == curtab.get() {
                            weight += 1;
                            if old_curwin == wp {
                                weight += 1;
                            }
                        }
                        if weight > *(*aall).opened.offset(i as isize) as ::core::ffi::c_int {
                            *(*aall).opened.offset(i as isize) = weight as uint8_t;
                            if i == 0 as ::core::ffi::c_int {
                                if !(*aall).new_curwin.is_null() {
                                    (*(*aall).new_curwin).w_arg_idx = (*aall).opened_len;
                                }
                                (*aall).new_curwin = wp;
                                (*aall).new_curtab = curtab.get();
                            }
                        } else if (*aall).keep_tabs {
                            i = (*aall).opened_len;
                        }
                        if (*wp).w_alist != (*aall).alist {
                            alist_unlink((*wp).w_alist);
                            (*wp).w_alist = (*aall).alist;
                            (*(*wp).w_alist).al_refcount += 1;
                        }
                        break;
                    } else {
                        i += 1;
                    }
                }
            }
            (*wp).w_arg_idx = i;
            's_31: {
                if i == (*aall).opened_len && !(*aall).keep_tabs {
                    if buf_hide(buf) as ::core::ffi::c_int != 0
                        || (*aall).forceit as ::core::ffi::c_int != 0
                        || (*buf).b_nwindows > 1 as ::core::ffi::c_int
                        || !bufIsChanged(buf)
                    {
                        if !buf_hide(buf)
                            && (*buf).b_nwindows <= 1 as ::core::ffi::c_int
                            && bufIsChanged(buf) as ::core::ffi::c_int != 0
                        {
                            let mut bufref: bufref_T = bufref_T {
                                br_buf: ::core::ptr::null_mut::<buf_T>(),
                                br_fnum: 0,
                                br_buf_free_count: 0,
                            };
                            set_bufref(&raw mut bufref, buf);
                            autowrite(buf, false_0 != 0);
                            if !win_valid(wp) || !bufref_valid(&raw mut bufref) {
                                wpnext = if (*lastwin.get()).w_floating as ::core::ffi::c_int != 0 {
                                    lastwin.get()
                                } else {
                                    firstwin.get()
                                };
                                break 's_31;
                            }
                        }
                        if firstwin.get() == lastwin.get()
                            && ((*first_tabpage.get()).tp_next.is_null() || (*aall).had_tab == 0)
                        {
                            (*aall).use_firstwin = true_0 != 0;
                        } else {
                            win_close(wp, !buf_hide(buf) && !bufIsChanged(buf), false_0 != 0);
                            if !win_valid(wpnext) {
                                wpnext = if (*lastwin.get()).w_floating as ::core::ffi::c_int != 0 {
                                    lastwin.get()
                                } else {
                                    firstwin.get()
                                };
                            }
                        }
                    }
                }
            }
            wp = wpnext;
        }
        if (*aall).had_tab == 0 as ::core::ffi::c_int || tpnext.is_null() {
            break;
        }
        if !valid_tabpage(tpnext) {
            tpnext = first_tabpage.get();
        }
        goto_tabpage_tp(tpnext, true_0 != 0, true_0 != 0);
    }
    (*tabpage_move_disallowed.ptr()) -= 1;
}
unsafe extern "C" fn arg_all_open_windows(
    mut aall: *mut arg_all_state_T,
    mut count: ::core::ffi::c_int,
) {
    let mut tab_drop_empty_window: bool = false_0 != 0;
    if (*aall).keep_tabs as ::core::ffi::c_int != 0
        && buf_is_empty(curbuf.get()) as ::core::ffi::c_int != 0
        && (*curbuf.get()).b_nwindows == 1 as ::core::ffi::c_int
        && (*curbuf.get()).b_ffname.is_null()
        && (*curbuf.get()).b_changed == 0
    {
        (*aall).use_firstwin = true_0 != 0;
        tab_drop_empty_window = true_0 != 0;
    }
    let mut split_ret: ::core::ffi::c_int = OK;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < count && !got_int.get() {
        if (*aall).alist == global_alist.ptr()
            && i == (*global_alist.ptr()).al_ga.ga_len - 1 as ::core::ffi::c_int
        {
            arg_had_last.set(true_0 != 0);
        }
        's_23: {
            if *(*aall).opened.offset(i as isize) as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
                if (*curwin.get()).w_arg_idx != i {
                    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                        firstwin.get()
                    } else {
                        (*curtab.get()).tp_firstwin
                    };
                    while !wp.is_null() {
                        if (*wp).w_arg_idx == i {
                            if (*aall).keep_tabs {
                                (*aall).new_curwin = wp;
                                (*aall).new_curtab = curtab.get();
                                break;
                            } else {
                                if (*wp).w_floating {
                                    break;
                                }
                                if (*(*wp).w_frame).fr_parent
                                    != (*(*curwin.get()).w_frame).fr_parent
                                {
                                    emsg(gettext(
                                        (e_window_layout_changed_unexpectedly.ptr() as *const _)
                                            as *const ::core::ffi::c_char,
                                    ));
                                    i = count;
                                    break;
                                } else {
                                    win_move_after(wp, curwin.get());
                                    break;
                                }
                            }
                        } else {
                            wp = (*wp).w_next;
                        }
                    }
                }
            } else if split_ret == OK {
                if tab_drop_empty_window as ::core::ffi::c_int != 0
                    && i == count - 1 as ::core::ffi::c_int
                {
                    (*autocmd_no_enter.ptr()) -= 1;
                }
                if !(*aall).use_firstwin {
                    let mut p_ea_save: bool = p_ea.get() != 0;
                    p_ea.set(true_0);
                    split_ret = win_split(
                        0 as ::core::ffi::c_int,
                        WSP_ROOM as ::core::ffi::c_int | WSP_BELOW as ::core::ffi::c_int,
                    );
                    p_ea.set(p_ea_save as ::core::ffi::c_int);
                    if split_ret == FAIL {
                        break 's_23;
                    }
                } else {
                    (*autocmd_no_leave.ptr()) -= 1;
                }
                (*curwin.get()).w_arg_idx = i;
                if i == 0 as ::core::ffi::c_int {
                    (*aall).new_curwin = curwin.get();
                    (*aall).new_curtab = curtab.get();
                }
                do_ecmd(
                    0 as ::core::ffi::c_int,
                    alist_name(
                        ((*(*aall).alist).al_ga.ga_data as *mut aentry_T).offset(i as isize),
                    ),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<exarg_T>(),
                    ECMD_ONE as ::core::ffi::c_int as linenr_T,
                    (if buf_hide((*curwin.get()).w_buffer) as ::core::ffi::c_int != 0
                        || bufIsChanged((*curwin.get()).w_buffer) as ::core::ffi::c_int != 0
                    {
                        ECMD_HIDE as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) + ECMD_OLDBUF as ::core::ffi::c_int,
                    curwin.get(),
                );
                if tab_drop_empty_window as ::core::ffi::c_int != 0
                    && i == count - 1 as ::core::ffi::c_int
                {
                    (*autocmd_no_enter.ptr()) += 1;
                }
                if (*aall).use_firstwin {
                    (*autocmd_no_leave.ptr()) += 1;
                }
                (*aall).use_firstwin = false_0 != 0;
            }
            os_breakcheck();
            if (*aall).had_tab > 0 as ::core::ffi::c_int
                && tabpage_index(::core::ptr::null_mut::<tabpage_T>()) as OptInt <= p_tpm.get()
            {
                (*cmdmod.ptr()).cmod_tab = 9999 as ::core::ffi::c_int;
            }
        }
        i += 1;
    }
}
unsafe extern "C" fn do_arg_all(
    mut count: ::core::ffi::c_int,
    mut forceit: ::core::ffi::c_int,
    mut keep_tabs: ::core::ffi::c_int,
) {
    let mut last_curwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut last_curtab: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    let mut prev_arglist_locked: bool = arglist_locked.get();
    '_c2rust_label: {
        if !(*firstwin.ptr()).is_null() {
        } else {
            __assert_fail(
                b"firstwin != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/arglist.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1068 as ::core::ffi::c_uint,
                b"void do_arg_all(int, int, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if cmdwin_type.get() != 0 as ::core::ffi::c_int {
        emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
        return;
    }
    if (*(*curwin.get()).w_alist).al_ga.ga_len <= 0 as ::core::ffi::c_int {
        return;
    }
    setpcmark();
    let mut aall: arg_all_state_T = arg_all_state_T {
        alist: ::core::ptr::null_mut::<alist_T>(),
        had_tab: (*cmdmod.ptr()).cmod_tab,
        keep_tabs: keep_tabs != 0,
        forceit: forceit != 0,
        use_firstwin: false_0 != 0,
        opened: xcalloc(
            (*(*curwin.get()).w_alist).al_ga.ga_len as size_t,
            1 as size_t,
        ) as *mut uint8_t,
        opened_len: (*(*curwin.get()).w_alist).al_ga.ga_len,
        new_curwin: ::core::ptr::null_mut::<win_T>(),
        new_curtab: ::core::ptr::null_mut::<tabpage_T>(),
    };
    aall.alist = (*curwin.get()).w_alist;
    (*aall.alist).al_refcount += 1;
    arglist_locked.set(true_0 != 0);
    let new_lu_tp: *mut tabpage_T = curtab.get();
    reset_VIsual_and_resel();
    arg_all_close_unused_windows(&raw mut aall);
    if count > aall.opened_len || count <= 0 as ::core::ffi::c_int {
        count = aall.opened_len;
    }
    (*autocmd_no_enter.ptr()) += 1;
    (*autocmd_no_leave.ptr()) += 1;
    last_curwin = curwin.get();
    last_curtab = curtab.get();
    win_enter(
        lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>()),
        false_0 != 0,
    );
    arg_all_open_windows(&raw mut aall, count);
    alist_unlink(aall.alist);
    arglist_locked.set(prev_arglist_locked);
    (*autocmd_no_enter.ptr()) -= 1;
    if last_curtab != aall.new_curtab {
        if valid_tabpage(last_curtab) {
            goto_tabpage_tp(last_curtab, true_0 != 0, true_0 != 0);
        }
        if win_valid(last_curwin) {
            win_enter(last_curwin, false_0 != 0);
        }
    }
    if valid_tabpage(aall.new_curtab) {
        goto_tabpage_tp(aall.new_curtab, true_0 != 0, true_0 != 0);
    }
    if valid_tabpage(new_lu_tp) {
        lastused_tabpage.set(new_lu_tp);
    }
    if win_valid(aall.new_curwin) {
        win_enter(aall.new_curwin, false_0 != 0);
    }
    (*autocmd_no_leave.ptr()) -= 1;
    xfree(aall.opened as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn ex_all(mut eap: *mut exarg_T) {
    if (*eap).addr_count == 0 as ::core::ffi::c_int {
        (*eap).line2 = 9999 as ::core::ffi::c_int as linenr_T;
    }
    do_arg_all(
        (*eap).line2 as ::core::ffi::c_int,
        (*eap).forceit,
        ((*eap).cmdidx as ::core::ffi::c_int == CMD_drop as ::core::ffi::c_int)
            as ::core::ffi::c_int,
    );
}
pub unsafe extern "C" fn arg_all() -> *mut ::core::ffi::c_char {
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    loop {
        let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while idx < (*(*curwin.get()).w_alist).al_ga.ga_len {
            let mut p: *mut ::core::ffi::c_char = alist_name(
                ((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T).offset(idx as isize),
            );
            if !p.is_null() {
                if len > 0 as ::core::ffi::c_int {
                    if !retval.is_null() {
                        *retval.offset(len as isize) = ' ' as ::core::ffi::c_char;
                    }
                    len += 1;
                }
                while *p as ::core::ffi::c_int != NUL {
                    if *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                        || *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                        || *p as ::core::ffi::c_int == '`' as ::core::ffi::c_int
                    {
                        if !retval.is_null() {
                            *retval.offset(len as isize) = '\\' as ::core::ffi::c_char;
                        }
                        len += 1;
                    }
                    if !retval.is_null() {
                        *retval.offset(len as isize) = *p;
                    }
                    len += 1;
                    p = p.offset(1);
                }
            }
            idx += 1;
        }
        if !retval.is_null() {
            *retval.offset(len as isize) = NUL as ::core::ffi::c_char;
            break;
        } else {
            retval = xmalloc((len as size_t).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        }
    }
    return retval;
}
pub unsafe extern "C" fn f_argc(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*rettv).vval.v_number = (*(*curwin.get()).w_alist).al_ga.ga_len as varnumber_T;
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) == -1 as varnumber_T
    {
        (*rettv).vval.v_number = (*global_alist.ptr()).al_ga.ga_len as varnumber_T;
    } else {
        let mut wp: *mut win_T =
            find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
        if !wp.is_null() {
            (*rettv).vval.v_number = (*(*wp).w_alist).al_ga.ga_len as varnumber_T;
        } else {
            (*rettv).vval.v_number = -1 as varnumber_T;
        }
    };
}
pub unsafe extern "C" fn f_argidx(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = (*curwin.get()).w_arg_idx as varnumber_T;
}
pub unsafe extern "C" fn f_arglistid(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    let mut wp: *mut win_T = find_tabwin(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        argvars.offset(1 as ::core::ffi::c_int as isize),
    );
    if !wp.is_null() {
        (*rettv).vval.v_number = (*(*wp).w_alist).id as varnumber_T;
    }
}
unsafe extern "C" fn get_arglist_as_rettv(
    mut arglist: *mut aentry_T,
    mut argcount: ::core::ffi::c_int,
    mut rettv: *mut typval_T,
) {
    tv_list_alloc_ret(rettv, argcount as ptrdiff_t);
    if !arglist.is_null() {
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while idx < argcount {
            tv_list_append_string(
                (*rettv).vval.v_list,
                alist_name(arglist.offset(idx as isize)),
                -1 as ssize_t,
            );
            idx += 1;
        }
    }
}
pub unsafe extern "C" fn f_argv(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut arglist: *mut aentry_T = ::core::ptr::null_mut::<aentry_T>();
    let mut argcount: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        get_arglist_as_rettv(
            (*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T,
            (*(*curwin.get()).w_alist).al_ga.ga_len,
            rettv,
        );
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        arglist = (*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T;
        argcount = (*(*curwin.get()).w_alist).al_ga.ga_len;
    } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) == -1 as varnumber_T
    {
        arglist = (*global_alist.ptr()).al_ga.ga_data as *mut aentry_T;
        argcount = (*global_alist.ptr()).al_ga.ga_len;
    } else {
        let mut wp: *mut win_T =
            find_win_by_nr_or_id(argvars.offset(1 as ::core::ffi::c_int as isize));
        if !wp.is_null() {
            arglist = (*(*wp).w_alist).al_ga.ga_data as *mut aentry_T;
            argcount = (*(*wp).w_alist).al_ga.ga_len;
        }
    }
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int;
    if !arglist.is_null() && idx >= 0 as ::core::ffi::c_int && idx < argcount {
        (*rettv).vval.v_string = xstrdup(alist_name(arglist.offset(idx as isize)));
    } else if idx == -1 as ::core::ffi::c_int {
        get_arglist_as_rettv(arglist, argcount, rettv);
    }
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isspace(mut c: ::core::ffi::c_int) -> bool {
    return c >= 9 as ::core::ffi::c_int && c <= 13 as ::core::ffi::c_int
        || c == ' ' as ::core::ffi::c_int;
}
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
