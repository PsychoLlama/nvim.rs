#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::extmark::nvim_create_namespace;
use crate::api::private::helpers::{
    api_free_array, arena_array, cstr_as_string, try_enter, try_leave,
};
use crate::api::vim::nvim_create_buf;
use crate::ascii::{ascii_isalpha, ascii_isdigit, ascii_isspace, ascii_iswhite};
use crate::autocmd::{
    apply_autocmds, aucmd_prepbuf, aucmd_restbuf, block_autocmds, has_event, unblock_autocmds,
};
use crate::buffer::{
    buf_clear, buf_get_changedtick, buf_open_scratch, buf_set_changedtick, buf_valid, close_buffer,
    do_buffer, find_buf,
};
use crate::charset::{
    ptr2cells, skipwhite, vim_is_ident_char, vim_isprintc, vim_iswordc, vim_str2nr,
};
use crate::cmdexpand::{
    clear_cmdline_orig, cmdline_pum_active, cmdline_pum_cleanup, cmdline_pum_remove,
    expand_cleanup, expand_init, expand_one, nextwild, set_expand_context, showmatches,
    wildmenu_cleanup, wildmenu_process_key, wildmenu_translate_key,
};
use crate::cmdhist::{
    add_to_history, get_hisidx, get_hislen, hist_char2type, hist_entry_ref, init_history,
};
use crate::cursor::{
    coladvance, gchar_cursor, get_cursor_line_len, get_cursor_line_ptr, get_cursor_pos_ptr,
};
use crate::digraph::{do_digraph, get_digraph};
use crate::drawscreen::{
    UPD_NOT_VALID, UPD_SOME_VALID, UPD_VALID, redraw_all_later, redraw_custom_title_later,
    redraw_later, redraw_statuslines, set_must_redraw, setcursor, status_redraw_all,
    status_redraw_curbuf, update_screen,
};
use crate::edit::get_literal;
use crate::eval::typval::{
    callback_free, tv_check_for_opt_number_arg, tv_check_for_string_arg, tv_clear, tv_copy,
    tv_dict_add_bool, tv_dict_add_nr, tv_dict_add_str, tv_dict_find, tv_dict_get_callback,
    tv_dict_get_number, tv_dict_get_string_buf_chk, tv_dict_set_keys_readonly, tv_get_number,
    tv_get_number_chk, tv_get_string_buf_chk, tv_list_first, tv_list_free, tv_list_last,
    tv_list_len,
};
use crate::eval::vars::{get_globvar_dict, heredoc_get, set_vim_var_char};
use crate::eval::{callback_call, eval_has_provider, get_echo_hl_id, get_v_event, restore_v_event};
use crate::ex_cmds::rename_buffer;
use crate::ex_docmd::{
    do_cmdline, execute_cmd, expr_map_locked, parse_cmd_address, parse_cmdline,
    parse_command_modifiers, set_no_hlsearch, skip_range, undo_cmdmod,
};
use crate::ex_eval::aborting;
use crate::extmark::extmark_clear;
use crate::getchar::{
    beep_flush, char_avail, getcmdkeycmd, ins_typebuf, map_execute_lua, plain_vgetc, stuff_empty,
    stuff_readbuf, stuff_readbuf_char, stuff_readbuf_one_line, vgetc, vpeekc, vpeekc_any, vungetc,
};
use crate::global_cell::GlobalCell;
use crate::highlight_group::{HLF_E, syn_id2attr, syn_name2id};
use crate::keycodes::{K_SPECIAL, get_special_key_name};
use crate::main::{
    Columns, KeyStuffed, KeyTyped, Rows, State, allbuf_lock, cmd_silent, cmdline_row, cmdline_star,
    cmdline_was_last_drawn, cmdline_win, cmdmod, cmdmsg_rl, cmdpreview, cmdwin_buf, cmdwin_level,
    cmdwin_old_curwin, cmdwin_result, cmdwin_type, cmdwin_win, curbuf, current_sctx, curwin,
    did_emsg, e_cannot_edit_other_buf, e_cmdwin, e_command_too_recursive, e_intern2, e_invarg,
    e_positive, e_textlock, emsg_on_display, ex_normal_busy, exec_from_reg, exmode_active,
    global_busy, got_int, highlight_match, lines_left, magic_overruled, mod_mask, mouse_col,
    mouse_row, msg_col, msg_didout, msg_no_more, msg_row, msg_scroll, msg_scrolled,
    need_wait_return, new_last_cmdline, no_abbr, no_hlsearch, p_ari, p_arshape, p_cedit, p_ch,
    p_cwh, p_hls, p_ic, p_icm, p_is, p_paste, p_ru, p_scs, p_stl, p_tal, p_tbidi, p_wbr, p_wc,
    p_wcm, p_wim, p_wmnu, pum_want, quit_more, redir_off, redraw_cmdline, redraw_tabline,
    redrawing_cmdline, restart_edit, search_first_line, search_last_line, search_match_endcol,
    search_match_lines, skip_redraw, skip_win_fix_cursor, textlock, wild_menu_showing, wim_flags,
};
use crate::map::{mh_get_ptr_t, mh_put_ptr_t};
use crate::mapping::{add_map, check_abbr, map_to_exists_mode};
use crate::mark::setpcmark;
use crate::mbyte::{
    mb_cptr2char_adv, mb_get_class, mb_off_next, mb_prevptr, mb_tolower, utf_char2bytes,
    utf_char2len, utf_head_off, utf_iscomposing_first, utf_ptr2cells, utf_ptr2char,
    utf8len_tab_zero, utfc_ptr2len,
};
use crate::memline::{decl, incl, ml_append, ml_replace};
use crate::memory::{
    ARENA_EMPTY, arena_alloc, arena_finish, arena_mem_free, xfree, xmalloc, xmallocz, xmemdupz,
    xrealloc, xstrdup,
};
use crate::message::{
    emsg, msg, msg_check, msg_clr_eos, msg_cursor_goto, msg_grid_validate, msg_outtrans_len,
    msg_putchar, msg_puts_hl, msg_puts_len, msg_start, msg_starthere, sb_text_end_cmdline,
    sb_text_restart_cmdline, sb_text_start_cmdline,
};
use crate::mouse::setmouse;
use crate::r#move::{
    changed_cline_bef_curs, changed_line_abv_curs, invalidate_botline_win, update_topline,
    validate_cursor,
};
use crate::normal::{clear_showcmd, normal_enter};
use crate::option::{
    csh_like_shell, magic_isset, set_iminsert_global, set_imsearch_global, set_option_direct,
    set_option_value_give_err, string_to_key,
};
use crate::options::{
    kOptBoFlagError, kOptBoFlagWildmode, kOptBufhidden, kOptFiletype, kOptInccommand,
    kOptWimFlagFull, kOptWimFlagLastused, kOptWimFlagList, kOptWimFlagLongest, kOptWimFlagNoselect,
};
use crate::os::cshim::{gettext, strncasecmp, strncmp};
use crate::os::env::home_replace_save;
use crate::os::input::line_breakcheck;
use crate::path::vim_ispathsep;
use crate::popupmenu::{pum_check_clear, pum_ext_want_done, pum_undisplay};
use crate::pos::{MAXCOL, MAXLNUM, clearpos, equalpos, lt};
use crate::profile::profile_setlimit;
use crate::regexp::{RE_SEARCH, skip_regexp_ex};
use crate::register::{
    cmdline_paste_reg, get_expr_line, get_expr_register, get_spec_reg, is_literal_register,
    valid_yank_reg,
};
use crate::search::{
    BACKWARD, FORWARD, SEARCH_COL, SEARCH_KEEP, SEARCH_NOOF, SEARCH_OPT, SEARCH_PEEK, SEARCH_START,
    do_search, last_search_pattern, last_search_pattern_len, pat_has_uppercase,
    restore_last_search_pattern, restore_search_patterns, save_last_search_pattern,
    save_search_patterns, searchit,
};
use crate::state::{
    MODE_CMDLINE, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL, may_trigger_modechanged,
    may_trigger_safestate, state_enter, state_handle_k_event,
};
use crate::strings::{vim_strchr, vim_strsave_escaped, xstrnsave};
use crate::types::AutoEvent;
use crate::types::CAR;
use crate::types::CmdIdx;
use crate::types::ESC;
use crate::types::NL;
use crate::types::TAB;
use crate::types::ui::{kUICmdline, kUIMessages};
use crate::types::{
    Arena, Array, BackslashEscape, Boolean, Callback, CmdAddr, CmdBuff, CmdParseInfo,
    CmdParseInfo_magic, CmdRedraw, CmdlineColorChunk, CmdlineInfo, ColoredCmdline, Direction,
    Error, EvalFuncData, ExArgt, ExpandContext, ExprAST, ExprASTNodeType, ExprAssignmentType,
    ExprCaseCompareStrategy, ExprComparisonType, ExprOptScope, ExprParserFlags, HistoryType,
    Integer, MHPutStatus, MapHash, MotionType, Object, OptInt, OptVal, ParserHighlight,
    ParserHighlightChunk, ParserLine, ParserPosition, ParserState, RemapValues, Set_ptr_t,
    String_0, TryState, UndoLink, UndoObjectType, VimState, aco_save_T, buf_T, cmdmod_T, colnr_T,
    cstack_T, dict_T, disptick_T, dobuf_action_values, dobuf_start_values, exarg_T, except_T,
    expand_T, handle_T, hashtab_T, linenr_T, list_T, listitem_T, magic_T, msglist_T, oparg_T,
    optmagic_T, optset_T, pos_T, proftime_T, ptr_t, ptrdiff_t, save_v_event_T, sctx_T,
    searchit_arg_T, size_t, tabpage_T, time_t, typval_T, typval_vval_union, uint8_t, uint32_t,
    uvarnumber_T, varnumber_T, win_T, xp_prefix_T,
};
use crate::ui::{
    ui_busy_start, ui_busy_stop, ui_call_cmdline_block_append, ui_call_cmdline_block_hide,
    ui_call_cmdline_block_show, ui_call_cmdline_hide, ui_call_cmdline_pos, ui_call_cmdline_show,
    ui_call_cmdline_special_char, ui_cursor_shape, ui_flush, ui_has, vim_beep,
};
use crate::undo::store::header_chain;
use crate::undo::{u_blockfree, u_clearall, u_sync, u_undo_and_forget};
use crate::usercmd::{cmdcomplete_type_to_str, parse_compl_arg};
use crate::viml::parser::expressions::{viml_pexpr_free_ast, viml_pexpr_parse};
use crate::viml::parser::parser::{parser_simple_get_line, viml_parser_destroy, viml_parser_init};
use crate::window::{
    WSP_BOT, close_windows, global_stl_height, last_window, lastwin_nofloating, win_close,
    win_enter, win_goto, win_size_restore, win_size_save, win_split, win_valid,
};
use crate::winlayer::Cc;
use ::libc::{abort, strcpy, strrchr};

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
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub const kMHExisting: MHPutStatus = 0;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub const OPTION_MAGIC_OFF: optmagic_T = 2;
pub const OPTION_MAGIC_ON: optmagic_T = 1;
pub const OPTION_MAGIC_NOT_SET: optmagic_T = 0;
pub const MAGIC_ALL: magic_T = 4;
pub const MAGIC_ON: magic_T = 3;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_GOTO: dobuf_action_values = 0;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const kMTCharWise: MotionType = 0;
pub const kCmdRedrawAll: CmdRedraw = 2;
pub const kCmdRedrawPos: CmdRedraw = 1;
pub const kCmdRedrawNone: CmdRedraw = 0;
pub const HIST_DEBUG: HistoryType = 4;
pub const HIST_SEARCH: HistoryType = 1;
pub const HIST_CMD: HistoryType = 0;
pub const HIST_INVALID: HistoryType = -1;
pub const REMAP_NONE: RemapValues = -1;
pub const VSE_BUFFER: ::core::ffi::c_int = 2;
pub const VSE_SHELL: ::core::ffi::c_int = 1;
/// `#[repr(C)]`: `state_enter` takes `&mut self.state` and the callbacks
/// cast that `*mut VimState` back to this type, which only works while
/// `state` is guaranteed to be the first field.
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
pub const MAX_CB_ERRORS: ::core::ffi::c_int = 1;
#[derive(Clone)]
pub struct CpInfo {
    pub win_info: CpWinInfoVec,
    pub buf_info: CpBufInfoVec,
    pub save_hls: bool,
    pub save_cmdmod: cmdmod_T,
    pub save_view: Vec<::core::ffi::c_int>,
}
#[derive(Copy, Clone)]
pub struct CpBufInfoVec {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut CpBufInfo,
}
#[derive(Copy, Clone)]
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
pub struct CpUndoInfo {
    pub save_b_u_oldhead: UndoLink,
    pub save_b_u_newhead: UndoLink,
    pub save_b_u_curhead: UndoLink,
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
pub struct CpWinInfoVec {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut CpWinInfo,
}
#[derive(Copy, Clone)]
pub struct CpWinInfo {
    pub win: *mut win_T,
    pub save_w_cursor: pos_T,
    pub save_viewstate: viewstate_T,
    pub save_w_p_cul: ::core::ffi::c_int,
    pub save_w_p_cuc: ::core::ffi::c_int,
}
/// What handling one command-line key did — C's anonymous
/// `CMDLINE_NOT_CHANGED` / `CMDLINE_CHANGED` / `GOTO_NORMAL_MODE` enum.
///
/// Four handlers answer with it and three `match`es in
/// [`self::handlekey::command_line_dispatch_key`] read it, which is why it
/// lives on the parent rather than in any one child.  C's fourth value,
/// `PROCESS_NEXT_KEY`, belongs to `CTRL-\` alone and is
/// [`self::execute::CtrlBsl`]'s variant instead.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum KeyOutcome {
    /// The key was handled and the command line is unchanged.
    NotChanged,
    /// The command line changed.
    Changed,
    /// Leave the command line for Normal mode.
    GotoNormalMode,
}
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
unsafe fn set_has_ptr_t(mut set: *mut Set_ptr_t, mut key: ptr_t) -> bool {
    unsafe { mh_get_ptr_t(set, key) != MH_TOMBSTONE as uint32_t }
}
#[inline]
unsafe fn set_put_ptr_t(
    mut set: *mut Set_ptr_t,
    mut key: ptr_t,
    mut key_alloc: *mut *mut ptr_t,
) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = unsafe { mh_put_ptr_t(set, key, &raw mut status) };
    if !key_alloc.is_null() {
        unsafe { *key_alloc = (*set).keys.offset(k as isize) };
    }
    status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint
}
pub const B_IMODE_USE_INSERT: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const B_IMODE_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const B_IMODE_LMAP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
static last_prompt_id: GlobalCell<::core::ffi::c_uint> = GlobalCell::new(0 as ::core::ffi::c_uint);

/// An all-zero [`CmdlineInfo`].
///
/// This is C's `CLEAR_FIELD(ccline)`, and what [`save_cmdline`] leaves behind
/// when it moves the command line onto the saved stack.
/// `Callback::None` is discriminant 0, so this really is the
/// zero value and the C's `CLEAR_FIELD` and this constant agree bit for bit.
pub(crate) const CMDLINE_INFO_INIT: CmdlineInfo = CmdlineInfo {
    cmdbuff: CmdBuff::NONE,
    cmdpos: 0,
    cmdspos: 0,
    cmdfirstc: 0,
    cmdindent: 0,
    cmdprompt: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    hl_id: 0,
    overstrike: 0,
    xpc: ::core::ptr::null_mut::<expand_T>(),
    xp_context: ExpandContext::Nothing,
    xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    input_fn: 0,
    cmdbuff_replaced: false,
    prompt_id: 0,
    highlight_callback: Callback::None,
    last_colors: ColoredCmdline::NONE,
    level: 0,
    special_char: 0,
    special_shift: false,
    redraw_state: kCmdRedrawNone,
    one_key: false,
    mouse_used: ::core::ptr::null_mut::<bool>(),
};

/// An all-zero [`pos_T`].
pub(crate) const POS_INIT: pos_T = pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
};

/// An all-zero [`viewstate_T`], which `save_viewstate` fills.
pub(crate) const VIEWSTATE_INIT: viewstate_T = viewstate_T {
    vs_curswant: 0,
    vs_leftcol: 0,
    vs_skipcol: 0,
    vs_topline: 0,
    vs_topfill: 0,
    vs_botline: 0,
    vs_empty_rows: 0,
};

/// An all-zero [`incsearch_state_T`]; `init_incsearch_state` fills it.
pub(crate) const INCSEARCH_STATE_INIT: incsearch_state_T = incsearch_state_T {
    search_start: POS_INIT,
    save_cursor: POS_INIT,
    winid: 0,
    init_viewstate: VIEWSTATE_INIT,
    old_viewstate: VIEWSTATE_INIT,
    match_start: POS_INIT,
    match_end: POS_INIT,
    did_incsearch: false,
    incsearch_postponed: false,
    magic_overruled_save: OPTION_MAGIC_NOT_SET,
};

/// An all-zero [`expand_T`]; `expand_init` fills the fields that matter.
pub(crate) const EXPAND_T_INIT: expand_T = expand_T {
    xp_pattern: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    xp_context: ExpandContext::Nothing,
    xp_pattern_len: 0,
    xp_prefix: XP_PREFIX_NONE,
    xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    xp_luaref: 0,
    xp_script_ctx: sctx_T::NONE,
    xp_backslash: BackslashEscape::NONE,
    xp_shell: false,
    xp_numfiles: 0,
    xp_col: 0,
    xp_selected: 0,
    xp_orig: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    xp_files: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    xp_line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    xp_buf: [0; 1025],
    xp_search_dir: kDirectionNotSet,
    xp_pre_incsearch_pos: POS_INIT,
};

/// An all-zero [`TryState`], which is what the `TRY_WRAP` macro declares
/// (uninitialised in the C; `try_enter` fills every field).
pub(crate) const TRY_STATE_INIT: TryState = TryState {
    current_exception: ::core::ptr::null_mut::<except_T>(),
    private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
    msg_list: ::core::ptr::null::<*const msglist_T>(),
    got_int: 0,
    did_throw: false,
    need_rethrow: 0,
    did_emsg: 0,
};

/// An all-zero [`save_v_event_T`], the out-parameter of `get_v_event`.
/// C's `STATIC_CSTR_AS_OPTVAL`: a string option value borrowing a literal.
/// Nothing frees it.
pub(crate) const fn static_optval(value: &'static ::core::ffi::CStr) -> OptVal {
    OptVal::String(String_0::from_raw_parts(
        value.as_ptr() as *mut ::core::ffi::c_char,
        value.count_bytes() as size_t,
    ))
}

/// An all-zero [`exarg_T`]; `parse_cmdline` fills it.
pub(crate) const EXARG_T_INIT: exarg_T = exarg_T {
    arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    arglens: ::core::ptr::null_mut::<size_t>(),
    argc: 0,
    nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    cmdidx: CmdIdx::append,
    argt: ExArgt::NONE,
    skip: 0,
    forceit: 0,
    addr_count: 0,
    line1: 0,
    line2: 0,
    addr_type: CmdAddr::Lines,
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
    errmsg: None,
    ea_getline: None,
    cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    cstack: ::core::ptr::null_mut::<cstack_T>(),
};

/// An all-zero [`CmdParseInfo`]; `parse_cmdline` fills it.
pub(crate) const CMD_PARSE_INFO_INIT: CmdParseInfo = CmdParseInfo {
    cmdmod: cmdmod_T::NONE,
    magic: CmdParseInfo_magic {
        file: false,
        bar: false,
    },
};

/// An all-zero [`CpUndoInfo`], which `cmdpreview_save_undo` fills.
pub(crate) const CP_UNDO_INFO_INIT: CpUndoInfo = CpUndoInfo {
    save_b_u_oldhead: UndoLink::NONE,
    save_b_u_newhead: UndoLink::NONE,
    save_b_u_curhead: UndoLink::NONE,
    save_b_u_numhead: 0,
    save_b_u_synced: false,
    save_b_u_seq_last: 0,
    save_b_u_save_nr_last: 0,
    save_b_u_seq_cur: 0,
    save_b_u_time_cur: 0,
    save_b_u_save_nr_cur: 0,
    save_b_u_line_ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    save_b_u_line_lnum: 0,
    save_b_u_line_colnr: 0,
};

/// An all-zero [`CpBufInfo`]; every field is assigned before it is pushed.
pub(crate) const CP_BUF_INFO_INIT: CpBufInfo = CpBufInfo {
    buf: ::core::ptr::null_mut::<buf_T>(),
    save_b_p_ul: 0,
    save_b_p_ma: 0,
    save_b_changed: 0,
    save_b_op_start: POS_INIT,
    save_b_op_end: POS_INIT,
    save_changedtick: 0,
    undo_info: CP_UNDO_INFO_INIT,
};

/// An all-zero [`CpWinInfo`]; every field is assigned before it is pushed.
pub(crate) const CP_WIN_INFO_INIT: CpWinInfo = CpWinInfo {
    win: ::core::ptr::null_mut::<win_T>(),
    save_w_cursor: POS_INIT,
    save_viewstate: VIEWSTATE_INIT,
    save_w_p_cul: 0,
    save_w_p_cuc: 0,
};

/// An all-zero [`CpInfo`], C's two `kv_init`s plus an unset save area.
pub(crate) const CP_INFO_INIT: CpInfo = CpInfo {
    win_info: CpWinInfoVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<CpWinInfo>(),
    },
    buf_info: CpBufInfoVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<CpBufInfo>(),
    },
    save_hls: false,
    save_cmdmod: cmdmod_T::NONE,
    save_view: Vec::new(),
};

pub(crate) const SAVE_V_EVENT_INIT: save_v_event_T = save_v_event_T {
    sve_did_save: false,
    sve_hashtab: hashtab_T::new(),
};

static ccline: GlobalCell<CmdlineInfo> = GlobalCell::new(CMDLINE_INFO_INIT);
/// The command lines suspended under [`ccline`], innermost last.
///
/// C threads these on a `prev_ccline` pointer through the stack frame of
/// whoever called `save_cmdline()`, so the chain is only as sound as every
/// caller's promise to restore before returning. Here they are owned values
/// on an explicit stack: [`save_cmdline`] pushes and [`restore_cmdline`]
/// pops. The `Box` is what makes an entry's address stable, because the
/// Vimscript face still answers about the enclosing command line by pointer.
// The `Box` is the point, not an oversight: `get_ccline_ptr()` hands the
// address of an entry to the Vimscript functions, and a bare `Vec` would move
// every entry the next `save_cmdline` grows it.
#[allow(clippy::vec_box)]
static saved_cmdlines: GlobalCell<Vec<Box<CmdlineInfo>>> = GlobalCell::new(Vec::new());
static new_cmdpos: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static cmdline_block: GlobalCell<CmdlineBlock> = GlobalCell::new(CmdlineBlock::EMPTY);
static getln_interrupted_highlight: GlobalCell<bool> = GlobalCell::new(false);
static cedit_key: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
static cmdpreview_bufnr: GlobalCell<handle_T> = GlobalCell::new(0 as handle_T);
static cmdpreview_ns: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static e_active_window_or_buffer_changed_or_deleted: &::core::ffi::CStr =
    c"E199: Active window or buffer changed or deleted";
pub const PATH_ESC_CHARS: &::core::ffi::CStr = c" \t\n*?[{`$\\%#'\"|!<";
pub const SHELL_ESC_CHARS: &::core::ffi::CStr = c" \t\n*?[{`$\\%#'\"|!<>();&";
pub const BUFFER_ESC_CHARS: &::core::ffi::CStr = c" \t\n*?[`$\\%#'\"|!<";
pub const SID_NONE: ::core::ffi::c_int = -6 as ::core::ffi::c_int;
pub const ABBR_OFF: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const UINT_MAX: ::core::ffi::c_uint = (INT_MAX as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
