#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::api::private::helpers::{api_clear_error, api_free_object, cstr_as_string};
use crate::src::nvim::arglist::get_arglist_name;
use crate::src::nvim::ascii::{ascii_isdigit, ascii_isspace, ascii_iswhite};
use crate::src::nvim::autocmd::{
    expand_get_augroup_name, expand_get_event_name, set_context_in_autocmd,
};
use crate::src::nvim::buffer::ExpandBufnames;
use crate::src::nvim::charset::{
    backslash_halve_save, ptr2cells, rem_backslash, skipdigits, skiptowhite, skipwhite, transchar,
    transchar_byte, vim_isIDc, vim_isfilec_or_wc, vim_strsize,
};
use crate::src::nvim::cmdhist::get_history_arg;
use crate::src::nvim::drawscreen::{redraw_statuslines, update_screen, win_redraw_last_status};
use crate::src::nvim::eval::funcs::{get_expr_name, get_function_name};
use crate::src::nvim::eval::typval::{
    tv_check_for_string_arg, tv_clear, tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str,
    tv_dict_alloc_ret, tv_get_number_chk, tv_get_string, tv_list_alloc, tv_list_alloc_ret,
    tv_list_append_string, tv_list_unref,
};
use crate::src::nvim::eval::userfunc::get_user_func_name;
use crate::src::nvim::eval::vars::get_user_var_name;
use crate::src::nvim::eval::{call_func_retlist, call_func_retstr, set_context_for_expression};
use crate::src::nvim::ex_cmds::skip_vimgrep_pat;
use crate::src::nvim::ex_docmd::{
    ends_excmd, excmd_get_argt, excmd_get_cmdidx, expand_argopt, expand_findfunc, find_nextcmd,
    get_command_name, set_no_hlsearch, skip_cmd_arg, skip_range,
};
use crate::src::nvim::ex_getln::{
    cmd_screencol, cursorcmd, escape_fname, get_cmdline_info, get_cmdline_last_prompt_id,
    parse_pattern_and_range, put_on_cmdline, realloc_cmdbuff, redrawcmd, tilde_replace,
    vim_strsave_fnameescape,
};
use crate::src::nvim::fuzzy::{fuzzy_match_str, fuzzymatches_to_strmatches};
use crate::src::nvim::garray::{ga_append, ga_clear_strings, ga_concat_len, ga_grow, ga_init};
use crate::src::nvim::getchar::{beep_flush, char_avail, vpeekc};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{grid_line_fill, grid_line_flush, grid_line_puts, grid_line_start};
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::hashtab::{hash_add_item, hash_clear, hash_hash, hash_init, hash_lookup};
use crate::src::nvim::help::{cleanup_help_tags, find_help_tags};
use crate::src::nvim::highlight::win_hl_attr;
use crate::src::nvim::highlight_group::{
    HLF_D, HLF_NONE, HLF_T, HLF_WM, get_highlight_name, set_context_in_highlight_cmd,
};
use crate::src::nvim::insexpand::find_word_end;
use crate::src::nvim::keycodes::{K_DOWN, K_KENTER, K_LEFT, K_RIGHT, K_UP};
use crate::src::nvim::lua::executor::{
    nlua_call_user_expand_func, nlua_exec, nlua_expand_get_matches, nlua_expand_pat,
};
use crate::src::nvim::main::pum_want;
use crate::src::nvim::main::{
    Columns, KeyTyped, NameBuff, RedrawingDisabled, Rows, cmd_silent, cmdline_row, cmdline_win,
    curbuf, current_sctx, curwin, default_gridview, e_invarg, e_invarg2, e_nomatch2, e_toomany,
    emsg_off, got_int, hl_attr_active, lastwin, msg_col, msg_didany, msg_grid_adj, msg_row,
    msg_scrolled, msg_silent, p_fic, p_ic, p_ls, p_scs, p_wc, p_wic, p_wmh, p_wmnu, save_p_ls,
    save_p_wmh, search_first_line, search_last_line, topframe, wild_menu_showing, wop_flags,
};
use crate::src::nvim::mapping::{ExpandMappings, set_context_in_map_cmd};
use crate::src::nvim::mbyte::{mb_tolower, utf_head_off, utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memline::{ml_get, ml_get_len};
use crate::src::nvim::memory::{xfree, xmalloc, xmemcpyz, xmemdupz, xstpcpy, xstrdup};
use crate::src::nvim::menu::{
    get_menu_name, get_menu_names, menu_is_separator, set_context_in_menu_cmd,
};
use crate::src::nvim::message::{
    emsg, msg_advance, msg_clr_eos, msg_ext_set_kind, msg_outtrans, msg_outtrans_long, msg_putchar,
    msg_puts, msg_puts_hl, msg_scroll_up, msg_start, semsg,
};
use crate::src::nvim::option::{
    ExpandOldSetting, ExpandSettingSubtract, ExpandSettings, ExpandStringSetting, copy_option_part,
    csh_like_shell, get_findfunc, magic_isset, set_context_in_set_cmd,
};
use crate::src::nvim::options::{
    kOptBoFlagWildmode, kOptWopFlagExacttext, kOptWopFlagFuzzy, kOptWopFlagPum, kOptWopFlagTagfile,
};
use crate::src::nvim::os::env::{expand_env_save_opt, get_env_name, home_replace, vim_getenv};
use crate::src::nvim::os::fs::os_isdir;
use crate::src::nvim::os::lang::{get_lang_arg, get_locales};
use crate::src::nvim::os::libc::{
    gettext, memmove, qsort, snprintf, strchr, strcmp, strcpy, strlen, strncmp, strncpy,
};
use crate::src::nvim::os::users::{UserMatch, get_users, match_user};
use crate::src::nvim::path::{
    FreeWild, after_pathsep, expand_wildcards, expand_wildcards_eval, match_suffix,
    path_is_absolute, path_tail, vim_ispathsep,
};
use crate::src::nvim::popupmenu::{
    pum_clear, pum_display, pum_get_height, pum_undisplay, pum_visible,
};
use crate::src::nvim::pos::ltoreq;
use crate::src::nvim::profile::{get_profile_name, set_context_in_profile_cmd};
use crate::src::nvim::regexp::{RE_LAST, RE_MAGIC, RE_STRING, skip_regexp};
use crate::src::nvim::runtime::{
    DIP_OPT, DIP_START, ExpandPackAddDir, ExpandRTDir, expand_runtime_cmd, script_items,
    set_context_in_runtime_cmd,
};
use crate::src::nvim::search::{
    BACKWARD, FORWARD, SEARCH_NFMSG, SEARCH_NOOF, SEARCH_OPT, SEARCH_PEEK, SEARCH_START,
    ignorecase, pat_has_uppercase, searchit,
};
use crate::src::nvim::sign::{get_sign_name, set_context_in_sign_cmd};
use crate::src::nvim::statusline::fillchar_status;
use crate::src::nvim::strings::{
    sort_strings, strcase_save, vim_strchr, vim_strsave_escaped, xstrnsave,
};
use crate::src::nvim::syntax::{
    get_syntax_name, get_syntime_arg, reset_expand_highlight, set_context_in_echohl_cmd,
    set_context_in_syntax_cmd,
};
use crate::src::nvim::tag::expand_tags;
use crate::src::nvim::types::api::kErrorTypeNone;
use crate::src::nvim::types::ui::{kUICmdline, kUIMessages, kUIPopupmenu, kUIWildmenu};
use crate::src::nvim::types::{
    Arena, Array, BoolVarValue, CMD_index, CmdlineInfo, CompleteListItemGetter, Direction, Error,
    EvalFuncData, LuaRetMode, Object, OptInt, ScopeType, SpecialVarValue, VarLockStatus, VarType,
    buf_T, cmd_addr_T, cmdidx_T, colnr_T, dict_T, exarg_T, expand_T, fuzmatch_str_T, garray_T,
    hashtab_T, hlf_T, kObjectTypeArray, kObjectTypeString, list_T, listitem_T, pos_T, ptrdiff_t,
    pumitem_T, regmatch_T, regprog_T, scriptitem_T, sctx_T, size_t, ssize_t, typval_T,
    typval_vval_union, uint32_t, varnumber_T, xp_prefix_T,
};
use crate::src::nvim::ui::{ui_flush, ui_has, vim_beep};
use crate::src::nvim::usercmd::{
    cmdcomplete_str_to_type, cmdcomplete_type_to_str, find_ucmd, get_user_cmd_addr_type,
    get_user_cmd_complete, get_user_cmd_flags, get_user_cmd_nargs, get_user_commands,
    set_context_in_user_cmd, set_context_in_user_cmdarg,
};
use crate::src::nvim::window::{global_stl_height, last_status};
use core::ffi::CStr;

// The carve of the transpiled module; see each child's docs.
mod cmdidx;
pub(crate) use self::cmdidx::*;
mod escape;
pub use self::escape::*;
mod expandone;
pub use self::expandone::*;
mod pum;
pub use self::pum::*;
mod showmatch;
pub use self::showmatch::*;
mod context;
pub use self::context::*;
mod cmdname;
pub use self::cmdname::*;
mod generate;
pub(crate) use self::generate::*;
mod fromcontext;
pub use self::fromcontext::*;
mod userfunc;
pub use self::userfunc::*;
mod wildkey;
pub(crate) use self::wildkey::*;
mod eval;
pub use self::eval::*;
mod bufpat;
pub(crate) use self::bufpat::*;
unsafe extern "C" {
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
    fn vim_regexec_nl(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T)
    -> bool;
}
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
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
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_int;
pub const XP_BS_COMMA: C2Rust_Unnamed_13 = 4;
pub const XP_BS_THREE: C2Rust_Unnamed_13 = 2;
pub const XP_BS_ONE: C2Rust_Unnamed_13 = 1;
pub const XP_BS_NONE: C2Rust_Unnamed_13 = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_14 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_14 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_14 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_14 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_14 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_14 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_14 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_14 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_14 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_14 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_14 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_14 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_14 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_14 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_14 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_14 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_14 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_14 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_14 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_14 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_14 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_14 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_14 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_14 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_14 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_14 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_14 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_14 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_14 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_14 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_14 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_14 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_14 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_14 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_14 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_14 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_14 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_14 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_14 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_14 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_14 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_14 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_14 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_14 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_14 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_14 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_14 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_14 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_14 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_14 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_14 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_14 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_14 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_14 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_14 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_14 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_14 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_14 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_14 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_14 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_14 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_14 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_14 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_14 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_14 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_14 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_14 = -2;
pub const ADDR_LINES: cmd_addr_T = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_int;
pub const WILD_PUM_WANT: C2Rust_Unnamed_18 = 13;
pub const WILD_PAGEDOWN: C2Rust_Unnamed_18 = 12;
pub const WILD_PAGEUP: C2Rust_Unnamed_18 = 11;
pub const WILD_APPLY: C2Rust_Unnamed_18 = 10;
pub const WILD_CANCEL: C2Rust_Unnamed_18 = 9;
pub const WILD_ALL_KEEP: C2Rust_Unnamed_18 = 8;
pub const WILD_LONGEST: C2Rust_Unnamed_18 = 7;
pub const WILD_ALL: C2Rust_Unnamed_18 = 6;
pub const WILD_PREV: C2Rust_Unnamed_18 = 5;
pub const WILD_NEXT: C2Rust_Unnamed_18 = 4;
pub const WILD_EXPAND_FREE: C2Rust_Unnamed_18 = 2;
pub const WILD_FREE: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_int;
pub const WILD_FUNC_TRIGGER: C2Rust_Unnamed_19 = 65536;
pub const WILD_MAY_EXPAND_PATTERN: C2Rust_Unnamed_19 = 32768;
pub const WILD_NOSELECT: C2Rust_Unnamed_19 = 16384;
pub const BUF_DIFF_FILTER: C2Rust_Unnamed_19 = 8192;
pub const WILD_NOERROR: C2Rust_Unnamed_19 = 2048;
pub const WILD_ALLLINKS: C2Rust_Unnamed_19 = 512;
pub const WILD_ICASE: C2Rust_Unnamed_19 = 256;
pub const WILD_ESCAPE: C2Rust_Unnamed_19 = 128;
pub const WILD_SILENT: C2Rust_Unnamed_19 = 64;
pub const WILD_KEEP_ALL: C2Rust_Unnamed_19 = 32;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_19 = 16;
pub const WILD_NO_BEEP: C2Rust_Unnamed_19 = 8;
pub const WILD_USE_NL: C2Rust_Unnamed_19 = 4;
pub const WILD_HOME_REPLACE: C2Rust_Unnamed_19 = 2;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_19 = 1;
pub const VSE_NONE: C2Rust_Unnamed_24 = 0;
pub const VSE_BUFFER: C2Rust_Unnamed_24 = 2;
pub const VSE_SHELL: C2Rust_Unnamed_24 = 1;
pub const kRetObject: LuaRetMode = 0;
pub const EXP_BREAKPT_DEL: C2Rust_Unnamed_20 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const EXP_PROFDEL: C2Rust_Unnamed_20 = 2;
pub const EXP_BREAKPT_ADD: C2Rust_Unnamed_20 = 0;
pub const EXP_FILETYPECMD_ONOFF: C2Rust_Unnamed_21 = 3;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const EXP_FILETYPECMD_INDENT: C2Rust_Unnamed_21 = 2;
pub const EXP_FILETYPECMD_PLUGIN: C2Rust_Unnamed_21 = 1;
pub const EXP_FILETYPECMD_ALL: C2Rust_Unnamed_21 = 0;
pub const FUZZY_SCORE_NONE: C2Rust_Unnamed_25 = -2147483648;
pub const TAG_MANY: C2Rust_Unnamed_32 = 300;
pub const EW_DIR: C2Rust_Unnamed_28 = 1;
pub const EW_ALLLINKS: C2Rust_Unnamed_28 = 4096;
pub const EW_NOERROR: C2Rust_Unnamed_28 = 512;
pub const EW_SILENT: C2Rust_Unnamed_28 = 32;
pub const EW_KEEPALL: C2Rust_Unnamed_28 = 16;
pub const EW_ADDSLASH: C2Rust_Unnamed_28 = 8;
pub const EW_NOTFOUND: C2Rust_Unnamed_28 = 4;
pub const EW_SHELLCMD: C2Rust_Unnamed_28 = 8192;
pub const EW_EXEC: C2Rust_Unnamed_28 = 64;
pub const EW_FILE: C2Rust_Unnamed_28 = 2;
pub const EW_ICASE: C2Rust_Unnamed_28 = 256;
pub const EW_CDPATH: C2Rust_Unnamed_28 = 131072;
pub const EW_PATH: C2Rust_Unnamed_28 = 128;
pub const MB_MAXBYTES: C2Rust_Unnamed_23 = 21;
pub const WM_SCROLLED: C2Rust_Unnamed_26 = 2;
pub const WM_SHOWN: C2Rust_Unnamed_26 = 1;
pub const EXPAND_FILETYPECMD_INDENT: C2Rust_Unnamed_33 = 2;
pub const EXPAND_FILETYPECMD_PLUGIN: C2Rust_Unnamed_33 = 1;
pub const OPT_LOCAL: C2Rust_Unnamed_27 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_27 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_int;
pub const EW_NOTWILD: C2Rust_Unnamed_28 = 1024;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const EX_EXTRA: ::core::ffi::c_uint = 0x4 as ::core::ffi::c_uint;
pub const EX_XFILE: ::core::ffi::c_uint = 0x8 as ::core::ffi::c_uint;
pub const EX_TRLBAR: ::core::ffi::c_uint = 0x100 as ::core::ffi::c_uint;
pub const EX_NOTRLCOM: ::core::ffi::c_uint = 0x800 as ::core::ffi::c_uint;
pub const EX_CMDARG: ::core::ffi::c_uint = 0x4000 as ::core::ffi::c_uint;
pub const EX_ARGOPT: ::core::ffi::c_uint = 0x20000 as ::core::ffi::c_uint;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const Ctrl_N: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const Ctrl_P: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
static cmd_showtail: GlobalCell<bool> = GlobalCell::new(false);
static may_expand_pattern: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static pre_incsearch_pos: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
});
static compl_match_array: GlobalCell<*mut pumitem_T> =
    GlobalCell::new(::core::ptr::null_mut::<pumitem_T>());
static compl_match_arraysize: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static compl_startcol: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static compl_selected: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static cmdline_orig: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static filetype_expand_what: GlobalCell<C2Rust_Unnamed_21> = GlobalCell::new(EXP_FILETYPECMD_ALL);
static breakpt_expand_what: GlobalCell<C2Rust_Unnamed_20> = GlobalCell::new(EXP_BREAKPT_ADD);
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
