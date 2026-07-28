//! Everything an option *does*: `:set` and its relatives, the validation a
//! new value has to pass, the `did_set_*` callbacks that react to one, and
//! the per-scope plumbing that decides which copy of a value a window or
//! buffer is looking at.
//!
//! Everything an option *is* — its name, type, scopes, flags, variable and
//! default — lives in the generated [`crate::src::nvim::options`] table.

use crate::src::nvim::api::extmark::nvim_create_namespace;
use crate::src::nvim::api::private::helpers::{
    api_free_string, api_set_error, arena_dict, copy_string, cstr_as_string, cstr_to_string,
};
use crate::src::nvim::api::private::validate::api_err_invalid;
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::autocmd::{
    apply_autocmds, aucmd_prepbuf, aucmd_restbuf, do_filetype_autocmd,
};
use crate::src::nvim::buffer::{
    bt_prompt, buf_is_empty, do_autochdir, free_buf_options, maketitle,
};
use crate::src::nvim::change::save_file_ff;
use crate::src::nvim::charset::{
    buf_init_chartab, init_chartab, skiptowhite_esc, skipwhite, trans_characters, transchar,
    vim_str2nr, vim_strsize,
};
use crate::src::nvim::cmdexpand::cmdline_fuzzy_complete;
use crate::src::nvim::cursor_shape::parse_shape_opt;
use crate::src::nvim::decoration_provider::get_decor_provider;
use crate::src::nvim::diff::diff_buf_adjust;
use crate::src::nvim::drawscreen::{
    check_screensize, comp_col, redraw_all_later, redraw_buf_later, redraw_buf_status_later,
    redraw_later, screen_resize, showmode, status_redraw_all, status_redraw_curbuf,
};
use crate::src::nvim::eval::typval::{callback_free, tv_dict_add_tv, tv_dict_alloc, tv_free};
use crate::src::nvim::eval::vars::{
    get_vim_var_str, optval_as_tv, reset_v_option_vars, set_vim_var_string, set_vim_var_tv,
};
use crate::src::nvim::eval::window::{restore_win_noblock, switch_win_noblock};
use crate::src::nvim::eval_1::{callback_from_typval, eval_expr, last_set_msg};
use crate::src::nvim::ex_docmd::set_no_hlsearch;
use crate::src::nvim::ex_getln::{check_opt_wim, did_set_cedit, gotocmdline};
use crate::src::nvim::ex_session::{put_eol, put_line};
use crate::src::nvim::fold::{
    foldUpdateAll, foldmethodIsDiff, foldmethodIsIndent, foldmethodIsSyntax, newFoldLevel,
};
use crate::src::nvim::fuzzy::{fuzzy_match_str, fuzzymatches_to_strmatches};
use crate::src::nvim::garray::{ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight::{hl_invalidate_blends, ns_hl_def};
use crate::src::nvim::highlight_group::{highlight_changed, syn_check_group};
use crate::src::nvim::indent::{briopt_check, tabstop_set};
use crate::src::nvim::indent_c::parse_cino;
use crate::src::nvim::insexpand::{
    set_buflocal_cfu_callback, set_buflocal_cpt_callbacks, set_buflocal_ofu_callback,
};
use crate::src::nvim::keycodes::{
    find_special_key, find_special_key_in_table, get_special_key_code, get_special_key_name,
};
use crate::src::nvim::log::logmsg;
use crate::src::nvim::lua::executor::nlua_set_sctx;
use crate::src::nvim::main::{
    Columns, IObuff, NameBuff, Rows, State, bkc_flags, clear_cmdline, cmdline_row, cmdmod, curbuf,
    current_sctx, curtab, curwin, e_invarg, e_positive, e_sandbox, e_scroll, e_secure, e_trailing,
    e_unknown_option2, e_unsupportedoption, e_winheight, e_winwidth, empty_string_option,
    escape_chars, fenc_default, first_tabpage, firstbuf, firstwin, full_screen, got_int,
    info_message, lastwin, magic_overruled, need_maketitle, no_wait_return, p_ai, p_arshape,
    p_bdir, p_bin, p_bomb, p_bs, p_cdpath, p_cfu, p_ch, p_chi, p_ci, p_cin, p_cink, p_cino,
    p_cinsd, p_cinw, p_cms, p_columns, p_com, p_cpo, p_cpt, p_deco, p_dir, p_ea, p_enc, p_ep, p_et,
    p_fenc, p_fex, p_ff, p_ffs, p_ffu, p_fixeol, p_flp, p_fo, p_ft, p_hh, p_hlg, p_hls, p_icon,
    p_iminsert, p_imsearch, p_inde, p_indk, p_inex, p_inf, p_isk, p_keymap, p_kp, p_lines, p_lisp,
    p_lnr, p_lop, p_lrm, p_ma, p_magic, p_ml, p_mle, p_mouse, p_mps, p_nf, p_ofu, p_paste, p_path,
    p_pi, p_pp, p_qe, p_ri, p_rtp, p_ru, p_sbr, p_scbk, p_sh, p_shm, p_si, p_siso, p_sj, p_sm,
    p_smc, p_so, p_spc, p_spf, p_spl, p_spo, p_sps, p_sta, p_sts, p_sua, p_sw, p_swf, p_syn,
    p_tags, p_tbidi, p_tfu, p_title, p_titlelen, p_ts, p_tw, p_uc, p_udf, p_ul, p_vdir, p_verbose,
    p_vsts, p_vts, p_wbr, p_wc, p_wcm, p_wh, p_window, p_wiw, p_wm, p_wmh, p_wmw, readonlymode,
    redraw_tabline, sandbox, secure, silent_mode, spo_flags, starting, t_colors, topframe,
    updating_screen, ve_flags,
};
use crate::src::nvim::mapping::{langmap_init, put_escstr};
use crate::src::nvim::mbyte::{enc_locale, utfc_ptr2len};
use crate::src::nvim::memfile::mf_close_file;
use crate::src::nvim::memline::{ml_open_file, ml_open_files};
use crate::src::nvim::memory::{
    strequal, strnequal, xcalloc, xfree, xmalloc, xmemdupz, xrealloc, xstrchrnul, xstrdup, xstrlcpy,
};
use crate::src::nvim::message::{
    emsg, iemsg, message_filtered, msg, msg_advance, msg_ext_set_kind, msg_outtrans, msg_putchar,
    msg_puts, msg_puts_title, msg_source,
};
use crate::src::nvim::mouse::setmouse;
use crate::src::nvim::r#move::changed_window_setting;
use crate::src::nvim::normal::{do_check_scrollbind, get_vtopline};
use crate::src::nvim::options::*;
use crate::src::nvim::optionstr::{
    check_buf_options, check_illegal_path_names, check_signcolumn, check_string_option,
    clear_string_option, did_set_breakat, didset_string_options, free_string_option,
    set_chars_option,
};
use crate::src::nvim::os::env::{
    expand_env_esc, home_replace, os_env_exists, os_getenv, os_setenv, vim_getenv,
};
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::lang::{get_mess_lang, lang_init};
use crate::src::nvim::os::libc::{
    __assert_fail, abort, bind_textdomain_codeset, fprintf, fputs, gettext, getuid, memmove,
    memset, snprintf, strchr, strcmp, strcpy, strlen, strncasecmp, strncmp, strstr,
};
use crate::src::nvim::os::stdpaths::stdpaths_user_state_subpath;
use crate::src::nvim::path::{
    FullName_save, after_pathsep, invocation_path_tail, path_fnamecmp, path_tail, vim_ispathlistsep,
};
use crate::src::nvim::popupmenu::{pum_drawn, pum_redraw};
use crate::src::nvim::quickfix::qf_resize_stack;
use crate::src::nvim::runtime::{exestack, runtimepath_default, source_runtime_vim_lua};
use crate::src::nvim::spell::{
    compile_cap_prog, did_set_spell_option, init_spell_chartab, parse_spelllang,
};
use crate::src::nvim::spellfile::spell_check_msm;
use crate::src::nvim::spellsuggest::spell_check_sps;
use crate::src::nvim::strings::{
    vim_snprintf, vim_snprintf_safelen, vim_strchr, vim_strsave_escaped,
};
use crate::src::nvim::tag::set_buflocal_tfu_callback;
use crate::src::nvim::types::{
    __uid_t, Arena, CMD_index, Callback, Callback_data, CallbackType, CharsOption, DecorProvider,
    Dict, Error, ErrorType, FILE, HlAttrs, Integer, KeyDict_highlight, KeyValuePair, NS, Object,
    ObjectType, OptIndex, OptInt, OptScope, OptVal, OptValData, OptValType, RgbValue, String_0,
    Terminal, TriState, VarType, VimVarIndex, aco_save_T, auto_event, buf_T, bufref_T, colnr_T,
    dict_T, estack_T, event_T, exarg_T, expand_T, fuzmatch_str_T, garray_T, int16_t, int32_t,
    int64_t, key_value_pair, linenr_T, object, object_data, optexpand_T, optset_T, ptrdiff_t,
    regmatch_T, scid_T, sctx_T, size_t, ssize_t, switchwin_T, tabpage_T, typval_T, uint8_t,
    uint32_t, uint64_t, uvarnumber_T, vimoption_T, win_T, winopt_T, xp_prefix_T,
};
use crate::src::nvim::ui::ui_call_option_set;
use crate::src::nvim::undo::{bufIsChanged, curbufIsChanged, u_compute_hash, u_read_undo, u_sync};
use crate::src::nvim::window::{
    check_colorcolumn, command_height, frame_new_height, global_stl_height, last_status, min_rows,
    min_rows_for_all_tabpages, set_winbar, set_winbar_win, tabline_height, win_comp_pos,
    win_comp_scroll, win_default_scroll, win_equal, win_find_tabpage, win_new_screen_rows,
    win_setheight, win_setwidth,
};
use crate::src::nvim::winfloat::win_float_update_statusline;
use core::ffi::{c_char, c_int, c_uchar, c_uint, c_void};
unsafe extern "C" {
    fn vim_regexec(rmp: *mut regmatch_T, line: *const c_char, col: colnr_T) -> bool;
    fn on_scrollback_option_changed(term: *mut Terminal);
    fn ll_resize_stack(wp: *mut win_T, n: c_int);
}
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub const kCallbackNone: CallbackType = 0;
pub const VAR_STRING: VarType = 2;
pub const MAXCOL: c_uint = 2147483647;
pub const HL_GLOBAL: c_uint = 16384;
pub const HLF_W: c_uint = 26;
pub const NUMBUFLEN: c_uint = 65;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_BS_COMMA: c_uint = 4;
pub const XP_BS_THREE: c_uint = 2;
pub const XP_BS_ONE: c_uint = 1;
pub const EXPAND_KEYMAP: c_int = 55;
pub const EXPAND_SETTING_SUBTRACT: c_int = 53;
pub const EXPAND_STRING_SETTING: c_int = 52;
pub const EXPAND_OWNSYNTAX: c_int = 38;
pub const EXPAND_FILETYPE: c_int = 36;
pub const EXPAND_OLD_SETTING: c_int = 7;
pub const EXPAND_BOOL_SETTINGS: c_int = 5;
pub const EXPAND_SETTINGS: c_int = 4;
pub const EXPAND_DIRECTORIES: c_int = 3;
pub const EXPAND_FILES: c_int = 2;
pub const EXPAND_NOTHING: c_int = 0;
pub const EXPAND_UNSUCCESSFUL: c_int = -2;
pub type OptFlags = c_uint;
pub const kOptFlagColon: OptFlags = 33554432;
pub const kOptFlagFunc: OptFlags = 16777216;
pub const kOptFlagMLE: OptFlags = 8388608;
pub const kOptFlagHLOnly: OptFlags = 4194304;
pub const kOptFlagNDname: OptFlags = 2097152;
pub const kOptFlagCurswant: OptFlags = 1048576;
pub const kOptFlagPriMkrc: OptFlags = 524288;
pub const kOptFlagInsecure: OptFlags = 262144;
pub const kOptFlagNFname: OptFlags = 131072;
pub const kOptFlagNoGlob: OptFlags = 65536;
pub const kOptFlagGettext: OptFlags = 32768;
pub const kOptFlagSecure: OptFlags = 16384;
pub const kOptFlagFlagList: OptFlags = 8192;
pub const kOptFlagNoDup: OptFlags = 4096;
pub const kOptFlagOneComma: OptFlags = 3072;
pub const kOptFlagComma: OptFlags = 1024;
pub const kOptFlagRedrAll: OptFlags = 768;
pub const kOptFlagRedrBuf: OptFlags = 512;
pub const kOptFlagRedrWin: OptFlags = 256;
pub const kOptFlagRedrStat: OptFlags = 128;
pub const kOptFlagRedrTabl: OptFlags = 64;
pub const kOptFlagUIOption: OptFlags = 32;
pub const kOptFlagNoMkrc: OptFlags = 16;
pub const kOptFlagWasSet: OptFlags = 8;
pub const kOptFlagNoDefault: OptFlags = 4;
pub const kOptFlagNoDefExp: OptFlags = 2;
pub const kOptFlagExpand: OptFlags = 1;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
pub const kOptScopeBuf: OptScope = 2;
pub const kOptScopeWin: OptScope = 1;
pub const kOptScopeGlobal: OptScope = 0;
pub type set_op_T = c_uint;
pub const OP_REMOVING: set_op_T = 3;
pub const OP_PREPENDING: set_op_T = 2;
pub const OP_ADDING: set_op_T = 1;
pub const OP_NONE: set_op_T = 0;
pub const CMD_setlocal: CMD_index = 402;
pub const CMD_setglobal: CMD_index = 401;
pub const CMOD_NOSWAPFILE: c_uint = 8192;
pub const EVENT_SYNTAX: auto_event = 110;
pub const EVENT_OPTIONSET: auto_event = 85;
pub const EVENT_BUFDELETE: auto_event = 2;
pub const EVENT_BUFADD: auto_event = 0;
pub const SHM_WRI: c_uint = 119;
pub const SHM_LINES: c_uint = 108;
pub const SHM_MOD: c_uint = 109;
pub const SHM_RO: c_uint = 114;
pub const STR2NR_ALL: c_uint = 15;
pub const UPD_CLEAR: c_uint = 50;
pub const UPD_NOT_VALID: c_uint = 40;
pub const UPD_SOME_VALID: c_uint = 35;
pub const VV_OPTION_TYPE: VimVarIndex = 67;
pub const VV_OPTION_COMMAND: VimVarIndex = 66;
pub const VV_OPTION_OLDGLOBAL: VimVarIndex = 65;
pub const VV_OPTION_OLDLOCAL: VimVarIndex = 64;
pub const VV_OPTION_OLD: VimVarIndex = 63;
pub const VV_OPTION_NEW: VimVarIndex = 62;
pub const VV_WARNINGMSG: VimVarIndex = 4;
pub const FUZZY_SCORE_NONE: c_int = -2147483648;
pub const MODE_TERMINAL: c_uint = 128;
pub const FSK_SIMPLIFY: c_uint = 8;
pub const FSK_KEEP_X_KEY: c_uint = 2;
pub const FSK_KEYCODE: c_uint = 1;
pub const BCO_NOHELP: c_uint = 4;
pub const BCO_ALWAYS: c_uint = 2;
pub const BCO_ENTER: c_uint = 1;
pub const OPT_SKIPRTP: c_uint = 128;
pub const OPT_ONECOLUMN: c_uint = 32;
pub const OPT_NOWIN: c_uint = 16;
pub const OPT_WINONLY: c_uint = 8;
pub const OPT_MODELINE: c_uint = 4;
pub const OPT_LOCAL: c_uint = 2;
pub const OPT_GLOBAL: c_uint = 1;
pub const STATUS_HEIGHT: c_uint = 1;
pub const DIP_ALL: c_uint = 1;
pub const MIN_COLUMNS: c_uint = 12;
pub const MAX_SEARCH_COUNT: c_uint = 9999;
pub const kListchars: CharsOption = 1;
pub const kFillchars: CharsOption = 0;
pub type set_prefix_T = c_uint;
pub const PREFIX_INV: set_prefix_T = 2;
pub const PREFIX_NONE: set_prefix_T = 1;
pub const PREFIX_NO: set_prefix_T = 0;
pub const NULL: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub const DEFAULT_MAXPATHL: c_int = 4096 as c_int;
pub const MAXPATHL: c_int = DEFAULT_MAXPATHL;
pub const ROOT_UID: c_int = 0 as c_int;
pub const BF_SYN_SET: c_int = 0x200 as c_int;
pub const B_IMODE_USE_INSERT: c_int = -1 as c_int;
pub const B_IMODE_NONE: c_int = 0 as c_int;
pub const B_IMODE_LAST: c_int = 1 as c_int;
pub const KEYMAP_INIT: c_int = 1 as c_int;
pub const NULL_STRING: String_0 = String_0 {
    data: ::core::ptr::null_mut::<c_char>(),
    size: 0 as size_t,
};
pub const LOGLVL_INF: c_int = 2 as c_int;
pub const OK: c_int = 1 as c_int;
pub const FAIL: c_int = 0 as c_int;
pub const NUL: c_int = '\0' as c_int;
pub const TAB: c_int = '\t' as c_int;
pub const CTRL_F_STR: [c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [c_char; 2]>(*b"\x06\0") };
pub const Ctrl_C: c_int = 3 as c_int;
pub const PATHSEPSTR: [c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [c_char; 2]>(*b"/\0") };
pub const FORCE_BIN: c_int = 1 as c_int;
pub const HLATTRS_INIT: HlAttrs = HlAttrs {
    rgb_ae_attr: 0 as int32_t,
    cterm_ae_attr: 0 as int32_t,
    rgb_fg_color: -1 as RgbValue,
    rgb_bg_color: -1 as RgbValue,
    rgb_sp_color: -1 as RgbValue,
    cterm_fg_color: 0 as int16_t,
    cterm_bg_color: 0 as int16_t,
    hl_blend: -1 as int32_t,
    url: -1 as int32_t,
};
pub const HIGHLIGHT_INIT: [c_char; 779] = unsafe {
    ::core::mem::transmute::<
        [u8; 779],
        [c_char; 779],
    >(
        *b"8:SpecialKey,~:EndOfBuffer,z:TermCursor,@:NonText,d:Directory,e:ErrorMsg,i:IncSearch,l:Search,y:CurSearch,m:MoreMsg,M:ModeMsg,n:LineNr,a:LineNrAbove,b:LineNrBelow,N:CursorLineNr,G:CursorLineSign,O:CursorLineFold,r:Question,s:StatusLine,S:StatusLineNC,c:VertSplit,t:Title,v:Visual,V:VisualNOS,w:WarningMsg,W:WildMenu,f:Folded,F:FoldColumn,A:DiffAdd,C:DiffChange,D:DiffDelete,T:DiffText,E:DiffTextAdd,>:SignColumn,-:Conceal,B:SpellBad,P:SpellCap,R:SpellRare,L:SpellLocal,+:Pmenu,=:PmenuSel,k:PmenuMatch,<:PmenuMatchSel,[:PmenuKind,]:PmenuKindSel,{:PmenuExtra,}:PmenuExtraSel,x:PmenuSbar,X:PmenuThumb,*:TabLine,#:TabLineSel,_:TabLineFill,!:CursorColumn,.:CursorLine,o:ColorColumn,q:QuickFixLine,z:StatusLineTerm,Z:StatusLineTermNC,g:MsgArea,h:ComplMatchIns,0:Whitespace,I:PreInsert\0",
    )
};
pub const DFLT_EFM: [c_char; 667] = unsafe {
    ::core::mem::transmute::<
        [u8; 667],
        [c_char; 667],
    >(
        *b"%*[^\"]\"%f\"%*\\D%l: %m,\"%f\"%*\\D%l: %m,%-Gg%\\?make[%*\\d]: *** [%f:%l:%m,%-Gg%\\?make: *** [%f:%l:%m,%-G%f:%l: (Each undeclared identifier is reported only once,%-G%f:%l: for each function it appears in.),%-GIn file included from %f:%l:%c:,%-GIn file included from %f:%l:%c\\,,%-GIn file included from %f:%l:%c,%-GIn file included from %f:%l,%-G%*[ ]from %f:%l:%c,%-G%*[ ]from %f:%l:,%-G%*[ ]from %f:%l\\,,%-G%*[ ]from %f:%l,%f:%l:%c:%m,%f(%l):%m,%f:%l:%m,\"%f\"\\, line %l%*\\D%c%*[^ ] %m,%D%*\\a[%*\\d]: Entering directory %*[`']%f',%X%*\\a[%*\\d]: Leaving directory %*[`']%f',%D%*\\a: Entering directory %*[`']%f',%X%*\\a: Leaving directory %*[`']%f',%DMaking %*\\a in %f,%f|%l| %m\0",
    )
};
pub const DFLT_GFN: [c_char; 55] = unsafe {
    ::core::mem::transmute::<[u8; 55], [c_char; 55]>(
        *b"Source Code Pro,DejaVu Sans Mono,Courier New,monospace\0",
    )
};
pub const DFLT_GREPFORMAT: [c_char; 26] =
    unsafe { ::core::mem::transmute::<[u8; 26], [c_char; 26]>(*b"%f:%l:%m,%f:%l%m,%f  %l%m\0") };
pub const ENC_DFLT: [c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [c_char; 6]>(*b"utf-8\0") };
pub const EOL_UNIX: c_int = 0 as c_int;
pub const EOL_DOS: c_int = 1 as c_int;
pub const EOL_MAC: c_int = 2 as c_int;
pub const DFLT_FO_VIM: [c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [c_char; 5]>(*b"tcqj\0") };
pub const MAX_MCO: c_int = 6 as c_int;
pub const CPO_BUFOPT: c_int = 's' as c_int;
pub const CPO_BUFOPTGLOB: c_int = 'S' as c_int;
pub const CPO_VIM: [c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [c_char; 9]>(*b"aABceFs_\0") };
pub const BS_START: c_int = 's' as c_int;
pub const BS_NOSTOP: c_int = 'p' as c_int;
pub const LISPWORD_VALUE: [c_char; 746] = unsafe {
    ::core::mem::transmute::<
        [u8; 746],
        [c_char; 746],
    >(
        *b"defun,define,defmacro,set!,lambda,if,case,let,flet,let*,letrec,do,do*,define-syntax,let-syntax,letrec-syntax,destructuring-bind,defpackage,defparameter,defstruct,deftype,defvar,do-all-symbols,do-external-symbols,do-symbols,dolist,dotimes,ecase,etypecase,eval-when,labels,macrolet,multiple-value-bind,multiple-value-call,multiple-value-prog1,multiple-value-setq,prog1,progv,typecase,unless,unwind-protect,when,with-input-from-string,with-open-file,with-open-stream,with-output-to-string,with-package-iterator,define-condition,handler-bind,handler-case,restart-bind,restart-case,with-simple-restart,store-value,use-value,muffle-warning,abort,continue,with-slots,with-slots*,with-accessors,with-accessors*,defclass,defmethod,print-unreadable-object\0",
    )
};
pub static p_vfile: GlobalCell<*mut c_char> =
    GlobalCell::new((empty_string_option.as_raw() as *const _) as *mut c_char);
pub const NO_LOCAL_UNDOLEVEL: c_int = -123456 as c_int;
pub const SB_MAX: c_int = 1000000 as c_int;
pub const MAX_NUMBERWIDTH: c_int = 20 as c_int;
pub const TABSTOP_MAX: c_int = 9999 as c_int;
pub const SHAPE_CURSOR: c_int = 2 as c_int;
pub const IOSIZE: c_int = 1024 as c_int + 1 as c_int;
pub const DFLT_ERRORFILE: [c_char; 11] =
    unsafe { ::core::mem::transmute::<[u8; 11], [c_char; 11]>(*b"errors.err\0") };
pub const DFLT_HELPFILE: [c_char; 25] =
    unsafe { ::core::mem::transmute::<[u8; 25], [c_char; 25]>(*b"$VIMRUNTIME/doc/help.txt\0") };
pub const NO_SCREEN: c_int = 2 as c_int;
pub const DFLT_COLS: c_int = 80 as c_int;
pub const DFLT_ROWS: c_int = 24 as c_int;
pub const SID_NONE: c_int = -6 as c_int;
pub const K_ZERO: c_int = -(255 as c_int + (('X' as c_int) << 8 as c_int));
pub const K_KENTER: c_int = -('K' as c_int + (('A' as c_int) << 8 as c_int));
#[inline]
unsafe extern "C" fn is_power_of_two(mut x: uint64_t) -> bool {
    return x != 0 as uint64_t && x & x.wrapping_sub(1 as uint64_t) == 0 as uint64_t;
}
#[inline]
unsafe extern "C" fn optval_type_get_name(type_0: OptValType) -> *const c_char {
    match type_0 as c_int {
        -1 => return b"nil\0".as_ptr() as *const c_char,
        0 => return b"boolean\0".as_ptr() as *const c_char,
        1 => return b"number\0".as_ptr() as *const c_char,
        2 => return b"string\0".as_ptr() as *const c_char,
        _ => {}
    }
    unreachable!();
}
static e_unknown_option: GlobalCell<[c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [c_char; 21]>(*b"E518: Unknown option\0")
});
static e_not_allowed_in_modeline: GlobalCell<[c_char; 32]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 32], [c_char; 32]>(*b"E520: Not allowed in a modeline\0")
});
static e_not_allowed_in_modeline_when_modelineexpr_is_off: GlobalCell<[c_char; 59]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 59], [c_char; 59]>(
            *b"E992: Not allowed in a modeline when 'modelineexpr' is off\0",
        )
    });
static e_number_required_after_equal: GlobalCell<[c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [c_char; 30]>(*b"E521: Number required after =\0")
});
static e_preview_window_already_exists: GlobalCell<[c_char; 38]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 38], [c_char; 38]>(*b"E590: A preview window already exists\0")
});
static e_cannot_have_negative_or_zero_number_of_quickfix: GlobalCell<[c_char; 72]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 72], [c_char; 72]>(
            *b"E1542: Cannot have a negative or zero number of quickfix/location lists\0",
        )
    });
static e_cannot_have_more_than_hundred_quickfix: GlobalCell<[c_char; 63]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 63], [c_char; 63]>(
            *b"E1543: Cannot have more than a hundred quickfix/location lists\0",
        )
    });
static p_term: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
static p_ttytype: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
static p_et_nobin: GlobalCell<c_int> = GlobalCell::new(0);
static p_ml_nobin: GlobalCell<c_int> = GlobalCell::new(0);
static p_tw_nobin: GlobalCell<OptInt> = GlobalCell::new(0);
static p_wm_nobin: GlobalCell<OptInt> = GlobalCell::new(0);
static p_ai_nopaste: GlobalCell<c_int> = GlobalCell::new(0);
static p_et_nopaste: GlobalCell<c_int> = GlobalCell::new(0);
static p_sts_nopaste: GlobalCell<OptInt> = GlobalCell::new(0);
static p_tw_nopaste: GlobalCell<OptInt> = GlobalCell::new(0);
static p_wm_nopaste: GlobalCell<OptInt> = GlobalCell::new(0);
static p_vsts_nopaste: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub const OPTION_COUNT: usize = ::core::mem::size_of::<[vimoption_T; 374]>()
    .wrapping_div(::core::mem::size_of::<vimoption_T>())
    .wrapping_div(
        (::core::mem::size_of::<[vimoption_T; 374]>()
            .wrapping_rem(::core::mem::size_of::<vimoption_T>())
            == 0) as c_int as usize,
    );
static p_bin_dep_opts: GlobalCell<[c_int; 5]> = GlobalCell::new([
    kOptTextwidth as c_int,
    kOptWrapmargin as c_int,
    kOptModeline as c_int,
    kOptExpandtab as c_int,
    kOptInvalid as c_int,
]);
static p_paste_dep_opts: GlobalCell<[c_int; 11]> = GlobalCell::new([
    kOptAutoindent as c_int,
    kOptExpandtab as c_int,
    kOptRuler as c_int,
    kOptShowmatch as c_int,
    kOptSmarttab as c_int,
    kOptSofttabstop as c_int,
    kOptTextwidth as c_int,
    kOptWrapmargin as c_int,
    kOptRevins as c_int,
    kOptVarsofttabstop as c_int,
    kOptInvalid as c_int,
]);
pub unsafe extern "C" fn set_init_tablocal() {
    p_ch.set(
        (*options.ptr())[kOptCmdheight as c_int as usize]
            .def_val
            .data
            .number,
    );
}
unsafe extern "C" fn set_init_default_shell() {
    let mut shell: *mut c_char = os_getenv(b"SHELL\0".as_ptr() as *const c_char);
    if !shell.is_null() {
        if !vim_strchr(shell, ' ' as c_int).is_null() {
            let len: size_t = strlen(shell).wrapping_add(3 as size_t);
            let cmd: *mut c_char = xmalloc(len) as *mut c_char;
            snprintf(cmd, len, b"\"%s\"\0".as_ptr() as *const c_char, shell);
            set_string_default(kOptShell, cmd, true_0 != 0);
        } else {
            set_string_default(kOptShell, shell, false_0 != 0);
        }
        xfree(shell as *mut c_void);
    }
}
unsafe extern "C" fn set_init_default_backupskip() {
    static names: GlobalCell<[*mut c_char; 4]> = GlobalCell::new([
        b"\0".as_ptr() as *const c_char as *mut c_char,
        b"TMPDIR\0".as_ptr() as *const c_char as *mut c_char,
        b"TEMP\0".as_ptr() as *const c_char as *mut c_char,
        b"TMP\0".as_ptr() as *const c_char as *mut c_char,
    ]);
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<c_void>(),
    };
    let mut opt_idx: OptIndex = kOptBackupskip;
    ga_init(&raw mut ga, 1 as c_int, 100 as c_int);
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<[*mut c_char; 4]>()
        .wrapping_div(::core::mem::size_of::<*mut c_char>())
        .wrapping_div(
            (::core::mem::size_of::<[*mut c_char; 4]>()
                .wrapping_rem(::core::mem::size_of::<*mut c_char>())
                == 0) as c_int as usize,
        )
    {
        let mut mustfree: bool = true_0 != 0;
        let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
        let mut plen: size_t = 0;
        if *(*names.ptr())[i as usize] as c_int == NUL {
            p = b"/tmp\0".as_ptr() as *const c_char as *mut c_char;
            plen = ::core::mem::size_of::<[c_char; 5]>().wrapping_sub(1 as usize) as size_t;
            mustfree = false_0 != 0;
        } else {
            p = vim_getenv((*names.ptr())[i as usize] as *const c_char);
            plen = 0 as size_t;
        }
        if !p.is_null() && *p as c_int != NUL {
            let mut has_trailing_path_sep: bool = false_0 != 0;
            if plen == 0 as size_t {
                plen = strlen(p);
                if after_pathsep(p, p.offset(plen as isize)) != 0 {
                    has_trailing_path_sep = true_0 != 0;
                }
            }
            let mut itemsize: size_t = plen
                .wrapping_add(
                    (if has_trailing_path_sep as c_int != 0 {
                        0 as c_int
                    } else {
                        1 as c_int
                    }) as size_t,
                )
                .wrapping_add(2 as size_t);
            let mut item: *mut c_char = xmalloc(itemsize) as *mut c_char;
            let mut itemseplen: size_t = (if ga.ga_len == 0 as c_int {
                0 as c_int
            } else {
                1 as c_int
            }) as size_t;
            let mut itemlen: size_t = vim_snprintf(
                item,
                itemsize,
                b"%s%s*\0".as_ptr() as *const c_char,
                p,
                if has_trailing_path_sep as c_int != 0 {
                    b"\0".as_ptr() as *const c_char
                } else {
                    PATHSEPSTR.as_ptr()
                },
            ) as size_t;
            if find_dup_item(
                ga.ga_data as *const c_char,
                item,
                itemlen,
                (*options.ptr())[opt_idx as usize].flags,
            )
            .is_null()
            {
                ga_grow(
                    &raw mut ga,
                    itemseplen.wrapping_add(itemlen).wrapping_add(1 as size_t) as c_int,
                );
                ga.ga_len += vim_snprintf(
                    (ga.ga_data as *mut c_char).offset(ga.ga_len as isize),
                    itemseplen.wrapping_add(itemlen).wrapping_add(1 as size_t),
                    b"%s%s\0".as_ptr() as *const c_char,
                    if itemseplen > 0 as size_t {
                        b",\0".as_ptr() as *const c_char
                    } else {
                        b"\0".as_ptr() as *const c_char
                    },
                    item,
                );
            }
            xfree(item as *mut c_void);
        }
        if mustfree {
            xfree(p as *mut c_void);
        }
        i = i.wrapping_add(1);
    }
    if !ga.ga_data.is_null() {
        set_string_default(kOptBackupskip, ga.ga_data as *mut c_char, true_0 != 0);
    }
}
unsafe extern "C" fn set_init_default_cdpath() {
    let mut cdpath: *mut c_char = vim_getenv(b"CDPATH\0".as_ptr() as *const c_char);
    if cdpath.is_null() {
        return;
    }
    let mut buf: *mut c_char = xmalloc(
        (2 as size_t)
            .wrapping_mul(strlen(cdpath))
            .wrapping_add(2 as size_t),
    ) as *mut c_char;
    *buf.offset(0 as c_int as isize) = ',' as c_char;
    let mut j: c_int = 1 as c_int;
    let mut i: c_int = 0 as c_int;
    while *cdpath.offset(i as isize) as c_int != NUL {
        if vim_ispathlistsep(*cdpath.offset(i as isize) as c_int) {
            let c2rust_fresh0 = j;
            j = j + 1;
            *buf.offset(c2rust_fresh0 as isize) = ',' as c_char;
        } else {
            if *cdpath.offset(i as isize) as c_int == ' ' as c_int
                || *cdpath.offset(i as isize) as c_int == ',' as c_int
            {
                let c2rust_fresh1 = j;
                j = j + 1;
                *buf.offset(c2rust_fresh1 as isize) = '\\' as c_char;
            }
            let c2rust_fresh2 = j;
            j = j + 1;
            *buf.offset(c2rust_fresh2 as isize) = *cdpath.offset(i as isize);
        }
        i += 1;
    }
    *buf.offset(j as isize) = NUL as c_char;
    change_option_default(
        kOptCdpath,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(buf),
            },
        },
    );
    xfree(cdpath as *mut c_void);
}
unsafe extern "C" fn set_init_expand_env() {
    let mut opt_idx: OptIndex = kOptAleph;
    while (opt_idx as c_int) < kOptCount {
        let mut opt: *mut vimoption_T =
            (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
        if (*opt).flags & kOptFlagNoDefExp as c_int as uint32_t == 0 {
            let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
            if (*opt).flags & kOptFlagGettext as c_int as uint32_t != 0 && !(*opt).var.is_null() {
                p = gettext(*((*opt).var as *mut *mut c_char));
            } else {
                p = option_expand(opt_idx, ::core::ptr::null::<c_char>());
            }
            if !p.is_null() {
                set_option_varp(
                    opt_idx,
                    (*opt).var,
                    OptVal {
                        type_0: kOptValTypeString,
                        data: OptValData {
                            string: cstr_to_string(p),
                        },
                    },
                    true_0 != 0,
                );
                change_option_default(
                    opt_idx,
                    OptVal {
                        type_0: kOptValTypeString,
                        data: OptValData {
                            string: cstr_to_string(p),
                        },
                    },
                );
            }
        }
        opt_idx += 1;
    }
}
unsafe extern "C" fn set_init_fenc_default() {
    let mut p: *mut c_char = enc_locale();
    if p.is_null() {
        p = xmemdupz(
            b"utf-8\0".as_ptr() as *const c_char as *const c_void,
            ::core::mem::size_of::<[c_char; 6]>().wrapping_sub(1 as size_t),
        ) as *mut c_char;
    }
    fenc_default.set(p);
}
pub unsafe extern "C" fn set_init_1(mut clean_arg: bool) {
    langmap_init();
    alloc_options_default();
    set_init_default_shell();
    set_init_default_backupskip();
    set_init_default_cdpath();
    let mut backupdir: *mut c_char = stdpaths_user_state_subpath(
        b"backup\0".as_ptr() as *const c_char,
        2 as size_t,
        true_0 != 0,
    );
    let backupdir_len: size_t = strlen(backupdir);
    backupdir = xrealloc(
        backupdir as *mut c_void,
        backupdir_len.wrapping_add(3 as size_t),
    ) as *mut c_char;
    memmove(
        backupdir.offset(2 as c_int as isize) as *mut c_void,
        backupdir as *const c_void,
        backupdir_len.wrapping_add(1 as size_t),
    );
    memmove(
        backupdir as *mut c_void,
        b".,\0".as_ptr() as *const c_char as *const c_void,
        2 as size_t,
    );
    set_string_default(kOptBackupdir, backupdir, true_0 != 0);
    set_string_default(
        kOptViewdir,
        stdpaths_user_state_subpath(
            b"view\0".as_ptr() as *const c_char,
            2 as size_t,
            true_0 != 0,
        ),
        true_0 != 0,
    );
    set_string_default(
        kOptDirectory,
        stdpaths_user_state_subpath(
            b"swap\0".as_ptr() as *const c_char,
            2 as size_t,
            true_0 != 0,
        ),
        true_0 != 0,
    );
    set_string_default(
        kOptUndodir,
        stdpaths_user_state_subpath(
            b"undo\0".as_ptr() as *const c_char,
            2 as size_t,
            true_0 != 0,
        ),
        true_0 != 0,
    );
    let mut rtp: *mut c_char = runtimepath_default(clean_arg);
    if !rtp.is_null() {
        set_string_default(kOptRuntimepath, rtp, true_0 != 0);
        set_string_default(kOptPackpath, rtp, false_0 != 0);
        rtp = ::core::ptr::null_mut::<c_char>();
    }
    set_options_default(0 as c_int);
    (*curbuf.get()).b_p_initialized = true_0 != 0;
    (*curbuf.get()).b_p_ac = -1 as c_int;
    (*curbuf.get()).b_p_ar = -1 as c_int;
    (*curbuf.get()).b_p_fs = -1 as c_int;
    (*curbuf.get()).b_p_ul = NO_LOCAL_UNDOLEVEL as OptInt;
    check_buf_options(curbuf.get());
    check_win_options(curwin.get());
    check_options();
    last_status(false_0 != 0);
    didset_options();
    init_spell_chartab();
    set_init_expand_env();
    if os_env_exists(b"NVIM_NOTTYFAST\0".as_ptr() as *const c_char, false_0 != 0) {
        set_option_value_give_err(
            kOptTtyfast,
            OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData { boolean: kFalse },
            },
            0 as c_int,
        );
    }
    save_file_ff(curbuf.get());
    if os_env_exists(b"MLTERM\0".as_ptr() as *const c_char, false_0 != 0) {
        set_option_value_give_err(
            kOptTermbidi,
            OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData { boolean: kTrue },
            },
            0 as c_int,
        );
    }
    didset_options2();
    lang_init();
    set_init_fenc_default();
    bind_textdomain_codeset(PROJECT_NAME.as_ptr(), p_enc.get());
    set_helplang_default(get_mess_lang());
}
pub unsafe extern "C" fn get_option_default(opt_idx: OptIndex, mut opt_flags: c_int) -> OptVal {
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    let mut is_global_local_option: bool = option_is_global_local(opt_idx);
    if opt_idx as c_int == kOptModeline as c_int && getuid() == ROOT_UID as __uid_t {
        return OptVal {
            type_0: kOptValTypeBoolean,
            data: OptValData { boolean: kFalse },
        };
    }
    if opt_flags & OPT_LOCAL as c_int != 0 && is_global_local_option as c_int != 0 {
        return get_option_unset_value(opt_idx);
    } else if option_has_type(opt_idx, kOptValTypeString) as c_int != 0
        && (*opt).flags & kOptFlagNoDefExp as c_int as uint32_t == 0
    {
        let mut s: *mut c_char = option_expand(opt_idx, (*opt).def_val.data.string.data);
        return if s.is_null() {
            (*opt).def_val
        } else {
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(s),
                },
            }
        };
    } else {
        return (*opt).def_val;
    };
}
unsafe extern "C" fn alloc_options_default() {
    let mut opt_idx: OptIndex = kOptAleph;
    while (opt_idx as c_int) < kOptCount {
        (*options.ptr())[opt_idx as usize].def_val =
            optval_copy((*options.ptr())[opt_idx as usize].def_val);
        opt_idx += 1;
    }
}
unsafe extern "C" fn change_option_default(opt_idx: OptIndex, mut value: OptVal) {
    optval_free((*options.ptr())[opt_idx as usize].def_val);
    (*options.ptr())[opt_idx as usize].def_val = value;
}
unsafe extern "C" fn set_option_default(opt_idx: OptIndex, mut opt_flags: c_int) {
    let mut both: bool = opt_flags & (OPT_LOCAL as c_int | OPT_GLOBAL as c_int) == 0 as c_int;
    let mut def_val: OptVal = get_option_default(opt_idx, opt_flags);
    set_option_direct(opt_idx, def_val, opt_flags, (*current_sctx.ptr()).sc_sid);
    if opt_idx as c_int == kOptScroll as c_int {
        win_comp_scroll(curwin.get());
    }
    let mut flagsp: *mut uint32_t = insecure_flag(curwin.get(), opt_idx, opt_flags);
    *flagsp = *flagsp & !(kOptFlagInsecure as c_int as uint32_t);
    if both {
        flagsp = insecure_flag(curwin.get(), opt_idx, OPT_LOCAL as c_int);
        *flagsp = *flagsp & !(kOptFlagInsecure as c_int as uint32_t);
    }
}
unsafe extern "C" fn set_options_default(mut opt_flags: c_int) {
    let mut opt_idx: OptIndex = kOptAleph;
    while (opt_idx as c_int) < kOptCount {
        if (*options.ptr())[opt_idx as usize].flags & kOptFlagNoDefault as c_int as uint32_t == 0 {
            set_option_default(opt_idx, opt_flags);
        }
        opt_idx += 1;
    }
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            win_comp_scroll(wp);
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    parse_cino(curbuf.get());
}
unsafe extern "C" fn set_string_default(
    mut opt_idx: OptIndex,
    mut val: *mut c_char,
    mut allocated: bool,
) {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                546 as c_uint,
                b"void set_string_default(OptIndex, char *, _Bool)\0".as_ptr() as *const c_char,
            );
        }
    };
    change_option_default(
        opt_idx,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(if allocated as c_int != 0 {
                    val
                } else {
                    xstrdup(val)
                }),
            },
        },
    );
}
unsafe extern "C" fn find_dup_item(
    mut origval: *const c_char,
    mut newval: *const c_char,
    newvallen: size_t,
    mut flags: uint32_t,
) -> *const c_char {
    if origval.is_null() {
        return ::core::ptr::null::<c_char>();
    }
    let mut bs: c_int = 0 as c_int;
    let mut s: *const c_char = origval;
    while *s as c_int != NUL {
        if (flags & kOptFlagComma as c_int as uint32_t == 0
            || s == origval
            || *s.offset(-1 as c_int as isize) as c_int == ',' as c_int && bs & 1 as c_int == 0)
            && strncmp(s, newval, newvallen) == 0 as c_int
            && (flags & kOptFlagComma as c_int as uint32_t == 0
                || *s.offset(newvallen as isize) as c_int == ',' as c_int
                || *s.offset(newvallen as isize) as c_int == NUL)
        {
            return s;
        }
        if s > origval.offset(1 as c_int as isize)
            && *s.offset(-1 as c_int as isize) as c_int == '\\' as c_int
            && *s.offset(-2 as c_int as isize) as c_int != ',' as c_int
            || s == origval.offset(1 as c_int as isize)
                && *s.offset(-1 as c_int as isize) as c_int == '\\' as c_int
        {
            bs += 1;
        } else {
            bs = 0 as c_int;
        }
        s = s.offset(1);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn set_init_2(mut _headless: bool) {
    logmsg(
        LOGLVL_INF,
        ::core::ptr::null::<c_char>(),
        b"set_init_2\0".as_ptr() as *const c_char,
        613 as c_int,
        true_0 != 0,
        b"startup runtimepath/packpath value: %s\0".as_ptr() as *const c_char,
        p_rtp.get(),
    );
    if (*options.ptr())[kOptScroll as c_int as usize].flags & kOptFlagWasSet as c_int as uint32_t
        == 0
    {
        set_option_default(kOptScroll, OPT_LOCAL as c_int);
    }
    comp_col();
    if !option_was_set(kOptWindow) {
        p_window.set((Rows.get() - 1 as c_int) as OptInt);
    }
    change_option_default(
        kOptWindow,
        OptVal {
            type_0: kOptValTypeNumber,
            data: OptValData {
                number: (Rows.get() - 1 as c_int) as OptInt,
            },
        },
    );
}
pub unsafe extern "C" fn set_init_3() {
    parse_shape_opt(SHAPE_CURSOR);
    let mut do_srr: bool = (*options.ptr())[kOptShellredir as c_int as usize].flags
        & kOptFlagWasSet as c_int as uint32_t
        == 0;
    let mut do_sp: bool = (*options.ptr())[kOptShellpipe as c_int as usize].flags
        & kOptFlagWasSet as c_int as uint32_t
        == 0;
    let mut len: size_t = 0 as size_t;
    let mut p: *mut c_char = invocation_path_tail(p_sh.get(), &raw mut len) as *mut c_char;
    p = xmemdupz(p as *const c_void, len) as *mut c_char;
    let mut is_csh: bool = path_fnamecmp(p, b"csh\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"tcsh\0".as_ptr() as *const c_char) == 0 as c_int;
    let mut is_known_shell: bool = path_fnamecmp(p, b"sh\0".as_ptr() as *const c_char)
        == 0 as c_int
        || path_fnamecmp(p, b"ksh\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"mksh\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"pdksh\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"zsh\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"zsh-beta\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"bash\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"fish\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"ash\0".as_ptr() as *const c_char) == 0 as c_int
        || path_fnamecmp(p, b"dash\0".as_ptr() as *const c_char) == 0 as c_int;
    if is_csh as c_int != 0 || is_known_shell as c_int != 0 {
        if do_sp {
            let sp: OptVal = if is_csh as c_int != 0 {
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: String_0 {
                            data: b"|& tee\0".as_ptr() as *const c_char as *mut c_char,
                            size: ::core::mem::size_of::<[c_char; 7]>().wrapping_sub(1 as size_t),
                        },
                    },
                }
            } else {
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: String_0 {
                            data: b"2>&1| tee\0".as_ptr() as *const c_char as *mut c_char,
                            size: ::core::mem::size_of::<[c_char; 10]>().wrapping_sub(1 as size_t),
                        },
                    },
                }
            };
            set_option_direct(kOptShellpipe, sp, 0 as c_int, SID_NONE);
            change_option_default(kOptShellpipe, optval_copy(sp));
        }
        if do_srr {
            let srr: OptVal = if is_csh as c_int != 0 {
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: String_0 {
                            data: b">&\0".as_ptr() as *const c_char as *mut c_char,
                            size: ::core::mem::size_of::<[c_char; 3]>().wrapping_sub(1 as size_t),
                        },
                    },
                }
            } else {
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: String_0 {
                            data: b">%s 2>&1\0".as_ptr() as *const c_char as *mut c_char,
                            size: ::core::mem::size_of::<[c_char; 9]>().wrapping_sub(1 as size_t),
                        },
                    },
                }
            };
            set_option_direct(kOptShellredir, srr, 0 as c_int, SID_NONE);
            change_option_default(kOptShellredir, optval_copy(srr));
        }
    }
    xfree(p as *mut c_void);
    if buf_is_empty(curbuf.get()) {
        if (*options.ptr())[kOptFileformats as c_int as usize].flags
            & kOptFlagWasSet as c_int as uint32_t
            != 0
        {
            set_fileformat(default_fileformat(), OPT_LOCAL as c_int);
        }
    }
    set_title_defaults();
}
pub unsafe extern "C" fn set_helplang_default(mut lang: *const c_char) {
    if lang.is_null() {
        return;
    }
    let lang_len: size_t = strlen(lang);
    if lang_len < 2 as size_t {
        return;
    }
    if (*options.ptr())[kOptHelplang as c_int as usize].flags & kOptFlagWasSet as c_int as uint32_t
        != 0
    {
        return;
    }
    free_string_option(p_hlg.get());
    p_hlg.set(xmemdupz(lang as *const c_void, lang_len) as *mut c_char);
    if strncasecmp(
        p_hlg.get(),
        b"zh_\0".as_ptr() as *const c_char as *mut c_char,
        3 as c_int as size_t,
    ) == 0 as c_int
        && lang_len >= 5 as size_t
    {
        *(*p_hlg.ptr()).offset(0 as c_int as isize) =
            (if (*(*p_hlg.ptr()).offset(3 as c_int as isize) as c_int) < 'A' as c_int
                || *(*p_hlg.ptr()).offset(3 as c_int as isize) as c_int > 'Z' as c_int
            {
                *(*p_hlg.ptr()).offset(3 as c_int as isize) as c_int
            } else {
                *(*p_hlg.ptr()).offset(3 as c_int as isize) as c_int + ('a' as c_int - 'A' as c_int)
            }) as c_char;
        *(*p_hlg.ptr()).offset(1 as c_int as isize) =
            (if (*(*p_hlg.ptr()).offset(4 as c_int as isize) as c_int) < 'A' as c_int
                || *(*p_hlg.ptr()).offset(4 as c_int as isize) as c_int > 'Z' as c_int
            {
                *(*p_hlg.ptr()).offset(4 as c_int as isize) as c_int
            } else {
                *(*p_hlg.ptr()).offset(4 as c_int as isize) as c_int + ('a' as c_int - 'A' as c_int)
            }) as c_char;
    } else if lang_len != 0 && *p_hlg.get() as c_int == 'C' as c_int {
        *(*p_hlg.ptr()).offset(0 as c_int as isize) = 'e' as c_char;
        *(*p_hlg.ptr()).offset(1 as c_int as isize) = 'n' as c_char;
    }
    *(*p_hlg.ptr()).offset(2 as c_int as isize) = NUL as c_char;
}
pub unsafe extern "C" fn set_title_defaults() {
    if (*options.ptr())[kOptTitle as c_int as usize].flags & kOptFlagWasSet as c_int as uint32_t
        == 0
    {
        change_option_default(
            kOptTitle,
            OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData { boolean: kFalse },
            },
        );
        p_title.set(0 as c_int);
    }
    if (*options.ptr())[kOptIcon as c_int as usize].flags & kOptFlagWasSet as c_int as uint32_t == 0
    {
        change_option_default(
            kOptIcon,
            OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData { boolean: kFalse },
            },
        );
        p_icon.set(0 as c_int);
    }
}
pub unsafe extern "C" fn ex_set(mut eap: *mut exarg_T) {
    let mut flags: c_int = 0 as c_int;
    if (*eap).cmdidx as c_int == CMD_setlocal as c_int {
        flags = OPT_LOCAL as c_int;
    } else if (*eap).cmdidx as c_int == CMD_setglobal as c_int {
        flags = OPT_GLOBAL as c_int;
    }
    if (*eap).forceit != 0 {
        flags |= OPT_ONECOLUMN as c_int;
    }
    do_set((*eap).arg, flags);
}
unsafe extern "C" fn stropt_copy_value(
    mut origval: *const c_char,
    mut argp: *mut *mut c_char,
    mut op: set_op_T,
    mut _flags: uint32_t,
) -> *mut c_char {
    let mut arg: *mut c_char = *argp;
    let mut newlen: size_t = strlen(arg).wrapping_add(1 as size_t);
    if op as c_uint != OP_NONE as c_int as c_uint {
        newlen = newlen.wrapping_add(strlen(origval).wrapping_add(1 as size_t));
    }
    let mut newval: *mut c_char = xmalloc(newlen) as *mut c_char;
    let mut s: *mut c_char = newval;
    while *arg as c_int != NUL && !ascii_iswhite(*arg as c_int) {
        if *arg as c_int == '\\' as c_int && *arg.offset(1 as c_int as isize) as c_int != NUL {
            arg = arg.offset(1);
        }
        let mut i: c_int = utfc_ptr2len(arg);
        if i > 1 as c_int {
            memmove(s as *mut c_void, arg as *const c_void, i as size_t);
            arg = arg.offset(i as isize);
            s = s.offset(i as isize);
        } else {
            let c2rust_fresh4 = arg;
            arg = arg.offset(1);
            let c2rust_fresh5 = s;
            s = s.offset(1);
            *c2rust_fresh5 = *c2rust_fresh4;
        }
    }
    *s = NUL as c_char;
    *argp = arg;
    return newval;
}
unsafe extern "C" fn stropt_expand_envvar(
    mut opt_idx: OptIndex,
    mut origval: *const c_char,
    mut newval: *mut c_char,
    mut op: set_op_T,
) -> *mut c_char {
    let mut s: *mut c_char = option_expand(opt_idx, newval);
    if s.is_null() {
        return newval;
    }
    xfree(newval as *mut c_void);
    let mut newlen: uint32_t = (strlen(s) as uint32_t).wrapping_add(1 as uint32_t);
    if op as c_uint != OP_NONE as c_int as c_uint {
        newlen = (newlen as c_uint)
            .wrapping_add((strlen(origval) as c_uint).wrapping_add(1 as c_uint))
            as uint32_t;
    }
    newval = xmalloc(newlen as size_t) as *mut c_char;
    strcpy(newval, s);
    return newval;
}
unsafe extern "C" fn stropt_concat_with_comma(
    mut origval: *const c_char,
    mut newval: *mut c_char,
    mut op: set_op_T,
    mut flags: uint32_t,
) {
    let mut len: c_int = 0 as c_int;
    let mut comma: c_int = (flags & kOptFlagComma as c_int as uint32_t != 0
        && *origval as c_int != NUL
        && *newval as c_int != NUL) as c_int;
    if op as c_uint == OP_ADDING as c_int as c_uint {
        len = strlen(origval) as c_int;
        if comma != 0
            && len > 1 as c_int
            && flags & kOptFlagOneComma as c_int as uint32_t
                == kOptFlagOneComma as c_int as uint32_t
            && *origval.offset((len - 1 as c_int) as isize) as c_int == ',' as c_int
            && *origval.offset((len - 2 as c_int) as isize) as c_int != '\\' as c_int
        {
            len -= 1;
        }
        memmove(
            newval.offset(len as isize).offset(comma as isize) as *mut c_void,
            newval as *const c_void,
            strlen(newval).wrapping_add(1 as size_t),
        );
        memmove(
            newval as *mut c_void,
            origval as *const c_void,
            len as size_t,
        );
    } else {
        len = strlen(newval) as c_int;
        memmove(
            newval.offset(len as isize).offset(comma as isize) as *mut c_void,
            origval as *const c_void,
            strlen(origval).wrapping_add(1 as size_t),
        );
    }
    if comma != 0 {
        *newval.offset(len as isize) = ',' as c_char;
    }
}
unsafe extern "C" fn stropt_remove_val(
    mut origval: *const c_char,
    mut newval: *mut c_char,
    mut flags: uint32_t,
    mut strval: *const c_char,
    mut len: c_int,
) {
    strcpy(newval, origval as *mut c_char);
    if *strval != 0 {
        if flags & kOptFlagComma as c_int as uint32_t != 0 {
            if strval == origval {
                if *strval.offset(len as isize) as c_int == ',' as c_int {
                    len += 1;
                }
            } else {
                strval = strval.offset(-1);
                len += 1;
            }
        }
        memmove(
            newval.offset(strval.offset_from(origval) as isize) as *mut c_void,
            strval.offset(len as isize) as *const c_void,
            strlen(strval.offset(len as isize)).wrapping_add(1 as size_t),
        );
    }
}
unsafe extern "C" fn find_key_item(
    mut src: *mut c_char,
    mut key: *mut c_char,
    mut keylen: ptrdiff_t,
    mut itemlenp: *mut ptrdiff_t,
) -> *mut c_char {
    let mut p: *mut c_char = src;
    while *p as c_int != NUL {
        if (p == src || *p.offset(-(1 as c_int as isize)) as c_int == ',' as c_int)
            && strncmp(p, key, keylen as size_t) == 0 as c_int
        {
            let mut end: *mut c_char = vim_strchr(p, ',' as c_int);
            if end.is_null() {
                end = p.offset(strlen(p) as isize);
            }
            *itemlenp = end.offset_from(p) as ptrdiff_t;
            return p;
        }
        p = p.offset(1);
    }
    return ::core::ptr::null_mut::<c_char>();
}
unsafe extern "C" fn remove_comma_item(
    mut str: *const c_char,
    mut item: *mut c_char,
    mut itemlen: ptrdiff_t,
) {
    if *item.offset(itemlen as isize) as c_int == ',' as c_int {
        memmove(
            item as *mut c_void,
            item.offset(itemlen as isize).offset(1 as c_int as isize) as *const c_void,
            strlen(item.offset(itemlen as isize).offset(1 as c_int as isize))
                .wrapping_add(1 as size_t),
        );
    } else if item > str as *mut c_char
        && *item.offset(-(1 as c_int as isize)) as c_int == ',' as c_int
    {
        memmove(
            item.offset(-(1 as c_int as isize)) as *mut c_void,
            item.offset(itemlen as isize) as *const c_void,
            strlen(item.offset(itemlen as isize)).wrapping_add(1 as size_t),
        );
    } else {
        *item = NUL as c_char;
    };
}
unsafe extern "C" fn remove_key_item(
    mut str: *mut c_char,
    mut key: *mut c_char,
    mut keylen: ptrdiff_t,
    mut skip: *const c_char,
) {
    let mut itemlen: ptrdiff_t = 0;
    let mut found: *mut c_char = ::core::ptr::null_mut::<c_char>();
    loop {
        found = find_key_item(str, key, keylen, &raw mut itemlen);
        if found.is_null() {
            break;
        }
        if found == skip as *mut c_char {
            let mut next: *mut c_char = found.offset(itemlen as isize);
            if *next as c_int == ',' as c_int {
                next = next.offset(1);
            }
            found = find_key_item(next, key, keylen, &raw mut itemlen);
            if found.is_null() {
                break;
            }
        }
        remove_comma_item(str, found, itemlen);
    }
}
unsafe extern "C" fn append_item(
    mut str: *mut c_char,
    mut item: *mut c_char,
    mut item_len: ptrdiff_t,
) {
    let mut len: ptrdiff_t = strlen(str) as ptrdiff_t;
    if len > 0 as ptrdiff_t {
        let c2rust_fresh3 = len;
        len = len + 1;
        *str.offset(c2rust_fresh3 as isize) = ',' as c_char;
    }
    memmove(
        str.offset(len as isize) as *mut c_void,
        item as *const c_void,
        item_len as size_t,
    );
    *str.offset((len + item_len) as isize) = NUL as c_char;
}
unsafe extern "C" fn prepend_item(
    mut str: *mut c_char,
    mut item: *mut c_char,
    mut item_len: ptrdiff_t,
) {
    let mut len: ptrdiff_t = strlen(str) as ptrdiff_t;
    let mut comma: c_int = if len > 0 as ptrdiff_t {
        1 as c_int
    } else {
        0 as c_int
    };
    memmove(
        str.offset(item_len as isize).offset(comma as isize) as *mut c_void,
        str as *const c_void,
        (len as size_t).wrapping_add(1 as size_t),
    );
    memmove(
        str as *mut c_void,
        item as *const c_void,
        item_len as size_t,
    );
    if comma != 0 {
        *str.offset(item_len as isize) = ',' as c_char;
    }
}
unsafe extern "C" fn stropt_handle_keymatch(
    mut origval: *const c_char,
    mut newval: *mut c_char,
    mut op: set_op_T,
    mut _flags: uint32_t,
) -> bool {
    if vim_strchr(newval, ':' as c_int).is_null() && vim_strchr(newval, ',' as c_int).is_null() {
        return false_0 != 0;
    }
    let mut newval_copy: *mut c_char = xstrdup(newval);
    strcpy(newval, origval as *mut c_char);
    let mut item_start: *mut c_char = newval_copy;
    loop {
        let mut p: *mut c_char = vim_strchr(item_start, ',' as c_int);
        let mut item_len: ptrdiff_t = if p.is_null() {
            strlen(item_start) as ptrdiff_t
        } else {
            p.offset_from(item_start)
        };
        if item_len > 0 as ptrdiff_t {
            let mut colon: *mut c_char = vim_strchr(item_start, ':' as c_int);
            if !colon.is_null() && colon < item_start.offset(item_len as isize) {
                let mut keylen: ptrdiff_t = colon.offset_from(item_start) + 1 as ptrdiff_t;
                if op as c_uint == OP_ADDING as c_int as c_uint
                    || op as c_uint == OP_PREPENDING as c_int as c_uint
                {
                    let mut old_itemlen: ptrdiff_t = 0;
                    let mut found: *mut c_char =
                        find_key_item(newval, item_start, keylen, &raw mut old_itemlen);
                    if !found.is_null() {
                        if old_itemlen == item_len
                            && strncmp(found, item_start, item_len as size_t) == 0 as c_int
                        {
                            remove_key_item(newval, item_start, keylen, found);
                        } else {
                            remove_key_item(
                                newval,
                                item_start,
                                keylen,
                                ::core::ptr::null::<c_char>(),
                            );
                            if op as c_uint == OP_PREPENDING as c_int as c_uint {
                                prepend_item(newval, item_start, item_len);
                            } else {
                                append_item(newval, item_start, item_len);
                            }
                        }
                    } else if op as c_uint == OP_PREPENDING as c_int as c_uint {
                        prepend_item(newval, item_start, item_len);
                    } else {
                        append_item(newval, item_start, item_len);
                    }
                } else if op as c_uint == OP_REMOVING as c_int as c_uint {
                    remove_key_item(newval, item_start, keylen, ::core::ptr::null::<c_char>());
                }
            } else if op as c_uint == OP_ADDING as c_int as c_uint
                || op as c_uint == OP_PREPENDING as c_int as c_uint
            {
                let mut found_0: *const c_char = find_dup_item(
                    newval,
                    item_start,
                    item_len as size_t,
                    kOptFlagComma as c_int as uint32_t,
                );
                if found_0.is_null() {
                    if op as c_uint == OP_PREPENDING as c_int as c_uint {
                        prepend_item(newval, item_start, item_len);
                    } else {
                        append_item(newval, item_start, item_len);
                    }
                }
            } else if op as c_uint == OP_REMOVING as c_int as c_uint {
                let mut found_1: *mut c_char = find_dup_item(
                    newval,
                    item_start,
                    item_len as size_t,
                    kOptFlagComma as c_int as uint32_t,
                ) as *mut c_char;
                if !found_1.is_null() {
                    remove_comma_item(newval, found_1, item_len);
                }
            }
        }
        if p.is_null() {
            break;
        }
        item_start = p.offset(1 as c_int as isize);
    }
    xfree(newval_copy as *mut c_void);
    return true_0 != 0;
}
unsafe extern "C" fn stropt_remove_dupflags(mut newval: *mut c_char, mut flags: uint32_t) {
    let mut s: *mut c_char = newval;
    s = newval;
    while *s != 0 {
        if flags & kOptFlagOneComma as c_int as uint32_t != 0 {
            if *s as c_int != ',' as c_int
                && *s.offset(1 as c_int as isize) as c_int == ',' as c_int
                && !vim_strchr(s.offset(2 as c_int as isize), *s as uint8_t as c_int).is_null()
            {
                memmove(
                    s as *mut c_void,
                    s.offset(2 as c_int as isize) as *const c_void,
                    strlen(s.offset(2 as c_int as isize)).wrapping_add(1 as size_t),
                );
                continue;
            }
        } else if (flags & kOptFlagComma as c_int as uint32_t == 0 || *s as c_int != ',' as c_int)
            && !vim_strchr(s.offset(1 as c_int as isize), *s as uint8_t as c_int).is_null()
        {
            memmove(
                s as *mut c_void,
                s.offset(1 as c_int as isize) as *const c_void,
                strlen(s.offset(1 as c_int as isize)).wrapping_add(1 as size_t),
            );
            continue;
        }
        s = s.offset(1);
    }
}
unsafe extern "C" fn stropt_get_newval(
    mut _nextchar: c_int,
    mut opt_idx: OptIndex,
    mut argp: *mut *mut c_char,
    mut varp: *mut c_void,
    mut origval: *const c_char,
    mut op_arg: *mut set_op_T,
    mut flags: uint32_t,
) -> *mut c_char {
    let mut arg: *mut c_char = *argp;
    let mut op: set_op_T = *op_arg;
    let mut save_arg: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut newval: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut s: *const c_char = ::core::ptr::null::<c_char>();
    arg = arg.offset(1);
    if varp == p_kp.ptr() as *mut c_void && (*arg as c_int == NUL || *arg as c_int == ' ' as c_int)
    {
        save_arg = arg;
        arg = b":help\0".as_ptr() as *const c_char as *mut c_char;
    }
    newval = stropt_copy_value(origval, &raw mut arg, op, flags);
    if op as c_uint == OP_NONE as c_int as c_uint || flags & kOptFlagComma as c_int as uint32_t != 0
    {
        newval = stropt_expand_envvar(opt_idx, origval, newval, op);
    }
    if !(flags & kOptFlagComma as c_int as uint32_t != 0
        && flags & kOptFlagColon as c_int as uint32_t != 0
        && op as c_uint != OP_NONE as c_int as c_uint
        && stropt_handle_keymatch(origval, newval, op, flags) as c_int != 0)
    {
        let mut len: c_int = 0 as c_int;
        if op as c_uint == OP_REMOVING as c_int as c_uint
            || flags & kOptFlagNoDup as c_int as uint32_t != 0
        {
            len = strlen(newval) as c_int;
            s = find_dup_item(origval, newval, len as size_t, flags);
            if (op as c_uint == OP_ADDING as c_int as c_uint
                || op as c_uint == OP_PREPENDING as c_int as c_uint)
                && !s.is_null()
            {
                op = OP_NONE;
                strcpy(newval, origval as *mut c_char);
            }
            if s.is_null() {
                s = origval.offset(strlen(origval) as c_int as isize);
            }
        }
        if op as c_uint == OP_ADDING as c_int as c_uint
            || op as c_uint == OP_PREPENDING as c_int as c_uint
        {
            stropt_concat_with_comma(origval, newval, op, flags);
        } else if op as c_uint == OP_REMOVING as c_int as c_uint {
            stropt_remove_val(origval, newval, flags, s, len);
        }
    }
    if flags & kOptFlagFlagList as c_int as uint32_t != 0 {
        stropt_remove_dupflags(newval, flags);
    }
    if !save_arg.is_null() {
        arg = save_arg;
    }
    *argp = arg;
    *op_arg = op;
    return newval;
}
unsafe extern "C" fn get_op(mut arg: *const c_char) -> set_op_T {
    let mut op: set_op_T = OP_NONE;
    if *arg as c_int != NUL && *arg.offset(1 as c_int as isize) as c_int == '=' as c_int {
        if *arg as c_int == '+' as c_int {
            op = OP_ADDING;
        } else if *arg as c_int == '^' as c_int {
            op = OP_PREPENDING;
        } else if *arg as c_int == '-' as c_int {
            op = OP_REMOVING;
        }
    }
    return op;
}
unsafe extern "C" fn get_option_prefix(mut argp: *mut *mut c_char) -> set_prefix_T {
    if strncmp(*argp, b"no\0".as_ptr() as *const c_char, 2 as size_t) == 0 as c_int {
        *argp = (*argp).offset(2 as c_int as isize);
        return PREFIX_NO;
    } else if strncmp(*argp, b"inv\0".as_ptr() as *const c_char, 3 as size_t) == 0 as c_int {
        *argp = (*argp).offset(3 as c_int as isize);
        return PREFIX_INV;
    }
    return PREFIX_NONE;
}
unsafe extern "C" fn validate_opt_idx(
    mut win: *mut win_T,
    mut opt_idx: OptIndex,
    mut opt_flags: c_int,
    mut flags: uint32_t,
    mut prefix: set_prefix_T,
    mut errmsg: *mut *const c_char,
) -> c_int {
    if !option_has_type(opt_idx, kOptValTypeBoolean)
        && prefix as c_uint != PREFIX_NONE as c_int as c_uint
    {
        *errmsg = &raw const e_invarg as *const c_char;
        return FAIL;
    }
    if opt_flags & OPT_WINONLY as c_int != 0 && !option_is_window_local(opt_idx) {
        return FAIL;
    }
    if opt_flags & OPT_NOWIN as c_int != 0 && option_is_window_local(opt_idx) as c_int != 0 {
        return FAIL;
    }
    if opt_flags & OPT_MODELINE as c_int != 0 {
        if flags & kOptFlagSecure as c_int as uint32_t != 0 {
            *errmsg = (e_not_allowed_in_modeline.ptr() as *const _) as *const c_char;
            return FAIL;
        }
        if flags & kOptFlagMLE as c_int as uint32_t != 0 && p_mle.get() == 0 {
            *errmsg = (e_not_allowed_in_modeline_when_modelineexpr_is_off.ptr() as *const _)
                as *const c_char;
            return FAIL;
        }
        if (*win).w_onebuf_opt.wo_diff != 0
            && (opt_idx as c_int == kOptFoldmethod as c_int
                || opt_idx as c_int == kOptWrap as c_int)
        {
            return FAIL;
        }
    }
    if sandbox.get() != 0 as c_int && flags & kOptFlagSecure as c_int as uint32_t != 0 {
        *errmsg = &raw const e_sandbox as *const c_char;
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn find_tty_option_end(mut arg: *const c_char) -> *const c_char {
    if strequal(arg, b"term\0".as_ptr() as *const c_char) {
        return arg
            .offset(::core::mem::size_of::<[c_char; 5]>() as isize)
            .offset(-(1 as c_int as isize));
    } else if strequal(arg, b"ttytype\0".as_ptr() as *const c_char) {
        return arg
            .offset(::core::mem::size_of::<[c_char; 8]>() as isize)
            .offset(-(1 as c_int as isize));
    }
    let mut p: *const c_char = arg;
    let mut delimit: bool = false_0 != 0;
    if *arg.offset(0 as c_int as isize) as c_int == '<' as c_int {
        delimit = true_0 != 0;
        p = p.offset(1);
    }
    if *p.offset(0 as c_int as isize) as c_int == 't' as c_int
        && *p.offset(1 as c_int as isize) as c_int == '_' as c_int
        && *p.offset(2 as c_int as isize) as c_int != 0
        && *p.offset(3 as c_int as isize) as c_int != 0
    {
        p = p.offset(4 as c_int as isize);
    } else if delimit {
        while *p as c_int != NUL && *p as c_int != '>' as c_int {
            p = p.offset(1);
        }
    }
    if delimit {
        if *p as c_int != '>' as c_int {
            return ::core::ptr::null::<c_char>();
        }
        p = p.offset(1);
    }
    return if arg == p {
        ::core::ptr::null::<c_char>()
    } else {
        p
    };
}
pub unsafe extern "C" fn find_option_end(
    mut arg: *const c_char,
    mut opt_idxp: *mut OptIndex,
) -> *const c_char {
    let mut p: *const c_char = ::core::ptr::null::<c_char>();
    p = find_tty_option_end(arg);
    if !p.is_null() {
        *opt_idxp = kOptInvalid;
        return p;
    } else {
        p = arg;
    }
    if !(*p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
        || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint)
    {
        *opt_idxp = kOptInvalid;
        return ::core::ptr::null::<c_char>();
    }
    while *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
        || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
    {
        p = p.offset(1);
    }
    *opt_idxp = find_option_len(arg, p.offset_from(arg) as size_t);
    return p;
}
unsafe extern "C" fn get_option_newval(
    mut opt_idx: OptIndex,
    mut opt_flags: c_int,
    mut prefix: set_prefix_T,
    mut argp: *mut *mut c_char,
    mut nextchar: c_int,
    mut op: set_op_T,
    mut flags: uint32_t,
    mut varp: *mut c_void,
    mut _errbuf: *mut c_char,
    _errbuflen: size_t,
    mut errmsg: *mut *const c_char,
) -> OptVal {
    '_c2rust_label: {
        if !varp.is_null() {
        } else {
            __assert_fail(
                b"varp != NULL\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr()
                    as *const c_char,
                1322 as c_uint,
                b"OptVal get_option_newval(OptIndex, int, set_prefix_T, char **, int, set_op_T, uint32_t, void *, char *, const size_t, const char **)\0"
                    .as_ptr() as *const c_char,
            );
        }
    };
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    let mut arg: *mut c_char = *argp;
    let oldval_is_global: bool =
        option_is_global_local(opt_idx) as c_int != 0 && opt_flags & OPT_LOCAL as c_int != 0;
    let mut oldval: OptVal = optval_from_varp(
        opt_idx,
        if oldval_is_global as c_int != 0 {
            get_varp(opt)
        } else {
            varp
        },
    );
    let mut newval: OptVal = OptVal {
        type_0: kOptValTypeNil,
        data: OptValData { boolean: kFalse },
    };
    if nextchar == '&' as c_int {
        return optval_copy(get_option_default(opt_idx, OPT_GLOBAL as c_int));
    } else if nextchar == '<' as c_int {
        if option_is_global_local(opt_idx) as c_int != 0 && opt_flags & OPT_LOCAL as c_int == 0 {
            unset_option_local_value(opt_idx);
        }
        return get_option_value(opt_idx, OPT_GLOBAL as c_int);
    }
    match oldval.type_0 as c_int {
        -1 => {
            abort();
        }
        0 => {
            let mut newval_bool: TriState = kFalse;
            if nextchar == '!' as c_int {
                match oldval.data.boolean as c_int {
                    -1 => {
                        newval_bool = kNone;
                    }
                    1 => {
                        newval_bool = kFalse;
                    }
                    0 => {
                        newval_bool = kTrue;
                    }
                    _ => {}
                }
            } else if prefix as c_uint == PREFIX_INV as c_int as c_uint {
                newval_bool = (*(varp as *mut c_int) ^ 1 as c_int) as TriState;
            } else {
                newval_bool = (if prefix as c_uint == PREFIX_NO as c_int as c_uint {
                    0 as c_int
                } else {
                    1 as c_int
                }) as TriState;
            }
            newval = OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData {
                    boolean: newval_bool,
                },
            };
        }
        1 => {
            let mut oldval_num: OptInt = oldval.data.number;
            let mut newval_num: OptInt = 0;
            arg = arg.offset(1);
            if (varp as *mut OptInt == p_wc.ptr() || varp as *mut OptInt == p_wcm.ptr())
                && (*arg as c_int == '<' as c_int
                    || *arg as c_int == '^' as c_int
                    || *arg as c_int != NUL
                        && (*arg.offset(1 as c_int as isize) == 0
                            || ascii_iswhite(*arg.offset(1 as c_int as isize) as c_int) as c_int
                                != 0)
                        && !ascii_isdigit(*arg as c_int))
            {
                newval_num = string_to_key(arg) as OptInt;
                if newval_num == 0 as OptInt {
                    *errmsg = &raw const e_invarg as *const c_char;
                    return newval;
                }
            } else if *arg as c_int == '-' as c_int || ascii_isdigit(*arg as c_int) as c_int != 0 {
                let mut i: c_int = 0;
                vim_str2nr(
                    arg,
                    ::core::ptr::null_mut::<c_int>(),
                    &raw mut i,
                    STR2NR_ALL as c_int,
                    &raw mut newval_num,
                    ::core::ptr::null_mut::<uvarnumber_T>(),
                    0 as c_int,
                    true_0 != 0,
                    ::core::ptr::null_mut::<bool>(),
                );
                if i == 0 as c_int
                    || *arg.offset(i as isize) as c_int != NUL
                        && !ascii_iswhite(*arg.offset(i as isize) as c_int)
                {
                    *errmsg = (e_number_required_after_equal.ptr() as *const _) as *const c_char;
                    return newval;
                }
            } else {
                *errmsg = (e_number_required_after_equal.ptr() as *const _) as *const c_char;
                return newval;
            }
            if op as c_uint == OP_ADDING as c_int as c_uint {
                newval_num = oldval_num + newval_num;
            }
            if op as c_uint == OP_PREPENDING as c_int as c_uint {
                newval_num = oldval_num * newval_num;
            }
            if op as c_uint == OP_REMOVING as c_int as c_uint {
                newval_num = oldval_num - newval_num;
            }
            newval = OptVal {
                type_0: kOptValTypeNumber,
                data: OptValData { number: newval_num },
            };
        }
        2 => {
            let mut oldval_str: *const c_char = oldval.data.string.data;
            let mut newval_str: *const c_char = stropt_get_newval(
                nextchar,
                opt_idx,
                argp,
                varp,
                oldval_str,
                &raw mut op,
                flags,
            );
            newval = OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(newval_str),
                },
            };
        }
        _ => {}
    }
    return newval;
}
unsafe extern "C" fn do_one_set_option(
    mut opt_flags: c_int,
    mut argp: *mut *mut c_char,
    mut did_show: *mut bool,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
    mut errmsg: *mut *const c_char,
) {
    let mut prefix: set_prefix_T = get_option_prefix(argp);
    let mut arg: *mut c_char = *argp;
    let mut opt_idx: OptIndex = kOptAleph;
    let option_end: *const c_char = find_option_end(arg, &raw mut opt_idx);
    if opt_idx as c_int != kOptInvalid as c_int {
        '_c2rust_label: {
            if option_end >= arg as *const c_char {
            } else {
                __assert_fail(
                    b"option_end >= arg\0".as_ptr() as *const c_char,
                    b"src/nvim/option.rs\0".as_ptr()
                        as *const c_char,
                    1448 as c_uint,
                    b"void do_one_set_option(int, char **, _Bool *, char *, size_t, const char **)\0"
                        .as_ptr() as *const c_char,
                );
            }
        };
    } else if is_tty_option(arg) {
        return;
    } else {
        *errmsg = (e_unknown_option.ptr() as *const _) as *const c_char;
        return;
    }
    let mut afterchar: uint8_t = *option_end as uint8_t;
    let mut p: *mut c_char = option_end as *mut c_char;
    while ascii_iswhite(*p as c_int) {
        p = p.offset(1);
    }
    let mut op: set_op_T = get_op(p);
    if op as c_uint != OP_NONE as c_int as c_uint {
        p = p.offset(1);
    }
    let mut nextchar: uint8_t = *p as uint8_t;
    let mut flags: uint32_t = (*options.ptr())[opt_idx as usize].flags;
    let mut varp: *mut c_void = get_varp_scope(
        (options.ptr() as *mut vimoption_T).offset(opt_idx as isize),
        opt_flags,
    );
    if validate_opt_idx(curwin.get(), opt_idx, opt_flags, flags, prefix, errmsg) == FAIL {
        return;
    }
    if !vim_strchr(b"?=:!&<\0".as_ptr() as *const c_char, nextchar as c_int).is_null() {
        *argp = p;
        if nextchar as c_int == '&' as c_int
            && *(*argp).offset(1 as c_int as isize) as c_int == 'v' as c_int
            && *(*argp).offset(2 as c_int as isize) as c_int == 'i' as c_int
        {
            if *(*argp).offset(3 as c_int as isize) as c_int == 'm' as c_int {
                *argp = (*argp).offset(3 as c_int as isize);
            } else {
                *argp = (*argp).offset(2 as c_int as isize);
            }
        }
        if !vim_strchr(b"?!&<\0".as_ptr() as *const c_char, nextchar as c_int).is_null()
            && *(*argp).offset(1 as c_int as isize) as c_int != NUL
            && !ascii_iswhite(*(*argp).offset(1 as c_int as isize) as c_int)
        {
            *errmsg = &raw const e_trailing as *const c_char;
            return;
        }
    }
    if nextchar as c_int == '?' as c_int
        || prefix as c_uint == PREFIX_NONE as c_int as c_uint
            && vim_strchr(b"=:&<\0".as_ptr() as *const c_char, nextchar as c_int).is_null()
            && !option_has_type(opt_idx, kOptValTypeBoolean)
    {
        if *did_show {
            msg_putchar('\n' as c_int);
        } else {
            msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const c_char);
            gotocmdline(true_0 != 0);
            *did_show = true_0 != 0;
        }
        showoneopt(
            (options.ptr() as *mut vimoption_T).offset(opt_idx as isize),
            opt_flags,
        );
        if p_verbose.get() > 0 as OptInt {
            if varp == (*options.ptr())[opt_idx as usize].var {
                last_set_msg((*options.ptr())[opt_idx as usize].script_ctx);
            } else if option_has_scope(opt_idx, kOptScopeWin) {
                last_set_msg(
                    (*curwin.get()).w_onebuf_opt.wo_script_ctx
                        [option_scope_idx(opt_idx, kOptScopeWin) as usize],
                );
            } else if option_has_scope(opt_idx, kOptScopeBuf) {
                last_set_msg(
                    (*curbuf.get()).b_p_script_ctx
                        [option_scope_idx(opt_idx, kOptScopeBuf) as usize],
                );
            }
        }
        if nextchar as c_int != '?' as c_int
            && nextchar as c_int != NUL
            && !ascii_iswhite(afterchar as c_int)
        {
            *errmsg = &raw const e_trailing as *const c_char;
        }
        return;
    }
    if option_has_type(opt_idx, kOptValTypeBoolean) {
        if !vim_strchr(b"=:\0".as_ptr() as *const c_char, nextchar as c_int).is_null() {
            *errmsg = &raw const e_invarg as *const c_char;
            return;
        }
        if vim_strchr(b"!&<\0".as_ptr() as *const c_char, nextchar as c_int).is_null()
            && nextchar as c_int != NUL
            && !ascii_iswhite(afterchar as c_int)
        {
            *errmsg = &raw const e_trailing as *const c_char;
            return;
        }
    } else if vim_strchr(b"=:&<\0".as_ptr() as *const c_char, nextchar as c_int).is_null() {
        *errmsg = &raw const e_invarg as *const c_char;
        return;
    }
    let mut newval: OptVal = get_option_newval(
        opt_idx,
        opt_flags,
        prefix,
        argp,
        nextchar as c_int,
        op,
        flags,
        varp,
        errbuf,
        errbuflen,
        errmsg,
    );
    if newval.type_0 as c_int == kOptValTypeNil as c_int || !(*errmsg).is_null() {
        return;
    }
    *errmsg = set_option(
        opt_idx,
        newval,
        opt_flags,
        0 as scid_T,
        false_0 != 0,
        op as c_uint == OP_NONE as c_int as c_uint,
        errbuf,
        errbuflen,
    );
}
pub unsafe extern "C" fn do_set(mut arg: *mut c_char, mut opt_flags: c_int) -> c_int {
    let mut did_show: bool = false_0 != 0;
    if *arg as c_int == NUL {
        showoptions(false_0 != 0, opt_flags);
        did_show = true_0 != 0;
    } else {
        while *arg as c_int != NUL {
            if strncmp(arg, b"all\0".as_ptr() as *const c_char, 3 as size_t) == 0 as c_int
                && !(*arg.offset(3 as c_int as isize) as c_uint >= 'A' as c_uint
                    && *arg.offset(3 as c_int as isize) as c_uint <= 'Z' as c_uint
                    || *arg.offset(3 as c_int as isize) as c_uint >= 'a' as c_uint
                        && *arg.offset(3 as c_int as isize) as c_uint <= 'z' as c_uint)
                && opt_flags & OPT_MODELINE as c_int == 0
            {
                arg = arg.offset(3 as c_int as isize);
                if *arg as c_int == '&' as c_int {
                    arg = arg.offset(1);
                    set_options_default(opt_flags);
                    didset_options();
                    didset_options2();
                    ui_refresh_options();
                    redraw_all_later(UPD_CLEAR as c_int);
                } else {
                    showoptions(true_0 != 0, opt_flags);
                    did_show = true_0 != 0;
                }
            } else {
                let mut startarg: *mut c_char = arg;
                let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
                let mut errbuf: [c_char; 80] = [0; 80];
                do_one_set_option(
                    opt_flags,
                    &raw mut arg,
                    &raw mut did_show,
                    &raw mut errbuf as *mut c_char,
                    ::core::mem::size_of::<[c_char; 80]>(),
                    &raw mut errmsg,
                );
                let mut i: c_int = 0 as c_int;
                while i < 2 as c_int {
                    arg = skiptowhite_esc(arg);
                    arg = skipwhite(arg);
                    if *arg as c_int != '=' as c_int {
                        break;
                    }
                    i += 1;
                }
                if !errmsg.is_null() {
                    let mut i_0: c_int = vim_snprintf(
                        IObuff.ptr() as *mut c_char,
                        IOSIZE as size_t,
                        b"%s\0".as_ptr() as *const c_char,
                        gettext(errmsg),
                    ) + 2 as c_int;
                    if i_0 as isize + arg.offset_from(startarg) < IOSIZE as isize {
                        xstrlcpy(
                            (IObuff.ptr() as *mut c_char)
                                .offset(i_0 as isize)
                                .offset(-(2 as c_int as isize)),
                            b": \0".as_ptr() as *const c_char,
                            (IOSIZE - i_0 + 2 as c_int) as size_t,
                        );
                        '_c2rust_label: {
                            if arg >= startarg {
                            } else {
                                __assert_fail(
                                    b"arg >= startarg\0".as_ptr() as *const c_char,
                                    b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                                    1620 as c_uint,
                                    b"int do_set(char *, int)\0".as_ptr() as *const c_char,
                                );
                            }
                        };
                        memmove(
                            (IObuff.ptr() as *mut c_char).offset(i_0 as isize) as *mut c_void,
                            startarg as *const c_void,
                            arg.offset_from(startarg) as size_t,
                        );
                        (*IObuff.ptr())[(i_0 as isize + arg.offset_from(startarg)) as usize] =
                            NUL as c_char;
                    }
                    trans_characters(IObuff.ptr() as *mut c_char, IOSIZE);
                    (*no_wait_return.ptr()) += 1;
                    emsg(IObuff.ptr() as *mut c_char);
                    (*no_wait_return.ptr()) -= 1;
                    return FAIL;
                }
            }
            arg = skipwhite(arg);
        }
    }
    if silent_mode.get() as c_int != 0 && did_show as c_int != 0 {
        silent_mode.set(false_0 != 0);
        info_message.set(true_0 != 0);
        msg_putchar('\n' as c_int);
        silent_mode.set(true_0 != 0);
        info_message.set(false_0 != 0);
    }
    return OK;
}
unsafe extern "C" fn find_key_len(
    mut arg_arg: *const c_char,
    mut len: size_t,
    mut has_lt: bool,
) -> c_int {
    let mut key: c_int = 0 as c_int;
    let mut arg: *const c_char = arg_arg;
    if len >= 4 as size_t
        && *arg.offset(0 as c_int as isize) as c_int == 't' as c_int
        && *arg.offset(1 as c_int as isize) as c_int == '_' as c_int
    {
        if !has_lt || *arg.offset(4 as c_int as isize) as c_int == '>' as c_int {
            key = -(*arg.offset(2 as c_int as isize) as uint8_t as c_int
                + ((*arg.offset(3 as c_int as isize) as uint8_t as c_int) << 8 as c_int));
        }
    } else if has_lt {
        arg = arg.offset(-1);
        let mut modifiers: c_int = 0 as c_int;
        key = find_special_key(
            &raw mut arg,
            len.wrapping_add(1 as size_t),
            &raw mut modifiers,
            FSK_KEYCODE as c_int | FSK_KEEP_X_KEY as c_int | FSK_SIMPLIFY as c_int,
            ::core::ptr::null_mut::<bool>(),
        );
        if modifiers != 0 {
            key = 0 as c_int;
        }
    }
    return key;
}
pub unsafe extern "C" fn string_to_key(mut arg: *mut c_char) -> c_int {
    if *arg as c_int == '<' as c_int && *arg.offset(1 as c_int as isize) as c_int != 0 {
        return find_key_len(arg.offset(1 as c_int as isize), strlen(arg), true_0 != 0);
    }
    if *arg as c_int == '^' as c_int && *arg.offset(1 as c_int as isize) as c_int != 0 {
        let mut key: c_int = (if (*arg.offset(1 as c_int as isize) as uint8_t as c_int)
            < 'a' as c_int
            || *arg.offset(1 as c_int as isize) as uint8_t as c_int > 'z' as c_int
        {
            *arg.offset(1 as c_int as isize) as uint8_t as c_int
        } else {
            *arg.offset(1 as c_int as isize) as uint8_t as c_int - ('a' as c_int - 'A' as c_int)
        }) ^ 0x40 as c_int;
        if key == 0 as c_int {
            key = K_ZERO;
        }
        return key;
    }
    return *arg as uint8_t as c_int;
}
pub unsafe extern "C" fn did_set_title() {
    if starting.get() != NO_SCREEN {
        maketitle();
    }
}
pub unsafe extern "C" fn set_options_bin(
    mut oldval: c_int,
    mut newval: c_int,
    mut opt_flags: c_int,
) {
    if newval != 0 {
        if oldval == 0 {
            if opt_flags & OPT_GLOBAL as c_int == 0 {
                (*curbuf.get()).b_p_tw_nobin = (*curbuf.get()).b_p_tw;
                (*curbuf.get()).b_p_wm_nobin = (*curbuf.get()).b_p_wm;
                (*curbuf.get()).b_p_ml_nobin = (*curbuf.get()).b_p_ml;
                (*curbuf.get()).b_p_et_nobin = (*curbuf.get()).b_p_et;
            }
            if opt_flags & OPT_LOCAL as c_int == 0 {
                p_tw_nobin.set(p_tw.get());
                p_wm_nobin.set(p_wm.get());
                p_ml_nobin.set(p_ml.get());
                p_et_nobin.set(p_et.get());
            }
        }
        if opt_flags & OPT_GLOBAL as c_int == 0 {
            (*curbuf.get()).b_p_tw = 0 as OptInt;
            (*curbuf.get()).b_p_wm = 0 as OptInt;
            (*curbuf.get()).b_p_ml = 0 as c_int;
            (*curbuf.get()).b_p_et = 0 as c_int;
        }
        if opt_flags & OPT_LOCAL as c_int == 0 {
            p_tw.set(0 as OptInt);
            p_wm.set(0 as OptInt);
            p_ml.set(false_0);
            p_et.set(false_0);
            p_bin.set(true_0);
        }
    } else if oldval != 0 {
        if opt_flags & OPT_GLOBAL as c_int == 0 {
            (*curbuf.get()).b_p_tw = (*curbuf.get()).b_p_tw_nobin;
            (*curbuf.get()).b_p_wm = (*curbuf.get()).b_p_wm_nobin;
            (*curbuf.get()).b_p_ml = (*curbuf.get()).b_p_ml_nobin;
            (*curbuf.get()).b_p_et = (*curbuf.get()).b_p_et_nobin;
        }
        if opt_flags & OPT_LOCAL as c_int == 0 {
            p_tw.set(p_tw_nobin.get());
            p_wm.set(p_wm_nobin.get());
            p_ml.set(p_ml_nobin.get());
            p_et.set(p_et_nobin.get());
        }
    }
    didset_options_sctx(opt_flags, p_bin_dep_opts.ptr() as *mut c_int);
}
unsafe extern "C" fn option_expand(mut opt_idx: OptIndex, mut val: *const c_char) -> *mut c_char {
    if (*options.ptr())[opt_idx as usize].flags & kOptFlagExpand as c_int as uint32_t == 0
        || is_option_hidden(opt_idx) as c_int != 0
    {
        return ::core::ptr::null_mut::<c_char>();
    }
    if val.is_null() {
        val = *((*options.ptr())[opt_idx as usize].var as *mut *mut c_char);
    }
    if val.is_null() || strlen(val) > MAXPATHL as size_t {
        return ::core::ptr::null_mut::<c_char>();
    }
    let mut var: *mut *mut c_char = (*options.ptr())[opt_idx as usize].var as *mut *mut c_char;
    let mut esc: bool = var == p_tags.ptr() || var == p_path.ptr();
    expand_env_esc(
        val,
        NameBuff.ptr() as *mut c_char,
        MAXPATHL,
        esc,
        false_0 != 0,
        (if (*options.ptr())[opt_idx as usize].var as *mut *mut c_char == p_sps.ptr() {
            b"file:\0".as_ptr() as *const c_char
        } else {
            ::core::ptr::null::<c_char>()
        }) as *mut c_char,
    );
    if strcmp(NameBuff.ptr() as *mut c_char, val) == 0 as c_int {
        return ::core::ptr::null_mut::<c_char>();
    }
    return NameBuff.ptr() as *mut c_char;
}
unsafe extern "C" fn didset_options() {
    init_chartab();
    didset_string_options();
    spell_check_msm();
    spell_check_sps();
    compile_cap_prog((*curwin.get()).w_s);
    did_set_spell_option();
    did_set_cedit(::core::ptr::null_mut::<optset_T>());
    did_set_breakat(::core::ptr::null_mut::<optset_T>());
    didset_window_options(curwin.get(), true_0 != 0);
}
unsafe extern "C" fn didset_options2() {
    highlight_changed();
    set_chars_option(
        curwin.get(),
        (*curwin.get()).w_onebuf_opt.wo_fcs,
        kFillchars,
        true_0 != 0,
        ::core::ptr::null_mut::<c_char>(),
        0 as size_t,
    );
    set_chars_option(
        curwin.get(),
        (*curwin.get()).w_onebuf_opt.wo_lcs,
        kListchars,
        true_0 != 0,
        ::core::ptr::null_mut::<c_char>(),
        0 as size_t,
    );
    check_opt_wim();
    xfree((*curbuf.get()).b_p_vsts_array as *mut c_void);
    tabstop_set(
        (*curbuf.get()).b_p_vsts,
        &raw mut (*curbuf.get()).b_p_vsts_array,
    );
    xfree((*curbuf.get()).b_p_vts_array as *mut c_void);
    tabstop_set(
        (*curbuf.get()).b_p_vts,
        &raw mut (*curbuf.get()).b_p_vts_array,
    );
}
pub unsafe extern "C" fn check_options() {
    let mut opt_idx: OptIndex = kOptAleph;
    while (opt_idx as c_int) < kOptCount {
        if option_has_type(opt_idx, kOptValTypeString) as c_int != 0
            && !(*options.ptr())[opt_idx as usize].var.is_null()
        {
            check_string_option(get_varp(
                (options.ptr() as *mut vimoption_T).offset(opt_idx as isize),
            ) as *mut *mut c_char);
        }
        opt_idx += 1;
    }
}
pub unsafe extern "C" fn was_set_insecurely(
    wp: *mut win_T,
    mut opt_idx: OptIndex,
    mut opt_flags: c_int,
) -> c_int {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                1855 as c_uint,
                b"int was_set_insecurely(win_T *const, OptIndex, int)\0".as_ptr() as *const c_char,
            );
        }
    };
    let mut flagp: *mut uint32_t = insecure_flag(wp, opt_idx, opt_flags);
    return (*flagp & kOptFlagInsecure as c_int as uint32_t != 0 as uint32_t) as c_int;
}
pub unsafe extern "C" fn insecure_flag(
    wp: *mut win_T,
    mut opt_idx: OptIndex,
    mut opt_flags: c_int,
) -> *mut uint32_t {
    if opt_flags & OPT_LOCAL as c_int != 0 {
        '_c2rust_label: {
            if !wp.is_null() {
            } else {
                __assert_fail(
                    b"wp != NULL\0".as_ptr() as *const c_char,
                    b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                    1868 as c_uint,
                    b"uint32_t *insecure_flag(win_T *const, OptIndex, int)\0".as_ptr()
                        as *const c_char,
                );
            }
        };
        match opt_idx as c_int {
            367 => return &raw mut (*wp).w_onebuf_opt.wo_wrap_flags,
            294 => return &raw mut (*wp).w_onebuf_opt.wo_stl_flags,
            355 => return &raw mut (*wp).w_onebuf_opt.wo_wbr_flags,
            104 => return &raw mut (*wp).w_onebuf_opt.wo_fde_flags,
            113 => return &raw mut (*wp).w_onebuf_opt.wo_fdt_flags,
            148 => return &raw mut (*(*wp).w_buffer).b_p_inde_flags,
            114 => return &raw mut (*(*wp).w_buffer).b_p_fex_flags,
            146 => return &raw mut (*(*wp).w_buffer).b_p_inex_flags,
            _ => {}
        }
    } else {
        match opt_idx as c_int {
            367 => return &raw mut (*wp).w_allbuf_opt.wo_wrap_flags,
            104 => return &raw mut (*wp).w_allbuf_opt.wo_fde_flags,
            113 => return &raw mut (*wp).w_allbuf_opt.wo_fdt_flags,
            _ => {}
        }
    }
    return &raw mut (*(options.ptr() as *mut vimoption_T).offset(opt_idx as isize)).flags;
}
pub unsafe extern "C" fn redraw_titles() {
    need_maketitle.set(true_0 != 0);
    redraw_tabline.set(true_0 != 0);
}
pub unsafe extern "C" fn valid_name(mut val: *const c_char, mut allowed: *const c_char) -> bool {
    let mut s: *const c_char = val;
    while *s as c_int != NUL {
        if !(*s as c_uint >= 'A' as c_uint && *s as c_uint <= 'Z' as c_uint
            || *s as c_uint >= 'a' as c_uint && *s as c_uint <= 'z' as c_uint
            || ascii_isdigit(*s as c_int) as c_int != 0)
            && vim_strchr(allowed, *s as uint8_t as c_int).is_null()
        {
            return false_0 != 0;
        }
        s = s.offset(1);
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn check_blending(mut wp: *mut win_T) {
    (*wp).w_grid_alloc.blending = (*wp).w_onebuf_opt.wo_winbl > 0 as OptInt
        || (*wp).w_floating as c_int != 0 && (*wp).w_config.shadow as c_int != 0;
}
pub unsafe extern "C" fn parse_winhl_opt(mut winhl: *const c_char, mut wp: *mut win_T) -> bool {
    let mut p: *const c_char = empty_string_option.ptr() as *mut c_char;
    if !winhl.is_null() {
        p = winhl;
    } else if !wp.is_null() {
        p = (*wp).w_onebuf_opt.wo_winhl;
    }
    if *p == 0 {
        if !wp.is_null() && (*wp).w_ns_hl_winhl > 0 as c_int && (*wp).w_ns_hl == (*wp).w_ns_hl_winhl
        {
            (*wp).w_ns_hl = 0 as c_int;
            (*wp).w_hl_needs_update = true_0;
        }
        return true_0 != 0;
    }
    let mut ns_hl: c_int = 0 as c_int;
    if !wp.is_null() {
        if (*wp).w_ns_hl_winhl == 0 as c_int {
            (*wp).w_ns_hl_winhl = nvim_create_namespace(NULL_STRING) as c_int;
        } else {
            let mut dp: *mut DecorProvider =
                get_decor_provider((*wp).w_ns_hl_winhl as NS, true_0 != 0);
            (*dp).hl_valid += 1;
        }
        ns_hl = (*wp).w_ns_hl_winhl;
        if (*wp).w_ns_hl <= 0 as c_int {
            (*wp).w_ns_hl = (*wp).w_ns_hl_winhl;
        }
    }
    while *p != 0 {
        let mut colon: *const c_char = strchr(p, ':' as c_int);
        if colon.is_null() {
            return false_0 != 0;
        }
        let mut nlen: size_t = colon.offset_from(p) as size_t;
        let mut hi: *const c_char = colon.offset(1 as c_int as isize);
        let mut commap: *const c_char = xstrchrnul(hi, ',' as c_char);
        let mut len: size_t = commap.offset_from(hi) as size_t;
        let mut hl_id: c_int = if len != 0 {
            syn_check_group(hi, len)
        } else {
            -1 as c_int
        };
        if hl_id == 0 as c_int {
            return false_0 != 0;
        }
        let mut hl_id_link: c_int = if nlen != 0 {
            syn_check_group(p, nlen)
        } else {
            0 as c_int
        };
        if hl_id_link == 0 as c_int {
            return false_0 != 0;
        }
        if !wp.is_null() {
            let mut attrs: HlAttrs = HLATTRS_INIT;
            attrs.rgb_ae_attr = (attrs.rgb_ae_attr as c_int | HL_GLOBAL as c_int) as int32_t;
            ns_hl_def(
                ns_hl as NS,
                hl_id_link,
                attrs,
                hl_id,
                ::core::ptr::null_mut::<KeyDict_highlight>(),
            );
        }
        p = if *commap as c_int != 0 {
            commap.offset(1 as c_int as isize)
        } else {
            b"\0".as_ptr() as *const c_char
        };
    }
    if !wp.is_null() {
        (*wp).w_hl_needs_update = true_0;
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn get_option_sctx(mut opt_idx: OptIndex) -> *mut sctx_T {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                2008 as c_uint,
                b"sctx_T *get_option_sctx(OptIndex)\0".as_ptr() as *const c_char,
            );
        }
    };
    return &raw mut (*(options.ptr() as *mut vimoption_T).offset(opt_idx as isize)).script_ctx;
}
pub unsafe extern "C" fn set_option_sctx(
    mut opt_idx: OptIndex,
    mut opt_flags: c_int,
    mut script_ctx: sctx_T,
) {
    let mut both: bool = opt_flags & (OPT_LOCAL as c_int | OPT_GLOBAL as c_int) == 0 as c_int;
    if opt_flags & OPT_MODELINE as c_int == 0 {
        script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
        .es_lnum;
    }
    nlua_set_sctx(&raw mut script_ctx);
    if both as c_int != 0
        || opt_flags & OPT_GLOBAL as c_int != 0
        || option_is_global_only(opt_idx) as c_int != 0
    {
        (*options.ptr())[opt_idx as usize].script_ctx = script_ctx;
    }
    if both as c_int != 0 || opt_flags & OPT_LOCAL as c_int != 0 {
        if option_has_scope(opt_idx, kOptScopeBuf) {
            (*curbuf.get()).b_p_script_ctx[option_scope_idx(opt_idx, kOptScopeBuf) as usize] =
                script_ctx;
        } else if option_has_scope(opt_idx, kOptScopeWin) {
            (*curwin.get()).w_onebuf_opt.wo_script_ctx
                [option_scope_idx(opt_idx, kOptScopeWin) as usize] = script_ctx;
            if both {
                (*curwin.get()).w_allbuf_opt.wo_script_ctx
                    [option_scope_idx(opt_idx, kOptScopeWin) as usize] = script_ctx;
            }
        }
    }
}
unsafe extern "C" fn apply_optionset_autocmd(
    mut opt_idx: OptIndex,
    mut opt_flags: c_int,
    mut oldval: OptVal,
    mut oldval_g: OptVal,
    mut oldval_l: OptVal,
    mut newval: OptVal,
    mut errmsg: *const c_char,
) {
    if starting.get() != 0 || !errmsg.is_null() || *get_vim_var_str(VV_OPTION_TYPE) as c_int != NUL
    {
        return;
    }
    let mut buf_type: [c_char; 7] = [0; 7];
    let mut oldval_tv: typval_T = optval_as_tv(oldval, false_0 != 0);
    let mut oldval_g_tv: typval_T = optval_as_tv(oldval_g, false_0 != 0);
    let mut oldval_l_tv: typval_T = optval_as_tv(oldval_l, false_0 != 0);
    let mut newval_tv: typval_T = optval_as_tv(newval, false_0 != 0);
    set_vim_var_tv(VV_OPTION_OLD, &raw mut oldval_tv);
    set_vim_var_tv(VV_OPTION_NEW, &raw mut newval_tv);
    let mut typelen: size_t = vim_snprintf_safelen(
        &raw mut buf_type as *mut c_char,
        ::core::mem::size_of::<[c_char; 7]>(),
        b"%s\0".as_ptr() as *const c_char,
        if opt_flags & OPT_LOCAL as c_int != 0 {
            b"local\0".as_ptr() as *const c_char
        } else {
            b"global\0".as_ptr() as *const c_char
        },
    );
    set_vim_var_string(
        VV_OPTION_TYPE,
        &raw mut buf_type as *mut c_char,
        typelen as ptrdiff_t,
    );
    if opt_flags & OPT_LOCAL as c_int != 0 {
        set_vim_var_string(
            VV_OPTION_COMMAND,
            b"setlocal\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 9]>().wrapping_sub(1 as usize) as ptrdiff_t,
        );
        set_vim_var_tv(VV_OPTION_OLDLOCAL, &raw mut oldval_tv);
    }
    if opt_flags & OPT_GLOBAL as c_int != 0 {
        set_vim_var_string(
            VV_OPTION_COMMAND,
            b"setglobal\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 10]>().wrapping_sub(1 as usize) as ptrdiff_t,
        );
        set_vim_var_tv(VV_OPTION_OLDGLOBAL, &raw mut oldval_tv);
    }
    if opt_flags & (OPT_LOCAL as c_int | OPT_GLOBAL as c_int) == 0 as c_int {
        set_vim_var_string(
            VV_OPTION_COMMAND,
            b"set\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 4]>().wrapping_sub(1 as usize) as ptrdiff_t,
        );
        set_vim_var_tv(VV_OPTION_OLDLOCAL, &raw mut oldval_l_tv);
        set_vim_var_tv(VV_OPTION_OLDGLOBAL, &raw mut oldval_g_tv);
    }
    if opt_flags & OPT_MODELINE as c_int != 0 {
        set_vim_var_string(
            VV_OPTION_COMMAND,
            b"modeline\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 9]>().wrapping_sub(1 as usize) as ptrdiff_t,
        );
        set_vim_var_tv(VV_OPTION_OLDLOCAL, &raw mut oldval_tv);
    }
    apply_autocmds(
        EVENT_OPTIONSET,
        (*options.ptr())[opt_idx as usize].fullname,
        ::core::ptr::null_mut::<c_char>(),
        false_0 != 0,
        ::core::ptr::null_mut::<buf_T>(),
    );
    reset_v_option_vars();
}
pub unsafe extern "C" fn did_set_arabic(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
    if (*win).w_onebuf_opt.wo_arab != 0 {
        if p_tbidi.get() == 0 {
            if (*win).w_onebuf_opt.wo_rl == 0 {
                (*win).w_onebuf_opt.wo_rl = true_0;
                changed_window_setting(win);
            }
            if p_arshape.get() == 0 {
                p_arshape.set(true_0);
                redraw_all_later(UPD_NOT_VALID as c_int);
            }
        }
        if strcmp(p_enc.get(), b"utf-8\0".as_ptr() as *const c_char) != 0 as c_int {
            static w_arabic: GlobalCell<*mut c_char> = GlobalCell::new(
                b"W17: Arabic requires UTF-8, do ':set encoding=utf-8'\0".as_ptr() as *const c_char
                    as *mut c_char,
            );
            msg_source(HLF_W as c_int);
            msg(gettext(w_arabic.get()), HLF_W as c_int);
            set_vim_var_string(VV_WARNINGMSG, gettext(w_arabic.get()), -1 as ptrdiff_t);
        }
        p_deco.set(true_0);
        errmsg = set_option_value(
            kOptKeymap,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: b"arabic\0".as_ptr() as *const c_char as *mut c_char,
                        size: ::core::mem::size_of::<[c_char; 7]>().wrapping_sub(1 as size_t),
                    },
                },
            },
            OPT_LOCAL as c_int,
        );
    } else {
        if p_tbidi.get() == 0 {
            if (*win).w_onebuf_opt.wo_rl != 0 {
                (*win).w_onebuf_opt.wo_rl = false_0;
                changed_window_setting(win);
            }
        }
        (*(*win).w_buffer).b_p_iminsert = B_IMODE_NONE as OptInt;
        (*(*win).w_buffer).b_p_imsearch = B_IMODE_USE_INSERT as OptInt;
    }
    return errmsg;
}
pub unsafe extern "C" fn did_set_autochdir(mut _args: *mut optset_T) -> *const c_char {
    do_autochdir();
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_binary(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    set_options_bin(
        (*args).os_oldval.boolean as c_int,
        (*buf).b_p_bin,
        (*args).os_flags,
    );
    redraw_titles();
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_buflisted(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    if (*args).os_oldval.boolean as c_int != (*buf).b_p_bl {
        apply_autocmds(
            (if (*buf).b_p_bl != 0 {
                EVENT_BUFADD as c_int
            } else {
                EVENT_BUFDELETE as c_int
            }) as event_T,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            true_0 != 0,
            buf,
        );
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_cmdheight(mut args: *mut optset_T) -> *const c_char {
    let mut old_value: OptInt = (*args).os_oldval.number;
    if p_ch.get() > (Rows.get() - min_rows(curtab.get()) + 1 as c_int) as OptInt {
        p_ch.set((Rows.get() - min_rows(curtab.get()) + 1 as c_int) as OptInt);
    }
    if (p_ch.get() != old_value
        || (tabline_height() + global_stl_height() + (*topframe.get()).fr_height) as OptInt
            != Rows.get() as OptInt - p_ch.get())
        && full_screen.get() as c_int != 0
    {
        command_height();
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_diff(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    diff_buf_adjust(win);
    if foldmethodIsDiff(win) {
        foldUpdateAll(win);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_eof_eol_fixeol_bomb(mut _args: *mut optset_T) -> *const c_char {
    redraw_titles();
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_equalalways(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if p_ea.get() != 0 && (*args).os_oldval.boolean as u64 == 0 {
        win_equal(win, false_0 != 0, 0 as c_int);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_foldlevel(mut _args: *mut optset_T) -> *const c_char {
    newFoldLevel();
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_foldminlines(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    foldUpdateAll(win);
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_foldnestmax(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if foldmethodIsSyntax(win) as c_int != 0 || foldmethodIsIndent(win) as c_int != 0 {
        foldUpdateAll(win);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_helpheight(mut _args: *mut optset_T) -> *const c_char {
    if !(firstwin.get() == lastwin.get()) {
        if (*curbuf.get()).b_help as c_int != 0 && ((*curwin.get()).w_height as OptInt) < p_hh.get()
        {
            win_setheight(p_hh.get() as c_int);
        }
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_hlsearch(mut _args: *mut optset_T) -> *const c_char {
    set_no_hlsearch(false_0 != 0);
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_ignorecase(mut _args: *mut optset_T) -> *const c_char {
    if p_hls.get() != 0 {
        redraw_all_later(UPD_SOME_VALID as c_int);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_iminsert(mut _args: *mut optset_T) -> *const c_char {
    showmode();
    status_redraw_curbuf();
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_langnoremap(mut _args: *mut optset_T) -> *const c_char {
    p_lrm.set((p_lnr.get() == 0) as c_int);
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_langremap(mut _args: *mut optset_T) -> *const c_char {
    p_lnr.set((p_lrm.get() == 0) as c_int);
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_laststatus(mut args: *mut optset_T) -> *const c_char {
    let mut old_value: OptInt = (*args).os_oldval.number;
    let mut value: OptInt = (*args).os_newval.number;
    if value == 3 as OptInt && old_value != 3 as OptInt {
        frame_new_height(
            topframe.get(),
            (*topframe.get()).fr_height - STATUS_HEIGHT as c_int,
            false_0 != 0,
            false_0 != 0,
            false_0 != 0,
        );
        win_comp_pos();
        clear_cmdline.set(true_0 != 0);
    }
    if old_value == 3 as OptInt && value != 3 as OptInt {
        frame_new_height(
            topframe.get(),
            (*topframe.get()).fr_height + STATUS_HEIGHT as c_int,
            false_0 != 0,
            false_0 != 0,
            false_0 != 0,
        );
        win_comp_pos();
    }
    status_redraw_curbuf();
    last_status(false_0 != 0);
    win_float_update_statusline();
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_lines_or_columns(mut args: *mut optset_T) -> *const c_char {
    if p_lines.get() != Rows.get() as OptInt || p_columns.get() != Columns.get() as OptInt {
        if updating_screen.get() {
            let mut oldval: OptVal = OptVal {
                type_0: kOptValTypeNumber,
                data: (*args).os_oldval,
            };
            set_option_varp((*args).os_idx, (*args).os_varp, oldval, false_0 != 0);
        } else if full_screen.get() {
            screen_resize(p_columns.get() as c_int, p_lines.get() as c_int);
        } else {
            Rows.set(p_lines.get() as c_int);
            Columns.set(p_columns.get() as c_int);
            check_screensize();
            let mut new_row: c_int = (Rows.get() as OptInt
                - (if p_ch.get() > 1 as OptInt {
                    p_ch.get()
                } else {
                    1 as OptInt
                })) as c_int;
            if cmdline_row.get() > new_row && Rows.get() as OptInt > p_ch.get() {
                '_c2rust_label: {
                    if p_ch.get() >= 0 as OptInt && new_row <= 2147483647 as c_int {
                    } else {
                        __assert_fail(
                            b"p_ch >= 0 && new_row <= INT_MAX\0".as_ptr() as *const c_char,
                            b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                            2359 as c_uint,
                            b"const char *did_set_lines_or_columns(optset_T *)\0".as_ptr()
                                as *const c_char,
                        );
                    }
                };
                cmdline_row.set(new_row);
            }
        }
        if p_window.get() >= Rows.get() as OptInt || !option_was_set(kOptWindow) {
            p_window.set((Rows.get() - 1 as c_int) as OptInt);
        }
    }
    if p_sj.get() >= Rows.get() as OptInt && full_screen.get() as c_int != 0 {
        p_sj.set((Rows.get() / 2 as c_int) as OptInt);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_lisp(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    buf_init_chartab(buf, false);
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_modifiable(mut _args: *mut optset_T) -> *const c_char {
    redraw_titles();
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_modified(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    if (*args).os_newval.boolean as u64 == 0 {
        save_file_ff(buf);
    }
    redraw_titles();
    (*buf).b_modified_was_set = (*args).os_newval.boolean as c_int != 0;
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_number_relativenumber(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if *(*win).w_onebuf_opt.wo_stc as c_int != NUL {
        (*win).w_nrwidth_line_count = 0 as c_int as linenr_T;
    }
    check_signcolumn(::core::ptr::null_mut::<c_char>(), win);
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_numberwidth(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    (*win).w_nrwidth_line_count = 0 as c_int as linenr_T;
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_paste(mut _args: *mut optset_T) -> *const c_char {
    static old_p_paste: GlobalCell<c_int> = GlobalCell::new(false_0);
    static save_sm: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    static save_sta: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    static save_ru: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    static save_ri: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    if p_paste.get() != 0 {
        if old_p_paste.get() == 0 {
            let mut buf: *mut buf_T = firstbuf.get();
            while !buf.is_null() {
                (*buf).b_p_tw_nopaste = (*buf).b_p_tw;
                (*buf).b_p_wm_nopaste = (*buf).b_p_wm;
                (*buf).b_p_sts_nopaste = (*buf).b_p_sts;
                (*buf).b_p_ai_nopaste = (*buf).b_p_ai;
                (*buf).b_p_et_nopaste = (*buf).b_p_et;
                if !(*buf).b_p_vsts_nopaste.is_null() {
                    xfree((*buf).b_p_vsts_nopaste as *mut c_void);
                }
                (*buf).b_p_vsts_nopaste = if !(*buf).b_p_vsts.is_null()
                    && (*buf).b_p_vsts != empty_string_option.ptr() as *mut c_char
                {
                    xstrdup((*buf).b_p_vsts)
                } else {
                    ::core::ptr::null_mut::<c_char>()
                };
                buf = (*buf).b_next;
            }
            save_sm.set(p_sm.get());
            save_sta.set(p_sta.get());
            save_ru.set(p_ru.get());
            save_ri.set(p_ri.get());
            p_ai_nopaste.set(p_ai.get());
            p_et_nopaste.set(p_et.get());
            p_sts_nopaste.set(p_sts.get());
            p_tw_nopaste.set(p_tw.get());
            p_wm_nopaste.set(p_wm.get());
            if !(*p_vsts_nopaste.ptr()).is_null() {
                xfree(p_vsts_nopaste.get() as *mut c_void);
            }
            p_vsts_nopaste.set(
                if !(*p_vsts.ptr()).is_null()
                    && p_vsts.get() != empty_string_option.ptr() as *mut c_char
                {
                    xstrdup(p_vsts.get())
                } else {
                    ::core::ptr::null_mut::<c_char>()
                },
            );
        }
        let mut buf_0: *mut buf_T = firstbuf.get();
        while !buf_0.is_null() {
            (*buf_0).b_p_tw = 0 as OptInt;
            (*buf_0).b_p_wm = 0 as OptInt;
            (*buf_0).b_p_sts = 0 as OptInt;
            (*buf_0).b_p_ai = 0 as c_int;
            (*buf_0).b_p_et = 0 as c_int;
            if !(*buf_0).b_p_vsts.is_null() {
                free_string_option((*buf_0).b_p_vsts);
            }
            (*buf_0).b_p_vsts = empty_string_option.ptr() as *mut c_char;
            let mut ptr_: *mut *mut c_void = &raw mut (*buf_0).b_p_vsts_array as *mut *mut c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
            buf_0 = (*buf_0).b_next;
        }
        p_sm.set(0 as c_int);
        p_sta.set(0 as c_int);
        if p_ru.get() != 0 {
            status_redraw_all();
        }
        p_ru.set(0 as c_int);
        p_ri.set(0 as c_int);
        p_tw.set(0 as OptInt);
        p_wm.set(0 as OptInt);
        p_sts.set(0 as OptInt);
        p_ai.set(0 as c_int);
        p_et.set(0 as c_int);
        if !(*p_vsts.ptr()).is_null() {
            free_string_option(p_vsts.get());
        }
        p_vsts.set(empty_string_option.ptr() as *mut c_char);
    } else if old_p_paste.get() != 0 {
        let mut buf_1: *mut buf_T = firstbuf.get();
        while !buf_1.is_null() {
            (*buf_1).b_p_tw = (*buf_1).b_p_tw_nopaste;
            (*buf_1).b_p_wm = (*buf_1).b_p_wm_nopaste;
            (*buf_1).b_p_sts = (*buf_1).b_p_sts_nopaste;
            (*buf_1).b_p_ai = (*buf_1).b_p_ai_nopaste;
            (*buf_1).b_p_et = (*buf_1).b_p_et_nopaste;
            if !(*buf_1).b_p_vsts.is_null() {
                free_string_option((*buf_1).b_p_vsts);
            }
            (*buf_1).b_p_vsts = if !(*buf_1).b_p_vsts_nopaste.is_null() {
                xstrdup((*buf_1).b_p_vsts_nopaste)
            } else {
                empty_string_option.ptr() as *mut c_char
            };
            xfree((*buf_1).b_p_vsts_array as *mut c_void);
            if !(*buf_1).b_p_vsts.is_null()
                && (*buf_1).b_p_vsts != empty_string_option.ptr() as *mut c_char
            {
                tabstop_set((*buf_1).b_p_vsts, &raw mut (*buf_1).b_p_vsts_array);
            } else {
                (*buf_1).b_p_vsts_array = ::core::ptr::null_mut::<colnr_T>();
            }
            buf_1 = (*buf_1).b_next;
        }
        p_sm.set(save_sm.get());
        p_sta.set(save_sta.get());
        if p_ru.get() != save_ru.get() {
            status_redraw_all();
        }
        p_ru.set(save_ru.get());
        p_ri.set(save_ri.get());
        p_ai.set(p_ai_nopaste.get());
        p_et.set(p_et_nopaste.get());
        p_sts.set(p_sts_nopaste.get());
        p_tw.set(p_tw_nopaste.get());
        p_wm.set(p_wm_nopaste.get());
        if !(*p_vsts.ptr()).is_null() {
            free_string_option(p_vsts.get());
        }
        p_vsts.set(if !(*p_vsts_nopaste.ptr()).is_null() {
            xstrdup(p_vsts_nopaste.get())
        } else {
            empty_string_option.ptr() as *mut c_char
        });
    }
    old_p_paste.set(p_paste.get());
    didset_options_sctx(
        OPT_LOCAL as c_int | OPT_GLOBAL as c_int,
        p_paste_dep_opts.ptr() as *mut c_int,
    );
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_previewwindow(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if (*win).w_onebuf_opt.wo_pvw == 0 {
        return ::core::ptr::null::<c_char>();
    }
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_onebuf_opt.wo_pvw != 0 && wp != win {
            (*win).w_onebuf_opt.wo_pvw = false_0;
            return (e_preview_window_already_exists.ptr() as *const _) as *const c_char;
        }
        wp = (*wp).w_next;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_pumblend(mut _args: *mut optset_T) -> *const c_char {
    hl_invalidate_blends();
    if pum_drawn() {
        pum_redraw();
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_readonly(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    if (*buf).b_p_ro == 0 && (*args).os_flags & OPT_LOCAL as c_int == 0 as c_int {
        readonlymode.set(false_0 != 0);
    }
    if (*buf).b_p_ro != 0 {
        (*buf).b_did_warn = false_0 != 0;
    }
    redraw_titles();
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_scrollback(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut old_value: OptInt = (*args).os_oldval.number;
    let mut value: OptInt = (*args).os_newval.number;
    if !(*buf).terminal.is_null() && value < old_value {
        on_scrollback_option_changed((*buf).terminal);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_scrollbind(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if (*win).w_onebuf_opt.wo_scb == 0 {
        return ::core::ptr::null::<c_char>();
    }
    do_check_scrollbind(false_0 != 0);
    (*win).w_scbind_pos = get_vtopline(win);
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_shiftwidth_tabstop(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut pp: *mut OptInt = (*args).os_varp as *mut OptInt;
    if foldmethodIsIndent(win) {
        foldUpdateAll(win);
    }
    if pp == &raw mut (*buf).b_p_sw || (*buf).b_p_sw == 0 as OptInt {
        parse_cino(buf);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_showtabline(mut _args: *mut optset_T) -> *const c_char {
    win_new_screen_rows();
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_smoothscroll(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if (*win).w_onebuf_opt.wo_sms == 0 {
        (*win).w_skipcol = 0 as c_int as colnr_T;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_spell(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if (*win).w_onebuf_opt.wo_spell != 0 {
        return parse_spelllang(win);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_swapfile(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    if (*buf).b_p_swf != 0 && p_uc.get() != 0 {
        ml_open_file(buf);
    } else {
        mf_close_file(buf, true_0 != 0);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_textwidth(mut _args: *mut optset_T) -> *const c_char {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            check_colorcolumn(::core::ptr::null_mut::<c_char>(), wp);
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_title_icon(mut _args: *mut optset_T) -> *const c_char {
    did_set_title();
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_titlelen(mut args: *mut optset_T) -> *const c_char {
    let mut old_value: OptInt = (*args).os_oldval.number;
    if starting.get() != NO_SCREEN && old_value != p_titlelen.get() {
        need_maketitle.set(true_0 != 0);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_undofile(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    if (*buf).b_p_udf == 0 && p_udf.get() == 0 {
        return ::core::ptr::null::<c_char>();
    }
    let mut hash: [uint8_t; 32] = [0; 32];
    let mut bp: *mut buf_T = firstbuf.get();
    while !bp.is_null() {
        if (buf == bp
            || (*args).os_flags & OPT_GLOBAL as c_int != 0
            || (*args).os_flags == 0 as c_int)
            && !bufIsChanged(bp)
            && !(*bp).b_ml.ml_mfp.is_null()
        {
            u_compute_hash(bp, &raw mut hash as *mut uint8_t);
            u_read_undo(
                ::core::ptr::null_mut::<c_char>(),
                &raw mut hash as *mut uint8_t,
                (*bp).b_fname,
            );
        }
        bp = (*bp).b_next;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_global_undolevels(
    mut value: OptInt,
    mut old_value: OptInt,
) -> *const c_char {
    p_ul.set(old_value);
    u_sync(true_0 != 0);
    p_ul.set(value);
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_buflocal_undolevels(
    mut buf: *mut buf_T,
    mut value: OptInt,
    mut old_value: OptInt,
) -> *const c_char {
    (*buf).b_p_ul = old_value;
    u_sync(true_0 != 0);
    (*buf).b_p_ul = value;
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_undolevels(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut pp: *mut OptInt = (*args).os_varp as *mut OptInt;
    if pp == p_ul.ptr() {
        did_set_global_undolevels((*args).os_newval.number, (*args).os_oldval.number);
    } else if pp == &raw mut (*buf).b_p_ul {
        did_set_buflocal_undolevels(buf, (*args).os_newval.number, (*args).os_oldval.number);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_updatecount(mut args: *mut optset_T) -> *const c_char {
    let mut old_value: OptInt = (*args).os_oldval.number;
    if p_uc.get() != 0 && old_value == 0 {
        ml_open_files();
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_wildchar(mut args: *mut optset_T) -> *const c_char {
    let mut c: OptInt = *((*args).os_varp as *mut OptInt);
    if c == Ctrl_C as OptInt
        || c == '\n' as OptInt
        || c == '\r' as OptInt
        || c == K_KENTER as OptInt
    {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_winblend(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut old_value: OptInt = (*args).os_oldval.number;
    let mut value: OptInt = (*args).os_newval.number;
    if value != old_value {
        (*win).w_onebuf_opt.wo_winbl = if (if (*win).w_onebuf_opt.wo_winbl < 100 as OptInt {
            (*win).w_onebuf_opt.wo_winbl
        } else {
            100 as OptInt
        }) > 0 as OptInt
        {
            if (*win).w_onebuf_opt.wo_winbl < 100 as OptInt {
                (*win).w_onebuf_opt.wo_winbl
            } else {
                100 as OptInt
            }
        } else {
            0 as OptInt
        };
        (*win).w_hl_needs_update = true_0;
        check_blending(win);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_window(mut _args: *mut optset_T) -> *const c_char {
    if p_window.get() < 1 as OptInt {
        p_window.set((Rows.get() - 1 as c_int) as OptInt);
    } else if p_window.get() >= Rows.get() as OptInt {
        p_window.set((Rows.get() - 1 as c_int) as OptInt);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_winheight(mut _args: *mut optset_T) -> *const c_char {
    if !(firstwin.get() == lastwin.get()) {
        if ((*curwin.get()).w_height as OptInt) < p_wh.get() {
            win_setheight(p_wh.get() as c_int);
        }
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_winwidth(mut _args: *mut optset_T) -> *const c_char {
    if !(firstwin.get() == lastwin.get()) && ((*curwin.get()).w_width as OptInt) < p_wiw.get() {
        win_setwidth(p_wiw.get() as c_int);
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_wrap(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    if (*win).w_onebuf_opt.wo_wrap != 0 {
        (*win).w_leftcol = 0 as c_int as colnr_T;
    } else {
        (*win).w_skipcol = 0 as c_int as colnr_T;
    }
    return ::core::ptr::null::<c_char>();
}
pub unsafe extern "C" fn did_set_xhistory(mut args: *mut optset_T) -> *const c_char {
    let mut win: *mut win_T = (*args).os_win as *mut win_T;
    let mut is_p_chi: bool = (*args).os_varp as *mut OptInt == p_chi.ptr();
    let mut arg: *mut OptInt = if is_p_chi as c_int != 0 {
        p_chi.ptr()
    } else {
        (*args).os_varp as *mut OptInt
    };
    if is_p_chi {
        qf_resize_stack(*arg as c_int);
    } else {
        ll_resize_stack(win, *arg as c_int);
    }
    return ::core::ptr::null::<c_char>();
}
unsafe extern "C" fn do_syntax_autocmd(mut buf: *mut buf_T, mut value_changed: bool) {
    static syn_recursive: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    (*syn_recursive.ptr()) += 1;
    (*buf).b_flags |= BF_SYN_SET;
    apply_autocmds(
        EVENT_SYNTAX,
        (*buf).b_p_syn,
        (*buf).b_fname,
        value_changed as c_int != 0 || syn_recursive.get() == 1 as c_int,
        buf,
    );
    (*syn_recursive.ptr()) -= 1;
}
unsafe extern "C" fn do_spelllang_source(mut win: *mut win_T) {
    let mut fname: [c_char; 200] = [0; 200];
    let mut q: *mut c_char = (*(*win).w_s).b_p_spl;
    if strncmp(q, b"cjk,\0".as_ptr() as *const c_char, 4 as size_t) == 0 as c_int {
        q = q.offset(4 as c_int as isize);
    }
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    p = q;
    while *p as c_int != NUL {
        if !(*p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
            || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
            || ascii_isdigit(*p as c_int) as c_int != 0)
            && *p as c_int != '-' as c_int
        {
            break;
        }
        p = p.offset(1);
    }
    if p > q {
        vim_snprintf(
            &raw mut fname as *mut c_char,
            ::core::mem::size_of::<[c_char; 200]>(),
            b"spell/%.*s.*\0".as_ptr() as *const c_char,
            p.offset_from(q) as c_int,
            q,
        );
        source_runtime_vim_lua(&raw mut fname as *mut c_char, DIP_ALL as c_int);
    }
}
unsafe extern "C" fn check_num_option_bounds(
    mut opt_idx: OptIndex,
    mut newval: *mut OptInt,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
) -> *const c_char {
    let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
    match opt_idx as c_int {
        169 => {
            if *newval < min_rows_for_all_tabpages() as OptInt && full_screen.get() as c_int != 0 {
                vim_snprintf(
                    errbuf,
                    errbuflen,
                    gettext(b"E593: Need at least %d lines\0".as_ptr() as *const c_char),
                    min_rows_for_all_tabpages(),
                );
                errmsg = errbuf;
                *newval = min_rows_for_all_tabpages() as OptInt;
            }
            *newval = if *newval < 2147483647 as OptInt {
                *newval
            } else {
                2147483647 as OptInt
            };
        }
        47 => {
            if *newval < MIN_COLUMNS as c_int as OptInt && full_screen.get() as c_int != 0 {
                vim_snprintf(
                    errbuf,
                    errbuflen,
                    gettext(b"E594: Need at least %d columns\0".as_ptr() as *const c_char),
                    MIN_COLUMNS as c_int,
                );
                errmsg = errbuf;
                *newval = MIN_COLUMNS as c_int as OptInt;
            }
            *newval = if *newval < 2147483647 as OptInt {
                *newval
            } else {
                2147483647 as OptInt
            };
        }
        222 => {
            *newval = if (if *newval < 100 as OptInt {
                *newval
            } else {
                100 as OptInt
            }) > 0 as OptInt
            {
                if *newval < 100 as OptInt {
                    *newval
                } else {
                    100 as OptInt
                }
            } else {
                0 as OptInt
            };
        }
        246 => {
            if (*newval < -100 as OptInt || *newval >= Rows.get() as OptInt)
                && full_screen.get() as c_int != 0
            {
                errmsg = &raw const e_scroll as *const c_char;
                *newval = 1 as OptInt;
            }
        }
        243 => {
            if (*newval <= 0 as OptInt
                || *newval > (*curwin.get()).w_view_height as OptInt
                    && (*curwin.get()).w_view_height > 0 as c_int)
                && full_screen.get() as c_int != 0
            {
                if *newval != 0 as OptInt {
                    errmsg = &raw const e_scroll as *const c_char;
                }
                *newval = win_default_scroll(curwin.get());
            }
        }
        _ => {}
    }
    return errmsg;
}
unsafe extern "C" fn validate_num_option(
    mut opt_idx: OptIndex,
    mut newval: *mut OptInt,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
) -> *const c_char {
    let mut value: OptInt = *newval;
    if value < INT_MIN as OptInt || value > INT_MAX as OptInt {
        return &raw const e_invarg as *const c_char;
    }
    match opt_idx as c_int {
        129 | 325 | 335 | 236 | 336 | 275 | 106 | 266 | 318 | 373 | 323 => {
            if value < 0 as OptInt {
                return &raw const e_positive as *const c_char;
            }
        }
        362 => {
            if value < 1 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if p_wmh.get() > value {
                return &raw const e_winheight as *const c_char;
            }
        }
        364 => {
            if value < 0 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if value > p_wh.get() {
                return &raw const e_winheight as *const c_char;
            }
        }
        366 => {
            if value < 1 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if p_wmw.get() > value {
                return &raw const e_winwidth as *const c_char;
            }
        }
        365 => {
            if value < 0 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if value > p_wiw.get() {
                return &raw const e_winwidth as *const c_char;
            }
        }
        183 => {
            *newval = MAX_MCO as OptInt;
        }
        44 => {
            if value < 0 as OptInt {
                return &raw const e_positive as *const c_char;
            }
        }
        133 => {
            if value < 0 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if value > 10000 as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        227 => {
            if value == 0 as OptInt {
                *newval = 3 as OptInt;
            } else if value != 3 as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        233 => {
            if value < 0 as OptInt || value > 2 as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        247 => {
            if value < 0 as OptInt && full_screen.get() as c_int != 0 {
                return &raw const e_positive as *const c_char;
            }
        }
        276 => {
            if value < 0 as OptInt && full_screen.get() as c_int != 0 {
                return &raw const e_positive as *const c_char;
            }
        }
        45 => {
            if value < 1 as OptInt {
                return &raw const e_positive as *const c_char;
            }
        }
        58 => {
            if value < 0 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if value > 3 as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        207 => {
            if value < 1 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if value > MAX_NUMBERWIDTH as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        142 => {
            if value < 0 as OptInt || value > B_IMODE_LAST as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        143 => {
            if value < -1 as OptInt || value > B_IMODE_LAST as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        35 => return &raw const e_invarg as *const c_char,
        244 => {
            if value < -1 as OptInt || value > SB_MAX as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        304 => {
            if value < 1 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if value > TABSTOP_MAX as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        37 | 167 => {
            if value < 1 as OptInt {
                return (e_cannot_have_negative_or_zero_number_of_quickfix.ptr() as *const _)
                    as *const c_char;
            } else if value > 100 as OptInt {
                return (e_cannot_have_more_than_hundred_quickfix.ptr() as *const _)
                    as *const c_char;
            }
        }
        187 => {
            if value <= 0 as OptInt {
                return &raw const e_positive as *const c_char;
            } else if value > MAX_SEARCH_COUNT as c_int as OptInt {
                return &raw const e_invarg as *const c_char;
            }
        }
        _ => {}
    }
    return check_num_option_bounds(opt_idx, newval, errbuf, errbuflen);
}
pub unsafe extern "C" fn check_redraw_for(
    mut buf: *mut buf_T,
    mut win: *mut win_T,
    mut flags: uint32_t,
) {
    let mut all: bool =
        flags & kOptFlagRedrAll as c_int as uint32_t == kOptFlagRedrAll as c_int as uint32_t;
    if flags & kOptFlagRedrStat as c_int as uint32_t != 0 || all as c_int != 0 {
        status_redraw_all();
    }
    if flags & kOptFlagRedrTabl as c_int as uint32_t != 0 || all as c_int != 0 {
        redraw_tabline.set(true_0 != 0);
    }
    if flags & kOptFlagRedrBuf as c_int as uint32_t != 0
        || flags & kOptFlagRedrWin as c_int as uint32_t != 0
        || all as c_int != 0
    {
        if flags & kOptFlagHLOnly as c_int as uint32_t != 0 {
            redraw_later(win, UPD_NOT_VALID as c_int);
        } else {
            changed_window_setting(win);
        }
    }
    if flags & kOptFlagRedrBuf as c_int as uint32_t != 0 {
        redraw_buf_later(buf, UPD_NOT_VALID as c_int);
    }
    if all {
        redraw_all_later(UPD_NOT_VALID as c_int);
    }
}
pub unsafe extern "C" fn check_redraw(mut flags: uint32_t) {
    check_redraw_for(curbuf.get(), curwin.get(), flags);
}
pub unsafe extern "C" fn is_tty_option(mut name: *const c_char) -> bool {
    return !find_tty_option_end(name).is_null();
}
pub unsafe extern "C" fn get_tty_option(mut name: *const c_char) -> OptVal {
    let mut value: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if strequal(name, b"t_Co\0".as_ptr() as *const c_char) {
        if t_colors.get() <= 1 as c_int {
            value = xstrdup(b"\0".as_ptr() as *const c_char);
        } else {
            value = xmalloc(NUMBUFLEN as c_int as size_t) as *mut c_char;
            snprintf(
                value,
                NUMBUFLEN as c_int as size_t,
                b"%d\0".as_ptr() as *const c_char,
                t_colors.get(),
            );
        }
    } else if strequal(name, b"term\0".as_ptr() as *const c_char) {
        value = if !(*p_term.ptr()).is_null() {
            xstrdup(p_term.get())
        } else {
            xstrdup(b"nvim\0".as_ptr() as *const c_char)
        };
    } else if strequal(name, b"ttytype\0".as_ptr() as *const c_char) {
        value = if !(*p_ttytype.ptr()).is_null() {
            xstrdup(p_ttytype.get())
        } else {
            xstrdup(b"nvim\0".as_ptr() as *const c_char)
        };
    } else if is_tty_option(name) {
        value = xstrdup(b"\0".as_ptr() as *const c_char);
    }
    return if value.is_null() {
        OptVal {
            type_0: kOptValTypeNil,
            data: OptValData { boolean: kFalse },
        }
    } else {
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(value),
            },
        }
    };
}
pub unsafe extern "C" fn set_tty_option(mut name: *const c_char, mut value: *mut c_char) -> bool {
    if strequal(name, b"term\0".as_ptr() as *const c_char) {
        if !(*p_term.ptr()).is_null() {
            xfree(p_term.get() as *mut c_void);
        }
        p_term.set(value);
        return true_0 != 0;
    }
    if strequal(name, b"ttytype\0".as_ptr() as *const c_char) {
        if !(*p_ttytype.ptr()).is_null() {
            xfree(p_ttytype.get() as *mut c_void);
        }
        p_ttytype.set(value);
        return true_0 != 0;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn find_option_len(name: *const c_char, len: size_t) -> OptIndex {
    if len == 0 {
        return kOptInvalid;
    }
    // SAFETY: the caller passes `len` readable bytes at `name`.
    find_option_index(unsafe { ::core::slice::from_raw_parts(name.cast::<u8>(), len) })
}
pub unsafe extern "C" fn find_option(name: *const c_char) -> OptIndex {
    return find_option_len(name, strlen(name));
}
pub unsafe extern "C" fn optval_free(mut o: OptVal) {
    match o.type_0 as c_int {
        2 => {
            if o.data.string.data != empty_string_option.ptr() as *mut c_char {
                api_free_string(o.data.string);
            }
        }
        -1 | 0 | 1 | _ => {}
    };
}
pub unsafe extern "C" fn optval_copy(mut o: OptVal) -> OptVal {
    match o.type_0 as c_int {
        -1 | 0 | 1 => return o,
        2 => {
            return OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: copy_string(o.data.string, ::core::ptr::null_mut::<Arena>()),
                },
            };
        }
        _ => {}
    }
    unreachable!();
}
pub unsafe extern "C" fn optval_equal(mut o1: OptVal, mut o2: OptVal) -> bool {
    if o1.type_0 as c_int != o2.type_0 as c_int {
        return false_0 != 0;
    }
    match o1.type_0 as c_int {
        -1 => return true_0 != 0,
        0 => {
            return o1.data.boolean as c_int == o2.data.boolean as c_int;
        }
        1 => return o1.data.number == o2.data.number,
        2 => {
            return o1.data.string.size == o2.data.string.size
                && (o1.data.string.data == o2.data.string.data
                    || strnequal(
                        o1.data.string.data,
                        o2.data.string.data,
                        o1.data.string.size,
                    ) as c_int
                        != 0);
        }
        _ => {}
    }
    unreachable!();
}
unsafe extern "C" fn option_get_type(opt_idx: OptIndex) -> OptValType {
    return (*options.ptr())[opt_idx as usize].type_0;
}
pub unsafe extern "C" fn optval_from_varp(mut opt_idx: OptIndex, mut varp: *mut c_void) -> OptVal {
    if varp as *mut c_int == &raw mut (*curbuf.get()).b_changed {
        return OptVal {
            type_0: kOptValTypeBoolean,
            data: OptValData {
                boolean: curbufIsChanged() as TriState,
            },
        };
    }
    let mut type_0: OptValType = option_get_type(opt_idx);
    match type_0 as c_int {
        -1 => {
            return OptVal {
                type_0: kOptValTypeNil,
                data: OptValData { boolean: kFalse },
            };
        }
        0 => {
            return OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData {
                    boolean: (if *(varp as *mut c_int) == 0 as c_int {
                        kFalse as c_int
                    } else if *(varp as *mut c_int) >= 1 as c_int {
                        kTrue as c_int
                    } else {
                        kNone as c_int
                    }) as TriState,
                },
            };
        }
        1 => {
            return OptVal {
                type_0: kOptValTypeNumber,
                data: OptValData {
                    number: *(varp as *mut OptInt),
                },
            };
        }
        2 => {
            return OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(*(varp as *mut *mut c_char)),
                },
            };
        }
        _ => {}
    }
    unreachable!();
}
unsafe extern "C" fn set_option_varp(
    mut opt_idx: OptIndex,
    mut varp: *mut c_void,
    mut value: OptVal,
    mut free_oldval: bool,
) {
    '_c2rust_label: {
        if option_has_type(opt_idx, value.type_0) {
        } else {
            __assert_fail(
                b"option_has_type(opt_idx, value.type)\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                3401 as c_uint,
                b"void set_option_varp(OptIndex, void *, OptVal, _Bool)\0".as_ptr()
                    as *const c_char,
            );
        }
    };
    if free_oldval {
        optval_free(optval_from_varp(opt_idx, varp));
    }
    match value.type_0 as c_int {
        -1 => {
            abort();
        }
        0 => {
            *(varp as *mut c_int) = value.data.boolean as c_int;
            return;
        }
        1 => {
            *(varp as *mut OptInt) = value.data.number;
            return;
        }
        2 => {
            *(varp as *mut *mut c_char) = value.data.string.data;
            return;
        }
        _ => {}
    }
    unreachable!();
}
unsafe extern "C" fn optval_to_cstr(mut o: OptVal) -> *mut c_char {
    match o.type_0 as c_int {
        -1 => return xstrdup(b"\0".as_ptr() as *const c_char),
        0 => {
            return xstrdup(if o.data.boolean as c_int != 0 {
                b"true\0".as_ptr() as *const c_char
            } else {
                b"false\0".as_ptr() as *const c_char
            });
        }
        1 => {
            let mut buf: *mut c_char = xmalloc(NUMBUFLEN as c_int as size_t) as *mut c_char;
            snprintf(
                buf,
                NUMBUFLEN as c_int as size_t,
                b"%ld\0".as_ptr() as *const c_char,
                o.data.number,
            );
            return buf;
        }
        2 => {
            let mut buf_0: *mut c_char =
                xmalloc(o.data.string.size.wrapping_add(3 as size_t)) as *mut c_char;
            snprintf(
                buf_0,
                o.data.string.size.wrapping_add(3 as size_t),
                b"\"%s\"\0".as_ptr() as *const c_char,
                o.data.string.data,
            );
            return buf_0;
        }
        _ => {}
    }
    unreachable!();
}
pub unsafe extern "C" fn optval_as_object(mut o: OptVal) -> Object {
    match o.type_0 as c_int {
        -1 => {
            return object {
                type_0: kObjectTypeNil,
                data: object_data { boolean: false },
            };
        }
        0 => {
            match o.data.boolean as c_int {
                0 | 1 => {
                    return object {
                        type_0: kObjectTypeBoolean,
                        data: object_data {
                            boolean: o.data.boolean as u64 != 0,
                        },
                    };
                }
                -1 => {
                    return object {
                        type_0: kObjectTypeNil,
                        data: object_data { boolean: false },
                    };
                }
                _ => {}
            }
            unreachable!();
        }
        1 => {
            return object {
                type_0: kObjectTypeInteger,
                data: object_data {
                    integer: o.data.number,
                },
            };
        }
        2 => {
            return object {
                type_0: kObjectTypeString,
                data: object_data {
                    string: o.data.string,
                },
            };
        }
        _ => {}
    }
    unreachable!();
}
pub unsafe extern "C" fn object_as_optval(mut o: Object, mut error: *mut bool) -> OptVal {
    match o.type_0 as c_uint {
        0 => {
            return OptVal {
                type_0: kOptValTypeNil,
                data: OptValData { boolean: kFalse },
            };
        }
        1 => {
            return OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData {
                    boolean: o.data.boolean as TriState,
                },
            };
        }
        2 => {
            return OptVal {
                type_0: kOptValTypeNumber,
                data: OptValData {
                    number: o.data.integer,
                },
            };
        }
        4 => {
            return OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: o.data.string,
                },
            };
        }
        _ => {
            *error = true_0 != 0;
            return OptVal {
                type_0: kOptValTypeNil,
                data: OptValData { boolean: kFalse },
            };
        }
    };
}
pub unsafe extern "C" fn is_option_hidden(mut opt_idx: OptIndex) -> bool {
    return opt_idx as c_int != kOptInvalid as c_int
        && (*options.ptr())[opt_idx as usize].immutable as c_int != 0
        && (*options.ptr())[opt_idx as usize].var
            == &raw mut (*(options.ptr() as *mut vimoption_T).offset(opt_idx as isize))
                .def_val
                .data as *mut c_void;
}
pub unsafe extern "C" fn option_has_type(mut opt_idx: OptIndex, mut type_0: OptValType) -> bool {
    return opt_idx as c_int != kOptInvalid as c_int
        && (*options.ptr())[opt_idx as usize].type_0 as c_int == type_0 as c_int;
}
pub unsafe extern "C" fn option_has_scope(mut opt_idx: OptIndex, mut scope: OptScope) -> bool {
    '_c2rust_label: {
        if scope as c_uint >= kOptScopeGlobal as c_int as c_uint
            && (scope as c_uint) < (kOptScopeBuf as c_int + 1 as c_int) as c_uint
        {
        } else {
            __assert_fail(
                b"scope >= kOptScopeGlobal && scope < kOptScopeSize\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                3512 as c_uint,
                b"_Bool option_has_scope(OptIndex, OptScope)\0".as_ptr() as *const c_char,
            );
        }
    };
    return (*get_option(opt_idx)).scope_flags as c_int & (1 as c_int) << scope as c_uint != 0;
}
#[inline]
unsafe extern "C" fn option_is_global_local(mut opt_idx: OptIndex) -> bool {
    return opt_idx as c_int != kOptInvalid as c_int
        && !is_power_of_two((*options.ptr())[opt_idx as usize].scope_flags as uint64_t);
}
#[inline]
unsafe extern "C" fn option_is_global_only(mut opt_idx: OptIndex) -> bool {
    return opt_idx as c_int != kOptInvalid as c_int
        && is_power_of_two((*options.ptr())[opt_idx as usize].scope_flags as uint64_t) as c_int
            != 0
        && option_has_scope(opt_idx, kOptScopeGlobal) as c_int != 0;
}
#[inline]
unsafe extern "C" fn option_is_window_local(mut opt_idx: OptIndex) -> bool {
    return opt_idx as c_int != kOptInvalid as c_int
        && is_power_of_two((*options.ptr())[opt_idx as usize].scope_flags as uint64_t) as c_int
            != 0
        && option_has_scope(opt_idx, kOptScopeWin) as c_int != 0;
}
pub unsafe extern "C" fn option_scope_idx(mut opt_idx: OptIndex, mut scope: OptScope) -> ssize_t {
    return (*options.ptr())[opt_idx as usize].scope_idx[scope as usize];
}
pub unsafe extern "C" fn get_option_value(mut opt_idx: OptIndex, mut opt_flags: c_int) -> OptVal {
    if opt_idx as c_int == kOptInvalid as c_int {
        return OptVal {
            type_0: kOptValTypeNil,
            data: OptValData { boolean: kFalse },
        };
    }
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    let mut varp: *mut c_void = get_varp_scope(opt, opt_flags);
    return optval_copy(optval_from_varp(opt_idx, varp));
}
pub unsafe extern "C" fn get_option(mut opt_idx: OptIndex) -> *mut vimoption_T {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                3580 as c_uint,
                b"vimoption_T *get_option(OptIndex)\0".as_ptr() as *const c_char,
            );
        }
    };
    return (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
}
unsafe extern "C" fn get_option_unset_value(mut opt_idx: OptIndex) -> OptVal {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                3593 as c_uint,
                b"OptVal get_option_unset_value(OptIndex)\0".as_ptr() as *const c_char,
            );
        }
    };
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    if option_is_global_local(opt_idx) {
        if option_has_type(opt_idx, kOptValTypeString) {
            return OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: b"\0".as_ptr() as *const c_char as *mut c_char,
                        size: ::core::mem::size_of::<[c_char; 1]>().wrapping_sub(1 as size_t),
                    },
                },
            };
        }
        match opt_idx as c_int {
            6 | 10 | 118 => {
                return OptVal {
                    type_0: kOptValTypeBoolean,
                    data: OptValData { boolean: kNone },
                };
            }
            247 | 276 => {
                return OptVal {
                    type_0: kOptValTypeNumber,
                    data: OptValData {
                        number: -1 as OptInt,
                    },
                };
            }
            333 => {
                return OptVal {
                    type_0: kOptValTypeNumber,
                    data: OptValData {
                        number: -123456 as OptInt,
                    },
                };
            }
            _ => {
                abort();
            }
        }
    }
    return optval_from_varp(opt_idx, get_varp_scope(opt, OPT_GLOBAL as c_int));
}
unsafe extern "C" fn is_option_local_value_unset(mut opt_idx: OptIndex) -> bool {
    let mut opt: *mut vimoption_T = get_option(opt_idx);
    if !option_is_global_local(opt_idx) {
        return false_0 != 0;
    }
    let mut varp_local: *mut c_void = get_varp_scope(opt, OPT_LOCAL as c_int);
    let mut local_value: OptVal = optval_from_varp(opt_idx, varp_local);
    let mut unset_local_value: OptVal = get_option_unset_value(opt_idx);
    return optval_equal(local_value, unset_local_value);
}
unsafe extern "C" fn did_set_option(
    mut opt_idx: OptIndex,
    mut varp: *mut c_void,
    mut old_value: OptVal,
    mut new_value: OptVal,
    mut opt_flags: c_int,
    mut set_sid: scid_T,
    direct: bool,
    value_replaced: bool,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
) -> *const c_char {
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
    let mut restore_chartab: bool = false_0 != 0;
    let mut value_changed: bool = false_0 != 0;
    let mut value_checked: bool = false_0 != 0;
    let mut did_set_cb_args: optset_T = optset_T {
        os_varp: varp,
        os_idx: opt_idx,
        os_flags: opt_flags,
        os_oldval: old_value.data,
        os_newval: new_value.data,
        os_value_checked: false_0 != 0,
        os_value_changed: false_0 != 0,
        os_restore_chartab: false_0 != 0,
        os_errbuf: errbuf,
        os_errbuflen: errbuflen,
        os_win: curwin.get() as *mut c_void,
        os_buf: curbuf.get() as *mut c_void,
    };
    if !direct {
        if (*opt).immutable as c_int != 0 && !optval_equal(old_value, new_value) {
            errmsg = &raw const e_unsupportedoption as *const c_char;
        } else if (secure.get() != 0 || sandbox.get() != 0 as c_int)
            && (*opt).flags & kOptFlagSecure as c_int as uint32_t != 0
        {
            errmsg = &raw const e_secure as *const c_char;
        } else if new_value.type_0 as c_int == kOptValTypeString as c_int
            && check_illegal_path_names(*(varp as *mut *mut c_char), (*opt).flags) as c_int != 0
        {
            errmsg = &raw const e_invarg as *const c_char;
        } else if (*opt).opt_did_set_cb.is_some() {
            errmsg =
                (*opt).opt_did_set_cb.expect("non-null function pointer")(&raw mut did_set_cb_args);
            value_changed = did_set_cb_args.os_value_changed;
            value_checked = did_set_cb_args.os_value_checked;
            restore_chartab = did_set_cb_args.os_restore_chartab;
        }
    }
    if !errmsg.is_null() {
        set_option_varp(opt_idx, varp, old_value, true_0 != 0);
        if restore_chartab {
            buf_init_chartab(curbuf.get(), true);
        }
        return errmsg;
    }
    new_value = optval_from_varp(opt_idx, varp);
    if set_sid != SID_NONE {
        let mut script_ctx: sctx_T = if set_sid == 0 as c_int {
            current_sctx.get()
        } else {
            sctx_T {
                sc_sid: set_sid,
                sc_seq: 0,
                sc_lnum: 0,
                sc_chan: 0,
            }
        };
        set_option_sctx(opt_idx, opt_flags, script_ctx);
    }
    optval_free(old_value);
    let scope_both: bool = opt_flags & (OPT_LOCAL as c_int | OPT_GLOBAL as c_int) == 0 as c_int;
    if scope_both {
        if option_is_global_local(opt_idx) {
            let mut varp_local: *mut c_void = get_varp_scope(opt, OPT_LOCAL as c_int);
            let mut local_unset_value: OptVal = get_option_unset_value(opt_idx);
            set_option_varp(
                opt_idx,
                varp_local,
                optval_copy(local_unset_value),
                true_0 != 0,
            );
        } else {
            let mut varp_global: *mut c_void = get_varp_scope(opt, OPT_GLOBAL as c_int);
            set_option_varp(opt_idx, varp_global, optval_copy(new_value), true_0 != 0);
        }
    }
    if direct {
        return errmsg;
    }
    if varp == &raw mut (*curbuf.get()).b_p_syn as *mut c_void {
        do_syntax_autocmd(curbuf.get(), value_changed);
    } else if varp == &raw mut (*curbuf.get()).b_p_ft as *mut c_void {
        if opt_flags & OPT_MODELINE as c_int == 0 || value_changed as c_int != 0 {
            do_filetype_autocmd(curbuf.get(), value_changed);
        }
    } else if varp == &raw mut (*(*curwin.get()).w_s).b_p_spl as *mut c_void {
        do_spelllang_source(curwin.get());
    }
    comp_col();
    if varp == p_mouse.ptr() as *mut c_void {
        setmouse();
    } else if (varp == p_flp.ptr() as *mut c_void
        || varp == &raw mut (*curbuf.get()).b_p_flp as *mut c_void)
        && (*curwin.get()).w_briopt_list != 0
    {
        redraw_all_later(UPD_NOT_VALID as c_int);
    } else if varp == p_wbr.ptr() as *mut c_void
        || varp == &raw mut (*curwin.get()).w_onebuf_opt.wo_wbr as *mut c_void
    {
        set_winbar(true_0 != 0);
    }
    if (*curwin.get()).w_curswant != MAXCOL as c_int
        && (*opt).flags & (kOptFlagCurswant as c_int | kOptFlagRedrAll as c_int) as uint32_t
            != 0 as uint32_t
        && (*opt).flags & kOptFlagHLOnly as c_int as uint32_t == 0 as uint32_t
    {
        (*curwin.get()).w_set_curswant = true_0;
    }
    check_redraw((*opt).flags);
    if errmsg.is_null() {
        (*opt).flags |= kOptFlagWasSet as c_int as uint32_t;
        let mut flagsp: *mut uint32_t = insecure_flag(curwin.get(), opt_idx, opt_flags);
        let mut flagsp_local: *mut uint32_t = if scope_both as c_int != 0 {
            insecure_flag(curwin.get(), opt_idx, OPT_LOCAL as c_int)
        } else {
            ::core::ptr::null_mut::<uint32_t>()
        };
        if !value_checked
            && (secure.get() != 0
                || sandbox.get() != 0 as c_int
                || opt_flags & OPT_MODELINE as c_int != 0)
        {
            *flagsp |= kOptFlagInsecure as c_int as uint32_t;
            if !flagsp_local.is_null() {
                *flagsp_local |= kOptFlagInsecure as c_int as uint32_t;
            }
        } else if value_replaced {
            *flagsp = (*flagsp as c_uint & !(kOptFlagInsecure as c_int as c_uint)) as uint32_t;
            if !flagsp_local.is_null() {
                *flagsp_local =
                    (*flagsp_local as c_uint & !(kOptFlagInsecure as c_int as c_uint)) as uint32_t;
            }
        }
    }
    return errmsg;
}
unsafe extern "C" fn validate_option_value(
    opt_idx: OptIndex,
    mut newval: *mut OptVal,
    mut opt_flags: c_int,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
) -> *const c_char {
    let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    if option_is_global_local(opt_idx) as c_int != 0
        && opt_flags & OPT_LOCAL as c_int != 0
        && optval_equal(*newval, get_option_unset_value(opt_idx)) as c_int != 0
    {
        return ::core::ptr::null::<c_char>();
    }
    if (*newval).type_0 as c_int == kOptValTypeNil as c_int {
        if opt_flags == OPT_GLOBAL as c_int {
            errmsg = gettext(b"Cannot unset global option value\0".as_ptr() as *const c_char);
        } else {
            *newval = optval_copy(get_option_unset_value(opt_idx));
        }
    } else if !option_has_type(opt_idx, (*newval).type_0) {
        let mut rep: *mut c_char = optval_to_cstr(*newval);
        let mut type_str: *const c_char = optval_type_get_name((*opt).type_0);
        snprintf(
            errbuf,
            IOSIZE as size_t,
            gettext(
                b"Invalid value for option '%s': expected %s, got %s %s\0".as_ptr()
                    as *const c_char,
            ),
            (*opt).fullname,
            type_str,
            optval_type_get_name((*newval).type_0),
            rep,
        );
        xfree(rep as *mut c_void);
        errmsg = errbuf;
    } else if (*newval).type_0 as c_int == kOptValTypeNumber as c_int {
        errmsg = validate_num_option(opt_idx, &raw mut (*newval).data.number, errbuf, errbuflen);
    }
    return errmsg;
}
unsafe extern "C" fn set_option(
    opt_idx: OptIndex,
    mut value: OptVal,
    mut opt_flags: c_int,
    mut set_sid: scid_T,
    direct: bool,
    value_replaced: bool,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
) -> *const c_char {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr()
                    as *const c_char,
                3871 as c_uint,
                b"const char *set_option(const OptIndex, OptVal, int, scid_T, const _Bool, const _Bool, char *, size_t)\0"
                    .as_ptr() as *const c_char,
            );
        }
    };
    let mut errmsg: *const c_char = ::core::ptr::null::<c_char>();
    if !direct {
        errmsg = validate_option_value(opt_idx, &raw mut value, opt_flags, errbuf, errbuflen);
        if !errmsg.is_null() {
            optval_free(value);
            return errmsg;
        }
    }
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    let scope_local: bool = opt_flags & OPT_LOCAL as c_int != 0;
    let scope_global: bool = opt_flags & OPT_GLOBAL as c_int != 0;
    let scope_both: bool = !scope_local && !scope_global;
    let is_opt_local_unset: bool = is_option_local_value_unset(opt_idx);
    let mut varp: *mut c_void =
        if scope_both as c_int != 0 && option_is_global_local(opt_idx) as c_int != 0 {
            (*opt).var
        } else {
            get_varp_scope(opt, opt_flags)
        };
    let mut varp_local: *mut c_void = get_varp_scope(opt, OPT_LOCAL as c_int);
    let mut varp_global: *mut c_void = get_varp_scope(opt, OPT_GLOBAL as c_int);
    let mut old_value: OptVal = optval_from_varp(opt_idx, varp);
    let mut old_global_value: OptVal = optval_from_varp(opt_idx, varp_global);
    let mut old_local_value: OptVal = if is_opt_local_unset as c_int != 0 {
        old_global_value
    } else {
        optval_from_varp(opt_idx, varp_local)
    };
    let mut used_old_value: OptVal =
        if scope_local as c_int != 0 && is_opt_local_unset as c_int != 0 {
            optval_from_varp(opt_idx, get_varp(opt))
        } else {
            old_value
        };
    let mut saved_used_value: OptVal = optval_copy(used_old_value);
    let mut saved_old_global_value: OptVal = optval_copy(old_global_value);
    let mut saved_old_local_value: OptVal = optval_copy(old_local_value);
    let mut saved_new_value: OptVal = optval_copy(value);
    let mut p: *mut uint32_t = insecure_flag(curwin.get(), opt_idx, opt_flags);
    let secure_saved: c_int = secure.get();
    if opt_flags & OPT_MODELINE as c_int != 0
        || sandbox.get() != 0 as c_int
        || !value_replaced && *p & kOptFlagInsecure as c_int as uint32_t != 0
    {
        secure.set(1 as c_int);
    }
    set_option_varp(opt_idx, varp, value, false_0 != 0);
    errmsg = did_set_option(
        opt_idx,
        varp,
        old_value,
        value,
        opt_flags,
        set_sid,
        direct,
        value_replaced,
        errbuf,
        errbuflen,
    );
    secure.set(secure_saved);
    if errmsg.is_null() && !direct {
        if starting.get() == 0 {
            apply_optionset_autocmd(
                opt_idx,
                opt_flags,
                saved_used_value,
                saved_old_global_value,
                saved_old_local_value,
                saved_new_value,
                errmsg,
            );
        }
        if (*opt).flags & kOptFlagUIOption as c_int as uint32_t != 0 {
            ui_call_option_set(
                cstr_as_string((*opt).fullname),
                optval_as_object(saved_new_value),
            );
        }
    }
    optval_free(saved_used_value);
    optval_free(saved_old_local_value);
    optval_free(saved_old_global_value);
    optval_free(saved_new_value);
    return errmsg;
}
pub unsafe extern "C" fn set_option_direct(
    mut opt_idx: OptIndex,
    mut value: OptVal,
    mut opt_flags: c_int,
    mut set_sid: scid_T,
) {
    static errbuf: GlobalCell<[c_char; 1025]> = GlobalCell::new([0; 1025]);
    if is_option_hidden(opt_idx) {
        return;
    }
    let mut errmsg: *const c_char = set_option(
        opt_idx,
        optval_copy(value),
        opt_flags,
        set_sid,
        true_0 != 0,
        true_0 != 0,
        errbuf.ptr() as *mut c_char,
        ::core::mem::size_of::<[c_char; 1025]>(),
    );
    '_c2rust_label: {
        if errmsg.is_null() {
        } else {
            __assert_fail(
                b"errmsg == NULL\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                3975 as c_uint,
                b"void set_option_direct(OptIndex, OptVal, int, scid_T)\0".as_ptr()
                    as *const c_char,
            );
        }
    };
}
pub unsafe extern "C" fn set_option_direct_for(
    mut opt_idx: OptIndex,
    mut value: OptVal,
    mut opt_flags: c_int,
    mut set_sid: scid_T,
    mut scope: OptScope,
    from: *mut c_void,
) {
    let mut save_curbuf: *mut buf_T = curbuf.get();
    let mut save_curwin: *mut win_T = curwin.get();
    match scope as c_uint {
        1 => {
            curwin.set(from as *mut win_T);
            curbuf.set((*curwin.get()).w_buffer);
        }
        2 => {
            curbuf.set(from as *mut buf_T);
        }
        0 | _ => {}
    }
    set_option_direct(opt_idx, value, opt_flags, set_sid);
    curwin.set(save_curwin);
    curbuf.set(save_curbuf);
}
pub unsafe extern "C" fn set_option_value(
    opt_idx: OptIndex,
    value: OptVal,
    mut opt_flags: c_int,
) -> *const c_char {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                4025 as c_uint,
                b"const char *set_option_value(const OptIndex, const OptVal, int)\0".as_ptr()
                    as *const c_char,
            );
        }
    };
    static errbuf: GlobalCell<[c_char; 1025]> = GlobalCell::new([0; 1025]);
    let mut flags: uint32_t = (*options.ptr())[opt_idx as usize].flags;
    if sandbox.get() > 0 as c_int && flags & kOptFlagSecure as c_int as uint32_t != 0 {
        return gettext(&raw const e_sandbox as *const c_char);
    }
    return set_option(
        opt_idx,
        optval_copy(value),
        opt_flags,
        0 as scid_T,
        false_0 != 0,
        true_0 != 0,
        errbuf.ptr() as *mut c_char,
        ::core::mem::size_of::<[c_char; 1025]>(),
    );
}
#[inline]
unsafe extern "C" fn unset_option_local_value(opt_idx: OptIndex) -> *const c_char {
    '_c2rust_label: {
        if option_is_global_local(opt_idx) {
        } else {
            __assert_fail(
                b"option_is_global_local(opt_idx)\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                4045 as c_uint,
                b"const char *unset_option_local_value(const OptIndex)\0".as_ptr() as *const c_char,
            );
        }
    };
    return set_option_value(opt_idx, get_option_unset_value(opt_idx), OPT_LOCAL as c_int);
}
pub unsafe extern "C" fn set_option_value_handle_tty(
    mut name: *const c_char,
    mut opt_idx: OptIndex,
    value: OptVal,
    mut opt_flags: c_int,
) -> *const c_char {
    static errbuf: GlobalCell<[c_char; 1025]> = GlobalCell::new([0; 1025]);
    if opt_idx as c_int == kOptInvalid as c_int {
        if is_tty_option(name) {
            return ::core::ptr::null::<c_char>();
        }
        snprintf(
            errbuf.ptr() as *mut c_char,
            ::core::mem::size_of::<[c_char; 1025]>(),
            gettext(&raw const e_unknown_option2 as *const c_char),
            name,
        );
        return errbuf.ptr() as *mut c_char;
    }
    return set_option_value(opt_idx, value, opt_flags);
}
pub unsafe extern "C" fn set_option_value_give_err(
    opt_idx: OptIndex,
    mut value: OptVal,
    mut opt_flags: c_int,
) {
    let mut errmsg: *const c_char = set_option_value(opt_idx, value, opt_flags);
    if !errmsg.is_null() {
        emsg(gettext(errmsg));
    }
}
unsafe extern "C" fn switch_option_context(
    ctx: *mut c_void,
    mut scope: OptScope,
    from: *mut c_void,
    mut err: *mut Error,
) -> bool {
    match scope as c_uint {
        0 => return false_0 != 0,
        1 => {
            let win: *mut win_T = from as *mut win_T;
            let switchwin: *mut switchwin_T = ctx as *mut switchwin_T;
            if win == curwin.get() {
                return false_0 != 0;
            }
            if switch_win_noblock(switchwin, win, win_find_tabpage(win), true_0 != 0) == FAIL {
                restore_win_noblock(switchwin, true_0 != 0);
                if (*err).type_0 as c_int != kErrorTypeNone as c_int {
                    return false_0 != 0;
                }
                api_set_error(
                    err,
                    kErrorTypeException,
                    b"Problem while switching windows\0".as_ptr() as *const c_char,
                );
                return false_0 != 0;
            }
            return true_0 != 0;
        }
        2 => {
            let buf: *mut buf_T = from as *mut buf_T;
            let aco: *mut aco_save_T = ctx as *mut aco_save_T;
            if buf == curbuf.get() {
                return false_0 != 0;
            }
            aucmd_prepbuf(aco, buf);
            return true_0 != 0;
        }
        _ => {}
    }
    unreachable!();
}
unsafe extern "C" fn restore_option_context(ctx: *mut c_void, mut scope: OptScope) {
    match scope as c_uint {
        1 => {
            restore_win_noblock(ctx as *mut switchwin_T, true_0 != 0);
        }
        2 => {
            aucmd_restbuf(ctx as *mut aco_save_T);
        }
        0 | _ => {}
    };
}
pub unsafe extern "C" fn get_option_value_for(
    mut opt_idx: OptIndex,
    mut opt_flags: c_int,
    scope: OptScope,
    from: *mut c_void,
    mut err: *mut Error,
) -> OptVal {
    let mut switchwin: switchwin_T = switchwin_T {
        sw_curwin: ::core::ptr::null_mut::<win_T>(),
        sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
        sw_same_win: false,
        sw_visual_active: false,
    };
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
        tp_localdir: ::core::ptr::null_mut::<c_char>(),
        globaldir: ::core::ptr::null_mut::<c_char>(),
        save_VIsual_active: false,
        save_prompt_insert: 0,
    };
    let mut ctx: *mut c_void = if scope as c_uint == kOptScopeWin as c_int as c_uint {
        &raw mut switchwin as *mut c_void
    } else if scope as c_uint == kOptScopeBuf as c_int as c_uint {
        &raw mut aco as *mut c_void
    } else {
        NULL
    };
    let mut switched: bool = switch_option_context(ctx, scope, from, err);
    if (*err).type_0 as c_int != kErrorTypeNone as c_int {
        return OptVal {
            type_0: kOptValTypeNil,
            data: OptValData { boolean: kFalse },
        };
    }
    let mut retv: OptVal = get_option_value(opt_idx, opt_flags);
    if switched {
        restore_option_context(ctx, scope);
    }
    return retv;
}
pub unsafe extern "C" fn set_option_value_for(
    mut name: *const c_char,
    mut opt_idx: OptIndex,
    mut value: OptVal,
    opt_flags: c_int,
    scope: OptScope,
    from: *mut c_void,
    mut err: *mut Error,
) {
    let mut switchwin: switchwin_T = switchwin_T {
        sw_curwin: ::core::ptr::null_mut::<win_T>(),
        sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
        sw_same_win: false,
        sw_visual_active: false,
    };
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
        tp_localdir: ::core::ptr::null_mut::<c_char>(),
        globaldir: ::core::ptr::null_mut::<c_char>(),
        save_VIsual_active: false,
        save_prompt_insert: 0,
    };
    let mut ctx: *mut c_void = if scope as c_uint == kOptScopeWin as c_int as c_uint {
        &raw mut switchwin as *mut c_void
    } else if scope as c_uint == kOptScopeBuf as c_int as c_uint {
        &raw mut aco as *mut c_void
    } else {
        NULL
    };
    let mut switched: bool = switch_option_context(ctx, scope, from, err);
    if (*err).type_0 as c_int != kErrorTypeNone as c_int {
        return;
    }
    let errmsg: *const c_char = set_option_value_handle_tty(name, opt_idx, value, opt_flags);
    if !errmsg.is_null() {
        api_set_error(
            err,
            kErrorTypeException,
            b"%s\0".as_ptr() as *const c_char,
            errmsg,
        );
    }
    if switched {
        restore_option_context(ctx, scope);
    }
}
unsafe extern "C" fn showoptions(mut all: bool, mut opt_flags: c_int) {
    let mut items: *mut *mut vimoption_T =
        xmalloc(::core::mem::size_of::<*mut vimoption_T>().wrapping_mul(OPTION_COUNT))
            as *mut *mut vimoption_T;
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const c_char);
    if opt_flags & OPT_GLOBAL as c_int != 0 {
        msg_puts_title(gettext(
            b"\n--- Global option values ---\0".as_ptr() as *const c_char
        ));
    } else if opt_flags & OPT_LOCAL as c_int != 0 {
        msg_puts_title(gettext(
            b"\n--- Local option values ---\0".as_ptr() as *const c_char
        ));
    } else {
        msg_puts_title(gettext(b"\n--- Options ---\0".as_ptr() as *const c_char));
    }
    let mut run: c_int = 1 as c_int;
    while run <= 2 as c_int && !got_int.get() {
        let mut item_count: c_int = 0 as c_int;
        let mut opt: *mut vimoption_T = ::core::ptr::null_mut::<vimoption_T>();
        let mut opt_idx: OptIndex = kOptAleph;
        while (opt_idx as c_int) < kOptCount {
            opt = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
            if !message_filtered((*opt).fullname) {
                let mut varp: *mut c_void = NULL;
                if opt_flags & (OPT_LOCAL as c_int | OPT_GLOBAL as c_int) != 0 as c_int {
                    if !option_is_global_only(opt_idx) {
                        varp = get_varp_scope(opt, opt_flags);
                    }
                } else {
                    varp = get_varp(opt);
                }
                if !varp.is_null() && (all as c_int != 0 || optval_default(opt_idx, varp) == 0) {
                    let mut len: c_int = 0;
                    if opt_flags & OPT_ONECOLUMN as c_int != 0 {
                        len = Columns.get();
                    } else if option_has_type(opt_idx, kOptValTypeBoolean) {
                        len = 1 as c_int;
                    } else {
                        option_value2string(opt, opt_flags);
                        len = strlen((*opt).fullname) as c_int
                            + vim_strsize(NameBuff.ptr() as *mut c_char)
                            + 1 as c_int;
                    }
                    if len <= INC - GAP && run == 1 as c_int || len > INC - GAP && run == 2 as c_int
                    {
                        let c2rust_fresh6 = item_count;
                        item_count = item_count + 1;
                        let c2rust_lvalue_ptr = &raw mut *items.offset(c2rust_fresh6 as isize);
                        *c2rust_lvalue_ptr = opt;
                    }
                }
            }
            opt_idx += 1;
        }
        let mut rows: c_int = 0;
        if run == 1 as c_int {
            '_c2rust_label: {
                if Columns.get() <= 2147483647 as c_int - 3 as c_int
                    && Columns.get() + 3 as c_int >= -2147483647 as c_int - 1 as c_int + 3 as c_int
                    && (Columns.get() + 3 as c_int - 3 as c_int) / 20 as c_int
                        >= -2147483647 as c_int - 1 as c_int
                    && (Columns.get() + 3 as c_int - 3 as c_int) / 20 as c_int
                        <= 2147483647 as c_int
                {
                } else {
                    __assert_fail(
                        b"Columns <= INT_MAX - GAP && Columns + GAP >= INT_MIN + 3 && (Columns + GAP - 3) / INC >= INT_MIN && (Columns + GAP - 3) / INC <= INT_MAX\0"
                            .as_ptr() as *const c_char,
                        b"src/nvim/option.rs\0"
                            .as_ptr() as *const c_char,
                        4288 as c_uint,
                        b"void showoptions(_Bool, int)\0".as_ptr()
                            as *const c_char,
                    );
                }
            };
            let mut cols: c_int = (Columns.get() + GAP - 3 as c_int) / INC;
            if cols == 0 as c_int {
                cols = 1 as c_int;
            }
            rows = (item_count + cols - 1 as c_int) / cols;
        } else {
            rows = item_count;
        }
        let mut row: c_int = 0 as c_int;
        while row < rows && !got_int.get() {
            msg_putchar('\n' as c_int);
            if got_int.get() {
                break;
            }
            let mut col: c_int = 0 as c_int;
            let mut i: c_int = row;
            while i < item_count {
                msg_advance(col);
                showoneopt(*items.offset(i as isize), opt_flags);
                col += INC;
                i += rows;
            }
            os_breakcheck();
            row += 1;
        }
        run += 1;
    }
    xfree(items as *mut c_void);
}
pub const INC: c_int = 20 as c_int;
pub const GAP: c_int = 3 as c_int;
unsafe extern "C" fn optval_default(mut opt_idx: OptIndex, mut varp: *mut c_void) -> c_int {
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    if is_option_hidden(opt_idx) {
        return true_0;
    }
    let mut current_val: OptVal = optval_from_varp(opt_idx, varp);
    let mut default_val: OptVal = (*opt).def_val;
    return optval_equal(current_val, default_val) as c_int;
}
pub unsafe extern "C" fn ui_refresh_options() {
    let mut opt_idx: OptIndex = kOptAleph;
    while (opt_idx as c_int) < kOptCount {
        let mut flags: uint32_t = (*options.ptr())[opt_idx as usize].flags;
        if flags & kOptFlagUIOption as c_int as uint32_t != 0 {
            let mut name: String_0 = cstr_as_string((*options.ptr())[opt_idx as usize].fullname);
            let mut value: Object = optval_as_object(optval_from_varp(
                opt_idx,
                (*options.ptr())[opt_idx as usize].var,
            ));
            ui_call_option_set(name, value);
        }
        opt_idx += 1;
    }
    if !(*p_mouse.ptr()).is_null() {
        setmouse();
    }
}
unsafe extern "C" fn showoneopt(mut opt: *mut vimoption_T, mut opt_flags: c_int) {
    let mut save_silent: c_int = silent_mode.get() as c_int;
    silent_mode.set(false_0 != 0);
    info_message.set(true_0 != 0);
    let mut opt_idx: OptIndex = get_opt_idx(opt);
    let mut varp: *mut c_void = get_varp_scope(opt, opt_flags);
    if option_has_type(opt_idx, kOptValTypeBoolean) as c_int != 0
        && (if varp as *mut c_int == &raw mut (*curbuf.get()).b_changed {
            !curbufIsChanged() as c_int
        } else {
            (*(varp as *mut c_int) == 0) as c_int
        }) != 0
    {
        msg_puts(b"no\0".as_ptr() as *const c_char);
    } else if option_has_type(opt_idx, kOptValTypeBoolean) as c_int != 0
        && *(varp as *mut c_int) < 0 as c_int
    {
        msg_puts(b"--\0".as_ptr() as *const c_char);
    } else {
        msg_puts(b"  \0".as_ptr() as *const c_char);
    }
    msg_puts((*opt).fullname);
    if !option_has_type(opt_idx, kOptValTypeBoolean) {
        msg_putchar('=' as c_int);
        option_value2string(opt, opt_flags);
        if *(NameBuff.ptr() as *mut c_char) as c_int != NUL {
            msg_outtrans(NameBuff.ptr() as *mut c_char, 0 as c_int, false_0 != 0);
        }
    }
    silent_mode.set(save_silent != 0);
    info_message.set(false_0 != 0);
}
pub unsafe extern "C" fn makeset(
    mut fd: *mut FILE,
    mut opt_flags: c_int,
    mut local_only: c_int,
) -> c_int {
    let mut pri: c_int = 1 as c_int;
    while pri >= 0 as c_int {
        let mut opt: *mut vimoption_T = ::core::ptr::null_mut::<vimoption_T>();
        let mut opt_idx: OptIndex = kOptAleph;
        while (opt_idx as c_int) < kOptCount {
            opt = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
            's_14: {
                if (*opt).flags & kOptFlagNoMkrc as c_int as uint32_t == 0
                    && (pri == 1 as c_int) as c_int
                        == ((*opt).flags & kOptFlagPriMkrc as c_int as uint32_t != 0 as uint32_t)
                            as c_int
                {
                    if !(option_is_global_only(opt_idx) as c_int != 0
                        && opt_flags & OPT_GLOBAL as c_int == 0)
                    {
                        if !(opt_flags & OPT_GLOBAL as c_int != 0
                            && (*opt).flags & kOptFlagNoGlob as c_int as uint32_t != 0)
                        {
                            let mut varp: *mut c_void = get_varp_scope(opt, opt_flags);
                            if !varp.is_null() {
                                if !(opt_flags & OPT_GLOBAL as c_int != 0
                                    && optval_default(opt_idx, varp) != 0)
                                {
                                    if !(opt_flags & OPT_SKIPRTP as c_int != 0
                                        && ((*opt).var == p_rtp.ptr() as *mut c_void
                                            || (*opt).var == p_pp.ptr() as *mut c_void))
                                    {
                                        let mut round: c_int = 2 as c_int;
                                        let mut varp_local: *mut c_void = NULL;
                                        if option_is_window_local(opt_idx) {
                                            if opt_flags & OPT_LOCAL as c_int == 0 {
                                                break 's_14;
                                            } else if opt_flags & OPT_GLOBAL as c_int == 0
                                                && local_only == 0
                                            {
                                                let mut varp_fresh: *mut c_void =
                                                    get_varp_scope(opt, OPT_GLOBAL as c_int);
                                                if optval_default(opt_idx, varp_fresh) == 0 {
                                                    round = 1 as c_int;
                                                    varp_local = varp;
                                                    varp = varp_fresh;
                                                }
                                            }
                                        }
                                        while round <= 2 as c_int {
                                            let mut cmd: *mut c_char =
                                                ::core::ptr::null_mut::<c_char>();
                                            if round == 1 as c_int
                                                || opt_flags & OPT_GLOBAL as c_int != 0
                                            {
                                                cmd = b"set\0".as_ptr() as *const c_char
                                                    as *mut c_char;
                                            } else {
                                                cmd = b"setlocal\0".as_ptr() as *const c_char
                                                    as *mut c_char;
                                            }
                                            let mut do_endif: bool = false_0 != 0;
                                            if opt_idx as c_int == kOptSyntax as c_int
                                                || opt_idx as c_int == kOptFiletype as c_int
                                            {
                                                if fprintf(
                                                    fd,
                                                    b"if &%s != '%s'\0".as_ptr() as *const c_char,
                                                    (*opt).fullname,
                                                    *(varp as *mut *mut c_char),
                                                ) < 0 as c_int
                                                    || put_eol(fd) < 0 as c_int
                                                {
                                                    return FAIL;
                                                }
                                                do_endif = true_0 != 0;
                                            }
                                            if put_set(fd, cmd, opt_idx, varp) == FAIL {
                                                return FAIL;
                                            }
                                            if do_endif {
                                                if put_line(
                                                    fd,
                                                    b"endif\0".as_ptr() as *const c_char
                                                        as *mut c_char,
                                                ) == FAIL
                                                {
                                                    return FAIL;
                                                }
                                            }
                                            varp = varp_local;
                                            round += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            opt_idx += 1;
        }
        pri -= 1;
    }
    return OK;
}
pub unsafe extern "C" fn makefoldset(mut fd: *mut FILE) -> c_int {
    if put_set(
        fd,
        b"setlocal\0".as_ptr() as *const c_char as *mut c_char,
        kOptFoldmethod,
        &raw mut (*curwin.get()).w_onebuf_opt.wo_fdm as *mut c_void,
    ) == FAIL
        || put_set(
            fd,
            b"setlocal\0".as_ptr() as *const c_char as *mut c_char,
            kOptFoldexpr,
            &raw mut (*curwin.get()).w_onebuf_opt.wo_fde as *mut c_void,
        ) == FAIL
        || put_set(
            fd,
            b"setlocal\0".as_ptr() as *const c_char as *mut c_char,
            kOptFoldmarker,
            &raw mut (*curwin.get()).w_onebuf_opt.wo_fmr as *mut c_void,
        ) == FAIL
        || put_set(
            fd,
            b"setlocal\0".as_ptr() as *const c_char as *mut c_char,
            kOptFoldignore,
            &raw mut (*curwin.get()).w_onebuf_opt.wo_fdi as *mut c_void,
        ) == FAIL
        || put_set(
            fd,
            b"setlocal\0".as_ptr() as *const c_char as *mut c_char,
            kOptFoldlevel,
            &raw mut (*curwin.get()).w_onebuf_opt.wo_fdl as *mut c_void,
        ) == FAIL
        || put_set(
            fd,
            b"setlocal\0".as_ptr() as *const c_char as *mut c_char,
            kOptFoldminlines,
            &raw mut (*curwin.get()).w_onebuf_opt.wo_fml as *mut c_void,
        ) == FAIL
        || put_set(
            fd,
            b"setlocal\0".as_ptr() as *const c_char as *mut c_char,
            kOptFoldnestmax,
            &raw mut (*curwin.get()).w_onebuf_opt.wo_fdn as *mut c_void,
        ) == FAIL
        || put_set(
            fd,
            b"setlocal\0".as_ptr() as *const c_char as *mut c_char,
            kOptFoldenable,
            &raw mut (*curwin.get()).w_onebuf_opt.wo_fen as *mut c_void,
        ) == FAIL
    {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn put_set(
    mut fd: *mut FILE,
    mut cmd: *mut c_char,
    mut opt_idx: OptIndex,
    mut varp: *mut c_void,
) -> c_int {
    let mut value: OptVal = optval_from_varp(opt_idx, varp);
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    let mut name: *mut c_char = (*opt).fullname;
    let mut flags: uint64_t = (*opt).flags as uint64_t;
    if option_is_global_local(opt_idx) as c_int != 0
        && varp != (*opt).var
        && optval_equal(value, get_option_unset_value(opt_idx)) as c_int != 0
    {
        return OK;
    }
    match value.type_0 as c_int {
        -1 => {
            abort();
        }
        0 => {
            '_c2rust_label: {
                if value.data.boolean as c_int != kNone as c_int {
                } else {
                    __assert_fail(
                        b"value.data.boolean != kNone\0".as_ptr() as *const c_char,
                        b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                        4544 as c_uint,
                        b"int put_set(FILE *, char *, OptIndex, void *)\0".as_ptr()
                            as *const c_char,
                    );
                }
            };
            let mut value_bool: bool = if value.data.boolean as c_int == kTrue as c_int {
                true_0
            } else if value.data.boolean as c_int == kFalse as c_int {
                false_0
            } else {
                0 as c_int
            } != 0;
            if fprintf(
                fd,
                b"%s %s%s\0".as_ptr() as *const c_char,
                cmd,
                if value_bool as c_int != 0 {
                    b"\0".as_ptr() as *const c_char
                } else {
                    b"no\0".as_ptr() as *const c_char
                },
                name,
            ) < 0 as c_int
            {
                return FAIL;
            }
        }
        1 => {
            if fprintf(fd, b"%s %s=\0".as_ptr() as *const c_char, cmd, name) < 0 as c_int {
                return FAIL;
            }
            let mut value_num: OptInt = value.data.number;
            let mut wc: OptInt = 0;
            if wc_use_keyname(varp, &raw mut wc) != 0 {
                if fputs(get_special_key_name(wc as c_int, 0 as c_int), fd) < 0 as c_int {
                    return FAIL;
                }
            } else if fprintf(fd, b"%ld\0".as_ptr() as *const c_char, value_num) < 0 as c_int {
                return FAIL;
            }
        }
        2 => {
            if fprintf(fd, b"%s %s=\0".as_ptr() as *const c_char, cmd, name) < 0 as c_int {
                return FAIL;
            }
            let mut value_str: *const c_char = value.data.string.data;
            let mut buf: *mut c_char = ::core::ptr::null_mut::<c_char>();
            let mut part: *mut c_char = ::core::ptr::null_mut::<c_char>();
            if !value_str.is_null() {
                if flags & kOptFlagExpand as c_int as uint64_t != 0 as uint64_t {
                    let mut size: size_t = strlen(value_str).wrapping_add(1 as size_t);
                    buf = xmalloc(size) as *mut c_char;
                    home_replace(
                        ::core::ptr::null::<buf_T>(),
                        value_str,
                        buf,
                        size,
                        false_0 != 0,
                    );
                    if size >= MAXPATHL as size_t
                        && flags & kOptFlagComma as c_int as uint64_t != 0 as uint64_t
                        && !vim_strchr(value_str, ',' as c_int).is_null()
                    {
                        part = xmalloc(size) as *mut c_char;
                        '_fail: {
                            if put_eol(fd) != FAIL {
                                let mut p: *mut c_char = buf;
                                while *p as c_int != NUL {
                                    if fprintf(
                                        fd,
                                        b"%s %s+=\0".as_ptr() as *const c_char,
                                        cmd,
                                        name,
                                    ) < 0 as c_int
                                    {
                                        break '_fail;
                                    }
                                    copy_option_part(
                                        &raw mut p,
                                        part,
                                        size,
                                        b",\0".as_ptr() as *const c_char as *mut c_char,
                                    );
                                    if put_escstr(fd, part, 2 as c_int) == FAIL
                                        || put_eol(fd) == FAIL
                                    {
                                        break '_fail;
                                    }
                                }
                                xfree(buf as *mut c_void);
                                xfree(part as *mut c_void);
                                return OK;
                            }
                        }
                        xfree(buf as *mut c_void);
                        xfree(part as *mut c_void);
                        return FAIL;
                    } else {
                        if put_escstr(fd, buf, 2 as c_int) == FAIL {
                            xfree(buf as *mut c_void);
                            return FAIL;
                        }
                        xfree(buf as *mut c_void);
                    }
                } else if put_escstr(fd, value_str, 2 as c_int) == FAIL {
                    return FAIL;
                }
            }
        }
        _ => {}
    }
    if put_eol(fd) < 0 as c_int {
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn get_varp_scope_from(
    mut p: *mut vimoption_T,
    mut opt_flags: c_int,
    mut buf: *mut buf_T,
    mut win: *mut win_T,
) -> *mut c_void {
    let mut opt_idx: OptIndex = get_opt_idx(p);
    if opt_flags & OPT_GLOBAL as c_int != 0 && !option_is_global_only(opt_idx) {
        if option_is_window_local(opt_idx) {
            return (get_varp_from(p, buf, win) as *mut c_char)
                .offset(::core::mem::size_of::<winopt_T>() as isize)
                as *mut c_void;
        }
        return (*p).var;
    }
    if opt_flags & OPT_LOCAL as c_int != 0 && option_is_global_local(opt_idx) as c_int != 0 {
        match opt_idx as c_int {
            117 => return &raw mut (*buf).b_p_fp as *mut c_void,
            118 => return &raw mut (*buf).b_p_fs as *mut c_void,
            99 => return &raw mut (*buf).b_p_ffu as *mut c_void,
            87 => return &raw mut (*buf).b_p_efm as *mut c_void,
            120 => return &raw mut (*buf).b_p_gefm as *mut c_void,
            121 => return &raw mut (*buf).b_p_gp as *mut c_void,
            180 => return &raw mut (*buf).b_p_mp as *mut c_void,
            84 => return &raw mut (*buf).b_p_ep as *mut c_void,
            160 => return &raw mut (*buf).b_p_kp as *mut c_void,
            217 => return &raw mut (*buf).b_p_path as *mut c_void,
            6 => return &raw mut (*buf).b_p_ac as *mut c_void,
            10 => return &raw mut (*buf).b_p_ar as *mut c_void,
            310 => return &raw mut (*buf).b_p_tags as *mut c_void,
            306 => return &raw mut (*buf).b_p_tc as *mut c_void,
            276 => {
                return &raw mut (*win).w_onebuf_opt.wo_siso as *mut c_void;
            }
            247 => return &raw mut (*win).w_onebuf_opt.wo_so as *mut c_void,
            67 => return &raw mut (*buf).b_p_def as *mut c_void,
            145 => return &raw mut (*buf).b_p_inc as *mut c_void,
            54 => return &raw mut (*buf).b_p_cot as *mut c_void,
            69 => return &raw mut (*buf).b_p_dict as *mut c_void,
            71 => return &raw mut (*buf).b_p_dia as *mut c_void,
            319 => return &raw mut (*buf).b_p_tsr as *mut c_void,
            320 => return &raw mut (*buf).b_p_tsrfu as *mut c_void,
            307 => return &raw mut (*buf).b_p_tfu as *mut c_void,
            268 => return &raw mut (*win).w_onebuf_opt.wo_sbr as *mut c_void,
            294 => return &raw mut (*win).w_onebuf_opt.wo_stl as *mut c_void,
            355 => return &raw mut (*win).w_onebuf_opt.wo_wbr as *mut c_void,
            333 => return &raw mut (*buf).b_p_ul as *mut c_void,
            173 => return &raw mut (*buf).b_p_lw as *mut c_void,
            16 => return &raw mut (*buf).b_p_bkc as *mut c_void,
            179 => return &raw mut (*buf).b_p_menc as *mut c_void,
            98 => return &raw mut (*win).w_onebuf_opt.wo_fcs as *mut c_void,
            175 => return &raw mut (*win).w_onebuf_opt.wo_lcs as *mut c_void,
            343 => return &raw mut (*win).w_onebuf_opt.wo_ve as *mut c_void,
            _ => {
                abort();
            }
        }
    }
    return get_varp_from(p, buf, win);
}
pub unsafe extern "C" fn get_varp_scope(
    mut p: *mut vimoption_T,
    mut opt_flags: c_int,
) -> *mut c_void {
    return get_varp_scope_from(p, opt_flags, curbuf.get(), curwin.get());
}
pub unsafe extern "C" fn get_option_varp_scope_from(
    mut opt_idx: OptIndex,
    mut opt_flags: c_int,
    mut buf: *mut buf_T,
    mut win: *mut win_T,
) -> *mut c_void {
    return get_varp_scope_from(
        (options.ptr() as *mut vimoption_T).offset(opt_idx as isize),
        opt_flags,
        buf,
        win,
    );
}
pub unsafe extern "C" fn get_varp_from(
    mut p: *mut vimoption_T,
    mut buf: *mut buf_T,
    mut win: *mut win_T,
) -> *mut c_void {
    let mut opt_idx: OptIndex = get_opt_idx(p);
    if is_option_hidden(opt_idx) as c_int != 0 || option_is_global_only(opt_idx) as c_int != 0 {
        return (*p).var;
    }
    match opt_idx as c_int {
        84 => {
            return if *(*buf).b_p_ep as c_int != NUL {
                &raw mut (*buf).b_p_ep as *mut c_void
            } else {
                (*p).var
            };
        }
        160 => {
            return if *(*buf).b_p_kp as c_int != NUL {
                &raw mut (*buf).b_p_kp as *mut c_void
            } else {
                (*p).var
            };
        }
        217 => {
            return if *(*buf).b_p_path as c_int != NUL {
                &raw mut (*buf).b_p_path as *mut c_void
            } else {
                (*p).var
            };
        }
        6 => {
            return if (*buf).b_p_ac >= 0 as c_int {
                &raw mut (*buf).b_p_ac as *mut c_void
            } else {
                (*p).var
            };
        }
        10 => {
            return if (*buf).b_p_ar >= 0 as c_int {
                &raw mut (*buf).b_p_ar as *mut c_void
            } else {
                (*p).var
            };
        }
        310 => {
            return if *(*buf).b_p_tags as c_int != NUL {
                &raw mut (*buf).b_p_tags as *mut c_void
            } else {
                (*p).var
            };
        }
        306 => {
            return if *(*buf).b_p_tc as c_int != NUL {
                &raw mut (*buf).b_p_tc as *mut c_void
            } else {
                (*p).var
            };
        }
        276 => {
            return if (*win).w_onebuf_opt.wo_siso >= 0 as OptInt {
                &raw mut (*win).w_onebuf_opt.wo_siso as *mut c_void
            } else {
                (*p).var
            };
        }
        247 => {
            return if (*win).w_onebuf_opt.wo_so >= 0 as OptInt {
                &raw mut (*win).w_onebuf_opt.wo_so as *mut c_void
            } else {
                (*p).var
            };
        }
        16 => {
            return if *(*buf).b_p_bkc as c_int != NUL {
                &raw mut (*buf).b_p_bkc as *mut c_void
            } else {
                (*p).var
            };
        }
        67 => {
            return if *(*buf).b_p_def as c_int != NUL {
                &raw mut (*buf).b_p_def as *mut c_void
            } else {
                (*p).var
            };
        }
        145 => {
            return if *(*buf).b_p_inc as c_int != NUL {
                &raw mut (*buf).b_p_inc as *mut c_void
            } else {
                (*p).var
            };
        }
        54 => {
            return if *(*buf).b_p_cot as c_int != NUL {
                &raw mut (*buf).b_p_cot as *mut c_void
            } else {
                (*p).var
            };
        }
        69 => {
            return if *(*buf).b_p_dict as c_int != NUL {
                &raw mut (*buf).b_p_dict as *mut c_void
            } else {
                (*p).var
            };
        }
        71 => {
            return if *(*buf).b_p_dia as c_int != NUL {
                &raw mut (*buf).b_p_dia as *mut c_void
            } else {
                (*p).var
            };
        }
        319 => {
            return if *(*buf).b_p_tsr as c_int != NUL {
                &raw mut (*buf).b_p_tsr as *mut c_void
            } else {
                (*p).var
            };
        }
        320 => {
            return if *(*buf).b_p_tsrfu as c_int != NUL {
                &raw mut (*buf).b_p_tsrfu as *mut c_void
            } else {
                (*p).var
            };
        }
        117 => {
            return if *(*buf).b_p_fp as c_int != NUL {
                &raw mut (*buf).b_p_fp as *mut c_void
            } else {
                (*p).var
            };
        }
        118 => {
            return if (*buf).b_p_fs >= 0 as c_int {
                &raw mut (*buf).b_p_fs as *mut c_void
            } else {
                (*p).var
            };
        }
        99 => {
            return if *(*buf).b_p_ffu as c_int != NUL {
                &raw mut (*buf).b_p_ffu as *mut c_void
            } else {
                (*p).var
            };
        }
        87 => {
            return if *(*buf).b_p_efm as c_int != NUL {
                &raw mut (*buf).b_p_efm as *mut c_void
            } else {
                (*p).var
            };
        }
        120 => {
            return if *(*buf).b_p_gefm as c_int != NUL {
                &raw mut (*buf).b_p_gefm as *mut c_void
            } else {
                (*p).var
            };
        }
        121 => {
            return if *(*buf).b_p_gp as c_int != NUL {
                &raw mut (*buf).b_p_gp as *mut c_void
            } else {
                (*p).var
            };
        }
        180 => {
            return if *(*buf).b_p_mp as c_int != NUL {
                &raw mut (*buf).b_p_mp as *mut c_void
            } else {
                (*p).var
            };
        }
        268 => {
            return if *(*win).w_onebuf_opt.wo_sbr as c_int != NUL {
                &raw mut (*win).w_onebuf_opt.wo_sbr as *mut c_void
            } else {
                (*p).var
            };
        }
        294 => {
            return if *(*win).w_onebuf_opt.wo_stl as c_int != NUL {
                &raw mut (*win).w_onebuf_opt.wo_stl as *mut c_void
            } else {
                (*p).var
            };
        }
        355 => {
            return if *(*win).w_onebuf_opt.wo_wbr as c_int != NUL {
                &raw mut (*win).w_onebuf_opt.wo_wbr as *mut c_void
            } else {
                (*p).var
            };
        }
        333 => {
            return if (*buf).b_p_ul != NO_LOCAL_UNDOLEVEL as OptInt {
                &raw mut (*buf).b_p_ul as *mut c_void
            } else {
                (*p).var
            };
        }
        173 => {
            return if *(*buf).b_p_lw as c_int != NUL {
                &raw mut (*buf).b_p_lw as *mut c_void
            } else {
                (*p).var
            };
        }
        179 => {
            return if *(*buf).b_p_menc as c_int != NUL {
                &raw mut (*buf).b_p_menc as *mut c_void
            } else {
                (*p).var
            };
        }
        98 => {
            return if *(*win).w_onebuf_opt.wo_fcs as c_int != NUL {
                &raw mut (*win).w_onebuf_opt.wo_fcs as *mut c_void
            } else {
                (*p).var
            };
        }
        175 => {
            return if *(*win).w_onebuf_opt.wo_lcs as c_int != NUL {
                &raw mut (*win).w_onebuf_opt.wo_lcs as *mut c_void
            } else {
                (*p).var
            };
        }
        343 => {
            return if *(*win).w_onebuf_opt.wo_ve as c_int != NUL {
                &raw mut (*win).w_onebuf_opt.wo_ve as *mut c_void
            } else {
                (*p).var
            };
        }
        3 => return &raw mut (*win).w_onebuf_opt.wo_arab as *mut c_void,
        174 => return &raw mut (*win).w_onebuf_opt.wo_list as *mut c_void,
        283 => return &raw mut (*win).w_onebuf_opt.wo_spell as *mut c_void,
        63 => return &raw mut (*win).w_onebuf_opt.wo_cuc as *mut c_void,
        64 => return &raw mut (*win).w_onebuf_opt.wo_cul as *mut c_void,
        65 => return &raw mut (*win).w_onebuf_opt.wo_culopt as *mut c_void,
        46 => return &raw mut (*win).w_onebuf_opt.wo_cc as *mut c_void,
        70 => return &raw mut (*win).w_onebuf_opt.wo_diff as *mut c_void,
        89 => return &raw mut (*win).w_onebuf_opt.wo_eiw as *mut c_void,
        102 => return &raw mut (*win).w_onebuf_opt.wo_fdc as *mut c_void,
        103 => return &raw mut (*win).w_onebuf_opt.wo_fen as *mut c_void,
        105 => return &raw mut (*win).w_onebuf_opt.wo_fdi as *mut c_void,
        106 => return &raw mut (*win).w_onebuf_opt.wo_fdl as *mut c_void,
        109 => return &raw mut (*win).w_onebuf_opt.wo_fdm as *mut c_void,
        110 => return &raw mut (*win).w_onebuf_opt.wo_fml as *mut c_void,
        111 => return &raw mut (*win).w_onebuf_opt.wo_fdn as *mut c_void,
        104 => return &raw mut (*win).w_onebuf_opt.wo_fde as *mut c_void,
        113 => return &raw mut (*win).w_onebuf_opt.wo_fdt as *mut c_void,
        108 => return &raw mut (*win).w_onebuf_opt.wo_fmr as *mut c_void,
        206 => return &raw mut (*win).w_onebuf_opt.wo_nu as *mut c_void,
        234 => return &raw mut (*win).w_onebuf_opt.wo_rnu as *mut c_void,
        207 => return &raw mut (*win).w_onebuf_opt.wo_nuw as *mut c_void,
        359 => return &raw mut (*win).w_onebuf_opt.wo_wfb as *mut c_void,
        360 => return &raw mut (*win).w_onebuf_opt.wo_wfh as *mut c_void,
        361 => return &raw mut (*win).w_onebuf_opt.wo_wfw as *mut c_void,
        220 => return &raw mut (*win).w_onebuf_opt.wo_pvw as *mut c_void,
        167 => return &raw mut (*win).w_onebuf_opt.wo_lhi as *mut c_void,
        238 => return &raw mut (*win).w_onebuf_opt.wo_rl as *mut c_void,
        239 => return &raw mut (*win).w_onebuf_opt.wo_rlc as *mut c_void,
        243 => return &raw mut (*win).w_onebuf_opt.wo_scr as *mut c_void,
        281 => return &raw mut (*win).w_onebuf_opt.wo_sms as *mut c_void,
        367 => return &raw mut (*win).w_onebuf_opt.wo_wrap as *mut c_void,
        168 => return &raw mut (*win).w_onebuf_opt.wo_lbr as *mut c_void,
        24 => return &raw mut (*win).w_onebuf_opt.wo_bri as *mut c_void,
        25 => return &raw mut (*win).w_onebuf_opt.wo_briopt as *mut c_void,
        245 => return &raw mut (*win).w_onebuf_opt.wo_scb as *mut c_void,
        62 => return &raw mut (*win).w_onebuf_opt.wo_crb as *mut c_void,
        57 => return &raw mut (*win).w_onebuf_opt.wo_cocu as *mut c_void,
        58 => return &raw mut (*win).w_onebuf_opt.wo_cole as *mut c_void,
        9 => return &raw mut (*buf).b_p_ai as *mut c_void,
        21 => return &raw mut (*buf).b_p_bin as *mut c_void,
        22 => return &raw mut (*buf).b_p_bomb as *mut c_void,
        27 => return &raw mut (*buf).b_p_bh as *mut c_void,
        29 => return &raw mut (*buf).b_p_bt as *mut c_void,
        28 => return &raw mut (*buf).b_p_bl as *mut c_void,
        30 => return &raw mut (*buf).b_p_busy as *mut c_void,
        35 => return &raw mut (*buf).b_p_channel as *mut c_void,
        60 => return &raw mut (*buf).b_p_ci as *mut c_void,
        38 => return &raw mut (*buf).b_p_cin as *mut c_void,
        39 => return &raw mut (*buf).b_p_cink as *mut c_void,
        40 => return &raw mut (*buf).b_p_cino as *mut c_void,
        41 => return &raw mut (*buf).b_p_cinsd as *mut c_void,
        42 => return &raw mut (*buf).b_p_cinw as *mut c_void,
        48 => return &raw mut (*buf).b_p_com as *mut c_void,
        49 => return &raw mut (*buf).b_p_cms as *mut c_void,
        51 => return &raw mut (*buf).b_p_cpt as *mut c_void,
        52 => return &raw mut (*buf).b_p_cfu as *mut c_void,
        208 => return &raw mut (*buf).b_p_ofu as *mut c_void,
        81 => return &raw mut (*buf).b_p_eof as *mut c_void,
        82 => return &raw mut (*buf).b_p_eol as *mut c_void,
        100 => return &raw mut (*buf).b_p_fixeol as *mut c_void,
        90 => return &raw mut (*buf).b_p_et as *mut c_void,
        92 => return &raw mut (*buf).b_p_fenc as *mut c_void,
        94 => return &raw mut (*buf).b_p_ff as *mut c_void,
        97 => return &raw mut (*buf).b_p_ft as *mut c_void,
        116 => return &raw mut (*buf).b_p_fo as *mut c_void,
        115 => return &raw mut (*buf).b_p_flp as *mut c_void,
        142 => return &raw mut (*buf).b_p_iminsert as *mut c_void,
        143 => return &raw mut (*buf).b_p_imsearch as *mut c_void,
        150 => return &raw mut (*buf).b_p_inf as *mut c_void,
        154 => return &raw mut (*buf).b_p_isk as *mut c_void,
        146 => return &raw mut (*buf).b_p_inex as *mut c_void,
        148 => return &raw mut (*buf).b_p_inde as *mut c_void,
        149 => return &raw mut (*buf).b_p_indk as *mut c_void,
        114 => return &raw mut (*buf).b_p_fex as *mut c_void,
        171 => return &raw mut (*buf).b_p_lisp as *mut c_void,
        172 => return &raw mut (*buf).b_p_lop as *mut c_void,
        191 => return &raw mut (*buf).b_p_ml as *mut c_void,
        181 => return &raw mut (*buf).b_p_mps as *mut c_void,
        194 => return &raw mut (*buf).b_p_ma as *mut c_void,
        195 => return &raw mut (*buf).b_changed as *mut c_void,
        205 => return &raw mut (*buf).b_p_nf as *mut c_void,
        218 => return &raw mut (*buf).b_p_pi as *mut c_void,
        229 => return &raw mut (*buf).b_p_qe as *mut c_void,
        230 => return &raw mut (*buf).b_p_ro as *mut c_void,
        244 => return &raw mut (*buf).b_p_scbk as *mut c_void,
        279 => return &raw mut (*buf).b_p_si as *mut c_void,
        282 => return &raw mut (*buf).b_p_sts as *mut c_void,
        296 => return &raw mut (*buf).b_p_sua as *mut c_void,
        297 => return &raw mut (*buf).b_p_swf as *mut c_void,
        299 => return &raw mut (*buf).b_p_smc as *mut c_void,
        300 => return &raw mut (*buf).b_p_syn as *mut c_void,
        284 => return &raw mut (*(*win).w_s).b_p_spc as *mut c_void,
        285 => return &raw mut (*(*win).w_s).b_p_spf as *mut c_void,
        286 => return &raw mut (*(*win).w_s).b_p_spl as *mut c_void,
        287 => return &raw mut (*(*win).w_s).b_p_spo as *mut c_void,
        266 => return &raw mut (*buf).b_p_sw as *mut c_void,
        307 => return &raw mut (*buf).b_p_tfu as *mut c_void,
        304 => return &raw mut (*buf).b_p_ts as *mut c_void,
        318 => return &raw mut (*buf).b_p_tw as *mut c_void,
        332 => return &raw mut (*buf).b_p_udf as *mut c_void,
        368 => return &raw mut (*buf).b_p_wm as *mut c_void,
        337 => return &raw mut (*buf).b_p_vsts as *mut c_void,
        338 => return &raw mut (*buf).b_p_vts as *mut c_void,
        158 => return &raw mut (*buf).b_p_keymap as *mut c_void,
        277 => return &raw mut (*win).w_onebuf_opt.wo_scl as *mut c_void,
        363 => return &raw mut (*win).w_onebuf_opt.wo_winhl as *mut c_void,
        356 => return &raw mut (*win).w_onebuf_opt.wo_winbl as *mut c_void,
        293 => return &raw mut (*win).w_onebuf_opt.wo_stc as *mut c_void,
        _ => {
            iemsg(gettext(b"E356: get_varp ERROR\0".as_ptr() as *const c_char));
        }
    }
    return &raw mut (*buf).b_p_wm as *mut c_void;
}
#[inline]
unsafe extern "C" fn get_opt_idx(mut opt: *mut vimoption_T) -> OptIndex {
    return opt.offset_from(options.ptr() as *mut vimoption_T) as OptIndex;
}
#[inline]
unsafe extern "C" fn get_varp(mut p: *mut vimoption_T) -> *mut c_void {
    return get_varp_from(p, curbuf.get(), curwin.get());
}
pub unsafe extern "C" fn get_equalprg() -> *mut c_char {
    if *(*curbuf.get()).b_p_ep as c_int == NUL {
        return p_ep.get();
    }
    return (*curbuf.get()).b_p_ep;
}
pub unsafe extern "C" fn get_findfunc() -> *mut c_char {
    if *(*curbuf.get()).b_p_ffu as c_int == NUL {
        return p_ffu.get();
    }
    return (*curbuf.get()).b_p_ffu;
}
pub unsafe extern "C" fn win_copy_options(mut wp_from: *mut win_T, mut wp_to: *mut win_T) {
    copy_winopt(
        &raw mut (*wp_from).w_onebuf_opt,
        &raw mut (*wp_to).w_onebuf_opt,
    );
    copy_winopt(
        &raw mut (*wp_from).w_allbuf_opt,
        &raw mut (*wp_to).w_allbuf_opt,
    );
    didset_window_options(wp_to, true_0 != 0);
}
unsafe extern "C" fn copy_option_val(mut val: *const c_char) -> *mut c_char {
    if val == empty_string_option.ptr() as *mut c_char as *const c_char {
        return empty_string_option.ptr() as *mut c_char;
    }
    return xstrdup(val);
}
pub unsafe extern "C" fn copy_winopt(mut from: *mut winopt_T, mut to: *mut winopt_T) {
    (*to).wo_arab = (*from).wo_arab;
    (*to).wo_list = (*from).wo_list;
    (*to).wo_lcs = copy_option_val((*from).wo_lcs);
    (*to).wo_fcs = copy_option_val((*from).wo_fcs);
    (*to).wo_nu = (*from).wo_nu;
    (*to).wo_rnu = (*from).wo_rnu;
    (*to).wo_ve = copy_option_val((*from).wo_ve);
    (*to).wo_ve_flags = (*from).wo_ve_flags;
    (*to).wo_nuw = (*from).wo_nuw;
    (*to).wo_rl = (*from).wo_rl;
    (*to).wo_rlc = copy_option_val((*from).wo_rlc);
    (*to).wo_sbr = copy_option_val((*from).wo_sbr);
    (*to).wo_stl = copy_option_val((*from).wo_stl);
    (*to).wo_wbr = copy_option_val((*from).wo_wbr);
    (*to).wo_wrap = (*from).wo_wrap;
    (*to).wo_wrap_save = (*from).wo_wrap_save;
    (*to).wo_lbr = (*from).wo_lbr;
    (*to).wo_bri = (*from).wo_bri;
    (*to).wo_briopt = copy_option_val((*from).wo_briopt);
    (*to).wo_scb = (*from).wo_scb;
    (*to).wo_scb_save = (*from).wo_scb_save;
    (*to).wo_sms = (*from).wo_sms;
    (*to).wo_crb = (*from).wo_crb;
    (*to).wo_crb_save = (*from).wo_crb_save;
    (*to).wo_siso = (*from).wo_siso;
    (*to).wo_so = (*from).wo_so;
    (*to).wo_spell = (*from).wo_spell;
    (*to).wo_cuc = (*from).wo_cuc;
    (*to).wo_cul = (*from).wo_cul;
    (*to).wo_culopt = copy_option_val((*from).wo_culopt);
    (*to).wo_cc = copy_option_val((*from).wo_cc);
    (*to).wo_diff = (*from).wo_diff;
    (*to).wo_diff_saved = (*from).wo_diff_saved;
    (*to).wo_eiw = copy_option_val((*from).wo_eiw);
    (*to).wo_cocu = copy_option_val((*from).wo_cocu);
    (*to).wo_cole = (*from).wo_cole;
    (*to).wo_fdc = copy_option_val((*from).wo_fdc);
    (*to).wo_fdc_save = if (*from).wo_diff_saved != 0 {
        xstrdup((*from).wo_fdc_save)
    } else {
        empty_string_option.ptr() as *mut c_char
    };
    (*to).wo_fen = (*from).wo_fen;
    (*to).wo_fen_save = (*from).wo_fen_save;
    (*to).wo_fdi = copy_option_val((*from).wo_fdi);
    (*to).wo_fml = (*from).wo_fml;
    (*to).wo_fdl = (*from).wo_fdl;
    (*to).wo_fdl_save = (*from).wo_fdl_save;
    (*to).wo_fdm = copy_option_val((*from).wo_fdm);
    (*to).wo_fdm_save = if (*from).wo_diff_saved != 0 {
        xstrdup((*from).wo_fdm_save)
    } else {
        empty_string_option.ptr() as *mut c_char
    };
    (*to).wo_fdn = (*from).wo_fdn;
    (*to).wo_fde = copy_option_val((*from).wo_fde);
    (*to).wo_fdt = copy_option_val((*from).wo_fdt);
    (*to).wo_fmr = copy_option_val((*from).wo_fmr);
    (*to).wo_scl = copy_option_val((*from).wo_scl);
    (*to).wo_lhi = (*from).wo_lhi;
    (*to).wo_winhl = copy_option_val((*from).wo_winhl);
    (*to).wo_winbl = (*from).wo_winbl;
    (*to).wo_stc = copy_option_val((*from).wo_stc);
    (*to).wo_wrap_flags = (*from).wo_wrap_flags;
    (*to).wo_stl_flags = (*from).wo_stl_flags;
    (*to).wo_wbr_flags = (*from).wo_wbr_flags;
    (*to).wo_fde_flags = (*from).wo_fde_flags;
    (*to).wo_fdt_flags = (*from).wo_fdt_flags;
    memmove(
        &raw mut (*to).wo_script_ctx as *mut sctx_T as *mut c_void,
        &raw mut (*from).wo_script_ctx as *mut sctx_T as *const c_void,
        ::core::mem::size_of::<[sctx_T; 51]>(),
    );
    check_winopt(to);
}
unsafe extern "C" fn check_win_options(mut win: *mut win_T) {
    check_winopt(&raw mut (*win).w_onebuf_opt);
    check_winopt(&raw mut (*win).w_allbuf_opt);
}
unsafe extern "C" fn check_winopt(mut wop: *mut winopt_T) {
    check_string_option(&raw mut (*wop).wo_fdc);
    check_string_option(&raw mut (*wop).wo_fdc_save);
    check_string_option(&raw mut (*wop).wo_fdi);
    check_string_option(&raw mut (*wop).wo_fdm);
    check_string_option(&raw mut (*wop).wo_fdm_save);
    check_string_option(&raw mut (*wop).wo_fde);
    check_string_option(&raw mut (*wop).wo_fdt);
    check_string_option(&raw mut (*wop).wo_fmr);
    check_string_option(&raw mut (*wop).wo_eiw);
    check_string_option(&raw mut (*wop).wo_scl);
    check_string_option(&raw mut (*wop).wo_rlc);
    check_string_option(&raw mut (*wop).wo_sbr);
    check_string_option(&raw mut (*wop).wo_stl);
    check_string_option(&raw mut (*wop).wo_culopt);
    check_string_option(&raw mut (*wop).wo_cc);
    check_string_option(&raw mut (*wop).wo_cocu);
    check_string_option(&raw mut (*wop).wo_briopt);
    check_string_option(&raw mut (*wop).wo_winhl);
    check_string_option(&raw mut (*wop).wo_lcs);
    check_string_option(&raw mut (*wop).wo_fcs);
    check_string_option(&raw mut (*wop).wo_ve);
    check_string_option(&raw mut (*wop).wo_wbr);
    check_string_option(&raw mut (*wop).wo_stc);
}
pub unsafe extern "C" fn clear_winopt(mut wop: *mut winopt_T) {
    clear_string_option(&raw mut (*wop).wo_fdc);
    clear_string_option(&raw mut (*wop).wo_fdc_save);
    clear_string_option(&raw mut (*wop).wo_fdi);
    clear_string_option(&raw mut (*wop).wo_fdm);
    clear_string_option(&raw mut (*wop).wo_fdm_save);
    clear_string_option(&raw mut (*wop).wo_fde);
    clear_string_option(&raw mut (*wop).wo_fdt);
    clear_string_option(&raw mut (*wop).wo_fmr);
    clear_string_option(&raw mut (*wop).wo_eiw);
    clear_string_option(&raw mut (*wop).wo_scl);
    clear_string_option(&raw mut (*wop).wo_rlc);
    clear_string_option(&raw mut (*wop).wo_sbr);
    clear_string_option(&raw mut (*wop).wo_stl);
    clear_string_option(&raw mut (*wop).wo_culopt);
    clear_string_option(&raw mut (*wop).wo_cc);
    clear_string_option(&raw mut (*wop).wo_cocu);
    clear_string_option(&raw mut (*wop).wo_briopt);
    clear_string_option(&raw mut (*wop).wo_winhl);
    clear_string_option(&raw mut (*wop).wo_lcs);
    clear_string_option(&raw mut (*wop).wo_fcs);
    clear_string_option(&raw mut (*wop).wo_ve);
    clear_string_option(&raw mut (*wop).wo_wbr);
    clear_string_option(&raw mut (*wop).wo_stc);
}
pub unsafe extern "C" fn didset_window_options(mut wp: *mut win_T, mut valid_cursor: bool) {
    if (*wp).w_onebuf_opt.wo_wrap != 0 {
        (*wp).w_leftcol = 0 as c_int as colnr_T;
    } else {
        (*wp).w_skipcol = 0 as c_int as colnr_T;
    }
    check_colorcolumn(::core::ptr::null_mut::<c_char>(), wp);
    briopt_check(::core::ptr::null_mut::<c_char>(), wp);
    fill_culopt_flags(::core::ptr::null_mut::<c_char>(), wp);
    set_chars_option(
        wp,
        (*wp).w_onebuf_opt.wo_fcs,
        kFillchars,
        true_0 != 0,
        ::core::ptr::null_mut::<c_char>(),
        0 as size_t,
    );
    set_chars_option(
        wp,
        (*wp).w_onebuf_opt.wo_lcs,
        kListchars,
        true_0 != 0,
        ::core::ptr::null_mut::<c_char>(),
        0 as size_t,
    );
    parse_winhl_opt(::core::ptr::null::<c_char>(), wp);
    check_blending(wp);
    set_winbar_win(wp, false_0 != 0, valid_cursor);
    check_signcolumn(::core::ptr::null_mut::<c_char>(), wp);
    (*wp).w_grid_alloc.blending = (*wp).w_onebuf_opt.wo_winbl > 0 as OptInt;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn buf_copy_options(mut buf: *mut buf_T, mut flags: c_int) {
    let mut should_copy: bool = true_0 != 0;
    let mut save_p_isk: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut did_isk: bool = false_0 != 0;
    if !(*p_cpo.ptr()).is_null() {
        if (vim_strchr(p_cpo.get(), CPO_BUFOPTGLOB).is_null() || flags & BCO_ENTER as c_int == 0)
            && ((*buf).b_p_initialized as c_int != 0
                || flags & BCO_ENTER as c_int == 0
                    && !vim_strchr(p_cpo.get(), CPO_BUFOPT).is_null())
        {
            should_copy = false_0 != 0;
        }
        if should_copy as c_int != 0 || flags & BCO_ALWAYS as c_int != 0 {
            memset(
                &raw mut (*buf).b_p_script_ctx as *mut c_void,
                0 as c_int,
                ::core::mem::size_of::<[sctx_T; 92]>(),
            );
            let mut dont_do_help: bool = flags & BCO_NOHELP as c_int != 0
                && (*buf).b_help as c_int != 0
                || (*buf).b_p_initialized as c_int != 0;
            if dont_do_help {
                save_p_isk = (*buf).b_p_isk;
                (*buf).b_p_isk = ::core::ptr::null_mut::<c_char>();
            }
            if !(*buf).b_p_initialized {
                free_buf_options(buf, true_0 != 0);
                (*buf).b_p_ro = false_0;
                (*buf).b_p_fenc = xstrdup(p_fenc.get());
                match *p_ffs.get() as c_int {
                    109 => {
                        (*buf).b_p_ff = xstrdup(b"mac\0".as_ptr() as *const c_char);
                    }
                    100 => {
                        (*buf).b_p_ff = xstrdup(b"dos\0".as_ptr() as *const c_char);
                    }
                    117 => {
                        (*buf).b_p_ff = xstrdup(b"unix\0".as_ptr() as *const c_char);
                    }
                    _ => {
                        (*buf).b_p_ff = xstrdup(p_ff.get());
                    }
                }
                (*buf).b_p_bh = empty_string_option.ptr() as *mut c_char;
                (*buf).b_p_bt = empty_string_option.ptr() as *mut c_char;
            } else {
                free_buf_options(buf, false_0 != 0);
            }
            (*buf).b_p_ai = p_ai.get();
            (*buf).b_p_script_ctx[kBufOptAutoindent as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptAutoindent as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_ai_nopaste = p_ai_nopaste.get();
            (*buf).b_p_sw = p_sw.get();
            (*buf).b_p_script_ctx[kBufOptShiftwidth as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptShiftwidth as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_scbk = p_scbk.get();
            (*buf).b_p_script_ctx[kBufOptScrollback as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptScrollback as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_tw = p_tw.get();
            (*buf).b_p_script_ctx[kBufOptTextwidth as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptTextwidth as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_tw_nopaste = p_tw_nopaste.get();
            (*buf).b_p_tw_nobin = p_tw_nobin.get();
            (*buf).b_p_wm = p_wm.get();
            (*buf).b_p_script_ctx[kBufOptWrapmargin as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptWrapmargin as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_wm_nopaste = p_wm_nopaste.get();
            (*buf).b_p_wm_nobin = p_wm_nobin.get();
            (*buf).b_p_bin = p_bin.get();
            (*buf).b_p_script_ctx[kBufOptBinary as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptBinary as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_bomb = p_bomb.get();
            (*buf).b_p_script_ctx[kBufOptBomb as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex).offset(kBufOptBomb as c_int as isize)
                    as usize]
                .script_ctx;
            (*buf).b_p_et = p_et.get();
            (*buf).b_p_script_ctx[kBufOptExpandtab as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptExpandtab as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_fixeol = p_fixeol.get();
            (*buf).b_p_script_ctx[kBufOptFixendofline as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptFixendofline as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_et_nobin = p_et_nobin.get();
            (*buf).b_p_et_nopaste = p_et_nopaste.get();
            (*buf).b_p_ml = p_ml.get();
            (*buf).b_p_script_ctx[kBufOptModeline as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptModeline as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_ml_nobin = p_ml_nobin.get();
            (*buf).b_p_inf = p_inf.get();
            (*buf).b_p_script_ctx[kBufOptInfercase as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptInfercase as c_int as isize) as usize]
                .script_ctx;
            if (*cmdmod.ptr()).cmod_flags & CMOD_NOSWAPFILE as c_int != 0 {
                (*buf).b_p_swf = false_0;
            } else {
                (*buf).b_p_swf = p_swf.get();
                (*buf).b_p_script_ctx[kBufOptSwapfile as c_int as usize] = (*options.ptr())
                    [*(&raw const buf_opt_idx as *const OptIndex)
                        .offset(kBufOptSwapfile as c_int as isize) as usize]
                    .script_ctx;
            }
            (*buf).b_p_cpt = xstrdup(p_cpt.get());
            (*buf).b_p_script_ctx[kBufOptComplete as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptComplete as c_int as isize) as usize]
                .script_ctx;
            set_buflocal_cpt_callbacks(buf);
            (*buf).b_p_cfu = xstrdup(p_cfu.get());
            (*buf).b_p_script_ctx[kBufOptCompletefunc as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptCompletefunc as c_int as isize) as usize]
                .script_ctx;
            set_buflocal_cfu_callback(buf);
            (*buf).b_p_ofu = xstrdup(p_ofu.get());
            (*buf).b_p_script_ctx[kBufOptOmnifunc as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptOmnifunc as c_int as isize) as usize]
                .script_ctx;
            set_buflocal_ofu_callback(buf);
            (*buf).b_p_tfu = xstrdup(p_tfu.get());
            (*buf).b_p_script_ctx[kBufOptTagfunc as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptTagfunc as c_int as isize) as usize]
                .script_ctx;
            set_buflocal_tfu_callback(buf);
            (*buf).b_p_sts = p_sts.get();
            (*buf).b_p_script_ctx[kBufOptSofttabstop as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptSofttabstop as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_sts_nopaste = p_sts_nopaste.get();
            (*buf).b_p_vsts = xstrdup(p_vsts.get());
            (*buf).b_p_script_ctx[kBufOptVarsofttabstop as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptVarsofttabstop as c_int as isize) as usize]
                .script_ctx;
            if !(*p_vsts.ptr()).is_null()
                && p_vsts.get() != empty_string_option.ptr() as *mut c_char
            {
                tabstop_set(p_vsts.get(), &raw mut (*buf).b_p_vsts_array);
            } else {
                (*buf).b_p_vsts_array = ::core::ptr::null_mut::<colnr_T>();
            }
            (*buf).b_p_vsts_nopaste = if !(*p_vsts_nopaste.ptr()).is_null() {
                xstrdup(p_vsts_nopaste.get())
            } else {
                ::core::ptr::null_mut::<c_char>()
            };
            (*buf).b_p_com = xstrdup(p_com.get());
            (*buf).b_p_script_ctx[kBufOptComments as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptComments as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_cms = xstrdup(p_cms.get());
            (*buf).b_p_script_ctx[kBufOptCommentstring as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptCommentstring as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_fo = xstrdup(p_fo.get());
            (*buf).b_p_script_ctx[kBufOptFormatoptions as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptFormatoptions as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_flp = xstrdup(p_flp.get());
            (*buf).b_p_script_ctx[kBufOptFormatlistpat as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptFormatlistpat as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_nf = xstrdup(p_nf.get());
            (*buf).b_p_script_ctx[kBufOptNrformats as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptNrformats as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_mps = xstrdup(p_mps.get());
            (*buf).b_p_script_ctx[kBufOptMatchpairs as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptMatchpairs as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_si = p_si.get();
            (*buf).b_p_script_ctx[kBufOptSmartindent as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptSmartindent as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_channel = 0 as OptInt;
            (*buf).b_p_ci = p_ci.get();
            (*buf).b_p_script_ctx[kBufOptCopyindent as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptCopyindent as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_cin = p_cin.get();
            (*buf).b_p_script_ctx[kBufOptCindent as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptCindent as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_cink = xstrdup(p_cink.get());
            (*buf).b_p_script_ctx[kBufOptCinkeys as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptCinkeys as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_cino = xstrdup(p_cino.get());
            (*buf).b_p_script_ctx[kBufOptCinoptions as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptCinoptions as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_cinsd = xstrdup(p_cinsd.get());
            (*buf).b_p_script_ctx[kBufOptCinscopedecls as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptCinscopedecls as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_lop = xstrdup(p_lop.get());
            (*buf).b_p_script_ctx[kBufOptLispoptions as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptLispoptions as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_ft = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_pi = p_pi.get();
            (*buf).b_p_script_ctx[kBufOptPreserveindent as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptPreserveindent as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_cinw = xstrdup(p_cinw.get());
            (*buf).b_p_script_ctx[kBufOptCinwords as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptCinwords as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_lisp = p_lisp.get();
            (*buf).b_p_script_ctx[kBufOptLisp as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex).offset(kBufOptLisp as c_int as isize)
                    as usize]
                .script_ctx;
            (*buf).b_p_syn = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_smc = p_smc.get();
            (*buf).b_p_script_ctx[kBufOptSynmaxcol as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptSynmaxcol as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_s.b_syn_isk = empty_string_option.ptr() as *mut c_char;
            (*buf).b_s.b_p_spc = xstrdup(p_spc.get());
            (*buf).b_p_script_ctx[kBufOptSpellcapcheck as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptSpellcapcheck as c_int as isize) as usize]
                .script_ctx;
            compile_cap_prog(&raw mut (*buf).b_s);
            (*buf).b_s.b_p_spf = xstrdup(p_spf.get());
            (*buf).b_p_script_ctx[kBufOptSpellfile as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptSpellfile as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_s.b_p_spl = xstrdup(p_spl.get());
            (*buf).b_p_script_ctx[kBufOptSpelllang as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptSpelllang as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_s.b_p_spo = xstrdup(p_spo.get());
            (*buf).b_p_script_ctx[kBufOptSpelloptions as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptSpelloptions as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_s.b_p_spo_flags = spo_flags.get();
            (*buf).b_p_inde = xstrdup(p_inde.get());
            (*buf).b_p_script_ctx[kBufOptIndentexpr as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptIndentexpr as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_indk = xstrdup(p_indk.get());
            (*buf).b_p_script_ctx[kBufOptIndentkeys as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptIndentkeys as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_fp = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_fex = xstrdup(p_fex.get());
            (*buf).b_p_script_ctx[kBufOptFormatexpr as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptFormatexpr as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_sua = xstrdup(p_sua.get());
            (*buf).b_p_script_ctx[kBufOptSuffixesadd as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptSuffixesadd as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_keymap = xstrdup(p_keymap.get());
            (*buf).b_p_script_ctx[kBufOptKeymap as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptKeymap as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_kmap_state = ((*buf).b_kmap_state as c_int | KEYMAP_INIT) as int16_t;
            (*buf).b_p_iminsert = p_iminsert.get();
            (*buf).b_p_script_ctx[kBufOptIminsert as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptIminsert as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_imsearch = p_imsearch.get();
            (*buf).b_p_script_ctx[kBufOptImsearch as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptImsearch as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_ac = -1 as c_int;
            (*buf).b_p_ar = -1 as c_int;
            (*buf).b_p_fs = -1 as c_int;
            (*buf).b_p_ul = NO_LOCAL_UNDOLEVEL as OptInt;
            (*buf).b_p_bkc = empty_string_option.ptr() as *mut c_char;
            (*buf).b_bkc_flags = 0 as c_uint;
            (*buf).b_p_gefm = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_gp = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_mp = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_efm = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_ep = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_ffu = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_kp = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_path = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_tags = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_tc = empty_string_option.ptr() as *mut c_char;
            (*buf).b_tc_flags = 0 as c_uint;
            (*buf).b_p_def = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_inc = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_inex = xstrdup(p_inex.get());
            (*buf).b_p_script_ctx[kBufOptIncludeexpr as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptIncludeexpr as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_cot = empty_string_option.ptr() as *mut c_char;
            (*buf).b_cot_flags = 0 as c_uint;
            (*buf).b_p_dict = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_dia = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_tsr = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_tsrfu = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_qe = xstrdup(p_qe.get());
            (*buf).b_p_script_ctx[kBufOptQuoteescape as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptQuoteescape as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_udf = p_udf.get();
            (*buf).b_p_script_ctx[kBufOptUndofile as c_int as usize] = (*options.ptr())
                [*(&raw const buf_opt_idx as *const OptIndex)
                    .offset(kBufOptUndofile as c_int as isize) as usize]
                .script_ctx;
            (*buf).b_p_lw = empty_string_option.ptr() as *mut c_char;
            (*buf).b_p_menc = empty_string_option.ptr() as *mut c_char;
            if dont_do_help {
                (*buf).b_p_isk = save_p_isk;
                if !(*p_vts.ptr()).is_null()
                    && *p_vts.get() as c_int != NUL
                    && (*buf).b_p_vts_array.is_null()
                {
                    tabstop_set(p_vts.get(), &raw mut (*buf).b_p_vts_array);
                } else {
                    (*buf).b_p_vts_array = ::core::ptr::null_mut::<colnr_T>();
                }
            } else {
                (*buf).b_p_isk = xstrdup(p_isk.get());
                (*buf).b_p_script_ctx[kBufOptIskeyword as c_int as usize] = (*options.ptr())
                    [*(&raw const buf_opt_idx as *const OptIndex)
                        .offset(kBufOptIskeyword as c_int as isize) as usize]
                    .script_ctx;
                did_isk = true_0 != 0;
                (*buf).b_p_ts = p_ts.get();
                (*buf).b_p_script_ctx[kBufOptTabstop as c_int as usize] = (*options.ptr())
                    [*(&raw const buf_opt_idx as *const OptIndex)
                        .offset(kBufOptTabstop as c_int as isize) as usize]
                    .script_ctx;
                (*buf).b_p_vts = xstrdup(p_vts.get());
                (*buf).b_p_script_ctx[kBufOptVartabstop as c_int as usize] = (*options.ptr())
                    [*(&raw const buf_opt_idx as *const OptIndex)
                        .offset(kBufOptVartabstop as c_int as isize) as usize]
                    .script_ctx;
                if !(*p_vts.ptr()).is_null()
                    && *p_vts.get() as c_int != NUL
                    && (*buf).b_p_vts_array.is_null()
                {
                    tabstop_set(p_vts.get(), &raw mut (*buf).b_p_vts_array);
                } else {
                    (*buf).b_p_vts_array = ::core::ptr::null_mut::<colnr_T>();
                }
                (*buf).b_help = false_0 != 0;
                if *(*buf).b_p_bt.offset(0 as c_int as isize) as c_int == 'h' as c_int {
                    clear_string_option(&raw mut (*buf).b_p_bt);
                }
                (*buf).b_p_ma = p_ma.get();
                (*buf).b_p_script_ctx[kBufOptModifiable as c_int as usize] = (*options.ptr())
                    [*(&raw const buf_opt_idx as *const OptIndex)
                        .offset(kBufOptModifiable as c_int as isize) as usize]
                    .script_ctx;
            }
        }
        if should_copy {
            (*buf).b_p_initialized = true_0 != 0;
        }
    }
    check_buf_options(buf);
    if did_isk {
        buf_init_chartab(buf, false);
    }
}
pub unsafe extern "C" fn reset_modifiable() {
    (*curbuf.get()).b_p_ma = false_0;
    p_ma.set(false_0);
    change_option_default(
        kOptModifiable,
        OptVal {
            type_0: kOptValTypeBoolean,
            data: OptValData { boolean: kFalse },
        },
    );
}
pub unsafe extern "C" fn set_iminsert_global(mut buf: *mut buf_T) {
    p_iminsert.set((*buf).b_p_iminsert);
}
pub unsafe extern "C" fn set_imsearch_global(mut buf: *mut buf_T) {
    p_imsearch.set((*buf).b_p_imsearch);
}
static expand_option_idx: GlobalCell<OptIndex> = GlobalCell::new(kOptInvalid);
static expand_option_start_col: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
static expand_option_name: GlobalCell<[c_char; 5]> = GlobalCell::new([
    't' as c_char,
    '_' as c_char,
    NUL as c_char,
    NUL as c_char,
    NUL as c_char,
]);
static expand_option_flags: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
static expand_option_append: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub unsafe extern "C" fn set_context_in_set_cmd(
    mut xp: *mut expand_T,
    mut arg: *mut c_char,
    mut opt_flags: c_int,
) {
    expand_option_flags.set(opt_flags);
    (*xp).xp_context = EXPAND_SETTINGS as c_int;
    if *arg as c_int == NUL {
        (*xp).xp_pattern = arg;
        return;
    }
    let argend: *mut c_char = arg.offset(strlen(arg) as isize);
    let mut p: *mut c_char = argend.offset(-(1 as c_int as isize));
    if *p as c_int == ' ' as c_int && *p.offset(-(1 as c_int as isize)) as c_int != '\\' as c_int {
        (*xp).xp_pattern = p.offset(1 as c_int as isize);
        return;
    }
    while p > arg {
        let mut s: *mut c_char = p;
        if *p as c_int == ' ' as c_int || *p as c_int == ',' as c_int {
            while s > arg && *s.offset(-(1 as c_int as isize)) as c_int == '\\' as c_int {
                s = s.offset(-1);
            }
        }
        if *p as c_int == ' ' as c_int && p.offset_from(s) & 1 as isize == 0 as isize {
            p = p.offset(1);
            break;
        } else {
            p = p.offset(-1);
        }
    }
    if strncmp(p, b"no\0".as_ptr() as *const c_char, 2 as size_t) == 0 as c_int {
        (*xp).xp_context = EXPAND_BOOL_SETTINGS as c_int;
        (*xp).xp_prefix = XP_PREFIX_NO;
        p = p.offset(2 as c_int as isize);
    } else if strncmp(p, b"inv\0".as_ptr() as *const c_char, 3 as size_t) == 0 as c_int {
        (*xp).xp_context = EXPAND_BOOL_SETTINGS as c_int;
        (*xp).xp_prefix = XP_PREFIX_INV;
        p = p.offset(3 as c_int as isize);
    }
    (*xp).xp_pattern = p;
    arg = p;
    let mut nextchar: c_char = 0;
    let mut flags: uint32_t = 0 as uint32_t;
    let mut opt_idx: OptIndex = kOptAleph;
    let mut is_term_option: bool = false_0 != 0;
    if *arg as c_int == '<' as c_int {
        while *p as c_int != '>' as c_int {
            let c2rust_fresh10 = p;
            p = p.offset(1);
            if *c2rust_fresh10 as c_int == NUL {
                return;
            }
        }
        let mut key: c_int = get_special_key_code(arg.offset(1 as c_int as isize));
        if key == 0 as c_int {
            (*xp).xp_context = EXPAND_NOTHING as c_int;
            return;
        }
        p = p.offset(1);
        nextchar = *p;
        is_term_option = true_0 != 0;
        (*expand_option_name.ptr())[2 as c_int as usize] =
            (-key & 0xff as c_int) as uint8_t as c_char;
        (*expand_option_name.ptr())[3 as c_int as usize] =
            (-key as c_uint >> 8 as c_int & 0xff as c_uint) as uint8_t as c_char;
    } else if *p.offset(0 as c_int as isize) as c_int == 't' as c_int
        && *p.offset(1 as c_int as isize) as c_int == '_' as c_int
    {
        p = p.offset(2 as c_int as isize);
        if *p as c_int != NUL {
            p = p.offset(1);
        }
        if *p as c_int == NUL {
            return;
        }
        p = p.offset(1);
        nextchar = *p;
        is_term_option = true_0 != 0;
        (*expand_option_name.ptr())[2 as c_int as usize] = *p.offset(-2 as c_int as isize);
        (*expand_option_name.ptr())[3 as c_int as usize] = *p.offset(-1 as c_int as isize);
    } else {
        while *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
            || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
            || ascii_isdigit(*p as c_int) as c_int != 0
            || *p as c_int == '_' as c_int
            || *p as c_int == '*' as c_int
        {
            p = p.offset(1);
        }
        if *p as c_int == NUL {
            return;
        }
        nextchar = *p;
        opt_idx = find_option_len(arg, p.offset_from(arg) as size_t);
        if opt_idx as c_int == kOptInvalid as c_int || is_option_hidden(opt_idx) as c_int != 0 {
            (*xp).xp_context = EXPAND_NOTHING as c_int;
            return;
        }
        flags = (*options.ptr())[opt_idx as usize].flags;
        if option_has_type(opt_idx, kOptValTypeBoolean) {
            (*xp).xp_context = EXPAND_NOTHING as c_int;
            return;
        }
    }
    expand_option_append.set(false_0 != 0);
    let mut expand_option_subtract: bool = false_0 != 0;
    if (nextchar as c_int == '-' as c_int
        || nextchar as c_int == '+' as c_int
        || nextchar as c_int == '^' as c_int)
        && *p.offset(1 as c_int as isize) as c_int == '=' as c_int
    {
        if nextchar as c_int == '-' as c_int {
            expand_option_subtract = true_0 != 0;
        }
        if nextchar as c_int == '+' as c_int || nextchar as c_int == '^' as c_int {
            expand_option_append.set(true_0 != 0);
        }
        p = p.offset(1);
        nextchar = '=' as c_char;
    }
    if nextchar as c_int != '=' as c_int && nextchar as c_int != ':' as c_int
        || (*xp).xp_context == EXPAND_BOOL_SETTINGS as c_int
    {
        (*xp).xp_context = EXPAND_UNSUCCESSFUL as c_int;
        return;
    }
    if is_term_option {
        expand_option_idx.set(kOptInvalid);
    } else {
        expand_option_idx.set(opt_idx);
    }
    (*xp).xp_pattern = p.offset(1 as c_int as isize);
    expand_option_start_col.set(p.offset(1 as c_int as isize).offset_from((*xp).xp_line) as c_int);
    if (*options.ptr())[opt_idx as usize].var == p_syn.ptr() as *mut c_void {
        (*xp).xp_context = EXPAND_OWNSYNTAX as c_int;
        return;
    }
    if (*options.ptr())[opt_idx as usize].var == p_ft.ptr() as *mut c_void {
        (*xp).xp_context = EXPAND_FILETYPE as c_int;
        return;
    }
    if (*options.ptr())[opt_idx as usize].var == p_keymap.ptr() as *mut c_void {
        (*xp).xp_context = EXPAND_KEYMAP as c_int;
        return;
    }
    if expand_option_subtract {
        (*xp).xp_context = EXPAND_SETTING_SUBTRACT as c_int;
        return;
    } else if expand_option_idx.get() as c_int != kOptInvalid as c_int
        && (*options.ptr())[expand_option_idx.get() as usize]
            .opt_expand_cb
            .is_some()
    {
        (*xp).xp_context = EXPAND_STRING_SETTING as c_int;
    } else if *(*xp).xp_pattern as c_int == NUL {
        (*xp).xp_context = EXPAND_OLD_SETTING as c_int;
        return;
    } else {
        (*xp).xp_context = EXPAND_NOTHING as c_int;
    }
    if is_term_option as c_int != 0 || option_has_type(opt_idx, kOptValTypeNumber) as c_int != 0 {
        return;
    }
    if flags & kOptFlagExpand as c_int as uint32_t != 0 {
        p = (*options.ptr())[opt_idx as usize].var as *mut c_char;
        if p == p_bdir.ptr() as *mut c_char
            || p == p_dir.ptr() as *mut c_char
            || p == p_path.ptr() as *mut c_char
            || p == p_pp.ptr() as *mut c_char
            || p == p_rtp.ptr() as *mut c_char
            || p == p_cdpath.ptr() as *mut c_char
            || p == p_vdir.ptr() as *mut c_char
        {
            (*xp).xp_context = EXPAND_DIRECTORIES as c_int;
            if p == p_path.ptr() as *mut c_char || p == p_cdpath.ptr() as *mut c_char {
                (*xp).xp_backslash = XP_BS_THREE as c_int;
            } else {
                (*xp).xp_backslash = XP_BS_ONE as c_int;
            }
        } else {
            (*xp).xp_context = EXPAND_FILES as c_int;
            if p == p_tags.ptr() as *mut c_char {
                (*xp).xp_backslash = XP_BS_THREE as c_int;
            } else {
                (*xp).xp_backslash = XP_BS_ONE as c_int;
            }
        }
        if flags & kOptFlagComma as c_int as uint32_t != 0 {
            (*xp).xp_backslash |= XP_BS_COMMA as c_int;
        }
    }
    if flags & kOptFlagExpand as c_int as uint32_t != 0
        || flags & kOptFlagComma as c_int as uint32_t != 0
        || flags & kOptFlagColon as c_int as uint32_t != 0
    {
        p = argend.offset(-(1 as c_int as isize));
        while p > (*xp).xp_pattern {
            if *p as c_int == ' ' as c_int
                || *p as c_int == ',' as c_int
                || *p as c_int == ':' as c_int && flags & kOptFlagColon as c_int as uint32_t != 0
            {
                let mut s_0: *mut c_char = p;
                while s_0 > (*xp).xp_pattern
                    && *s_0.offset(-(1 as c_int as isize)) as c_int == '\\' as c_int
                {
                    s_0 = s_0.offset(-1);
                }
                if *p as c_int == ' ' as c_int
                    && ((*xp).xp_backslash & XP_BS_THREE as c_int != 0
                        && p.offset_from(s_0) < 3 as isize)
                    || *p as c_int == ',' as c_int
                        && flags & kOptFlagComma as c_int as uint32_t != 0
                        && p.offset_from(s_0) < 2 as isize
                    || *p as c_int == ':' as c_int
                        && flags & kOptFlagColon as c_int as uint32_t != 0
                {
                    (*xp).xp_pattern = p.offset(1 as c_int as isize);
                    break;
                }
            }
            p = p.offset(-1);
        }
    }
    if flags & kOptFlagFlagList as c_int as uint32_t != 0 {
        (*xp).xp_pattern = argend;
    }
    if (*options.ptr())[opt_idx as usize].var == p_sps.ptr() as *mut c_void {
        if strncmp(
            (*xp).xp_pattern,
            b"file:\0".as_ptr() as *const c_char,
            5 as size_t,
        ) == 0 as c_int
        {
            (*xp).xp_pattern = (*xp).xp_pattern.offset(5 as c_int as isize);
            return;
        } else if (*options.ptr())[expand_option_idx.get() as usize]
            .opt_expand_cb
            .is_some()
        {
            (*xp).xp_context = EXPAND_STRING_SETTING as c_int;
        }
    }
}
unsafe extern "C" fn match_str(
    str: *mut c_char,
    regmatch: *mut regmatch_T,
    matches: *mut *mut c_char,
    idx: c_int,
    test_only: bool,
    fuzzy: bool,
    fuzzystr: *const c_char,
    fuzmatch: *mut fuzmatch_str_T,
) -> bool {
    if !fuzzy {
        if vim_regexec(regmatch, str, 0 as colnr_T) {
            if !test_only {
                *matches.offset(idx as isize) = xstrdup(str);
            }
            return true_0 != 0;
        }
    } else {
        let score: c_int = fuzzy_match_str(str, fuzzystr);
        if score != FUZZY_SCORE_NONE as c_int {
            if !test_only {
                (*fuzmatch.offset(idx as isize)).idx = idx;
                (*fuzmatch.offset(idx as isize)).str = xstrdup(str);
                (*fuzmatch.offset(idx as isize)).score = score;
            }
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn ExpandSettings(
    mut xp: *mut expand_T,
    mut regmatch: *mut regmatch_T,
    mut fuzzystr: *mut c_char,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
    can_fuzzy: bool,
) -> c_int {
    let mut num_normal: c_int = 0 as c_int;
    let mut count: c_int = 0 as c_int;
    static names: GlobalCell<[*mut c_char; 1]> =
        GlobalCell::new([b"all\0".as_ptr() as *const c_char as *mut c_char]);
    let mut ic: c_int = (*regmatch).rm_ic as c_int;
    let mut fuzmatch: *mut fuzmatch_str_T = ::core::ptr::null_mut::<fuzmatch_str_T>();
    let fuzzy: bool = can_fuzzy as c_int != 0 && cmdline_fuzzy_complete(fuzzystr) as c_int != 0;
    let mut loop_0: c_int = 0 as c_int;
    while loop_0 <= 1 as c_int {
        (*regmatch).rm_ic = ic != 0;
        if (*xp).xp_context != EXPAND_BOOL_SETTINGS as c_int {
            let mut match_0: c_int = 0 as c_int;
            while match_0
                < ::core::mem::size_of::<[*mut c_char; 1]>()
                    .wrapping_div(::core::mem::size_of::<*mut c_char>())
                    .wrapping_div(
                        (::core::mem::size_of::<[*mut c_char; 1]>()
                            .wrapping_rem(::core::mem::size_of::<*mut c_char>())
                            == 0) as c_int as usize,
                    ) as c_int
            {
                if match_str(
                    (*names.ptr())[match_0 as usize] as *mut c_char,
                    regmatch,
                    *matches,
                    count,
                    loop_0 == 0 as c_int,
                    fuzzy,
                    fuzzystr,
                    fuzmatch,
                ) {
                    if loop_0 == 0 as c_int {
                        num_normal += 1;
                    } else {
                        count += 1;
                    }
                }
                match_0 += 1;
            }
        }
        let mut str: *mut c_char = ::core::ptr::null_mut::<c_char>();
        let mut opt_idx: OptIndex = kOptAleph;
        while (opt_idx as c_int) < kOptCount {
            str = (*options.ptr())[opt_idx as usize].fullname;
            if !is_option_hidden(opt_idx) {
                if !((*xp).xp_context == EXPAND_BOOL_SETTINGS as c_int
                    && !option_has_type(opt_idx, kOptValTypeBoolean))
                {
                    if match_str(
                        str,
                        regmatch,
                        *matches,
                        count,
                        loop_0 == 0 as c_int,
                        fuzzy,
                        fuzzystr,
                        fuzmatch,
                    ) {
                        if loop_0 == 0 as c_int {
                            num_normal += 1;
                        } else {
                            count += 1;
                        }
                    } else if !fuzzy
                        && !(*options.ptr())[opt_idx as usize].shortname.is_null()
                        && vim_regexec(
                            regmatch,
                            (*options.ptr())[opt_idx as usize].shortname,
                            0 as colnr_T,
                        ) as c_int
                            != 0
                    {
                        if loop_0 == 0 as c_int {
                            num_normal += 1;
                        } else {
                            let c2rust_fresh11 = count;
                            count = count + 1;
                            let c2rust_lvalue_ptr =
                                &raw mut *(*matches).offset(c2rust_fresh11 as isize);
                            *c2rust_lvalue_ptr = xstrdup(str);
                        }
                    }
                }
            }
            opt_idx += 1;
        }
        if loop_0 == 0 as c_int {
            if num_normal > 0 as c_int {
                *numMatches = num_normal;
            } else {
                return OK;
            }
            if !fuzzy {
                *matches = xmalloc(
                    (*numMatches as size_t).wrapping_mul(::core::mem::size_of::<*mut c_char>()),
                ) as *mut *mut c_char;
            } else {
                fuzmatch = xmalloc(
                    (*numMatches as size_t).wrapping_mul(::core::mem::size_of::<fuzmatch_str_T>()),
                ) as *mut fuzmatch_str_T;
            }
        }
        loop_0 += 1;
    }
    if fuzzy {
        fuzzymatches_to_strmatches(fuzmatch, matches, count, false_0 != 0);
    }
    return OK;
}
unsafe extern "C" fn escape_option_str_cmdline(mut var: *mut c_char) -> *mut c_char {
    let mut buf: *mut c_char = vim_strsave_escaped(var, escape_chars.get());
    return buf;
}
pub unsafe extern "C" fn ExpandOldSetting(
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    let mut var: *mut c_char = ::core::ptr::null_mut::<c_char>();
    *numMatches = 0 as c_int;
    *matches = xmalloc(::core::mem::size_of::<*mut c_char>()) as *mut *mut c_char;
    if expand_option_idx.get() as c_int == kOptInvalid as c_int {
        expand_option_idx.set(find_option(expand_option_name.ptr() as *mut c_char));
    }
    if expand_option_idx.get() as c_int != kOptInvalid as c_int {
        option_value2string(
            (options.ptr() as *mut vimoption_T).offset(expand_option_idx.get() as isize),
            expand_option_flags.get(),
        );
        var = NameBuff.ptr() as *mut c_char;
    } else {
        var = b"\0".as_ptr() as *const c_char as *mut c_char;
    }
    let mut buf: *mut c_char = escape_option_str_cmdline(var);
    *(*matches).offset(0 as c_int as isize) = buf;
    *numMatches = 1 as c_int;
    return OK;
}
pub unsafe extern "C" fn ExpandStringSetting(
    mut xp: *mut expand_T,
    mut regmatch: *mut regmatch_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    if expand_option_idx.get() as c_int == kOptInvalid as c_int
        || (*options.ptr())[expand_option_idx.get() as usize]
            .opt_expand_cb
            .is_none()
    {
        return FAIL;
    }
    let mut args: optexpand_T = optexpand_T {
        oe_varp: get_varp_scope(
            (options.ptr() as *mut vimoption_T).offset(expand_option_idx.get() as isize),
            expand_option_flags.get(),
        ) as *mut c_char,
        oe_idx: expand_option_idx.get(),
        oe_opt_value: ::core::ptr::null_mut::<c_char>(),
        oe_append: expand_option_append.get(),
        oe_include_orig_val: false,
        oe_regmatch: regmatch,
        oe_xp: xp,
        oe_set_arg: (*xp).xp_line.offset(expand_option_start_col.get() as isize),
    };
    args.oe_include_orig_val = !expand_option_append.get() && *args.oe_set_arg as c_int == NUL;
    option_value2string(
        (options.ptr() as *mut vimoption_T).offset(expand_option_idx.get() as isize),
        expand_option_flags.get(),
    );
    let mut var: *mut c_char = NameBuff.ptr() as *mut c_char;
    let mut buf: *mut c_char = escape_option_str_cmdline(var);
    args.oe_opt_value = buf;
    let mut num_ret: c_int =
        (*options.ptr())[expand_option_idx.get() as usize]
            .opt_expand_cb
            .expect("non-null function pointer")(&raw mut args, numMatches, matches);
    xfree(buf as *mut c_void);
    return num_ret;
}
pub unsafe extern "C" fn ExpandSettingSubtract(
    mut xp: *mut expand_T,
    mut regmatch: *mut regmatch_T,
    mut numMatches: *mut c_int,
    mut matches: *mut *mut *mut c_char,
) -> c_int {
    if expand_option_idx.get() as c_int == kOptInvalid as c_int {
        return ExpandOldSetting(numMatches, matches);
    }
    let mut option_val: *mut c_char = *(get_option_varp_scope_from(
        expand_option_idx.get(),
        expand_option_flags.get(),
        curbuf.get(),
        curwin.get(),
    ) as *mut *mut c_char);
    let mut option_flags: uint32_t = (*options.ptr())[expand_option_idx.get() as usize].flags;
    if option_has_type(expand_option_idx.get(), kOptValTypeNumber) {
        return ExpandOldSetting(numMatches, matches);
    } else if option_flags & kOptFlagComma as c_int as uint32_t != 0 {
        if *option_val as c_int == NUL {
            return FAIL;
        }
        let mut option_copy: *mut c_char = xstrdup(option_val);
        let mut next_val: *mut c_char = option_copy;
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<c_void>(),
        };
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<*mut c_char>() as c_int,
            10 as c_int,
        );
        loop {
            let mut item: *mut c_char = next_val;
            let mut comma: *mut c_char = vim_strchr(next_val, ',' as c_int);
            while !comma.is_null()
                && comma != next_val
                && *comma.offset(-(1 as c_int as isize)) as c_int == '\\' as c_int
            {
                comma = vim_strchr(comma.offset(1 as c_int as isize), ',' as c_int);
            }
            if !comma.is_null() {
                *comma = NUL as c_char;
                next_val = comma.offset(1 as c_int as isize);
            } else {
                next_val = ::core::ptr::null_mut::<c_char>();
            }
            if *item as c_int != NUL {
                if vim_regexec(regmatch, item, 0 as colnr_T) {
                    let mut buf: *mut c_char = escape_option_str_cmdline(item);
                    ga_grow(&raw mut ga, 1 as c_int);
                    *(ga.ga_data as *mut *mut c_char).offset(ga.ga_len as isize) = buf;
                    ga.ga_len += 1;
                }
            }
            if next_val.is_null() {
                break;
            }
        }
        xfree(option_copy as *mut c_void);
        *matches = ga.ga_data as *mut *mut c_char;
        *numMatches = ga.ga_len;
        return OK;
    } else if option_flags & kOptFlagFlagList as c_int as uint32_t != 0 {
        if *(*xp).xp_pattern as c_int != NUL {
            return FAIL;
        }
        let mut num_flags: size_t = strlen(option_val);
        if num_flags == 0 as size_t {
            return FAIL;
        }
        *matches = xmalloc(
            ::core::mem::size_of::<*mut c_char>().wrapping_mul(num_flags.wrapping_add(1 as size_t)),
        ) as *mut *mut c_char;
        let mut count: c_int = 0 as c_int;
        let c2rust_fresh12 = count;
        count = count + 1;
        let c2rust_lvalue_ptr = &raw mut *(*matches).offset(c2rust_fresh12 as isize);
        *c2rust_lvalue_ptr = xmemdupz(option_val as *const c_void, num_flags) as *mut c_char;
        if num_flags > 1 as size_t {
            let mut flag: *mut c_char = option_val;
            while *flag as c_int != NUL {
                let c2rust_fresh13 = count;
                count = count + 1;
                let c2rust_lvalue_ptr_0 = &raw mut *(*matches).offset(c2rust_fresh13 as isize);
                *c2rust_lvalue_ptr_0 = xmemdupz(flag as *const c_void, 1 as size_t) as *mut c_char;
                flag = flag.offset(1);
            }
        }
        *numMatches = count;
        return OK;
    }
    return ExpandOldSetting(numMatches, matches);
}
unsafe extern "C" fn option_value2string(mut opt: *mut vimoption_T, mut opt_flags: c_int) {
    let mut varp: *mut c_void = get_varp_scope(opt, opt_flags);
    '_c2rust_label: {
        if !varp.is_null() {
        } else {
            __assert_fail(
                b"varp != NULL\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                6126 as c_uint,
                b"void option_value2string(vimoption_T *, int)\0".as_ptr() as *const c_char,
            );
        }
    };
    if option_has_type(get_opt_idx(opt), kOptValTypeNumber) {
        let mut wc: OptInt = 0 as OptInt;
        if wc_use_keyname(varp, &raw mut wc) != 0 {
            xstrlcpy(
                NameBuff.ptr() as *mut c_char,
                get_special_key_name(wc as c_int, 0 as c_int),
                ::core::mem::size_of::<[c_char; 4096]>(),
            );
        } else if wc != 0 as OptInt {
            xstrlcpy(
                NameBuff.ptr() as *mut c_char,
                transchar(wc as c_int),
                ::core::mem::size_of::<[c_char; 4096]>(),
            );
        } else {
            snprintf(
                NameBuff.ptr() as *mut c_char,
                ::core::mem::size_of::<[c_char; 4096]>(),
                b"%ld\0".as_ptr() as *const c_char,
                *(varp as *mut OptInt),
            );
        }
    } else {
        varp = *(varp as *mut *mut c_char) as *mut c_void;
        if (*opt).flags & kOptFlagExpand as c_int as uint32_t != 0 {
            home_replace(
                ::core::ptr::null::<buf_T>(),
                varp as *const c_char,
                NameBuff.ptr() as *mut c_char,
                MAXPATHL as size_t,
                false_0 != 0,
            );
        } else {
            xstrlcpy(
                NameBuff.ptr() as *mut c_char,
                varp as *const c_char,
                MAXPATHL as size_t,
            );
        }
    };
}
unsafe extern "C" fn wc_use_keyname(mut varp: *const c_void, mut wcp: *mut OptInt) -> c_int {
    if varp as *mut OptInt == p_wc.ptr() || varp as *mut OptInt == p_wcm.ptr() {
        *wcp = *(varp as *mut OptInt);
        if *wcp < 0 as OptInt || find_special_key_in_table(*wcp as c_int) >= 0 as c_int {
            return true_0;
        }
    }
    return false_0;
}
pub unsafe extern "C" fn shortmess(mut x: c_int) -> bool {
    return !(*p_shm.ptr()).is_null()
        && (!vim_strchr(p_shm.get(), x).is_null()
            || !vim_strchr(p_shm.get(), 'a' as c_int).is_null() && {
                let mut c2rust_lvalue: [c_char; 5] = [
                    SHM_RO as c_int as c_char,
                    SHM_MOD as c_int as c_char,
                    SHM_LINES as c_int as c_char,
                    SHM_WRI as c_int as c_char,
                    0 as c_char,
                ];
                !vim_strchr(&raw mut c2rust_lvalue as *mut c_char, x).is_null()
            });
}
pub unsafe extern "C" fn vimrc_found(mut fname: *mut c_char, mut envname: *mut c_char) {
    if !fname.is_null() && !envname.is_null() {
        let mut p: *mut c_char = vim_getenv(envname);
        if p.is_null() {
            p = FullName_save(fname, false_0 != 0);
            if !p.is_null() {
                os_setenv(envname, p, 1 as c_int);
                xfree(p as *mut c_void);
            }
        } else {
            xfree(p as *mut c_void);
        }
    }
}
pub unsafe extern "C" fn option_was_set(mut opt_idx: OptIndex) -> bool {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                6204 as c_uint,
                b"_Bool option_was_set(OptIndex)\0".as_ptr() as *const c_char,
            );
        }
    };
    return (*options.ptr())[opt_idx as usize].flags & kOptFlagWasSet as c_int as uint32_t != 0;
}
pub unsafe extern "C" fn reset_option_was_set(mut opt_idx: OptIndex) {
    '_c2rust_label: {
        if opt_idx as c_int != kOptInvalid as c_int {
        } else {
            __assert_fail(
                b"opt_idx != kOptInvalid\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                6213 as c_uint,
                b"void reset_option_was_set(OptIndex)\0".as_ptr() as *const c_char,
            );
        }
    };
    (*options.ptr())[opt_idx as usize].flags = ((*options.ptr())[opt_idx as usize].flags as c_uint
        & !(kOptFlagWasSet as c_int as c_uint))
        as uint32_t;
}
pub unsafe extern "C" fn fill_culopt_flags(mut val: *mut c_char, mut wp: *mut win_T) -> c_int {
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut culopt_flags_new: uint8_t = 0 as uint8_t;
    if val.is_null() {
        p = (*wp).w_onebuf_opt.wo_culopt;
    } else {
        p = val;
    }
    while *p as c_int != NUL {
        if strncmp(p, b"line\0".as_ptr() as *const c_char, 4 as size_t) == 0 as c_int {
            p = p.offset(4 as c_int as isize);
            culopt_flags_new = (culopt_flags_new as c_int | kOptCuloptFlagLine as c_int) as uint8_t;
        } else if strncmp(p, b"both\0".as_ptr() as *const c_char, 4 as size_t) == 0 as c_int {
            p = p.offset(4 as c_int as isize);
            culopt_flags_new = (culopt_flags_new as c_int
                | (kOptCuloptFlagLine as c_int | kOptCuloptFlagNumber as c_int))
                as uint8_t;
        } else if strncmp(p, b"number\0".as_ptr() as *const c_char, 6 as size_t) == 0 as c_int {
            p = p.offset(6 as c_int as isize);
            culopt_flags_new =
                (culopt_flags_new as c_int | kOptCuloptFlagNumber as c_int) as uint8_t;
        } else if strncmp(p, b"screenline\0".as_ptr() as *const c_char, 10 as size_t) == 0 as c_int
        {
            p = p.offset(10 as c_int as isize);
            culopt_flags_new =
                (culopt_flags_new as c_int | kOptCuloptFlagScreenline as c_int) as uint8_t;
        }
        if *p as c_int != ',' as c_int && *p as c_int != NUL {
            return FAIL;
        }
        if *p as c_int == ',' as c_int {
            p = p.offset(1);
        }
    }
    if culopt_flags_new as c_int & kOptCuloptFlagLine as c_int != 0
        && culopt_flags_new as c_int & kOptCuloptFlagScreenline as c_int != 0
    {
        return FAIL;
    }
    (*wp).w_p_culopt_flags = culopt_flags_new;
    return OK;
}
pub unsafe extern "C" fn magic_isset() -> bool {
    match magic_overruled.get() as c_uint {
        1 => return true_0 != 0,
        2 => return false_0 != 0,
        0 | _ => {}
    }
    return p_magic.get() != 0;
}
pub unsafe extern "C" fn option_set_callback_func(
    mut optval: *mut c_char,
    mut optcb: *mut Callback,
) -> c_int {
    if optval.is_null() || *optval as c_int == NUL {
        callback_free(optcb);
        return OK;
    }
    let mut tv: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
    if *optval as c_int == '{' as c_int
        || strncmp(
            optval,
            b"function(\0".as_ptr() as *const c_char,
            9 as size_t,
        ) == 0 as c_int
        || strncmp(optval, b"funcref(\0".as_ptr() as *const c_char, 8 as size_t) == 0 as c_int
    {
        tv = eval_expr(optval, ::core::ptr::null_mut::<exarg_T>());
        if tv.is_null() {
            return FAIL;
        }
    } else {
        tv = xcalloc(1 as size_t, ::core::mem::size_of::<typval_T>()) as *mut typval_T;
        (*tv).v_type = VAR_STRING;
        (*tv).vval.v_string = xstrdup(optval);
    }
    let mut cb: Callback = Callback {
        data: Callback_data {
            funcref: ::core::ptr::null_mut::<c_char>(),
        },
        type_0: kCallbackNone,
    };
    if !callback_from_typval(&raw mut cb, tv)
        || cb.type_0 as c_uint == kCallbackNone as c_int as c_uint
    {
        tv_free(tv);
        return FAIL;
    }
    callback_free(optcb);
    *optcb = cb;
    tv_free(tv);
    return OK;
}
unsafe extern "C" fn didset_options_sctx(mut opt_flags: c_int, mut buf: *mut c_int) {
    let mut i: c_int = 0 as c_int;
    while *buf.offset(i as isize) != kOptInvalid as c_int {
        set_option_sctx(
            *buf.offset(i as isize) as OptIndex,
            opt_flags,
            current_sctx.get(),
        );
        i += 1;
    }
}
pub unsafe extern "C" fn can_bs(mut what: c_int) -> bool {
    if what == BS_START && bt_prompt(curbuf.get()) as c_int != 0 {
        return false_0 != 0;
    }
    if *p_bs.get() as c_int == '2' as c_int {
        return what != BS_NOSTOP;
    }
    return !vim_strchr(p_bs.get(), what).is_null();
}
pub unsafe extern "C" fn get_bkc_flags(mut buf: *mut buf_T) -> c_uint {
    return if (*buf).b_bkc_flags != 0 {
        (*buf).b_bkc_flags
    } else {
        bkc_flags.get()
    };
}
pub unsafe extern "C" fn get_flp_value(mut buf: *mut buf_T) -> *mut c_char {
    if (*buf).b_p_flp.is_null() || *(*buf).b_p_flp as c_int == NUL {
        return p_flp.get();
    }
    return (*buf).b_p_flp;
}
pub unsafe extern "C" fn get_ve_flags(mut wp: *mut win_T) -> c_uint {
    return (if (*wp).w_onebuf_opt.wo_ve_flags != 0 {
        (*wp).w_onebuf_opt.wo_ve_flags
    } else {
        ve_flags.get()
    }) & !((kOptVeFlagNone as c_int | kOptVeFlagNoneU as c_int) as c_uint);
}
pub unsafe extern "C" fn get_showbreak_value(win: *mut win_T) -> *mut c_char {
    if (*win).w_onebuf_opt.wo_sbr.is_null() || *(*win).w_onebuf_opt.wo_sbr as c_int == NUL {
        return p_sbr.get();
    }
    if strcmp(
        (*win).w_onebuf_opt.wo_sbr,
        b"NONE\0".as_ptr() as *const c_char,
    ) == 0 as c_int
    {
        return empty_string_option.ptr() as *mut c_char;
    }
    return (*win).w_onebuf_opt.wo_sbr;
}
pub unsafe extern "C" fn get_fileformat(mut buf: *const buf_T) -> c_int {
    let mut c: c_int = *(*buf).b_p_ff as c_uchar as c_int;
    if (*buf).b_p_bin != 0 || c == 'u' as c_int {
        return EOL_UNIX;
    }
    if c == 'm' as c_int {
        return EOL_MAC;
    }
    return EOL_DOS;
}
pub unsafe extern "C" fn get_fileformat_force(
    mut buf: *const buf_T,
    mut eap: *const exarg_T,
) -> c_int {
    let mut c: c_int = 0;
    if !eap.is_null() && (*eap).force_ff != 0 as c_int {
        c = (*eap).force_ff;
    } else {
        if if !eap.is_null() && (*eap).force_bin != 0 as c_int {
            ((*eap).force_bin == FORCE_BIN) as c_int
        } else {
            (*buf).b_p_bin
        } != 0
        {
            return EOL_UNIX;
        }
        c = *(*buf).b_p_ff as c_uchar as c_int;
    }
    if c == 'u' as c_int {
        return EOL_UNIX;
    }
    if c == 'm' as c_int {
        return EOL_MAC;
    }
    return EOL_DOS;
}
pub unsafe extern "C" fn default_fileformat() -> c_int {
    match *p_ffs.get() as c_int {
        109 => return EOL_MAC,
        100 => return EOL_DOS,
        _ => {}
    }
    return EOL_UNIX;
}
pub unsafe extern "C" fn set_fileformat(mut eol_style: c_int, mut opt_flags: c_int) {
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    match eol_style {
        EOL_UNIX => {
            p = b"unix\0".as_ptr() as *const c_char as *mut c_char;
        }
        EOL_MAC => {
            p = b"mac\0".as_ptr() as *const c_char as *mut c_char;
        }
        EOL_DOS => {
            p = b"dos\0".as_ptr() as *const c_char as *mut c_char;
        }
        _ => {}
    }
    if !p.is_null() {
        set_option_direct(
            kOptFileformat,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(p),
                },
            },
            opt_flags,
            0 as scid_T,
        );
    }
    redraw_buf_status_later(curbuf.get());
    redraw_tabline.set(true_0 != 0);
    need_maketitle.set(true_0 != 0);
}
pub unsafe extern "C" fn skip_to_option_part(mut p: *const c_char) -> *mut c_char {
    if *p as c_int == ',' as c_int {
        p = p.offset(1);
    }
    while *p as c_int == ' ' as c_int {
        p = p.offset(1);
    }
    return p as *mut c_char;
}
pub unsafe extern "C" fn copy_option_part(
    mut option: *mut *mut c_char,
    mut buf: *mut c_char,
    mut maxlen: size_t,
    mut sep_chars: *mut c_char,
) -> size_t {
    let mut len: size_t = 0 as size_t;
    let mut p: *mut c_char = *option;
    if *p as c_int == '.' as c_int {
        let c2rust_fresh7 = p;
        p = p.offset(1);
        let c2rust_fresh8 = len;
        len = len.wrapping_add(1);
        *buf.offset(c2rust_fresh8 as isize) = *c2rust_fresh7;
    }
    while *p as c_int != NUL && vim_strchr(sep_chars, *p as uint8_t as c_int).is_null() {
        if *p.offset(0 as c_int as isize) as c_int == '\\' as c_int
            && !vim_strchr(
                sep_chars,
                *p.offset(1 as c_int as isize) as uint8_t as c_int,
            )
            .is_null()
        {
            p = p.offset(1);
        }
        if len < maxlen.wrapping_sub(1 as size_t) {
            let c2rust_fresh9 = len;
            len = len.wrapping_add(1);
            *buf.offset(c2rust_fresh9 as isize) = *p;
        }
        p = p.offset(1);
    }
    *buf.offset(len as isize) = NUL as c_char;
    if *p as c_int != NUL && *p as c_int != ',' as c_int {
        p = p.offset(1);
    }
    p = skip_to_option_part(p);
    *option = p;
    return len;
}
pub unsafe extern "C" fn csh_like_shell() -> c_int {
    return !strstr(path_tail(p_sh.get()), b"csh\0".as_ptr() as *const c_char).is_null() as c_int;
}
pub unsafe extern "C" fn fish_like_shell() -> bool {
    return !strstr(path_tail(p_sh.get()), b"fish\0".as_ptr() as *const c_char).is_null();
}
pub unsafe extern "C" fn get_winbuf_options(bufopt: c_int) -> *mut dict_T {
    let d: *mut dict_T = tv_dict_alloc();
    let mut opt_idx: OptIndex = kOptAleph;
    while (opt_idx as c_int) < kOptCount {
        let mut opt: *mut vimoption_T =
            (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
        if bufopt != 0 && option_has_scope(opt_idx, kOptScopeBuf) as c_int != 0
            || bufopt == 0 && option_has_scope(opt_idx, kOptScopeWin) as c_int != 0
        {
            let mut varp: *mut c_void = get_varp(opt);
            if !varp.is_null() {
                let mut opt_tv: typval_T =
                    optval_as_tv(optval_from_varp(opt_idx, varp), true_0 != 0);
                tv_dict_add_tv(d, (*opt).fullname, strlen((*opt).fullname), &raw mut opt_tv);
            }
        }
        opt_idx += 1;
    }
    return d;
}
pub unsafe extern "C" fn get_scrolloff_value(mut wp: *mut win_T) -> int64_t {
    if State.get() & MODE_TERMINAL as c_int != 0 && !(*(*wp).w_buffer).terminal.is_null() {
        return 0 as int64_t;
    }
    return if (*wp).w_onebuf_opt.wo_so < 0 as OptInt {
        p_so.get() as int64_t
    } else {
        (*wp).w_onebuf_opt.wo_so as int64_t
    };
}
pub unsafe extern "C" fn get_sidescrolloff_value(mut wp: *mut win_T) -> int64_t {
    return if (*wp).w_onebuf_opt.wo_siso < 0 as OptInt {
        p_siso.get() as int64_t
    } else {
        (*wp).w_onebuf_opt.wo_siso as int64_t
    };
}
pub unsafe extern "C" fn get_vimoption(
    mut name: String_0,
    mut opt_flags: c_int,
    mut buf: *mut buf_T,
    mut win: *mut win_T,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut opt_idx: OptIndex = find_option_len(name.data, name.size);
    if !(opt_idx as c_int != kOptInvalid as c_int) {
        api_err_invalid(
            err,
            b"option (not found)\0".as_ptr() as *const c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
    }
    return vimoption2dict(
        (options.ptr() as *mut vimoption_T).offset(opt_idx as isize),
        opt_flags,
        buf,
        win,
        arena,
    );
}
pub unsafe extern "C" fn get_all_vimoptions(mut arena: *mut Arena) -> Dict {
    let mut retval: Dict = arena_dict(arena, kOptCount as size_t);
    let mut opt_idx: OptIndex = kOptAleph;
    while (opt_idx as c_int) < kOptCount {
        let mut opt_dict: Dict = vimoption2dict(
            (options.ptr() as *mut vimoption_T).offset(opt_idx as isize),
            OPT_GLOBAL as c_int,
            curbuf.get(),
            curwin.get(),
            arena,
        );
        let c2rust_fresh27 = retval.size;
        retval.size = retval.size.wrapping_add(1);
        *retval.items.offset(c2rust_fresh27 as isize) = key_value_pair {
            key: cstr_as_string((*options.ptr())[opt_idx as usize].fullname),
            value: object {
                type_0: kObjectTypeDict,
                data: object_data { dict: opt_dict },
            },
        };
        opt_idx += 1;
    }
    return retval;
}
unsafe extern "C" fn vimoption2dict(
    mut opt: *mut vimoption_T,
    mut opt_flags: c_int,
    mut buf: *mut buf_T,
    mut win: *mut win_T,
    mut arena: *mut Arena,
) -> Dict {
    let mut opt_idx: OptIndex = get_opt_idx(opt);
    let mut dict: Dict = arena_dict(arena, 13 as size_t);
    let c2rust_fresh14 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh14 as isize) = key_value_pair {
        key: cstr_as_string(b"name\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeString,
            data: object_data {
                string: cstr_as_string((*opt).fullname),
            },
        },
    };
    let c2rust_fresh15 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh15 as isize) = key_value_pair {
        key: cstr_as_string(b"shortname\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeString,
            data: object_data {
                string: cstr_as_string((*opt).shortname),
            },
        },
    };
    let mut scope: *const c_char = ::core::ptr::null::<c_char>();
    if option_has_scope(opt_idx, kOptScopeBuf) {
        scope = b"buf\0".as_ptr() as *const c_char;
    } else if option_has_scope(opt_idx, kOptScopeWin) {
        scope = b"win\0".as_ptr() as *const c_char;
    } else {
        scope = b"global\0".as_ptr() as *const c_char;
    }
    let c2rust_fresh16 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh16 as isize) = key_value_pair {
        key: cstr_as_string(b"scope\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeString,
            data: object_data {
                string: cstr_as_string(scope),
            },
        },
    };
    let c2rust_fresh17 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh17 as isize) = key_value_pair {
        key: cstr_as_string(b"global_local\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: object_data {
                boolean: option_is_global_local(opt_idx),
            },
        },
    };
    let c2rust_fresh18 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh18 as isize) = key_value_pair {
        key: cstr_as_string(b"commalist\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: object_data {
                boolean: (*opt).flags & kOptFlagComma as c_int as uint32_t != 0,
            },
        },
    };
    let c2rust_fresh19 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh19 as isize) = key_value_pair {
        key: cstr_as_string(b"flaglist\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: object_data {
                boolean: (*opt).flags & kOptFlagFlagList as c_int as uint32_t != 0,
            },
        },
    };
    let c2rust_fresh20 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh20 as isize) = key_value_pair {
        key: cstr_as_string(b"was_set\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: object_data {
                boolean: (*opt).flags & kOptFlagWasSet as c_int as uint32_t != 0,
            },
        },
    };
    let mut script_ctx: sctx_T = sctx_T {
        sc_sid: 0 as scid_T,
        sc_seq: 0,
        sc_lnum: 0,
        sc_chan: 0,
    };
    if opt_flags == OPT_GLOBAL as c_int {
        script_ctx = (*opt).script_ctx;
    } else {
        if option_has_scope(opt_idx, kOptScopeBuf) {
            script_ctx =
                (*buf).b_p_script_ctx[(*opt).scope_idx[kOptScopeBuf as c_int as usize] as usize];
        }
        if option_has_scope(opt_idx, kOptScopeWin) {
            script_ctx = (*win).w_onebuf_opt.wo_script_ctx
                [(*opt).scope_idx[kOptScopeWin as c_int as usize] as usize];
        }
        if opt_flags != OPT_LOCAL as c_int && script_ctx.sc_sid == 0 as c_int {
            script_ctx = (*opt).script_ctx;
        }
    }
    let c2rust_fresh21 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh21 as isize) = key_value_pair {
        key: cstr_as_string(b"last_set_sid\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: object_data {
                integer: script_ctx.sc_sid as Integer,
            },
        },
    };
    let c2rust_fresh22 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh22 as isize) = key_value_pair {
        key: cstr_as_string(b"last_set_linenr\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: object_data {
                integer: script_ctx.sc_lnum as Integer,
            },
        },
    };
    let c2rust_fresh23 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh23 as isize) = key_value_pair {
        key: cstr_as_string(b"last_set_chan\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: object_data {
                integer: script_ctx.sc_chan as int64_t,
            },
        },
    };
    let c2rust_fresh24 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh24 as isize) = key_value_pair {
        key: cstr_as_string(b"type\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeString,
            data: object_data {
                string: cstr_as_string(optval_type_get_name(option_get_type(get_opt_idx(opt)))),
            },
        },
    };
    let c2rust_fresh25 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh25 as isize) = key_value_pair {
        key: cstr_as_string(b"default\0".as_ptr() as *const c_char),
        value: optval_as_object((*opt).def_val),
    };
    let c2rust_fresh26 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh26 as isize) = key_value_pair {
        key: cstr_as_string(b"allows_duplicates\0".as_ptr() as *const c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: object_data {
                boolean: (*opt).flags & kOptFlagNoDup as c_int as uint32_t == 0,
            },
        },
    };
    return dict;
}
pub const INT_MIN: c_int = -INT_MAX - 1 as c_int;
pub const INT_MAX: c_int = __INT_MAX__;
pub const true_0: c_int = 1 as c_int;
pub const false_0: c_int = 0 as c_int;
pub const PROJECT_NAME: [c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [c_char; 5]>(*b"nvim\0") };
pub const __INT_MAX__: c_int = 2147483647 as c_int;
