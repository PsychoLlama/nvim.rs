use crate::src::nvim::charset::{skiptowhite, skipwhite};
use crate::src::nvim::debugger::ex_breakadd;
use crate::src::nvim::eval::userfunc::{func_tbl_get, get_current_funccal};
use crate::src::nvim::eval::vars::set_vim_var_nr;
use crate::src::nvim::event::libuv::uv_err_name;
use crate::src::nvim::fileio::vim_fgets;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::main::{current_sctx, do_profiling, e_notopen, time_fd, IObuff};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::os::env::expand_env_save_opt;
use crate::src::nvim::os::fs::os_fopen;
use crate::src::nvim::os::libc::{
    __assert_fail, fclose, fopen, fprintf, gettext, qsort, round, setvbuf, snprintf, stderr,
    strcmp, strncmp,
};
use crate::src::nvim::os::time::os_hrtime;
use crate::src::nvim::runtime::{exestack, get_scriptname, script_items};
pub use crate::src::nvim::types::{
    Array, AutoPat, AutoPatCmd, AutoPatCmd_S, BoolVarValue, Boolean, CMD_index, Dict, Direction,
    Float, Integer, KeyValuePair, LineGetter, LuaRef, Object, ObjectType, ScopeDictDictItem,
    ScopeType, SpecialVarValue, String_0, VarLockStatus, VarType, VimVarIndex, _IO_codecvt,
    _IO_lock_t, _IO_marker, _IO_wide_data, __compar_fn_t, __off64_t, __off_t, auto_event, blob_T,
    blobvar_S, cmd_addr_T, cmdidx_T, colnr_T, cstack_T, cstack_T_cs_pend as C2Rust_Unnamed_2,
    dict_T, dictvar_S, eslist_T, eslist_elem, estack_T, estack_T_es_info as C2Rust_Unnamed_3,
    etype_T, event_T, exarg, exarg_T, except_T, except_type_T, expand_T, float_T, funccall_S,
    funccall_S_fc_fixvar as C2Rust_Unnamed, funccall_T, garray_T, hash_T, hashitem_T, hashtab_T,
    int32_t, int64_t, key_value_pair, linenr_T, list_T, listitem_S, listitem_T, listvar_S,
    listwatch_S, listwatch_T, msglist, msglist_T, object, object_data as C2Rust_Unnamed_0,
    partial_S, partial_T, pos_T, proftime_T, queue, regprog, regprog_T, scid_T, scriptitem_T,
    scriptvar_T, sctx_T, size_t, typval_T, typval_vval_union, ufunc_S, ufunc_T, uint32_t, uint64_t,
    uint8_t, varnumber_T, vim_exception, xp_prefix_T, FILE, QUEUE, _IO_FILE,
};
extern "C" {
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
}
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
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub type C2Rust_Unnamed_1 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_1 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_1 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_1 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_1 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_1 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_1 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_1 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_1 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_1 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_1 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_1 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_1 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_1 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_1 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_1 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_1 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_1 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_1 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_1 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_1 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_1 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_1 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_1 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_1 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_1 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_1 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_1 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_1 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_1 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_1 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_1 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_1 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_1 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_1 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_1 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_1 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_1 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_1 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_1 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_1 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_1 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_1 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_1 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_1 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_1 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_1 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_1 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_1 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_1 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_1 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_1 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_1 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_1 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_1 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_1 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_1 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_1 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_1 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_1 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_1 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_1 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_1 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_1 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_1 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_1 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_1 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_1 = -2;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sn_prl_T {
    pub snp_count: ::core::ffi::c_int,
    pub sn_prl_total: proftime_T,
    pub sn_prl_self: proftime_T,
}
pub const PEXP_SUBCMD: C2Rust_Unnamed_4 = 0;
pub type C2Rust_Unnamed_4 = ::core::ffi::c_uint;
pub const PEXP_FUNC: C2Rust_Unnamed_4 = 1;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const UINT64_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const _IOFBF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const PROF_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PROF_YES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PROF_PAUSED: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
static prof_wait_time: GlobalCell<proftime_T> = GlobalCell::new(0);
static startuptime_buf: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
#[no_mangle]
pub unsafe extern "C" fn profile_start() -> proftime_T {
    return os_hrtime();
}
#[no_mangle]
pub unsafe extern "C" fn profile_end(mut tm: proftime_T) -> proftime_T {
    return profile_sub(os_hrtime(), tm);
}
#[no_mangle]
pub unsafe extern "C" fn profile_msg(mut tm: proftime_T) -> *const ::core::ffi::c_char {
    static buf: GlobalCell<[::core::ffi::c_char; 50]> = GlobalCell::new([0; 50]);
    snprintf(
        buf.ptr() as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 50]>(),
        b"%10.6lf\0".as_ptr() as *const ::core::ffi::c_char,
        profile_signed(tm) as ::core::ffi::c_double / 1000000000.0f64,
    );
    return buf.ptr() as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn profile_setlimit(mut msec: int64_t) -> proftime_T {
    if msec <= 0 as int64_t {
        return profile_zero();
    }
    '_c2rust_label: {
        if msec as ::core::ffi::c_longlong
            <= 9223372036854775807 as ::core::ffi::c_longlong / 1000000 as ::core::ffi::c_longlong
                - 1 as ::core::ffi::c_longlong
        {
        } else {
            __assert_fail(
                b"msec <= (INT64_MAX / 1000000LL) - 1\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/profile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                92 as ::core::ffi::c_uint,
                b"proftime_T profile_setlimit(int64_t)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut nsec: proftime_T = (msec as proftime_T as ::core::ffi::c_ulonglong)
        .wrapping_mul(1000000 as ::core::ffi::c_ulonglong)
        as proftime_T;
    return os_hrtime().wrapping_add(nsec);
}
#[no_mangle]
pub unsafe extern "C" fn profile_passed_limit(mut tm: proftime_T) -> bool {
    if tm == 0 as proftime_T {
        return false_0 != 0;
    }
    return profile_cmp(os_hrtime(), tm) < 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn profile_zero() -> proftime_T {
    return 0 as proftime_T;
}
#[no_mangle]
pub unsafe extern "C" fn profile_divide(
    mut tm: proftime_T,
    mut count: ::core::ffi::c_int,
) -> proftime_T {
    if count <= 0 as ::core::ffi::c_int {
        return profile_zero();
    }
    return round(tm as ::core::ffi::c_double / count as ::core::ffi::c_double) as proftime_T;
}
#[no_mangle]
pub unsafe extern "C" fn profile_add(mut tm1: proftime_T, mut tm2: proftime_T) -> proftime_T {
    return tm1.wrapping_add(tm2);
}
#[no_mangle]
pub unsafe extern "C" fn profile_sub(mut tm1: proftime_T, mut tm2: proftime_T) -> proftime_T {
    return tm1.wrapping_sub(tm2);
}
#[no_mangle]
pub unsafe extern "C" fn profile_self(
    mut self_0: proftime_T,
    mut total: proftime_T,
    mut children: proftime_T,
) -> proftime_T {
    if total <= children {
        return self_0;
    }
    return profile_sub(profile_add(self_0, total), children);
}
unsafe extern "C" fn profile_get_wait() -> proftime_T {
    return prof_wait_time.get();
}
#[no_mangle]
pub unsafe extern "C" fn profile_set_wait(mut wait: proftime_T) {
    prof_wait_time.set(wait);
}
#[no_mangle]
pub unsafe extern "C" fn profile_sub_wait(mut tm: proftime_T, mut tma: proftime_T) -> proftime_T {
    let mut tm3: proftime_T = profile_sub(profile_get_wait(), tm);
    return profile_sub(tma, tm3);
}
unsafe extern "C" fn profile_equal(mut tm1: proftime_T, mut tm2: proftime_T) -> bool {
    return tm1 == tm2;
}
pub unsafe extern "C" fn profile_signed(mut tm: proftime_T) -> int64_t {
    return if tm <= INT64_MAX as proftime_T {
        tm as int64_t
    } else {
        -((UINT64_MAX as proftime_T).wrapping_sub(tm) as int64_t)
    };
}
#[no_mangle]
pub unsafe extern "C" fn profile_cmp(
    mut tm1: proftime_T,
    mut tm2: proftime_T,
) -> ::core::ffi::c_int {
    if tm1 == tm2 {
        return 0 as ::core::ffi::c_int;
    }
    return if profile_signed(tm2.wrapping_sub(tm1)) < 0 as int64_t {
        -1 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
}
static profile_fname: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub unsafe extern "C" fn profile_reset() {
    let mut id: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while id <= (*script_items.ptr()).ga_len {
        let mut si: *mut scriptitem_T = *((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
            .offset((id - 1 as ::core::ffi::c_int) as isize);
        if (*si).sn_prof_on {
            (*si).sn_prof_on = false_0 != 0;
            (*si).sn_pr_force = false_0 != 0;
            (*si).sn_pr_child = profile_zero();
            (*si).sn_pr_nest = 0 as ::core::ffi::c_int;
            (*si).sn_pr_count = 0 as ::core::ffi::c_int;
            (*si).sn_pr_total = profile_zero();
            (*si).sn_pr_self = profile_zero();
            (*si).sn_pr_start = profile_zero();
            (*si).sn_pr_children = profile_zero();
            ga_clear(&raw mut (*si).sn_prl_ga);
            (*si).sn_prl_start = profile_zero();
            (*si).sn_prl_children = profile_zero();
            (*si).sn_prl_wait = profile_zero();
            (*si).sn_prl_idx = -1 as ::core::ffi::c_int as linenr_T;
            (*si).sn_prl_execed = 0 as ::core::ffi::c_int;
        }
        id += 1;
    }
    let functbl: *mut hashtab_T = func_tbl_get();
    let mut todo: size_t = (*functbl).ht_used;
    let mut hi: *mut hashitem_T = (*functbl).ht_array;
    while todo > 0 as size_t {
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            todo = todo.wrapping_sub(1);
            let mut uf: *mut ufunc_T =
                (*hi).hi_key.offset(-(240 as ::core::ffi::c_ulong as isize)) as *mut ufunc_T;
            if (*uf).uf_prof_initialized != 0 {
                (*uf).uf_profiling = 0 as ::core::ffi::c_int;
                (*uf).uf_tm_count = 0 as ::core::ffi::c_int;
                (*uf).uf_tm_total = profile_zero();
                (*uf).uf_tm_self = profile_zero();
                (*uf).uf_tm_children = profile_zero();
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < (*uf).uf_lines.ga_len {
                    *(*uf).uf_tml_count.offset(i as isize) = 0 as ::core::ffi::c_int;
                    *(*uf).uf_tml_self.offset(i as isize) = 0 as proftime_T;
                    *(*uf).uf_tml_total.offset(i as isize) = *(*uf).uf_tml_self.offset(i as isize);
                    i += 1;
                }
                (*uf).uf_tml_start = profile_zero();
                (*uf).uf_tml_children = profile_zero();
                (*uf).uf_tml_wait = profile_zero();
                (*uf).uf_tml_idx = -1 as ::core::ffi::c_int;
                (*uf).uf_tml_execed = 0 as ::core::ffi::c_int;
            }
        }
        hi = hi.offset(1);
    }
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        profile_fname.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
}
pub unsafe extern "C" fn ex_profile(mut eap: *mut exarg_T) {
    static pause_time: GlobalCell<proftime_T> = GlobalCell::new(0);
    let mut e: *mut ::core::ffi::c_char = skiptowhite((*eap).arg);
    let mut len: ::core::ffi::c_int = e.offset_from((*eap).arg) as ::core::ffi::c_int;
    e = skipwhite(e);
    if len == 5 as ::core::ffi::c_int
        && strncmp(
            (*eap).arg,
            b"start\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        && *e as ::core::ffi::c_int != NUL
    {
        xfree(profile_fname.get() as *mut ::core::ffi::c_void);
        profile_fname.set(expand_env_save_opt(e, true_0 != 0));
        do_profiling.set(PROF_YES);
        profile_set_wait(profile_zero());
        set_vim_var_nr(VV_PROFILING, 1 as varnumber_T);
    } else if do_profiling.get() == PROF_NONE {
        emsg(gettext(
            b"E750: First use \":profile start {fname}\"\0".as_ptr() as *const ::core::ffi::c_char,
        ));
    } else if strcmp((*eap).arg, b"stop\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        profile_dump();
        do_profiling.set(PROF_NONE);
        set_vim_var_nr(VV_PROFILING, 0 as varnumber_T);
        profile_reset();
    } else if strcmp(
        (*eap).arg,
        b"pause\0".as_ptr() as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        if do_profiling.get() == PROF_YES {
            pause_time.set(profile_start());
        }
        do_profiling.set(PROF_PAUSED);
    } else if strcmp(
        (*eap).arg,
        b"continue\0".as_ptr() as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        if do_profiling.get() == PROF_PAUSED {
            pause_time.set(profile_end(pause_time.get()));
            profile_set_wait(profile_add(profile_get_wait(), pause_time.get()));
        }
        do_profiling.set(PROF_YES);
    } else if strcmp((*eap).arg, b"dump\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        profile_dump();
    } else {
        ex_breakadd(eap);
    };
}
static pexpand_what: GlobalCell<C2Rust_Unnamed_4> = GlobalCell::new(PEXP_SUBCMD);
static pexpand_cmds: GlobalCell<[*mut ::core::ffi::c_char; 8]> = GlobalCell::new([
    b"continue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"dump\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"file\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"func\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"pause\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"start\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"stop\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ::core::ptr::null_mut::<::core::ffi::c_char>(),
]);
pub unsafe extern "C" fn get_profile_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    match pexpand_what.get() as ::core::ffi::c_uint {
        0 => return (*pexpand_cmds.ptr())[idx as usize],
        _ => return ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
}
pub unsafe extern "C" fn set_context_in_profile_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) {
    (*xp).xp_context = EXPAND_PROFILE as ::core::ffi::c_int;
    pexpand_what.set(PEXP_SUBCMD);
    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
    let end_subcmd: *mut ::core::ffi::c_char = skiptowhite(arg);
    if *end_subcmd as ::core::ffi::c_int == NUL {
        return;
    }
    if end_subcmd.offset_from(arg) == 5 as isize
        && strncmp(
            arg,
            b"start\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        || end_subcmd.offset_from(arg) == 4 as isize
            && strncmp(
                arg,
                b"file\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
    {
        (*xp).xp_context = EXPAND_FILES as ::core::ffi::c_int;
        (*xp).xp_pattern = skipwhite(end_subcmd);
        return;
    } else if end_subcmd.offset_from(arg) == 4 as isize
        && strncmp(
            arg,
            b"func\0".as_ptr() as *const ::core::ffi::c_char,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        (*xp).xp_context = EXPAND_USER_FUNC as ::core::ffi::c_int;
        (*xp).xp_pattern = skipwhite(end_subcmd);
        return;
    }
    (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
}
static wait_time: GlobalCell<proftime_T> = GlobalCell::new(0);
pub unsafe extern "C" fn prof_input_start() {
    wait_time.set(profile_start());
}
pub unsafe extern "C" fn prof_input_end() {
    wait_time.set(profile_end(wait_time.get()));
    profile_set_wait(profile_add(profile_get_wait(), wait_time.get()));
}
pub unsafe extern "C" fn prof_def_func() -> bool {
    if (*current_sctx.ptr()).sc_sid > 0 as ::core::ffi::c_int {
        return (**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T).offset(
            ((*current_sctx.ptr()).sc_sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize,
        ))
        .sn_pr_force;
    }
    return false_0 != 0;
}
unsafe extern "C" fn prof_func_line(
    mut fd: *mut FILE,
    mut count: ::core::ffi::c_int,
    mut total: *const proftime_T,
    mut self_0: *const proftime_T,
    mut prefer_self: bool,
) {
    if count > 0 as ::core::ffi::c_int {
        fprintf(fd, b"%5d \0".as_ptr() as *const ::core::ffi::c_char, count);
        if prefer_self as ::core::ffi::c_int != 0
            && profile_equal(*total, *self_0) as ::core::ffi::c_int != 0
        {
            fprintf(fd, b"           \0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            fprintf(
                fd,
                b"%s \0".as_ptr() as *const ::core::ffi::c_char,
                profile_msg(*total),
            );
        }
        if !prefer_self && profile_equal(*total, *self_0) as ::core::ffi::c_int != 0 {
            fprintf(fd, b"           \0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            fprintf(
                fd,
                b"%s \0".as_ptr() as *const ::core::ffi::c_char,
                profile_msg(*self_0),
            );
        }
    } else {
        fprintf(
            fd,
            b"                            \0".as_ptr() as *const ::core::ffi::c_char,
        );
    };
}
unsafe extern "C" fn prof_sort_list(
    mut fd: *mut FILE,
    mut sorttab: *mut *mut ufunc_T,
    mut st_len: ::core::ffi::c_int,
    mut title: *mut ::core::ffi::c_char,
    mut prefer_self: bool,
) {
    fprintf(
        fd,
        b"FUNCTIONS SORTED ON %s TIME\n\0".as_ptr() as *const ::core::ffi::c_char,
        title,
    );
    fprintf(
        fd,
        b"count  total (s)   self (s)  function\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 20 as ::core::ffi::c_int && i < st_len {
        let mut fp: *mut ufunc_T = *sorttab.offset(i as isize);
        prof_func_line(
            fd,
            (*fp).uf_tm_count,
            &raw mut (*fp).uf_tm_total,
            &raw mut (*fp).uf_tm_self,
            prefer_self,
        );
        if *(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char)
            .offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            == K_SPECIAL
        {
            fprintf(
                fd,
                b" <SNR>%s()\n\0".as_ptr() as *const ::core::ffi::c_char,
                (&raw mut (*fp).uf_name as *mut ::core::ffi::c_char)
                    .offset(3 as ::core::ffi::c_int as isize),
            );
        } else {
            fprintf(
                fd,
                b" %s()\n\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
            );
        }
        i += 1;
    }
    fprintf(fd, b"\n\0".as_ptr() as *const ::core::ffi::c_char);
}
unsafe extern "C" fn prof_total_cmp(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut p1: *mut ufunc_T = *(s1 as *mut *mut ufunc_T);
    let mut p2: *mut ufunc_T = *(s2 as *mut *mut ufunc_T);
    return profile_cmp((*p1).uf_tm_total, (*p2).uf_tm_total);
}
unsafe extern "C" fn prof_self_cmp(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut p1: *mut ufunc_T = *(s1 as *mut *mut ufunc_T);
    let mut p2: *mut ufunc_T = *(s2 as *mut *mut ufunc_T);
    return profile_cmp((*p1).uf_tm_self, (*p2).uf_tm_self);
}
pub unsafe extern "C" fn func_do_profile(mut fp: *mut ufunc_T) {
    let mut len: ::core::ffi::c_int = (*fp).uf_lines.ga_len;
    if (*fp).uf_prof_initialized == 0 {
        if len == 0 as ::core::ffi::c_int {
            len = 1 as ::core::ffi::c_int;
        }
        (*fp).uf_tm_count = 0 as ::core::ffi::c_int;
        (*fp).uf_tm_self = profile_zero();
        (*fp).uf_tm_total = profile_zero();
        if (*fp).uf_tml_count.is_null() {
            (*fp).uf_tml_count =
                xcalloc(len as size_t, ::core::mem::size_of::<::core::ffi::c_int>())
                    as *mut ::core::ffi::c_int;
        }
        if (*fp).uf_tml_total.is_null() {
            (*fp).uf_tml_total =
                xcalloc(len as size_t, ::core::mem::size_of::<proftime_T>()) as *mut proftime_T;
        }
        if (*fp).uf_tml_self.is_null() {
            (*fp).uf_tml_self =
                xcalloc(len as size_t, ::core::mem::size_of::<proftime_T>()) as *mut proftime_T;
        }
        (*fp).uf_tml_idx = -1 as ::core::ffi::c_int;
        (*fp).uf_prof_initialized = true_0;
    }
    (*fp).uf_profiling = true_0;
}
pub unsafe extern "C" fn prof_child_enter(mut tm: *mut proftime_T) {
    let mut fc: *mut funccall_T = get_current_funccal();
    if !fc.is_null() && (*(*fc).fc_func).uf_profiling != 0 {
        (*fc).fc_prof_child = profile_start();
    }
    script_prof_save(tm);
}
pub unsafe extern "C" fn prof_child_exit(mut tm: *mut proftime_T) {
    let mut fc: *mut funccall_T = get_current_funccal();
    if !fc.is_null() && (*(*fc).fc_func).uf_profiling != 0 {
        (*fc).fc_prof_child = profile_end((*fc).fc_prof_child);
        (*fc).fc_prof_child = profile_sub_wait(*tm, (*fc).fc_prof_child);
        (*(*fc).fc_func).uf_tm_children =
            profile_add((*(*fc).fc_func).uf_tm_children, (*fc).fc_prof_child);
        (*(*fc).fc_func).uf_tml_children =
            profile_add((*(*fc).fc_func).uf_tml_children, (*fc).fc_prof_child);
    }
    script_prof_restore(tm);
}
pub unsafe extern "C" fn func_line_start(mut cookie: *mut ::core::ffi::c_void) {
    let mut fcp: *mut funccall_T = cookie as *mut funccall_T;
    let mut fp: *mut ufunc_T = (*fcp).fc_func;
    if (*fp).uf_profiling != 0
        && (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum
            >= 1 as linenr_T
        && (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum
            <= (*fp).uf_lines.ga_len as linenr_T
    {
        (*fp).uf_tml_idx = ((*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum
            - 1 as linenr_T) as ::core::ffi::c_int;
        while (*fp).uf_tml_idx > 0 as ::core::ffi::c_int
            && (*((*fp).uf_lines.ga_data as *mut *mut ::core::ffi::c_char)
                .offset((*fp).uf_tml_idx as isize))
            .is_null()
        {
            (*fp).uf_tml_idx -= 1;
        }
        (*fp).uf_tml_execed = false_0;
        (*fp).uf_tml_start = profile_start();
        (*fp).uf_tml_children = profile_zero();
        (*fp).uf_tml_wait = profile_get_wait();
    }
}
pub unsafe extern "C" fn func_line_exec(mut cookie: *mut ::core::ffi::c_void) {
    let mut fcp: *mut funccall_T = cookie as *mut funccall_T;
    let mut fp: *mut ufunc_T = (*fcp).fc_func;
    if (*fp).uf_profiling != 0 && (*fp).uf_tml_idx >= 0 as ::core::ffi::c_int {
        (*fp).uf_tml_execed = true_0;
    }
}
pub unsafe extern "C" fn func_line_end(mut cookie: *mut ::core::ffi::c_void) {
    let mut fcp: *mut funccall_T = cookie as *mut funccall_T;
    let mut fp: *mut ufunc_T = (*fcp).fc_func;
    if (*fp).uf_profiling != 0 && (*fp).uf_tml_idx >= 0 as ::core::ffi::c_int {
        if (*fp).uf_tml_execed != 0 {
            *(*fp).uf_tml_count.offset((*fp).uf_tml_idx as isize) += 1;
            (*fp).uf_tml_start = profile_end((*fp).uf_tml_start);
            (*fp).uf_tml_start = profile_sub_wait((*fp).uf_tml_wait, (*fp).uf_tml_start);
            *(*fp).uf_tml_total.offset((*fp).uf_tml_idx as isize) = profile_add(
                *(*fp).uf_tml_total.offset((*fp).uf_tml_idx as isize),
                (*fp).uf_tml_start,
            );
            *(*fp).uf_tml_self.offset((*fp).uf_tml_idx as isize) = profile_self(
                *(*fp).uf_tml_self.offset((*fp).uf_tml_idx as isize),
                (*fp).uf_tml_start,
                (*fp).uf_tml_children,
            );
        }
        (*fp).uf_tml_idx = -1 as ::core::ffi::c_int;
    }
}
unsafe extern "C" fn func_dump_profile(mut fd: *mut FILE) {
    let functbl: *mut hashtab_T = func_tbl_get();
    let mut st_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut todo: ::core::ffi::c_int = (*functbl).ht_used as ::core::ffi::c_int;
    if todo == 0 as ::core::ffi::c_int {
        return;
    }
    let mut sorttab: *mut *mut ufunc_T =
        xmalloc(::core::mem::size_of::<*mut ufunc_T>().wrapping_mul(todo as size_t))
            as *mut *mut ufunc_T;
    let mut hi: *mut hashitem_T = (*functbl).ht_array;
    while todo > 0 as ::core::ffi::c_int {
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            todo -= 1;
            let mut fp: *mut ufunc_T =
                (*hi).hi_key.offset(-(240 as ::core::ffi::c_ulong as isize)) as *mut ufunc_T;
            if (*fp).uf_prof_initialized != 0 {
                let c2rust_fresh0 = st_len;
                st_len = st_len + 1;
                let c2rust_lvalue_ptr = &raw mut *sorttab.offset(c2rust_fresh0 as isize);
                *c2rust_lvalue_ptr = fp;
                if *(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char)
                    .offset(0 as ::core::ffi::c_int as isize) as uint8_t
                    as ::core::ffi::c_int
                    == K_SPECIAL
                {
                    fprintf(
                        fd,
                        b"FUNCTION  <SNR>%s()\n\0".as_ptr() as *const ::core::ffi::c_char,
                        (&raw mut (*fp).uf_name as *mut ::core::ffi::c_char)
                            .offset(3 as ::core::ffi::c_int as isize),
                    );
                } else {
                    fprintf(
                        fd,
                        b"FUNCTION  %s()\n\0".as_ptr() as *const ::core::ffi::c_char,
                        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
                    );
                }
                if (*fp).uf_script_ctx.sc_sid != 0 as ::core::ffi::c_int {
                    let mut should_free: bool = false;
                    let mut p: *mut ::core::ffi::c_char =
                        get_scriptname((*fp).uf_script_ctx, &raw mut should_free);
                    fprintf(
                        fd,
                        b"    Defined: %s:%d\n\0".as_ptr() as *const ::core::ffi::c_char,
                        p,
                        (*fp).uf_script_ctx.sc_lnum,
                    );
                    if should_free {
                        xfree(p as *mut ::core::ffi::c_void);
                    }
                }
                if (*fp).uf_tm_count == 1 as ::core::ffi::c_int {
                    fprintf(
                        fd,
                        b"Called 1 time\n\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                } else {
                    fprintf(
                        fd,
                        b"Called %d times\n\0".as_ptr() as *const ::core::ffi::c_char,
                        (*fp).uf_tm_count,
                    );
                }
                fprintf(
                    fd,
                    b"Total time: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                    profile_msg((*fp).uf_tm_total),
                );
                fprintf(
                    fd,
                    b" Self time: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                    profile_msg((*fp).uf_tm_self),
                );
                fprintf(fd, b"\n\0".as_ptr() as *const ::core::ffi::c_char);
                fprintf(
                    fd,
                    b"count  total (s)   self (s)\n\0".as_ptr() as *const ::core::ffi::c_char,
                );
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < (*fp).uf_lines.ga_len {
                    if !(*((*fp).uf_lines.ga_data as *mut *mut ::core::ffi::c_char)
                        .offset(i as isize))
                    .is_null()
                    {
                        prof_func_line(
                            fd,
                            *(*fp).uf_tml_count.offset(i as isize),
                            (*fp).uf_tml_total.offset(i as isize),
                            (*fp).uf_tml_self.offset(i as isize),
                            true_0 != 0,
                        );
                        fprintf(
                            fd,
                            b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
                            *((*fp).uf_lines.ga_data as *mut *mut ::core::ffi::c_char)
                                .offset(i as isize),
                        );
                    }
                    i += 1;
                }
                fprintf(fd, b"\n\0".as_ptr() as *const ::core::ffi::c_char);
            }
        }
        hi = hi.offset(1);
    }
    if st_len > 0 as ::core::ffi::c_int {
        qsort(
            sorttab as *mut ::core::ffi::c_void,
            st_len as size_t,
            ::core::mem::size_of::<*mut ufunc_T>(),
            Some(
                prof_total_cmp
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        prof_sort_list(
            fd,
            sorttab,
            st_len,
            b"TOTAL\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            false_0 != 0,
        );
        qsort(
            sorttab as *mut ::core::ffi::c_void,
            st_len as size_t,
            ::core::mem::size_of::<*mut ufunc_T>(),
            Some(
                prof_self_cmp
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        prof_sort_list(
            fd,
            sorttab,
            st_len,
            b"SELF\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            true_0 != 0,
        );
    }
    xfree(sorttab as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn profile_init(mut si: *mut scriptitem_T) {
    (*si).sn_pr_count = 0 as ::core::ffi::c_int;
    (*si).sn_pr_total = profile_zero();
    (*si).sn_pr_self = profile_zero();
    ga_init(
        &raw mut (*si).sn_prl_ga,
        ::core::mem::size_of::<sn_prl_T>() as ::core::ffi::c_int,
        100 as ::core::ffi::c_int,
    );
    (*si).sn_prl_idx = -1 as ::core::ffi::c_int as linenr_T;
    (*si).sn_prof_on = true_0 != 0;
    (*si).sn_pr_nest = 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn script_prof_save(mut tm: *mut proftime_T) {
    if (*current_sctx.ptr()).sc_sid > 0 as ::core::ffi::c_int
        && (*current_sctx.ptr()).sc_sid <= (*script_items.ptr()).ga_len
    {
        let mut si: *mut scriptitem_T = *((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
            .offset(
                ((*current_sctx.ptr()).sc_sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    as isize,
            );
        if (*si).sn_prof_on as ::core::ffi::c_int != 0 && {
            let c2rust_fresh1 = (*si).sn_pr_nest;
            (*si).sn_pr_nest = (*si).sn_pr_nest + 1;
            c2rust_fresh1 == 0 as ::core::ffi::c_int
        } {
            (*si).sn_pr_child = profile_start();
        }
    }
    *tm = profile_get_wait();
}
pub unsafe extern "C" fn script_prof_restore(mut tm: *const proftime_T) {
    if !((*current_sctx.ptr()).sc_sid > 0 as ::core::ffi::c_int
        && (*current_sctx.ptr()).sc_sid <= (*script_items.ptr()).ga_len)
    {
        return;
    }
    let mut si: *mut scriptitem_T = *((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
        .offset(
            ((*current_sctx.ptr()).sc_sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize,
        );
    if (*si).sn_prof_on as ::core::ffi::c_int != 0 && {
        (*si).sn_pr_nest -= 1;
        (*si).sn_pr_nest == 0 as ::core::ffi::c_int
    } {
        (*si).sn_pr_child = profile_end((*si).sn_pr_child);
        (*si).sn_pr_child = profile_sub_wait(*tm, (*si).sn_pr_child);
        (*si).sn_pr_children = profile_add((*si).sn_pr_children, (*si).sn_pr_child);
        (*si).sn_prl_children = profile_add((*si).sn_prl_children, (*si).sn_pr_child);
    }
}
unsafe extern "C" fn script_dump_profile(mut fd: *mut FILE) {
    let mut pp: *mut sn_prl_T = ::core::ptr::null_mut::<sn_prl_T>();
    let mut id: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while id <= (*script_items.ptr()).ga_len {
        let mut si: *mut scriptitem_T = *((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
            .offset((id - 1 as ::core::ffi::c_int) as isize);
        if (*si).sn_prof_on {
            fprintf(
                fd,
                b"SCRIPT  %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                (*si).sn_name,
            );
            if (*si).sn_pr_count == 1 as ::core::ffi::c_int {
                fprintf(
                    fd,
                    b"Sourced 1 time\n\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                fprintf(
                    fd,
                    b"Sourced %d times\n\0".as_ptr() as *const ::core::ffi::c_char,
                    (*si).sn_pr_count,
                );
            }
            fprintf(
                fd,
                b"Total time: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                profile_msg((*si).sn_pr_total),
            );
            fprintf(
                fd,
                b" Self time: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                profile_msg((*si).sn_pr_self),
            );
            fprintf(fd, b"\n\0".as_ptr() as *const ::core::ffi::c_char);
            fprintf(
                fd,
                b"count  total (s)   self (s)\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            let mut sfd: *mut FILE =
                os_fopen((*si).sn_name, b"r\0".as_ptr() as *const ::core::ffi::c_char);
            if sfd.is_null() {
                fprintf(
                    fd,
                    b"Cannot open file!\n\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while !vim_fgets(IObuff.ptr() as *mut ::core::ffi::c_char, IOSIZE, sfd) {
                    if (*IObuff.ptr())[(IOSIZE - 2 as ::core::ffi::c_int) as usize]
                        as ::core::ffi::c_int
                        != NUL
                        && (*IObuff.ptr())[(IOSIZE - 2 as ::core::ffi::c_int) as usize]
                            as ::core::ffi::c_int
                            != NL
                    {
                        let mut n: ::core::ffi::c_int = IOSIZE - 2 as ::core::ffi::c_int;
                        while n > 0 as ::core::ffi::c_int
                            && (*IObuff.ptr())[n as usize] as ::core::ffi::c_int
                                & 0xc0 as ::core::ffi::c_int
                                == 0x80 as ::core::ffi::c_int
                        {
                            n -= 1;
                        }
                        (*IObuff.ptr())[n as usize] = NL as ::core::ffi::c_char;
                        (*IObuff.ptr())[(n + 1 as ::core::ffi::c_int) as usize] =
                            NUL as ::core::ffi::c_char;
                    }
                    if i < (*si).sn_prl_ga.ga_len && {
                        pp = ((*si).sn_prl_ga.ga_data as *mut sn_prl_T).offset(i as isize);
                        (*pp).snp_count > 0 as ::core::ffi::c_int
                    } {
                        fprintf(
                            fd,
                            b"%5d \0".as_ptr() as *const ::core::ffi::c_char,
                            (*pp).snp_count,
                        );
                        if profile_equal((*pp).sn_prl_total, (*pp).sn_prl_self) {
                            fprintf(fd, b"           \0".as_ptr() as *const ::core::ffi::c_char);
                        } else {
                            fprintf(
                                fd,
                                b"%s \0".as_ptr() as *const ::core::ffi::c_char,
                                profile_msg((*pp).sn_prl_total),
                            );
                        }
                        fprintf(
                            fd,
                            b"%s \0".as_ptr() as *const ::core::ffi::c_char,
                            profile_msg((*pp).sn_prl_self),
                        );
                    } else {
                        fprintf(
                            fd,
                            b"                            \0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                    fprintf(
                        fd,
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                    );
                    i += 1;
                }
                fclose(sfd);
            }
            fprintf(fd, b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        }
        id += 1;
    }
}
pub unsafe extern "C" fn profile_dump() {
    if (*profile_fname.ptr()).is_null() {
        return;
    }
    let mut fd: *mut FILE = os_fopen(
        profile_fname.get(),
        b"w\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if fd.is_null() {
        semsg(
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            profile_fname.get(),
        );
    } else {
        script_dump_profile(fd);
        func_dump_profile(fd);
        fclose(fd);
    };
}
pub unsafe extern "C" fn script_line_start() {
    if (*current_sctx.ptr()).sc_sid <= 0 as ::core::ffi::c_int
        || (*current_sctx.ptr()).sc_sid > (*script_items.ptr()).ga_len
    {
        return;
    }
    let mut si: *mut scriptitem_T = *((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
        .offset(
            ((*current_sctx.ptr()).sc_sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize,
        );
    if (*si).sn_prof_on as ::core::ffi::c_int != 0
        && (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum
            >= 1 as linenr_T
    {
        ga_grow(
            &raw mut (*si).sn_prl_ga,
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum as ::core::ffi::c_int
                - (*si).sn_prl_ga.ga_len,
        );
        (*si).sn_prl_idx = (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum
            - 1 as linenr_T;
        while (*si).sn_prl_ga.ga_len as linenr_T <= (*si).sn_prl_idx
            && (*si).sn_prl_ga.ga_len < (*si).sn_prl_ga.ga_maxlen
        {
            let mut pp: *mut sn_prl_T =
                ((*si).sn_prl_ga.ga_data as *mut sn_prl_T).offset((*si).sn_prl_ga.ga_len as isize);
            (*pp).snp_count = 0 as ::core::ffi::c_int;
            (*pp).sn_prl_total = profile_zero();
            (*pp).sn_prl_self = profile_zero();
            (*si).sn_prl_ga.ga_len += 1;
        }
        (*si).sn_prl_execed = false_0;
        (*si).sn_prl_start = profile_start();
        (*si).sn_prl_children = profile_zero();
        (*si).sn_prl_wait = profile_get_wait();
    }
}
pub unsafe extern "C" fn script_line_exec() {
    if (*current_sctx.ptr()).sc_sid <= 0 as ::core::ffi::c_int
        || (*current_sctx.ptr()).sc_sid > (*script_items.ptr()).ga_len
    {
        return;
    }
    let mut si: *mut scriptitem_T = *((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
        .offset(
            ((*current_sctx.ptr()).sc_sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize,
        );
    if (*si).sn_prof_on as ::core::ffi::c_int != 0 && (*si).sn_prl_idx >= 0 as linenr_T {
        (*si).sn_prl_execed = true_0;
    }
}
pub unsafe extern "C" fn script_line_end() {
    if (*current_sctx.ptr()).sc_sid <= 0 as ::core::ffi::c_int
        || (*current_sctx.ptr()).sc_sid > (*script_items.ptr()).ga_len
    {
        return;
    }
    let mut si: *mut scriptitem_T = *((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
        .offset(
            ((*current_sctx.ptr()).sc_sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize,
        );
    if (*si).sn_prof_on as ::core::ffi::c_int != 0
        && (*si).sn_prl_idx >= 0 as linenr_T
        && (*si).sn_prl_idx < (*si).sn_prl_ga.ga_len as linenr_T
    {
        if (*si).sn_prl_execed != 0 {
            let mut pp: *mut sn_prl_T =
                ((*si).sn_prl_ga.ga_data as *mut sn_prl_T).offset((*si).sn_prl_idx as isize);
            (*pp).snp_count += 1;
            (*si).sn_prl_start = profile_end((*si).sn_prl_start);
            (*si).sn_prl_start = profile_sub_wait((*si).sn_prl_wait, (*si).sn_prl_start);
            (*pp).sn_prl_total = profile_add((*pp).sn_prl_total, (*si).sn_prl_start);
            (*pp).sn_prl_self =
                profile_self((*pp).sn_prl_self, (*si).sn_prl_start, (*si).sn_prl_children);
        }
        (*si).sn_prl_idx = -1 as ::core::ffi::c_int as linenr_T;
    }
}
static g_start_time: GlobalCell<proftime_T> = GlobalCell::new(0);
static g_prev_time: GlobalCell<proftime_T> = GlobalCell::new(0);
pub unsafe extern "C" fn time_push(mut rel: *mut proftime_T, mut start: *mut proftime_T) {
    let mut now: proftime_T = profile_start();
    *rel = profile_sub(now, g_prev_time.get());
    *start = now;
    g_prev_time.set(now);
}
pub unsafe extern "C" fn time_pop(mut tp: proftime_T) {
    g_prev_time.set((*g_prev_time.ptr()).wrapping_sub(tp));
}
unsafe extern "C" fn time_diff(mut then: proftime_T, mut now: proftime_T) {
    let mut diff: proftime_T = profile_sub(now, then);
    fprintf(
        time_fd.get(),
        b"%07.3lf\0".as_ptr() as *const ::core::ffi::c_char,
        diff as ::core::ffi::c_double / 1.0E6f64,
    );
}
pub unsafe extern "C" fn time_start(mut message: *const ::core::ffi::c_char) {
    if (*time_fd.ptr()).is_null() {
        return;
    }
    g_start_time.set(profile_start());
    g_prev_time.set(g_start_time.get());
    fprintf(
        time_fd.get(),
        b"\ntimes in msec\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    fprintf(
        time_fd.get(),
        b" clock   self+sourced   self:  sourced script\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    fprintf(
        time_fd.get(),
        b" clock   elapsed:              other lines\n\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    time_msg(message, ::core::ptr::null::<proftime_T>());
}
pub unsafe extern "C" fn time_msg(
    mut mesg: *const ::core::ffi::c_char,
    mut start: *const proftime_T,
) {
    if (*time_fd.ptr()).is_null() {
        return;
    }
    let mut now: proftime_T = profile_start();
    time_diff(g_start_time.get(), now);
    if !start.is_null() {
        fprintf(
            time_fd.get(),
            b"  \0".as_ptr() as *const ::core::ffi::c_char,
        );
        time_diff(*start, now);
    }
    fprintf(
        time_fd.get(),
        b"  \0".as_ptr() as *const ::core::ffi::c_char,
    );
    time_diff(g_prev_time.get(), now);
    g_prev_time.set(now);
    fprintf(
        time_fd.get(),
        b": %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        mesg,
    );
}
pub unsafe extern "C" fn time_init(
    mut fname: *const ::core::ffi::c_char,
    mut proc_name: *const ::core::ffi::c_char,
) {
    let bufsize: size_t = 8192 as size_t;
    time_fd.set(fopen(fname, b"a\0".as_ptr() as *const ::core::ffi::c_char) as *mut FILE);
    if (*time_fd.ptr()).is_null() {
        fprintf(
            stderr,
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            fname,
        );
        return;
    }
    startuptime_buf.set(xmalloc(
        ::core::mem::size_of::<::core::ffi::c_char>()
            .wrapping_mul(bufsize.wrapping_add(1 as size_t)),
    ) as *mut ::core::ffi::c_char);
    let mut r: ::core::ffi::c_int = setvbuf(
        time_fd.get(),
        startuptime_buf.get(),
        _IOFBF,
        bufsize.wrapping_add(1 as size_t),
    );
    if r != 0 as ::core::ffi::c_int {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            startuptime_buf.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        fclose(time_fd.get());
        time_fd.set(::core::ptr::null_mut::<FILE>());
        fprintf(
            stderr,
            b"time_init: setvbuf failed: %d %s\0".as_ptr() as *const ::core::ffi::c_char,
            r,
            uv_err_name(r),
        );
        return;
    }
    fprintf(
        time_fd.get(),
        b"--- Startup times for process: %s ---\n\0".as_ptr() as *const ::core::ffi::c_char,
        proc_name,
    );
}
pub unsafe extern "C" fn time_finish() {
    if (*time_fd.ptr()).is_null() {
        return;
    }
    '_c2rust_label: {
        if !(*startuptime_buf.ptr()).is_null() {
        } else {
            __assert_fail(
                b"startuptime_buf != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/profile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                978 as ::core::ffi::c_uint,
                b"void time_finish(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"--- NVIM STARTED ---\n\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    fclose(time_fd.get());
    time_fd.set(::core::ptr::null_mut::<FILE>());
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        startuptime_buf.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
