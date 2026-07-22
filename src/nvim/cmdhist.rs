use crate::src::nvim::charset::vim_strsize;
use crate::src::nvim::eval::typval::{
    tv_get_number, tv_get_number_chk, tv_get_string_buf, tv_get_string_chk,
};
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::ex_getln::{get_cmdline_firstc, get_list_range};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    cmdmod, e_trailing_arg, e_val_too_large, got_int, maptick, p_hi, Columns, IObuff,
};
use crate::src::nvim::memory::{xfree, xmalloc, xstrlcpy};
use crate::src::nvim::message::{
    message_filtered, msg, msg_ext_set_kind, msg_outtrans, msg_putchar, msg_puts_title, semsg,
    trunc_string,
};
use crate::src::nvim::os::libc::{
    __assert_fail, gettext, memcpy, memset, snprintf, strcmp, strlen, strncasecmp,
};
use crate::src::nvim::os::time::os_time;
use crate::src::nvim::strings::{vim_snprintf, vim_strchr, xstrnsave};
pub use crate::src::nvim::types::{
    blob_T, blobvar_S, cmd_addr_T, cmdidx_T, cmdmod_T, colnr_T, cstack_T,
    cstack_T_cs_pend as C2Rust_Unnamed_0, dict_T, dictvar_S, eslist_T, eslist_elem, exarg, exarg_T,
    expand_T, float_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed, funccall_T, garray_T,
    hash_T, hashitem_T, hashtab_T, histentry_T, int32_t, int64_t, linenr_T, list_T, listitem_S,
    listitem_T, listvar_S, listwatch_S, listwatch_T, partial_S, partial_T, pos_T, proftime_T,
    queue, regmatch_T, regprog, regprog_T, scid_T, sctx_T, size_t, typval_T, typval_vval_union,
    ufunc_S, ufunc_T, uint32_t, uint64_t, uint8_t, varnumber_T, xp_prefix_T, AdditionalData,
    BoolVarValue, CMD_index, Direction, EvalFuncData, HistoryType, LineGetter, LuaRef,
    MsgpackRpcRequestHandler, OptInt, ScopeDictDictItem, ScopeType, SpecialVarValue, Timestamp,
    VarLockStatus, VarType, QUEUE,
};
extern "C" {
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
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
pub type C2Rust_Unnamed_1 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_1 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_1 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_1 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_1 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_1 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_1 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_1 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_1 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_1 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_1 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_1 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_1 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_1 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_1 = 1;
pub const HIST_DEBUG: HistoryType = 4;
pub const HIST_INPUT: HistoryType = 3;
pub const HIST_EXPR: HistoryType = 2;
pub const HIST_SEARCH: HistoryType = 1;
pub const HIST_CMD: HistoryType = 0;
pub const HIST_INVALID: HistoryType = -1;
pub const HIST_DEFAULT: HistoryType = -2;
pub type C2Rust_Unnamed_2 = ::core::ffi::c_uint;
pub const HIST_COUNT: C2Rust_Unnamed_2 = 5;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static history: GlobalCell<[*mut histentry_T; 5]> = GlobalCell::new([
    ::core::ptr::null_mut::<histentry_T>(),
    ::core::ptr::null_mut::<histentry_T>(),
    ::core::ptr::null_mut::<histentry_T>(),
    ::core::ptr::null_mut::<histentry_T>(),
    ::core::ptr::null_mut::<histentry_T>(),
]);
static hisidx: GlobalCell<[::core::ffi::c_int; 5]> = GlobalCell::new([
    -1 as ::core::ffi::c_int,
    -1 as ::core::ffi::c_int,
    -1 as ::core::ffi::c_int,
    -1 as ::core::ffi::c_int,
    -1 as ::core::ffi::c_int,
]);
static hisnum: GlobalCell<[::core::ffi::c_int; 5]> = GlobalCell::new([
    0 as ::core::ffi::c_int,
    0 as ::core::ffi::c_int,
    0 as ::core::ffi::c_int,
    0 as ::core::ffi::c_int,
    0 as ::core::ffi::c_int,
]);
static hislen: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub unsafe extern "C" fn get_hislen() -> ::core::ffi::c_int {
    return hislen.get();
}
pub unsafe extern "C" fn get_histentry(mut hist_type: ::core::ffi::c_int) -> *mut histentry_T {
    return (*history.ptr())[hist_type as usize] as *mut histentry_T;
}
pub unsafe extern "C" fn get_hisidx(mut hist_type: ::core::ffi::c_int) -> *mut ::core::ffi::c_int {
    return (hisidx.ptr() as *mut ::core::ffi::c_int).offset(hist_type as isize);
}
pub unsafe extern "C" fn hist_char2type(c: ::core::ffi::c_int) -> HistoryType {
    match c {
        58 => return HIST_CMD,
        61 => return HIST_EXPR,
        64 => return HIST_INPUT,
        62 => return HIST_DEBUG,
        NUL | 47 | 63 => return HIST_SEARCH,
        _ => return HIST_INVALID,
    };
}
static history_names: GlobalCell<[*mut ::core::ffi::c_char; 6]> = GlobalCell::new([
    b"cmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"search\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"expr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"input\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"debug\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ::core::ptr::null_mut::<::core::ffi::c_char>(),
]);
pub unsafe extern "C" fn get_history_arg(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut short_names: *const ::core::ffi::c_char =
        b":=@>?/\0".as_ptr() as *const ::core::ffi::c_char;
    let short_names_count: ::core::ffi::c_int = strlen(short_names) as ::core::ffi::c_int;
    let history_name_count: ::core::ffi::c_int =
        ::core::mem::size_of::<[*mut ::core::ffi::c_char; 6]>()
            .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*mut ::core::ffi::c_char; 6]>()
                    .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            )
            .wrapping_sub(1 as usize) as ::core::ffi::c_int;
    if idx < short_names_count {
        (*xp).xp_buf[0 as ::core::ffi::c_int as usize] = *short_names.offset(idx as isize);
        (*xp).xp_buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        return &raw mut (*xp).xp_buf as *mut ::core::ffi::c_char;
    }
    if idx < short_names_count + history_name_count {
        return (*history_names.ptr())[(idx - short_names_count) as usize]
            as *mut ::core::ffi::c_char;
    }
    if idx == short_names_count + history_name_count {
        return b"all\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn init_history() {
    '_c2rust_label: {
        if p_hi.get() >= 0 as OptInt && p_hi.get() <= 2147483647 as OptInt {
        } else {
            __assert_fail(
                b"p_hi >= 0 && p_hi <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/cmdhist.rs\0".as_ptr() as *const ::core::ffi::c_char,
                130 as ::core::ffi::c_uint,
                b"void init_history(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut newlen: ::core::ffi::c_int = p_hi.get() as ::core::ffi::c_int;
    let mut oldlen: ::core::ffi::c_int = hislen.get();
    if newlen == oldlen {
        return;
    }
    let mut type_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while type_0 < HIST_COUNT as ::core::ffi::c_int {
        let mut temp: *mut histentry_T = (if newlen > 0 as ::core::ffi::c_int {
            xmalloc((newlen as size_t).wrapping_mul(::core::mem::size_of::<histentry_T>()))
        } else {
            NULL
        }) as *mut histentry_T;
        let mut j: ::core::ffi::c_int = (*hisidx.ptr())[type_0 as usize];
        if j >= 0 as ::core::ffi::c_int {
            let mut l1: ::core::ffi::c_int = if (j + 1 as ::core::ffi::c_int) < newlen {
                j + 1 as ::core::ffi::c_int
            } else {
                newlen
            };
            let mut l2: ::core::ffi::c_int = (if newlen < oldlen { newlen } else { oldlen }) - l1;
            let mut i1: ::core::ffi::c_int = j + 1 as ::core::ffi::c_int - l1;
            let mut i2: ::core::ffi::c_int = if l1 > oldlen - newlen + l1 {
                l1
            } else {
                oldlen - newlen + l1
            };
            if newlen != 0 {
                memcpy(
                    temp.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                    (*(history.ptr() as *mut *mut histentry_T).offset(type_0 as isize))
                        .offset(i2 as isize) as *const ::core::ffi::c_void,
                    (l2 as size_t).wrapping_mul(::core::mem::size_of::<histentry_T>()),
                );
                memcpy(
                    temp.offset(l2 as isize) as *mut ::core::ffi::c_void,
                    (*(history.ptr() as *mut *mut histentry_T).offset(type_0 as isize))
                        .offset(i1 as isize) as *const ::core::ffi::c_void,
                    (l1 as size_t).wrapping_mul(::core::mem::size_of::<histentry_T>()),
                );
            }
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < i1 {
                hist_free_entry((*history.ptr())[type_0 as usize].offset(i as isize));
                i += 1;
            }
            let mut i_0: ::core::ffi::c_int = i1 + l1;
            while i_0 < i2 {
                hist_free_entry((*history.ptr())[type_0 as usize].offset(i_0 as isize));
                i_0 += 1;
            }
        }
        let mut l3: ::core::ffi::c_int = if j < 0 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else if newlen < oldlen {
            newlen
        } else {
            oldlen
        };
        if newlen > 0 as ::core::ffi::c_int {
            memset(
                temp.offset(l3 as isize) as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ((newlen - l3) as size_t).wrapping_mul(::core::mem::size_of::<histentry_T>()),
            );
        }
        (*hisidx.ptr())[type_0 as usize] = l3 - 1 as ::core::ffi::c_int;
        xfree((*history.ptr())[type_0 as usize] as *mut ::core::ffi::c_void);
        (*history.ptr())[type_0 as usize] = temp as *mut histentry_T;
        type_0 += 1;
    }
    hislen.set(newlen);
}
#[inline]
unsafe extern "C" fn hist_free_entry(mut hisptr: *mut histentry_T) {
    xfree((*hisptr).hisstr as *mut ::core::ffi::c_void);
    xfree((*hisptr).additional_data as *mut ::core::ffi::c_void);
    clear_hist_entry(hisptr);
}
#[inline]
unsafe extern "C" fn clear_hist_entry(mut hisptr: *mut histentry_T) {
    memset(
        hisptr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<histentry_T>(),
    );
}
unsafe extern "C" fn in_history(
    mut type_0: ::core::ffi::c_int,
    mut str: *const ::core::ffi::c_char,
    mut move_to_front: ::core::ffi::c_int,
    mut sep: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut last_i: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if (*hisidx.ptr())[type_0 as usize] < 0 as ::core::ffi::c_int {
        return false_0;
    }
    let mut i: ::core::ffi::c_int = (*hisidx.ptr())[type_0 as usize];
    loop {
        if (*(*history.ptr())[type_0 as usize].offset(i as isize))
            .hisstr
            .is_null()
        {
            return false_0;
        }
        let mut p: *mut ::core::ffi::c_char =
            (*(*history.ptr())[type_0 as usize].offset(i as isize)).hisstr;
        if strcmp(str, p) == 0 as ::core::ffi::c_int
            && (type_0 != HIST_SEARCH as ::core::ffi::c_int
                || sep
                    == *p.offset(
                        (*(*history.ptr())[type_0 as usize].offset(i as isize))
                            .hisstrlen
                            .wrapping_add(1 as size_t) as isize,
                    ) as ::core::ffi::c_int)
        {
            if move_to_front == 0 {
                return true_0;
            }
            last_i = i;
            break;
        } else {
            i -= 1;
            if i < 0 as ::core::ffi::c_int {
                i = hislen.get() - 1 as ::core::ffi::c_int;
            }
            if i == (*hisidx.ptr())[type_0 as usize] {
                break;
            }
        }
    }
    if last_i < 0 as ::core::ffi::c_int {
        return false_0;
    }
    let mut ad: *mut AdditionalData =
        (*(*history.ptr())[type_0 as usize].offset(i as isize)).additional_data;
    let save_hisstr: *mut ::core::ffi::c_char =
        (*(*history.ptr())[type_0 as usize].offset(i as isize)).hisstr;
    let save_hisstrlen: size_t = (*(*history.ptr())[type_0 as usize].offset(i as isize)).hisstrlen;
    while i != (*hisidx.ptr())[type_0 as usize] {
        i += 1;
        if i >= hislen.get() {
            i = 0 as ::core::ffi::c_int;
        }
        *(*history.ptr())[type_0 as usize].offset(last_i as isize) =
            *(*history.ptr())[type_0 as usize].offset(i as isize);
        last_i = i;
    }
    xfree(ad as *mut ::core::ffi::c_void);
    (*hisnum.ptr())[type_0 as usize] += 1;
    (*(*history.ptr())[type_0 as usize].offset(i as isize)).hisnum =
        (*hisnum.ptr())[type_0 as usize];
    (*(*history.ptr())[type_0 as usize].offset(i as isize)).hisstr = save_hisstr;
    (*(*history.ptr())[type_0 as usize].offset(i as isize)).hisstrlen = save_hisstrlen;
    (*(*history.ptr())[type_0 as usize].offset(i as isize)).timestamp = os_time();
    (*(*history.ptr())[type_0 as usize].offset(i as isize)).additional_data =
        ::core::ptr::null_mut::<AdditionalData>();
    return true_0;
}
unsafe extern "C" fn get_histtype(
    name: *const ::core::ffi::c_char,
    len: size_t,
    return_default: bool,
) -> HistoryType {
    if len == 0 as size_t {
        return (if return_default as ::core::ffi::c_int != 0 {
            HIST_DEFAULT as ::core::ffi::c_int
        } else {
            hist_char2type(get_cmdline_firstc()) as ::core::ffi::c_int
        }) as HistoryType;
    }
    let mut i: HistoryType = HIST_CMD;
    while !(*history_names.ptr())[i as usize].is_null() {
        if strncasecmp(
            name as *mut ::core::ffi::c_char,
            (*history_names.ptr())[i as usize],
            len,
        ) == 0 as ::core::ffi::c_int
        {
            return i;
        }
        i += 1;
    }
    if !vim_strchr(
        b":=@>?/\0".as_ptr() as *const ::core::ffi::c_char,
        *name.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int,
    )
    .is_null()
        && len == 1 as size_t
    {
        return hist_char2type(*name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int);
    }
    return HIST_INVALID;
}
static last_maptick: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
pub unsafe extern "C" fn add_to_history(
    mut histype: ::core::ffi::c_int,
    mut new_entry: *const ::core::ffi::c_char,
    mut new_entrylen: size_t,
    mut in_map: bool,
    mut sep: ::core::ffi::c_int,
) {
    let mut hisptr: *mut histentry_T = ::core::ptr::null_mut::<histentry_T>();
    if hislen.get() == 0 as ::core::ffi::c_int || histype == HIST_INVALID as ::core::ffi::c_int {
        return;
    }
    '_c2rust_label: {
        if histype != HIST_DEFAULT as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"histype != HIST_DEFAULT\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/cmdhist.rs\0".as_ptr() as *const ::core::ffi::c_char,
                306 as ::core::ffi::c_uint,
                b"void add_to_history(int, const char *, size_t, _Bool, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if (*cmdmod.ptr()).cmod_flags & CMOD_KEEPPATTERNS as ::core::ffi::c_int != 0
        && histype == HIST_SEARCH as ::core::ffi::c_int
    {
        return;
    }
    if histype == HIST_SEARCH as ::core::ffi::c_int && in_map as ::core::ffi::c_int != 0 {
        if maptick.get() == last_maptick.get()
            && (*hisidx.ptr())[HIST_SEARCH as ::core::ffi::c_int as usize]
                >= 0 as ::core::ffi::c_int
        {
            hisptr = (*(history.ptr() as *mut *mut histentry_T)
                .offset(HIST_SEARCH as ::core::ffi::c_int as isize))
            .offset(
                *(hisidx.ptr() as *mut ::core::ffi::c_int)
                    .offset(HIST_SEARCH as ::core::ffi::c_int as isize) as isize,
            );
            hist_free_entry(hisptr);
            (*hisnum.ptr())[histype as usize] -= 1;
            (*hisidx.ptr())[HIST_SEARCH as ::core::ffi::c_int as usize] -= 1;
            if (*hisidx.ptr())[HIST_SEARCH as ::core::ffi::c_int as usize] < 0 as ::core::ffi::c_int
            {
                (*hisidx.ptr())[HIST_SEARCH as ::core::ffi::c_int as usize] =
                    hislen.get() - 1 as ::core::ffi::c_int;
            }
        }
        last_maptick.set(-1 as ::core::ffi::c_int);
    }
    if in_history(histype, new_entry, true_0, sep) != 0 {
        return;
    }
    (*hisidx.ptr())[histype as usize] += 1;
    if (*hisidx.ptr())[histype as usize] == hislen.get() {
        (*hisidx.ptr())[histype as usize] = 0 as ::core::ffi::c_int;
    }
    hisptr = (*(history.ptr() as *mut *mut histentry_T).offset(histype as isize))
        .offset(*(hisidx.ptr() as *mut ::core::ffi::c_int).offset(histype as isize) as isize);
    hist_free_entry(hisptr);
    (*hisptr).hisstr = xstrnsave(new_entry, new_entrylen.wrapping_add(2 as size_t));
    (*hisptr).timestamp = os_time();
    (*hisptr).additional_data = ::core::ptr::null_mut::<AdditionalData>();
    *(*hisptr)
        .hisstr
        .offset(new_entrylen.wrapping_add(1 as size_t) as isize) = sep as ::core::ffi::c_char;
    (*hisptr).hisstrlen = new_entrylen;
    (*hisnum.ptr())[histype as usize] += 1;
    (*hisptr).hisnum = (*hisnum.ptr())[histype as usize];
    if histype == HIST_SEARCH as ::core::ffi::c_int && in_map as ::core::ffi::c_int != 0 {
        last_maptick.set(maptick.get());
    }
}
unsafe extern "C" fn get_history_idx(mut histype: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if hislen.get() == 0 as ::core::ffi::c_int
        || histype < 0 as ::core::ffi::c_int
        || histype >= HIST_COUNT as ::core::ffi::c_int
        || (*hisidx.ptr())[histype as usize] < 0 as ::core::ffi::c_int
    {
        return -1 as ::core::ffi::c_int;
    }
    return (*(*history.ptr())[histype as usize]
        .offset((*hisidx.ptr())[histype as usize] as isize))
    .hisnum;
}
unsafe extern "C" fn calc_hist_idx(
    mut histype: ::core::ffi::c_int,
    mut num: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    if hislen.get() == 0 as ::core::ffi::c_int
        || histype < 0 as ::core::ffi::c_int
        || histype >= HIST_COUNT as ::core::ffi::c_int
        || {
            i = (*hisidx.ptr())[histype as usize];
            i < 0 as ::core::ffi::c_int
        }
        || num == 0 as ::core::ffi::c_int
    {
        return -1 as ::core::ffi::c_int;
    }
    let mut hist: *mut histentry_T = (*history.ptr())[histype as usize] as *mut histentry_T;
    if num > 0 as ::core::ffi::c_int {
        let mut wrapped: bool = false_0 != 0;
        while (*hist.offset(i as isize)).hisnum > num {
            i -= 1;
            if i >= 0 as ::core::ffi::c_int {
                continue;
            }
            if wrapped {
                break;
            }
            i += hislen.get();
            wrapped = true_0 != 0;
        }
        if i >= 0 as ::core::ffi::c_int
            && (*hist.offset(i as isize)).hisnum == num
            && !(*hist.offset(i as isize)).hisstr.is_null()
        {
            return i;
        }
    } else if -num <= hislen.get() {
        i += num + 1 as ::core::ffi::c_int;
        if i < 0 as ::core::ffi::c_int {
            i += hislen.get();
        }
        if !(*hist.offset(i as isize)).hisstr.is_null() {
            return i;
        }
    }
    return -1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn clr_history(histype: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if hislen.get() != 0 as ::core::ffi::c_int
        && histype >= 0 as ::core::ffi::c_int
        && histype < HIST_COUNT as ::core::ffi::c_int
    {
        let mut hisptr: *mut histentry_T = (*history.ptr())[histype as usize] as *mut histentry_T;
        let mut i: ::core::ffi::c_int = hislen.get();
        loop {
            let c2rust_fresh0 = i;
            i = i - 1;
            if c2rust_fresh0 == 0 {
                break;
            }
            hist_free_entry(hisptr);
            hisptr = hisptr.offset(1);
        }
        (*hisidx.ptr())[histype as usize] = -1 as ::core::ffi::c_int;
        (*hisnum.ptr())[histype as usize] = 0 as ::core::ffi::c_int;
        return OK;
    }
    return FAIL;
}
unsafe extern "C" fn del_history_entry(
    mut histype: ::core::ffi::c_int,
    mut str: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if hislen.get() == 0 as ::core::ffi::c_int
        || histype < 0 as ::core::ffi::c_int
        || histype >= HIST_COUNT as ::core::ffi::c_int
        || *str as ::core::ffi::c_int == NUL
        || (*hisidx.ptr())[histype as usize] < 0 as ::core::ffi::c_int
    {
        return false_0;
    }
    let idx: ::core::ffi::c_int = (*hisidx.ptr())[histype as usize];
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    regmatch.regprog = vim_regcomp(str, RE_MAGIC + RE_STRING);
    if regmatch.regprog.is_null() {
        return false_0;
    }
    regmatch.rm_ic = false_0 != 0;
    let mut found: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = idx;
    let mut last: ::core::ffi::c_int = idx;
    loop {
        let mut hisptr: *mut histentry_T =
            (*(history.ptr() as *mut *mut histentry_T).offset(histype as isize)).offset(i as isize);
        if (*hisptr).hisstr.is_null() {
            break;
        }
        if vim_regexec(&raw mut regmatch, (*hisptr).hisstr, 0 as colnr_T) {
            found = true_0 != 0;
            hist_free_entry(hisptr);
        } else {
            if i != last {
                *(*history.ptr())[histype as usize].offset(last as isize) = *hisptr;
                clear_hist_entry(hisptr);
            }
            last -= 1;
            if last < 0 as ::core::ffi::c_int {
                last += hislen.get();
            }
        }
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            i += hislen.get();
        }
        if i == idx {
            break;
        }
    }
    if (*(*history.ptr())[histype as usize].offset(idx as isize))
        .hisstr
        .is_null()
    {
        (*hisidx.ptr())[histype as usize] = -1 as ::core::ffi::c_int;
    }
    vim_regfree(regmatch.regprog);
    return found as ::core::ffi::c_int;
}
unsafe extern "C" fn del_history_idx(
    mut histype: ::core::ffi::c_int,
    mut idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = calc_hist_idx(histype, idx);
    if i < 0 as ::core::ffi::c_int {
        return false_0;
    }
    idx = (*hisidx.ptr())[histype as usize];
    hist_free_entry(
        (*(history.ptr() as *mut *mut histentry_T).offset(histype as isize)).offset(i as isize),
    );
    if histype == HIST_SEARCH as ::core::ffi::c_int
        && maptick.get() == last_maptick.get()
        && i == idx
    {
        last_maptick.set(-1 as ::core::ffi::c_int);
    }
    while i != idx {
        let mut j: ::core::ffi::c_int = (i + 1 as ::core::ffi::c_int) % hislen.get();
        *(*history.ptr())[histype as usize].offset(i as isize) =
            *(*history.ptr())[histype as usize].offset(j as isize);
        i = j;
    }
    clear_hist_entry(
        (*(history.ptr() as *mut *mut histentry_T).offset(histype as isize)).offset(idx as isize),
    );
    i -= 1;
    if i < 0 as ::core::ffi::c_int {
        i += hislen.get();
    }
    (*hisidx.ptr())[histype as usize] = i;
    return true_0;
}
pub unsafe extern "C" fn f_histadd(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = false_0 as varnumber_T;
    if check_secure() {
        return;
    }
    let mut str: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut histype: HistoryType = (if !str.is_null() {
        get_histtype(str, strlen(str), false_0 != 0) as ::core::ffi::c_int
    } else {
        HIST_INVALID as ::core::ffi::c_int
    }) as HistoryType;
    if histype as ::core::ffi::c_int == HIST_INVALID as ::core::ffi::c_int {
        return;
    }
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    str = tv_get_string_buf(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    if *str as ::core::ffi::c_int == NUL {
        return;
    }
    init_history();
    add_to_history(
        histype as ::core::ffi::c_int,
        str,
        strlen(str),
        false_0 != 0,
        NUL,
    );
    (*rettv).vval.v_number = true_0 as varnumber_T;
}
pub unsafe extern "C" fn f_histdel(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut n: ::core::ffi::c_int = 0;
    let str: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if str.is_null() {
        n = 0 as ::core::ffi::c_int;
    } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        n = clr_history(get_histtype(str, strlen(str), false_0 != 0) as ::core::ffi::c_int);
    } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        n = del_history_idx(
            get_histtype(str, strlen(str), false_0 != 0) as ::core::ffi::c_int,
            tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int,
        );
    } else {
        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
        n = del_history_entry(
            get_histtype(str, strlen(str), false_0 != 0) as ::core::ffi::c_int,
            tv_get_string_buf(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut buf as *mut ::core::ffi::c_char,
            ) as *mut ::core::ffi::c_char,
        );
    }
    (*rettv).vval.v_number = n as varnumber_T;
}
pub unsafe extern "C" fn f_histget(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let str: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if str.is_null() {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        let mut idx: ::core::ffi::c_int = 0;
        let mut type_0: HistoryType = get_histtype(str, strlen(str), false_0 != 0);
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            idx = get_history_idx(type_0 as ::core::ffi::c_int);
        } else {
            idx = tv_get_number_chk(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                ::core::ptr::null_mut::<bool>(),
            ) as ::core::ffi::c_int;
        }
        idx = calc_hist_idx(type_0 as ::core::ffi::c_int, idx);
        if idx < 0 as ::core::ffi::c_int {
            (*rettv).vval.v_string =
                xstrnsave(b"\0".as_ptr() as *const ::core::ffi::c_char, 0 as size_t);
        } else {
            (*rettv).vval.v_string = xstrnsave(
                (*(*history.ptr())[type_0 as usize].offset(idx as isize)).hisstr,
                (*(*history.ptr())[type_0 as usize].offset(idx as isize)).hisstrlen,
            );
        }
    }
    (*rettv).v_type = VAR_STRING;
}
pub unsafe extern "C" fn f_histnr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let histname: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut i: HistoryType = (if histname.is_null() {
        HIST_INVALID as ::core::ffi::c_int
    } else {
        get_histtype(histname, strlen(histname), false_0 != 0) as ::core::ffi::c_int
    }) as HistoryType;
    if i as ::core::ffi::c_int != HIST_INVALID as ::core::ffi::c_int {
        (*rettv).vval.v_number = get_history_idx(i as ::core::ffi::c_int) as varnumber_T;
    } else {
        (*rettv).vval.v_number = HIST_INVALID as ::core::ffi::c_int as varnumber_T;
    };
}
pub unsafe extern "C" fn ex_history(mut eap: *mut exarg_T) {
    let mut histype1: ::core::ffi::c_int = HIST_CMD as ::core::ffi::c_int;
    let mut histype2: ::core::ffi::c_int = HIST_CMD as ::core::ffi::c_int;
    let mut hisidx1: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut hisidx2: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    if hislen.get() == 0 as ::core::ffi::c_int {
        msg(
            gettext(b"'history' option is zero\0".as_ptr() as *const ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
        return;
    }
    if !(ascii_isdigit(*arg as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        || *arg as ::core::ffi::c_int == '-' as ::core::ffi::c_int
        || *arg as ::core::ffi::c_int == ',' as ::core::ffi::c_int)
    {
        end = arg;
        while *end as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *end as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *end as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *end as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || !vim_strchr(
                b":=@>/?\0".as_ptr() as *const ::core::ffi::c_char,
                *end as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
        {
            end = end.offset(1);
        }
        histype1 =
            get_histtype(arg, end.offset_from(arg) as size_t, false_0 != 0) as ::core::ffi::c_int;
        if histype1 == HIST_INVALID as ::core::ffi::c_int {
            if strncasecmp(
                arg,
                b"all\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                end.offset_from(arg) as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                histype1 = 0 as ::core::ffi::c_int;
                histype2 = HIST_COUNT as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
            } else {
                semsg(
                    gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                    arg,
                );
                return;
            }
        } else {
            histype2 = histype1;
        }
    } else {
        end = arg;
    }
    if get_list_range(&raw mut end, &raw mut hisidx1, &raw mut hisidx2) == 0
        || *end as ::core::ffi::c_int != NUL
    {
        if *end as ::core::ffi::c_int != NUL {
            semsg(
                gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                end,
            );
        } else {
            semsg(
                gettext(&raw const e_val_too_large as *const ::core::ffi::c_char),
                arg,
            );
        }
        return;
    }
    while !got_int.get() && histype1 <= histype2 {
        '_c2rust_label: {
            if !(*history_names.ptr())[histype1 as usize].is_null() {
            } else {
                __assert_fail(
                    b"history_names[histype1] != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/cmdhist.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    641 as ::core::ffi::c_uint,
                    b"void ex_history(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        vim_snprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            b"\n      #  %s history\0".as_ptr() as *const ::core::ffi::c_char,
            (*history_names.ptr())[histype1 as usize],
        );
        msg_puts_title(IObuff.ptr() as *mut ::core::ffi::c_char);
        let mut idx: ::core::ffi::c_int = (*hisidx.ptr())[histype1 as usize];
        let mut hist: *mut histentry_T = (*history.ptr())[histype1 as usize] as *mut histentry_T;
        let mut j: ::core::ffi::c_int = hisidx1;
        let mut k: ::core::ffi::c_int = hisidx2;
        if j < 0 as ::core::ffi::c_int {
            j = if -j > hislen.get() {
                0 as ::core::ffi::c_int
            } else {
                (*hist.offset(
                    ((hislen.get() + j + idx + 1 as ::core::ffi::c_int) % hislen.get()) as isize,
                ))
                .hisnum
            };
        }
        if k < 0 as ::core::ffi::c_int {
            k = if -k > hislen.get() {
                0 as ::core::ffi::c_int
            } else {
                (*hist.offset(
                    ((hislen.get() + k + idx + 1 as ::core::ffi::c_int) % hislen.get()) as isize,
                ))
                .hisnum
            };
        }
        if idx >= 0 as ::core::ffi::c_int && j <= k {
            let mut i: ::core::ffi::c_int = idx + 1 as ::core::ffi::c_int;
            while !got_int.get() {
                if i == hislen.get() {
                    i = 0 as ::core::ffi::c_int;
                }
                if !(*hist.offset(i as isize)).hisstr.is_null()
                    && (*hist.offset(i as isize)).hisnum >= j
                    && (*hist.offset(i as isize)).hisnum <= k
                    && !message_filtered((*hist.offset(i as isize)).hisstr)
                {
                    msg_putchar('\n' as ::core::ffi::c_int);
                    let mut len: ::core::ffi::c_int = snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        b"%c%6d  \0".as_ptr() as *const ::core::ffi::c_char,
                        if i == idx {
                            '>' as ::core::ffi::c_int
                        } else {
                            ' ' as ::core::ffi::c_int
                        },
                        (*hist.offset(i as isize)).hisnum,
                    );
                    if vim_strsize((*hist.offset(i as isize)).hisstr)
                        > Columns.get() - 10 as ::core::ffi::c_int
                    {
                        trunc_string(
                            (*hist.offset(i as isize)).hisstr,
                            (IObuff.ptr() as *mut ::core::ffi::c_char).offset(len as isize),
                            Columns.get() - 10 as ::core::ffi::c_int,
                            IOSIZE - len,
                        );
                    } else {
                        xstrlcpy(
                            (IObuff.ptr() as *mut ::core::ffi::c_char).offset(len as isize),
                            (*hist.offset(i as isize)).hisstr,
                            (IOSIZE - len) as size_t,
                        );
                    }
                    msg_outtrans(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        0 as ::core::ffi::c_int,
                        false_0 != 0,
                    );
                }
                if i == idx {
                    break;
                }
                i += 1;
            }
        }
        histype1 += 1;
    }
}
pub unsafe extern "C" fn hist_iter(
    iter: *const ::core::ffi::c_void,
    history_type: uint8_t,
    zero: bool,
    hist: *mut histentry_T,
) -> *const ::core::ffi::c_void {
    *hist = histentry_T {
        hisnum: 0,
        hisstr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        hisstrlen: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    };
    if (*hisidx.ptr())[history_type as usize] == -1 as ::core::ffi::c_int {
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
    let hstart: *mut histentry_T = (*(history.ptr() as *mut *mut histentry_T)
        .offset(history_type as isize))
    .offset(0 as ::core::ffi::c_int as isize);
    let hlast: *mut histentry_T = (*(history.ptr() as *mut *mut histentry_T)
        .offset(history_type as isize))
    .offset(*(hisidx.ptr() as *mut ::core::ffi::c_int).offset(history_type as isize) as isize);
    let hend: *const histentry_T = (*(history.ptr() as *mut *mut histentry_T)
        .offset(history_type as isize))
    .offset((hislen.get() - 1 as ::core::ffi::c_int) as isize);
    let mut hiter: *mut histentry_T = ::core::ptr::null_mut::<histentry_T>();
    if iter.is_null() {
        let mut hfirst: *mut histentry_T = hlast;
        loop {
            hfirst = hfirst.offset(1);
            if hfirst > hend as *mut histentry_T {
                hfirst = hstart;
            }
            if !(*hfirst).hisstr.is_null() {
                break;
            }
            if hfirst == hlast {
                break;
            }
        }
        hiter = hfirst;
    } else {
        hiter = iter as *mut histentry_T;
    }
    if hiter.is_null() {
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
    *hist = *hiter;
    if zero {
        memset(
            hiter as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<histentry_T>(),
        );
    }
    if hiter == hlast {
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
    hiter = hiter.offset(1);
    return (if hiter > hend as *mut histentry_T {
        hstart
    } else {
        hiter
    }) as *const ::core::ffi::c_void;
}
pub unsafe extern "C" fn hist_get_array(
    history_type: uint8_t,
    new_hisidx: *mut *mut ::core::ffi::c_int,
    new_hisnum: *mut *mut ::core::ffi::c_int,
) -> *mut histentry_T {
    init_history();
    *new_hisidx = (hisidx.ptr() as *mut ::core::ffi::c_int).offset(history_type as isize);
    *new_hisnum = (hisnum.ptr() as *mut ::core::ffi::c_int).offset(history_type as isize);
    return (*history.ptr())[history_type as usize] as *mut histentry_T;
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RE_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
