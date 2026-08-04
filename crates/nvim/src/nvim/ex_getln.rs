#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::api::extmark::nvim_create_namespace;
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_array, arena_array, cstr_as_string, try_enter, try_leave,
};
use crate::src::nvim::api::vim::nvim_create_buf;
use crate::src::nvim::ascii::{ascii_isdigit, ascii_isspace, ascii_iswhite};
use crate::src::nvim::autocmd::{
    EVENT_CMDLINECHANGED, EVENT_CMDLINEENTER, EVENT_CMDLINELEAVE, EVENT_CMDLINELEAVEPRE,
    EVENT_CMDWINENTER, EVENT_CMDWINLEAVE, EVENT_CURSORMOVEDC, apply_autocmds, aucmd_prepbuf,
    aucmd_restbuf, block_autocmds, has_event, unblock_autocmds,
};
use crate::src::nvim::buffer::buf_get_changedtick;
use crate::src::nvim::buffer::{
    buf_clear, buf_open_scratch, buf_set_changedtick, buf_valid, buflist_findnr, bufref_valid,
    close_buffer, do_buffer, set_bufref,
};
use crate::src::nvim::charset::{
    ptr2cells, skipwhite, vim_isIDc, vim_isprintc, vim_iswordc, vim_str2nr,
};
use crate::src::nvim::cmdexpand::{
    ExpandCleanup, ExpandInit, ExpandOne, clear_cmdline_orig, cmdline_pum_active,
    cmdline_pum_cleanup, cmdline_pum_remove, nextwild, set_expand_context, showmatches,
    wildmenu_cleanup, wildmenu_process_key, wildmenu_translate_key,
};
use crate::src::nvim::cmdhist::{
    add_to_history, get_hisidx, get_hislen, hist_char2type, hist_entry_ref, init_history,
};
use crate::src::nvim::cursor::{
    coladvance, gchar_cursor, get_cursor_line_len, get_cursor_line_ptr, get_cursor_pos_ptr,
};
use crate::src::nvim::digraph::{do_digraph, get_digraph};
use crate::src::nvim::drawscreen::{
    UPD_NOT_VALID, UPD_SOME_VALID, UPD_VALID, redraw_all_later, redraw_custom_title_later,
    redraw_later, redraw_statuslines, set_must_redraw, setcursor, status_redraw_all,
    status_redraw_curbuf, update_screen,
};
use crate::src::nvim::edit::get_literal;
use crate::src::nvim::eval::typval::{
    callback_free, kCallbackNone, tv_check_for_opt_number_arg, tv_check_for_string_arg, tv_clear,
    tv_copy, tv_dict_add_bool, tv_dict_add_nr, tv_dict_add_str, tv_dict_find, tv_dict_get_callback,
    tv_dict_get_number, tv_dict_get_string_buf_chk, tv_dict_set_keys_readonly, tv_get_number,
    tv_get_number_chk, tv_get_string, tv_get_string_buf_chk, tv_get_string_chk, tv_list_free,
};
use crate::src::nvim::eval::typval::{tv_list_first, tv_list_last, tv_list_len};
use crate::src::nvim::eval::vars::{get_globvar_dict, heredoc_get, set_vim_var_char};
use crate::src::nvim::eval::{
    callback_call, eval_has_provider, get_echo_hl_id, get_v_event, restore_v_event,
};
use crate::src::nvim::ex_cmds::rename_buffer;
use crate::src::nvim::ex_docmd::{
    do_cmdline, execute_cmd, expr_map_locked, parse_cmd_address, parse_cmdline,
    parse_command_modifiers, set_no_hlsearch, skip_range, undo_cmdmod,
};
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::extmark::extmark_clear;
use crate::src::nvim::garray::{ga_append, ga_clear, ga_concat, ga_init};
use crate::src::nvim::getchar::{
    beep_flush, char_avail, getcmdkeycmd, ins_typebuf, map_execute_lua, plain_vgetc, stuff_empty,
    stuffReadbuff, stuffReadbuffSpec, stuffcharReadbuff, vgetc, vpeekc, vpeekc_any, vungetc,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight_group::{HLF_E, syn_id2attr, syn_name2id};
use crate::src::nvim::keycodes::{
    K_BS, K_C_END, K_C_HOME, K_DEL, K_DOWN, K_END, K_HOME, K_INS, K_KEND, K_KENTER, K_KHOME,
    K_KINS, K_KPAGEDOWN, K_KPAGEUP, K_LEFT, K_LEFTDRAG, K_LEFTMOUSE, K_MIDDLEDRAG, K_MIDDLEMOUSE,
    K_MIDDLERELEASE, K_MOUSEDOWN, K_MOUSELEFT, K_MOUSEMOVE, K_MOUSERIGHT, K_MOUSEUP, K_PAGEDOWN,
    K_PAGEUP, K_RIGHT, K_RIGHTDRAG, K_RIGHTMOUSE, K_S_END, K_S_HOME, K_S_LEFT, K_S_RIGHT, K_S_TAB,
    K_SELECT, K_SPECIAL, K_UP, K_X1DRAG, K_X1MOUSE, K_X1RELEASE, K_X2DRAG, K_X2MOUSE, K_X2RELEASE,
    K_ZERO, get_special_key_name,
};
use crate::src::nvim::main::{
    Columns, IObuff, KeyStuffed, KeyTyped, RedrawingDisabled, Rows, State, allbuf_lock, allow_keys,
    cmd_silent, cmdline_row, cmdline_star, cmdline_was_last_drawn, cmdline_win, cmdmod, cmdmsg_rl,
    cmdpreview, cmdwin_buf, cmdwin_level, cmdwin_old_curwin, cmdwin_result, cmdwin_type,
    cmdwin_win, curbuf, current_sctx, curtab, curwin, did_emsg, e_cannot_edit_other_buf, e_cmdwin,
    e_command_too_recursive, e_intern2, e_invarg, e_positive, e_textlock, emsg_off,
    emsg_on_display, emsg_silent, ex_normal_busy, exec_from_reg, exmode_active, firstwin,
    global_busy, got_int, highlight_match, lines_left, magic_overruled, mod_mask, mouse_col,
    mouse_row, msg_col, msg_didout, msg_no_more, msg_row, msg_scroll, msg_scrolled, msg_silent,
    need_wait_return, new_last_cmdline, no_abbr, no_hlsearch, no_mapping, p_ari, p_arshape,
    p_cedit, p_ch, p_cpo, p_cwh, p_hls, p_ic, p_icm, p_is, p_paste, p_ru, p_scs, p_stl, p_tal,
    p_tbidi, p_wbr, p_wc, p_wcm, p_wim, p_wmnu, quit_more, redir_off, redraw_cmdline,
    redraw_tabline, redrawing_cmdline, restart_edit, search_first_line, search_last_line,
    search_match_endcol, search_match_lines, skip_redraw, skip_win_fix_cursor, textlock, typebuf,
    wild_menu_showing, wim_flags,
};
use crate::src::nvim::map::{mh_get_ptr_t, mh_put_ptr_t};
use crate::src::nvim::mapping::{add_map, check_abbr, map_to_exists_mode};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::mbyte::{
    mb_cptr2char_adv, mb_get_class, mb_off_next, mb_prevptr, mb_tolower, utf_char2bytes,
    utf_char2len, utf_head_off, utf_iscomposing_first, utf_ptr2cells, utf_ptr2char,
    utf8len_tab_zero, utfc_ptr2len,
};
use crate::src::nvim::memline::{decl, incl, ml_append, ml_replace};
use crate::src::nvim::memory::{
    ARENA_EMPTY, arena_alloc, arena_finish, arena_mem_free, xfree, xmalloc, xmallocz, xmemdupz,
    xrealloc, xstrdup,
};
use crate::src::nvim::message::{
    emsg, msg, msg_check, msg_clr_eos, msg_cursor_goto, msg_grid_validate, msg_outtrans_len,
    msg_putchar, msg_puts_hl, msg_puts_len, msg_start, msg_starthere, sb_text_end_cmdline,
    sb_text_restart_cmdline, sb_text_start_cmdline, smsg,
};
use crate::src::nvim::mouse::setmouse;
use crate::src::nvim::r#move::{
    changed_cline_bef_curs, changed_line_abv_curs, invalidate_botline_win, update_topline,
    validate_cursor,
};
use crate::src::nvim::normal::{clear_showcmd, normal_enter};
use crate::src::nvim::option::{
    csh_like_shell, magic_isset, set_iminsert_global, set_imsearch_global, set_option_direct,
    set_option_value_give_err, string_to_key,
};
use crate::src::nvim::options::{
    kOptBoFlagError, kOptBoFlagWildmode, kOptBufhidden, kOptFiletype, kOptInccommand,
    kOptWimFlagFull, kOptWimFlagLastused, kOptWimFlagList, kOptWimFlagLongest, kOptWimFlagNoselect,
};
use crate::src::nvim::os::env::home_replace_save;
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, abort, gettext, memcpy, memmove, memset, strcmp, strcpy, strlen, strncasecmp,
    strncmp, strrchr,
};
use crate::src::nvim::path::vim_ispathsep;
use crate::src::nvim::popupmenu::{pum_check_clear, pum_undisplay};
use crate::src::nvim::pos::{MAXCOL, MAXLNUM, clearpos, equalpos, lt};
use crate::src::nvim::profile::profile_setlimit;
use crate::src::nvim::regexp::{RE_SEARCH, skip_regexp_ex};
use crate::src::nvim::register::is_literal_register;
use crate::src::nvim::register::{
    cmdline_paste_reg, get_expr_line, get_expr_register, get_spec_reg, valid_yank_reg,
};
use crate::src::nvim::search::{
    BACKWARD, FORWARD, SEARCH_COL, SEARCH_KEEP, SEARCH_NOOF, SEARCH_OPT, SEARCH_PEEK, SEARCH_START,
    do_search, last_search_pattern, last_search_pattern_len, pat_has_uppercase,
    restore_last_search_pattern, restore_search_patterns, save_last_search_pattern,
    save_search_patterns, searchit,
};
use crate::src::nvim::state::{
    MODE_CMDLINE, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL, may_trigger_modechanged,
    may_trigger_safestate, state_enter, state_handle_k_event,
};
use crate::src::nvim::strings::{vim_strchr, vim_strsave_escaped, xstrnsave};
use crate::src::nvim::types::api::{kErrorTypeException, kErrorTypeNone};
use crate::src::nvim::types::ui::{kUICmdline, kUIMessages};
use crate::src::nvim::types::{
    Arena, Array, BoolVarValue, Boolean, CMD_index, Callback, Callback_data as C2Rust_Unnamed_5,
    CmdParseInfo, CmdParseInfo_magic as C2Rust_Unnamed_21, CmdRedraw, CmdlineColorChunk,
    CmdlineColors, CmdlineInfo, ColoredCmdline, Direction, Error, EvalFuncData, ExprAST,
    ExprASTNodeType, ExprAssignmentType, ExprCaseCompareStrategy, ExprComparisonType, ExprOptScope,
    ExprParserFlags, HistoryType, Integer, MHPutStatus, MapHash, MotionType, Object, OptInt,
    OptVal, OptValData, OptValType, ParserHighlight, ParserHighlightChunk, ParserLine,
    ParserPosition, ParserState, RemapValues, ScopeType, Set_ptr_t, SpecialVarValue, String_0,
    TryState, UndoObjectType, VarLockStatus, VarType, VimState, aco_save_T, buf_T, bufref_T,
    cmd_addr_T, cmdmod_T, colnr_T, cstack_T, dict_T, dictitem_T, disptick_T, dobuf_action_values,
    dobuf_start_values, event_T, exarg, exarg_T, except_T, expand_T, garray_T, handle_T,
    hashitem_T, hashtab_T, int64_t, kObjectTypeArray, kObjectTypeInteger, kObjectTypeString,
    key_extra, linenr_T, list_T, listitem_T, magic_T, msglist_T, object,
    object_data as C2Rust_Unnamed, oparg_T, optmagic_T, optset_T, pos_T, proftime_T, ptr_t,
    ptrdiff_t, regmatch_T, regprog_T, save_v_event_T, sctx_T, searchit_arg_T, size_t,
    state_check_callback, state_execute_callback, tabpage_T, time_t, typval_T, typval_vval_union,
    u_header_T, uint8_t, uint32_t, uvarnumber_T, varnumber_T, win_T, xp_prefix_T,
};
use crate::src::nvim::ui::{
    ui_busy_start, ui_busy_stop, ui_call_cmdline_block_append, ui_call_cmdline_block_hide,
    ui_call_cmdline_block_show, ui_call_cmdline_hide, ui_call_cmdline_pos, ui_call_cmdline_show,
    ui_call_cmdline_special_char, ui_cursor_shape, ui_flush, ui_has, vim_beep,
};
use crate::src::nvim::undo::{u_blockfree, u_clearall, u_sync, u_undo_and_forget};
use crate::src::nvim::usercmd::{cmdcomplete_type_to_str, parse_compl_arg};
use crate::src::nvim::viml::parser::expressions::{viml_pexpr_free_ast, viml_pexpr_parse};
use crate::src::nvim::viml::parser::parser::{
    parser_simple_get_line, viml_parser_destroy, viml_parser_init,
};
use crate::src::nvim::window::{
    WSP_BOT, close_windows, global_stl_height, last_window, lastwin_nofloating, win_close,
    win_enter, win_goto, win_size_restore, win_size_save, win_split, win_valid,
};

// The carve of the transpiled module; see each child's docs.
mod incsearch;
pub use self::incsearch::*;
mod enter;
pub use self::enter::*;
mod execute;
pub(crate) use self::execute::*;
mod handlekey;
pub(crate) use self::handlekey::*;
mod wildchar;
pub use self::wildchar::*;
mod history;
pub use self::history::*;
mod cmdpreview;
pub use self::cmdpreview::*;
mod color;
pub(crate) use self::color::*;
mod draw;
pub use self::draw::*;
mod uiext;
pub use self::uiext::*;
mod buffer;
pub use self::buffer::*;
mod cmdwin;
pub use self::cmdwin::*;
mod eval;
pub use self::eval::*;
mod prompt;
pub use self::prompt::*;
unsafe extern "C" {
    static pum_want: GlobalCell<C2Rust_Unnamed_51>;
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
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_int;
pub const kMHExisting: MHPutStatus = 0;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_int;
pub const XP_BS_NONE: C2Rust_Unnamed_17 = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_int;
pub const EXPAND_NOTHING: C2Rust_Unnamed_18 = 0;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_18 = -2;
pub const OPTION_MAGIC_OFF: optmagic_T = 2;
pub const OPTION_MAGIC_ON: optmagic_T = 1;
pub const OPTION_MAGIC_NOT_SET: optmagic_T = 0;
pub const MAGIC_ALL: magic_T = 4;
pub const MAGIC_ON: magic_T = 3;
pub const kOptValTypeString: OptValType = 2;
pub const CMD_snext: CMD_index = 414;
pub const CMD_drop: CMD_index = 130;
pub const CMD_arglocal: CMD_index = 14;
pub const CMD_argglobal: CMD_index = 13;
pub const CMD_argdo: CMD_index = 10;
pub const CMD_args: CMD_index = 7;
pub const CMD_append: CMD_index = 0;
pub const ADDR_LINES: cmd_addr_T = 0;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_int;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_20 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_20 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_20 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_20 = 1024;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_GOTO: dobuf_action_values = 0;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const kMTCharWise: MotionType = 0;
pub const kCmdRedrawAll: CmdRedraw = 2;
pub const kCmdRedrawPos: CmdRedraw = 1;
pub const kCmdRedrawNone: CmdRedraw = 0;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_int;
pub const WILD_PUM_WANT: C2Rust_Unnamed_24 = 13;
pub const WILD_PAGEDOWN: C2Rust_Unnamed_24 = 12;
pub const WILD_PAGEUP: C2Rust_Unnamed_24 = 11;
pub const WILD_APPLY: C2Rust_Unnamed_24 = 10;
pub const WILD_CANCEL: C2Rust_Unnamed_24 = 9;
pub const WILD_LONGEST: C2Rust_Unnamed_24 = 7;
pub const WILD_ALL: C2Rust_Unnamed_24 = 6;
pub const WILD_PREV: C2Rust_Unnamed_24 = 5;
pub const WILD_NEXT: C2Rust_Unnamed_24 = 4;
pub const WILD_EXPAND_KEEP: C2Rust_Unnamed_24 = 3;
pub const WILD_FREE: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_int;
pub const WILD_FUNC_TRIGGER: C2Rust_Unnamed_25 = 65536;
pub const WILD_MAY_EXPAND_PATTERN: C2Rust_Unnamed_25 = 32768;
pub const WILD_NOSELECT: C2Rust_Unnamed_25 = 16384;
pub const WILD_BUFLASTUSED: C2Rust_Unnamed_25 = 4096;
pub const WILD_NO_BEEP: C2Rust_Unnamed_25 = 8;
pub const HIST_DEBUG: HistoryType = 4;
pub const HIST_SEARCH: HistoryType = 1;
pub const HIST_CMD: HistoryType = 0;
pub const HIST_INVALID: HistoryType = -1;
pub const REMAP_NONE: RemapValues = -1;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_int;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_28 = 2;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_int;
pub const VSE_BUFFER: C2Rust_Unnamed_29 = 2;
pub const VSE_SHELL: C2Rust_Unnamed_29 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CommandLineState {
    pub state: VimState,
    pub firstc: ::core::ffi::c_int,
    pub count: ::core::ffi::c_int,
    pub indent: ::core::ffi::c_int,
    pub c: ::core::ffi::c_int,
    pub gotesc: bool,
    pub do_abbr: bool,
    pub lookfor: *mut ::core::ffi::c_char,
    pub lookforlen: ::core::ffi::c_int,
    pub hiscnt: ::core::ffi::c_int,
    pub save_hiscnt: ::core::ffi::c_int,
    pub histype: ::core::ffi::c_int,
    pub is_state: incsearch_state_T,
    pub did_wild_list: bool,
    pub wim_index: ::core::ffi::c_int,
    pub save_msg_scroll: ::core::ffi::c_int,
    pub save_State: ::core::ffi::c_int,
    pub prev_cmdpos: ::core::ffi::c_int,
    pub prev_cmdbuff: *mut ::core::ffi::c_char,
    pub save_p_icm: *mut ::core::ffi::c_char,
    pub skip_pum_redraw: bool,
    pub some_key_typed: bool,
    pub ignore_drag_release: bool,
    pub break_ctrl_c: bool,
    pub xpc: expand_T,
    pub b_im_ptr: *mut OptInt,
    pub b_im_ptr_buf: *mut buf_T,
    pub cmdline_type: ::core::ffi::c_int,
    pub event_cmdlineleavepre_triggered: bool,
    pub did_hist_navigate: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct incsearch_state_T {
    pub search_start: pos_T,
    pub save_cursor: pos_T,
    pub winid: handle_T,
    pub init_viewstate: viewstate_T,
    pub old_viewstate: viewstate_T,
    pub match_start: pos_T,
    pub match_end: pos_T,
    pub did_incsearch: bool,
    pub incsearch_postponed: bool,
    pub magic_overruled_save: optmagic_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct viewstate_T {
    pub vs_curswant: colnr_T,
    pub vs_leftcol: colnr_T,
    pub vs_skipcol: colnr_T,
    pub vs_topline: linenr_T,
    pub vs_topfill: ::core::ffi::c_int,
    pub vs_botline: linenr_T,
    pub vs_empty_rows: ::core::ffi::c_int,
}
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
pub const kExprFlagsDisallowEOC: ExprParserFlags = 2;
pub const MAX_CB_ERRORS: C2Rust_Unnamed_58 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CpInfo {
    pub win_info: C2Rust_Unnamed_50,
    pub buf_info: C2Rust_Unnamed_49,
    pub save_hls: bool,
    pub save_cmdmod: cmdmod_T,
    pub save_view: garray_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_49 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut CpBufInfo,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CpBufInfo {
    pub buf: *mut buf_T,
    pub save_b_p_ul: OptInt,
    pub save_b_p_ma: ::core::ffi::c_int,
    pub save_b_changed: ::core::ffi::c_int,
    pub save_b_op_start: pos_T,
    pub save_b_op_end: pos_T,
    pub save_changedtick: varnumber_T,
    pub undo_info: CpUndoInfo,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CpUndoInfo {
    pub save_b_u_oldhead: *mut u_header_T,
    pub save_b_u_newhead: *mut u_header_T,
    pub save_b_u_curhead: *mut u_header_T,
    pub save_b_u_numhead: ::core::ffi::c_int,
    pub save_b_u_synced: bool,
    pub save_b_u_seq_last: ::core::ffi::c_int,
    pub save_b_u_save_nr_last: ::core::ffi::c_int,
    pub save_b_u_seq_cur: ::core::ffi::c_int,
    pub save_b_u_time_cur: time_t,
    pub save_b_u_save_nr_cur: ::core::ffi::c_int,
    pub save_b_u_line_ptr: *mut ::core::ffi::c_char,
    pub save_b_u_line_lnum: linenr_T,
    pub save_b_u_line_colnr: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_50 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut CpWinInfo,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CpWinInfo {
    pub win: *mut win_T,
    pub save_w_cursor: pos_T,
    pub save_viewstate: viewstate_T,
    pub save_w_p_cul: ::core::ffi::c_int,
    pub save_w_p_cuc: ::core::ffi::c_int,
}
pub const GOTO_NORMAL_MODE: C2Rust_Unnamed_57 = 3;
pub const CMDLINE_CHANGED: C2Rust_Unnamed_57 = 2;
pub const CMDLINE_NOT_CHANGED: C2Rust_Unnamed_57 = 1;
pub const KE_S_DOWN: key_extra = 5;
pub const KE_S_UP: key_extra = 4;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_IGNORE: key_extra = 53;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_KDEL: key_extra = 80;
pub const KE_WILD: key_extra = 108;
pub const KE_XF2: key_extra = 58;
pub const KE_XF1: key_extra = 57;
pub const KE_NOP: key_extra = 97;
pub const OPT_LOCAL: C2Rust_Unnamed_53 = 2;
pub const KE_CMDWIN: key_extra = 84;
pub const PROCESS_NEXT_KEY: C2Rust_Unnamed_57 = 4;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_51 {
    pub active: bool,
    pub item: ::core::ffi::c_int,
    pub insert: bool,
    pub finish: bool,
}
pub const KE_COMMAND: key_extra = 104;
pub const KE_EVENT: key_extra = 102;
pub const KE_LUA: key_extra = 103;
pub type C2Rust_Unnamed_53 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_54 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_55 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_56 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_57 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_58 = ::core::ffi::c_int;
static prev_prompt_id: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0);
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARRAY_DICT_INIT: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_ptr_t = Set_ptr_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_has_ptr_t(mut set: *mut Set_ptr_t, mut key: ptr_t) -> bool {
    unsafe {
        return mh_get_ptr_t(set, key) != MH_TOMBSTONE as uint32_t;
    }
}
#[inline]
unsafe extern "C" fn set_put_ptr_t(
    mut set: *mut Set_ptr_t,
    mut key: ptr_t,
    mut key_alloc: *mut *mut ptr_t,
) -> bool {
    unsafe {
        let mut status: MHPutStatus = kMHExisting;
        let mut k: uint32_t = mh_put_ptr_t(set, key, &raw mut status);
        if !key_alloc.is_null() {
            *key_alloc = (*set).keys.offset(k as isize);
        }
        return status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const Ctrl_A: ::core::ffi::c_int = 1;
pub const Ctrl_B: ::core::ffi::c_int = 2;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const Ctrl_D: ::core::ffi::c_int = 4;
pub const Ctrl_E: ::core::ffi::c_int = 5;
pub const Ctrl_F: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const Ctrl_G: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const Ctrl_H: ::core::ffi::c_int = 8;
pub const Ctrl_K: ::core::ffi::c_int = 11;
pub const Ctrl_L: ::core::ffi::c_int = 12;
pub const Ctrl_N: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const Ctrl_O: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const Ctrl_P: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const Ctrl_Q: ::core::ffi::c_int = 17;
pub const Ctrl_R: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const Ctrl_T: ::core::ffi::c_int = 20;
pub const Ctrl_U: ::core::ffi::c_int = 21;
pub const Ctrl_V: ::core::ffi::c_int = 22;
pub const Ctrl_W: ::core::ffi::c_int = 23 as ::core::ffi::c_int;
pub const Ctrl_Y: ::core::ffi::c_int = 25 as ::core::ffi::c_int;
pub const Ctrl_Z: ::core::ffi::c_int = 26 as ::core::ffi::c_int;
pub const Ctrl_BSL: ::core::ffi::c_int = 28 as ::core::ffi::c_int;
pub const Ctrl_RSB: ::core::ffi::c_int = 29 as ::core::ffi::c_int;
pub const Ctrl_HAT: ::core::ffi::c_int = 30;
pub const Ctrl__: ::core::ffi::c_int = 31;
pub const EX_RANGE: ::core::ffi::c_uint = 0x1 as ::core::ffi::c_uint;
pub const EX_PREVIEW: ::core::ffi::c_uint = 0x8000000 as ::core::ffi::c_uint;
pub const CPO_ESC: ::core::ffi::c_int = 'x' as ::core::ffi::c_int;
pub const B_IMODE_USE_INSERT: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const B_IMODE_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const B_IMODE_LMAP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
static last_prompt_id: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0 as ::core::ffi::c_uint);
static ccline: GlobalCell<CmdlineInfo> = GlobalCell::new(CmdlineInfo {
    cmdbuff: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    cmdbufflen: 0,
    cmdlen: 0,
    cmdpos: 0,
    cmdspos: 0,
    cmdfirstc: 0,
    cmdindent: 0,
    cmdprompt: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    hl_id: 0,
    overstrike: 0,
    xpc: ::core::ptr::null_mut::<expand_T>(),
    xp_context: 0,
    xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    input_fn: 0,
    cmdbuff_replaced: false,
    prompt_id: 0,
    highlight_callback: Callback {
        data: C2Rust_Unnamed_5 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    },
    last_colors: ColoredCmdline {
        prompt_id: 0,
        cmdbuff: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        colors: CmdlineColors {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<CmdlineColorChunk>(),
        },
    },
    level: 0,
    prev_ccline: ::core::ptr::null_mut::<CmdlineInfo>(),
    special_char: 0,
    special_shift: false,
    redraw_state: kCmdRedrawNone,
    one_key: false,
    mouse_used: ::core::ptr::null_mut::<bool>(),
});
static new_cmdpos: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static cmdline_block: GlobalCell<Array> = GlobalCell::new(ARRAY_DICT_INIT);
static getln_interrupted_highlight: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static cedit_key: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
static cmdpreview_bufnr: GlobalCell<handle_T> = GlobalCell::new(0 as handle_T);
static cmdpreview_ns: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static e_active_window_or_buffer_changed_or_deleted: GlobalCell<[::core::ffi::c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
            *b"E199: Active window or buffer changed or deleted\0",
        )
    });
pub const PATH_ESC_CHARS: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b" \t\n*?[{`$\\%#'\"|!<\0")
};
pub const SHELL_ESC_CHARS: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b" \t\n*?[{`$\\%#'\"|!<>();&\0")
};
pub const BUFFER_ESC_CHARS: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b" \t\n*?[`$\\%#'\"|!<\0")
};
pub const SID_NONE: ::core::ffi::c_int = -6 as ::core::ffi::c_int;
pub const ABBR_OFF: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const MOD_MASK_SHIFT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOD_MASK_CTRL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const UINT_MAX: ::core::ffi::c_uint = (INT_MAX as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
unsafe extern "C" fn c2rust_run_static_initializers() {
    prev_prompt_id.set(UINT_MAX);
}
#[used]
#[cfg_attr(target_os = "linux", unsafe(link_section = ".init_array"))]
#[cfg_attr(target_os = "windows", unsafe(link_section = ".CRT$XIB"))]
#[cfg_attr(target_os = "macos", unsafe(link_section = "__DATA,__mod_init_func"))]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [c2rust_run_static_initializers];
