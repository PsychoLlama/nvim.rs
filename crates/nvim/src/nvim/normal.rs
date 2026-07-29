use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite, ascii_iswhite_or_nul};
use crate::src::nvim::autocmd::{apply_autocmds, has_event};
use crate::src::nvim::buffer::buf_get_changedtick;
use crate::src::nvim::buffer::{bt_prompt, bt_quickfix, buf_hide, buflist_getfile, fileinfo};
use crate::src::nvim::change::{
    changed_lines, del_chars, deleted_lines, get_leader_len, ins_char, ins_char_bytes, open_line,
};
use crate::src::nvim::charset::{skipwhite, transchar, vim_isprintc, vim_iswordp, vim_strsize};
use crate::src::nvim::cmdhist::{add_to_history, init_history};
use crate::src::nvim::cursor::{
    adjust_cursor_col, check_cursor, check_cursor_col, check_cursor_lnum, coladvance,
    coladvance_force, dec_cursor, gchar_cursor, get_cursor_line_len, get_cursor_line_ptr,
    get_cursor_pos_len, get_cursor_pos_ptr, getviscol, inc_cursor, set_leftcol,
};
use crate::src::nvim::decoration::{decor_conceal_line, win_lines_concealed};
use crate::src::nvim::diff::{diff_move_to, diff_set_topline, ex_diffupdate, nv_diffgetput};
use crate::src::nvim::digraph::get_digraph;
use crate::src::nvim::drawscreen::{
    conceal_check_cursor_line, redraw_curbuf_later, redraw_later, redraw_statuslines, setcursor,
    show_cursor_info_later, showmode, update_screen, win_cursorline_standout,
};
use crate::src::nvim::edit::{
    beginline, cursor_down, cursor_down_inner, cursor_up, cursor_up_inner, edit, get_literal,
    ins_copychar, oneleft, oneright, prompt_curpos_editable, set_last_insert,
};
use crate::src::nvim::eval::prompt_invoke_callback;
use crate::src::nvim::eval::vars::{set_reg_var, set_vcount, set_vim_var_string};
use crate::src::nvim::ex_cmds::{do_ascii, do_ecmd};
use crate::src::nvim::ex_cmds2::autowrite;
use crate::src::nvim::ex_docmd::{do_cmdline, do_cmdline_cmd, do_exmode, do_sleep};
use crate::src::nvim::ex_eval::discard_current_exception;
use crate::src::nvim::ex_getln::{
    compute_cmdrow, curbuf_locked, getcmdline, getexline, text_locked, text_locked_msg,
    vim_strsave_fnameescape,
};
use crate::src::nvim::file_search::grab_file_name;
use crate::src::nvim::fileio::check_timestamps;
use crate::src::nvim::fold::{
    clearFolding, closeFold, closeFoldRecurse, deleteFold, foldAdjustVisual, foldCheckClose,
    foldManualAllowed, foldMoveTo, foldOpenCursor, foldUpdateAfterInsert, foldmethodIsDiff,
    foldmethodIsManual, foldmethodIsMarker, getDeepestNesting, hasAnyFolding, hasFolding,
    newFoldLevel, openFold, openFoldRecurse,
};
use crate::src::nvim::getchar::{
    AppendCharToRedobuff, AppendNumberToRedobuff, AppendToRedobuff, ResetRedobuff, beep_flush,
    char_avail, getcmdkeycmd, gotchars_ignore, ins_char_typebuf, map_execute_lua, paste_repeat,
    plain_vgetc, readbuf1_empty, safe_vgetc, start_redo, stuff_empty, stuffReadbuff,
    stuffcharReadbuff, stuffnumReadbuff, typebuf_maplen, typebuf_typed, ungetchars, vgetc, vpeekc,
    vungetc,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{grid_line_flush, grid_line_puts, grid_line_start};
use crate::src::nvim::help::ex_help;
use crate::src::nvim::highlight::win_hl_attr;
use crate::src::nvim::keycodes::simplify_key;
use crate::src::nvim::main::{
    KeyStuffed, KeyTyped, Rows, State, VIsual, VIsual_active, VIsual_mode, VIsual_reselect,
    VIsual_select, VIsual_select_exclu_adj, VIsual_select_reg, allow_keys, arrow_used, cb_flags,
    clear_cmdline, cmdwin_result, cmdwin_type, curbuf, curtab, curwin, did_check_timestamps,
    did_cursorhold, did_emsg, did_syncbind, did_throw, did_wait_return, diff_need_scrollbind,
    do_redraw, e_modifiable, e_noident, empty_string_option, emsg_off, emsg_on_display,
    emsg_silent, ex_normal_busy, exmode_active, fdo_flags, finish_op, firstwin, g_tag_at_cursor,
    global_busy, got_int, hl_attr_active, in_assert_fails, ins_at_eol, jop_flags, keep_msg,
    keep_msg_hl_id, km_startsel, km_stopsel, langmap_mapchar, last_cursormoved,
    last_cursormoved_win, may_garbage_collect, mod_mask, mode_displayed, motion_force,
    mouse_dragging, msg_col, msg_didany, msg_didout, msg_grid_adj, msg_hist_off, msg_nowait,
    msg_scroll, msg_silent, must_redraw, need_check_timestamps, need_fileinfo, need_wait_return,
    no_hlsearch, no_mapping, no_smartcase, no_u_sync, no_zero_mapping, opcount, p_ch, p_cpo, p_hls,
    p_kp, p_langmap, p_lrm, p_sbo, p_sbr, p_sc, p_scs, p_sel, p_slm, p_sloc, p_smd, p_sta, p_tm,
    p_to, p_ttm, p_ws, p_ww, quit_more, redraw_cmdline, redraw_mode, redraw_tabline, reg_executing,
    reg_recorded, reg_recording, resel_VIsual_line_count, resel_VIsual_mode, resel_VIsual_vcol,
    restart_VIsual_select, restart_edit, sc_col, showcmd_buf, skip_redraw, time_fd,
    typebuf_was_empty, vgetc_busy, vgetc_char, vgetc_mod_mask,
};
use crate::src::nvim::mapping::{add_map, langmap_adjust_mb};
use crate::src::nvim::mark::{
    checkpcmark, get_changelist, get_jumplist, getnextmark, mark_get, mark_mb_adjustpos,
    mark_move_to, pos_to_mark, setmark, setpcmark,
};
use crate::src::nvim::mbyte::{
    mb_adjust_cursor, mb_charlen, mb_check_adjust_col, mb_get_class, mb_prevptr, show_utf8,
    utf_char2bytes, utf_char2len, utf_find_illegal, utf_head_off, utf_iscomposing, utf_ptr2cells,
    utf_ptr2char, utf8len_tab, utfc_ptr2len,
};
use crate::src::nvim::memline::{
    goto_byte, inc, ml_delete_flags, ml_get, ml_get_buf, ml_get_len, ml_get_pos,
};
use crate::src::nvim::memory::{strequal, xfree, xmalloc, xmemdupz, xrealloc, xstrdup};
use crate::src::nvim::message::{
    emsg, may_clear_sb_text, messaging, msg, msg_delay, msg_ext_set_trigger, msg_grid_validate,
    show_sb_text, wait_return,
};
use crate::src::nvim::mouse::{do_mouse, nv_mouse, nv_mousescroll, setmouse};
use crate::src::nvim::r#move::{
    adjust_skipcol, changed_window_setting, cursor_correct, do_check_cursorbind, pagescroll,
    scroll_cursor_bot, scroll_cursor_halfway, scroll_cursor_top, scroll_redraw, scrolldown,
    scrollup, sms_marker_overlap, update_curswant, update_curswant_force, update_topline,
    validate_botline_win, validate_cheight, validate_cursor, validate_virtcol, win_col_off,
    win_col_off2,
};
use crate::src::nvim::ops::{
    adjust_cursor_eol, clear_oparg, cursor_pos_info, do_join, do_pending_operator,
    get_extra_op_char, get_op_char, get_op_type, op_addsub, op_is_change, swapchar,
};
use crate::src::nvim::option::{
    get_showbreak_value, get_sidescrolloff_value, get_ve_flags, magic_isset, shortmess,
};
use crate::src::nvim::options::{
    kOptBoFlagEsc, kOptCbFlagUnnamed, kOptCbFlagUnnamedplus, kOptFdoFlagAll, kOptFdoFlagBlock,
    kOptFdoFlagHor, kOptFdoFlagJump, kOptFdoFlagMark, kOptFdoFlagPercent, kOptFdoFlagSearch,
    kOptJopFlagView, kOptVeFlagAll, kOptVeFlagBlock, kOptVeFlagOnemore,
};
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, __ctype_b_loc, gettext, memmove, memset, qsort, snprintf, strcat, strchr,
    strcmp, strcpy, strlen, time,
};
use crate::src::nvim::plines::{
    getvcol, getvcols, getvvcol, linetabsize, plines_m_win_fill, plines_win, win_get_fill,
};
use crate::src::nvim::pos::{clearpos, equalpos, lt};
use crate::src::nvim::profile::{time_finish, time_msg};
use crate::src::nvim::quickfix::qf_view_result;
use crate::src::nvim::register::{
    copy_register, do_execreg, do_put, do_record, free_register, get_default_register_name,
    get_expr_register, valid_yank_reg,
};
use crate::src::nvim::search::{
    current_search, do_search, find_pattern_in_path, findmatch, findmatchlimit, reset_search_dir,
    searchc, searchit,
};
use crate::src::nvim::spell::spell_move_to;
use crate::src::nvim::spellfile::spell_add_word;
use crate::src::nvim::spellsuggest::spell_suggest;
use crate::src::nvim::state::{
    get_real_state, may_trigger_modechanged, may_trigger_safestate, state_enter,
    state_handle_k_event, state_no_longer_safe, virtual_active,
};
use crate::src::nvim::statusline::{draw_tabline, win_redr_status};
use crate::src::nvim::strings::{vim_strchr, vim_strsave_shellescape, xstrnsave};
use crate::src::nvim::syntax::syn_stack_free_all;
use crate::src::nvim::tag::do_tag;
use crate::src::nvim::terminal::terminal_check_refresh;
use crate::src::nvim::textformat::{auto_format, has_format_option};
use crate::src::nvim::textobject::{
    bck_word, bckend_word, current_block, current_par, current_quote, current_sent,
    current_tagblock, current_word, end_word, findpar, findsent, fwd_word,
};
use crate::src::nvim::types::{
    Array, Direction, GraphemeState, Integer, MarkGet, MarkMove, MarkMoveRes, MotionType, Object,
    ObjectType, OptInt, SpellAddType, UIExtension, VimState, VimVarIndex, auto_event, buf_T,
    cmdarg_T, colnr_T, dict_T, exarg_T, fmark_T, getf_values, hlf_T, int16_t, int64_t, key_extra,
    linenr_T, object, object_data as C2Rust_Unnamed_0, oparg_T, pos_T, proftime_T, ptrdiff_t,
    searchit_arg_T, size_t, smt_T, state_check_callback, state_execute_callback, time_t, uint8_t,
    uint16_t, uint64_t, win_T, yankreg_T,
};
use crate::src::nvim::ui::{
    ui_call_msg_showcmd, ui_cursor_shape, ui_cursor_shape_no_check_conceal, ui_flush, ui_has,
    vim_beep,
};
use crate::src::nvim::undo::{
    anyBufIsChanged, curbufIsChanged, u_clearline, u_redo, u_save, u_save_cursor, u_savesub,
    u_undo, u_undoline, undo_time,
};
use crate::src::nvim::window::{
    check_can_set_curbuf_disabled, do_window, goto_tabpage, goto_tabpage_lastused,
    may_make_initial_scroll_size_snapshot, may_trigger_win_scrolled_resized, set_fraction,
    win_setheight,
};
use core::ffi::{CStr, c_char, c_int, c_uint, c_ulong, c_ushort, c_void};

mod state;
pub use self::state::*;
mod dispatch;
pub use self::dispatch::*;
mod showcmd;
pub use self::showcmd::*;
mod visual;
pub use self::visual::*;
mod ident;
pub use self::ident::*;
mod motion;
pub use self::motion::*;
mod search;
pub(crate) use self::search::*;
mod brackets;
pub(crate) use self::brackets::*;
mod scroll;
pub use self::scroll::*;
mod edit;
pub use self::edit::*;
mod operator;
pub(crate) use self::operator::*;
mod gcmd;
pub use self::gcmd::*;
mod misc;
pub(crate) use self::misc::*;
pub type C2Rust_Unnamed = c_uint;
pub const _ISlower: C2Rust_Unnamed = 512;
pub const _ISupper: C2Rust_Unnamed = 256;
pub const kObjectTypeArray: ObjectType = 5;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeNil: ObjectType = 0;
pub type C2Rust_Unnamed_14 = c_uint;
pub const MAXLNUM: C2Rust_Unnamed_14 = 2147483647;
pub type C2Rust_Unnamed_15 = c_uint;
pub const MAXCOL: C2Rust_Unnamed_15 = 2147483647;
pub const HLF_MSG: hlf_T = 63;
pub const HLF_LC: hlf_T = 9;
pub const HLF_L: hlf_T = 8;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const kMarkChangedCursor: MarkMoveRes = 32;
pub const kMarkChangedLine: MarkMoveRes = 16;
pub const kMarkSwitchedBuf: MarkMoveRes = 4;
pub const kMarkMoveFailed: MarkMoveRes = 2;
pub const kMarkMoveSuccess: MarkMoveRes = 1;
pub const kMarkJumpList: MarkMove = 16;
pub const kMarkSetView: MarkMove = 8;
pub const KMarkNoContext: MarkMove = 4;
pub const kMarkContext: MarkMove = 2;
pub const kMarkBeginLine: MarkMove = 1;
pub const kMarkAll: MarkGet = 1;
pub const EVENT_TEXTCHANGED: auto_event = 124;
pub const EVENT_CURSORMOVED: auto_event = 39;
pub const EVENT_BUFMODIFIEDSET: auto_event = 8;
pub const GETF_ALT: getf_values = 2;
pub const GETF_SETMARK: getf_values = 1;
pub type C2Rust_Unnamed_17 = c_uint;
pub const OPENLINE_DO_COM: C2Rust_Unnamed_17 = 2;
pub type C2Rust_Unnamed_23 = c_uint;
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_23 = 83;
pub const SHM_FILEINFO: C2Rust_Unnamed_23 = 70;
pub type C2Rust_Unnamed_24 = c_int;
pub const HIST_SEARCH: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_25 = c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_25 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_25 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_25 = 35;
pub const UPD_INVERTED: C2Rust_Unnamed_25 = 20;
pub const UPD_VALID: C2Rust_Unnamed_25 = 10;
pub type C2Rust_Unnamed_26 = c_uint;
pub const BL_FIX: C2Rust_Unnamed_26 = 4;
pub const BL_SOL: C2Rust_Unnamed_26 = 2;
pub const BL_WHITE: C2Rust_Unnamed_26 = 1;
pub const VV_OP: VimVarIndex = 55;
pub const kUIMessages: UIExtension = 4;
pub type C2Rust_Unnamed_27 = c_uint;
pub const ECMD_HIDE: C2Rust_Unnamed_27 = 1;
pub type C2Rust_Unnamed_28 = c_int;
pub const ECMD_LAST: C2Rust_Unnamed_28 = -1;
pub type C2Rust_Unnamed_29 = c_uint;
pub const DOCMD_KEEPLINE: C2Rust_Unnamed_29 = 32;
pub type C2Rust_Unnamed_30 = c_uint;
pub const VSE_NONE: C2Rust_Unnamed_30 = 0;
pub type C2Rust_Unnamed_31 = c_uint;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_31 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_31 = 288;
pub const MODE_REPLACE: C2Rust_Unnamed_31 = 272;
pub const MODE_TERMINAL: C2Rust_Unnamed_31 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_31 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_31 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_31 = 16;
pub const MODE_NORMAL: C2Rust_Unnamed_31 = 1;
pub const KE_COMMAND: key_extra = 104;
pub const KE_LUA: key_extra = 103;
pub const KE_EVENT: key_extra = 102;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_NOP: key_extra = 97;
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
pub const KE_KDEL: key_extra = 80;
pub const KE_KINS: key_extra = 79;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_LEFTRELEASE_NM: key_extra = 70;
pub const KE_LEFTMOUSE_NM: key_extra = 69;
pub const KE_XF1: key_extra = 57;
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
pub const KE_S_DOWN: key_extra = 5;
pub const KE_S_UP: key_extra = 4;
pub type C2Rust_Unnamed_32 = c_uint;
pub const ML_DEL_MESSAGE: C2Rust_Unnamed_32 = 1;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_33 = c_uint;
pub const CA_NO_ADJ_OP_END: C2Rust_Unnamed_33 = 2;
pub const CA_COMMAND_BUSY: C2Rust_Unnamed_33 = 1;
pub type C2Rust_Unnamed_34 = c_int;
pub const REPLACE_NL_NCHAR: C2Rust_Unnamed_34 = -2;
pub const REPLACE_CR_NCHAR: C2Rust_Unnamed_34 = -1;
pub type C2Rust_Unnamed_35 = c_uint;
pub const SHOWCMD_COLS: C2Rust_Unnamed_35 = 10;
pub type C2Rust_Unnamed_36 = c_uint;
pub const SHOWCMD_BUFLEN: C2Rust_Unnamed_36 = 41;
pub type C2Rust_Unnamed_37 = c_int;
pub const MSCR_RIGHT: C2Rust_Unnamed_37 = -2;
pub const MSCR_LEFT: C2Rust_Unnamed_37 = -1;
pub const MSCR_UP: C2Rust_Unnamed_37 = 1;
pub const MSCR_DOWN: C2Rust_Unnamed_37 = 0;
pub type C2Rust_Unnamed_38 = c_uint;
pub const FIND_EVAL: C2Rust_Unnamed_38 = 4;
pub const FIND_STRING: C2Rust_Unnamed_38 = 2;
pub const FIND_IDENT: C2Rust_Unnamed_38 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct nv_cmd {
    pub cmd_char: c_int,
    pub cmd_func: nv_func_T,
    pub cmd_flags: uint16_t,
    pub cmd_arg: int16_t,
}
pub type nv_func_T = Option<unsafe extern "C" fn(*mut cmdarg_T) -> ()>;
pub const OP_NOP: C2Rust_Unnamed_40 = 0;
pub const OP_YANK: C2Rust_Unnamed_40 = 2;
pub const OP_RSHIFT: C2Rust_Unnamed_40 = 5;
pub const OP_LSHIFT: C2Rust_Unnamed_40 = 4;
pub const OP_DELETE: C2Rust_Unnamed_40 = 1;
pub const PUT_LINE_FORWARD: C2Rust_Unnamed_39 = 32;
pub const PUT_LINE_SPLIT: C2Rust_Unnamed_39 = 16;
pub const PUT_LINE: C2Rust_Unnamed_39 = 8;
pub const PUT_BLOCK_INNER: C2Rust_Unnamed_39 = 64;
pub const PUT_CURSEND: C2Rust_Unnamed_39 = 2;
pub const PUT_FIXINDENT: C2Rust_Unnamed_39 = 1;
pub const SEARCH_START: C2Rust_Unnamed_43 = 256;
pub const FM_FORWARD: C2Rust_Unnamed_44 = 2;
pub const RE_LAST: C2Rust_Unnamed_45 = 2;
pub const SEARCH_MSG: C2Rust_Unnamed_43 = 12;
pub const SEARCH_ECHO: C2Rust_Unnamed_43 = 2;
pub const SEARCH_OPT: C2Rust_Unnamed_43 = 16;
pub const OP_CHANGE: C2Rust_Unnamed_40 = 3;
pub const OP_NR_SUB: C2Rust_Unnamed_40 = 29;
pub const OP_NR_ADD: C2Rust_Unnamed_40 = 28;
pub const OP_TILDE: C2Rust_Unnamed_40 = 7;
pub const SPELL_ADD_BAD: SpellAddType = 1;
pub const SPELL_ADD_GOOD: SpellAddType = 0;
pub const SMT_RARE: smt_T = 2;
pub const SMT_BAD: smt_T = 1;
pub const SMT_ALL: smt_T = 0;
pub const OP_FOLD: C2Rust_Unnamed_40 = 19;
pub const OP_LOWER: C2Rust_Unnamed_40 = 12;
pub const OP_FORMAT: C2Rust_Unnamed_40 = 9;
pub const SEARCH_MARK: C2Rust_Unnamed_43 = 512;
pub const FM_BACKWARD: C2Rust_Unnamed_44 = 1;
pub const ACTION_GOTO: C2Rust_Unnamed_42 = 2;
pub const ACTION_SHOW: C2Rust_Unnamed_42 = 1;
pub const ACTION_SHOW_ALL: C2Rust_Unnamed_42 = 4;
pub const FIND_ANY: C2Rust_Unnamed_41 = 1;
pub const FIND_DEFINE: C2Rust_Unnamed_41 = 2;
pub const OP_UPPER: C2Rust_Unnamed_40 = 11;
pub const SEARCH_REV: C2Rust_Unnamed_43 = 1;
pub const OP_ROT13: C2Rust_Unnamed_40 = 15;
pub const DT_POP: C2Rust_Unnamed_46 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct NormalState {
    pub state: VimState,
    pub command_finished: bool,
    pub ctrl_w: bool,
    pub need_flushbuf: bool,
    pub set_prevcount: bool,
    pub previous_got_int: bool,
    pub cmdwin: bool,
    pub noexmode: bool,
    pub toplevel: bool,
    pub oa: oparg_T,
    pub ca: cmdarg_T,
    pub mapped_len: c_int,
    pub old_mapped_len: c_int,
    pub idx: c_int,
    pub c: c_int,
    pub old_col: c_int,
    pub old_pos: pos_T,
}
pub const OP_COLON: C2Rust_Unnamed_40 = 10;
pub type C2Rust_Unnamed_39 = c_uint;
pub type C2Rust_Unnamed_40 = c_uint;
pub type C2Rust_Unnamed_41 = c_uint;
pub type C2Rust_Unnamed_42 = c_uint;
pub type C2Rust_Unnamed_43 = c_uint;
pub type C2Rust_Unnamed_44 = c_uint;
pub type C2Rust_Unnamed_45 = c_uint;
pub type C2Rust_Unnamed_46 = c_uint;
pub const NULL: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const ML_EMPTY: c_int = 0x1 as c_int;
pub const OK: c_int = 1 as c_int;
pub const FAIL: c_int = 0 as c_int;
pub const NUL: c_int = '\0' as c_int;
pub const TAB: c_int = 9;
pub const NL: c_int = '\n' as c_int;
pub const CAR: c_int = '\r' as c_int;
pub const ESC: c_int = '\u{1b}' as c_int;
pub const DEL: c_int = 0x7f as c_int;
pub const POUND: c_int = 0xa3 as c_int;
pub const Ctrl_A: c_int = 1 as c_int;
pub const Ctrl_B: c_int = 2 as c_int;
pub const Ctrl_C: c_int = 3 as c_int;
pub const Ctrl_D: c_int = 4 as c_int;
pub const Ctrl_E: c_int = 5 as c_int;
pub const Ctrl_F: c_int = 6 as c_int;
pub const Ctrl_G: c_int = 7;
pub const Ctrl_H: c_int = 8;
pub const Ctrl_I: c_int = 9 as c_int;
pub const Ctrl_K: c_int = 11 as c_int;
pub const Ctrl_L: c_int = 12 as c_int;
pub const Ctrl_N: c_int = 14 as c_int;
pub const Ctrl_O: c_int = 15 as c_int;
pub const Ctrl_P: c_int = 16 as c_int;
pub const Ctrl_Q: c_int = 17 as c_int;
pub const Ctrl_R: c_int = 18 as c_int;
pub const Ctrl_S: c_int = 19 as c_int;
pub const Ctrl_T: c_int = 20 as c_int;
pub const Ctrl_U: c_int = 21 as c_int;
pub const Ctrl_V: c_int = 22 as c_int;
pub const Ctrl_W: c_int = 23 as c_int;
pub const Ctrl_X: c_int = 24;
pub const Ctrl_Y: c_int = 25 as c_int;
pub const Ctrl_Z: c_int = 26 as c_int;
pub const Ctrl_BSL: c_int = 28 as c_int;
pub const Ctrl_RSB: c_int = 29 as c_int;
pub const Ctrl_HAT: c_int = 30 as c_int;
pub const Ctrl__: c_int = 31 as c_int;
pub const FO_OPEN_COMS: c_int = 'o' as c_int;
pub const CPO_DIGRAPH: c_int = 'D' as c_int;
pub const CPO_CHANGEW: c_int = '_' as c_int;
pub const VALID_WCOL: c_int = 0x2 as c_int;
pub const VALID_CROW: c_int = 0x10 as c_int;
pub const B_IMODE_LMAP: c_int = 1 as c_int;
pub const K_ZERO: c_int = -(255 as c_int + (('X' as c_int) << 8 as c_int));
pub const K_UP: c_int = -30059;
pub const K_DOWN: c_int = -25707;
pub const K_LEFT: c_int = -('k' as c_int + (('l' as c_int) << 8 as c_int));
pub const K_RIGHT: c_int = -('k' as c_int + (('r' as c_int) << 8 as c_int));
pub const K_S_LEFT: c_int = -('#' as c_int + (('4' as c_int) << 8 as c_int));
pub const K_S_RIGHT: c_int = -('%' as c_int + (('i' as c_int) << 8 as c_int));
pub const K_S_HOME: c_int = -('#' as c_int + (('2' as c_int) << 8 as c_int));
pub const K_S_END: c_int = -('*' as c_int + (('7' as c_int) << 8 as c_int));
pub const K_F1: c_int = -('k' as c_int + (('1' as c_int) << 8 as c_int));
pub const K_HELP: c_int = -('%' as c_int + (('1' as c_int) << 8 as c_int));
pub const K_UNDO: c_int = -('&' as c_int + (('8' as c_int) << 8 as c_int));
pub const K_BS: c_int = -25195;
pub const K_INS: c_int = -('k' as c_int + (('I' as c_int) << 8 as c_int));
pub const K_DEL: c_int = -('k' as c_int + (('D' as c_int) << 8 as c_int));
pub const K_HOME: c_int = -26731;
pub const K_KHOME: c_int = -12619;
pub const K_END: c_int = -('@' as c_int + (('7' as c_int) << 8 as c_int));
pub const K_KEND: c_int = -('K' as c_int + (('4' as c_int) << 8 as c_int));
pub const K_PAGEUP: c_int = -('k' as c_int + (('P' as c_int) << 8 as c_int));
pub const K_PAGEDOWN: c_int = -('k' as c_int + (('N' as c_int) << 8 as c_int));
pub const K_KPAGEUP: c_int = -('K' as c_int + (('3' as c_int) << 8 as c_int));
pub const K_KPAGEDOWN: c_int = -('K' as c_int + (('5' as c_int) << 8 as c_int));
pub const K_KENTER: c_int = -16715;
pub const K_PASTE_START: c_int = -('P' as c_int + (('S' as c_int) << 8 as c_int));
pub const K_SELECT: c_int = -(245 as c_int + (('X' as c_int) << 8 as c_int));
pub const MOD_MASK_SHIFT: c_int = 0x2 as c_int;
pub const MOD_MASK_CTRL: c_int = 0x4 as c_int;
pub const GRAPHEME_STATE_INIT: c_int = 0 as c_int;
static VIsual_mode_orig: GlobalCell<c_int> = GlobalCell::new(NUL);
const e_changelist_is_empty: &CStr = c"E664: Changelist is empty";
const e_cmdline_window_already_open: &CStr = c"E1292: Command-line window is already open";
pub const NV_NCH: c_int = 0x1 as c_int;
pub const NV_NCH_NOP: c_int = 0x2 as c_int | NV_NCH;
pub const NV_NCH_ALW: c_int = 0x4 as c_int | NV_NCH;
pub const NV_LANG: c_int = 0x8 as c_int;
pub const NV_SS: c_int = 0x10 as c_int;
pub const NV_SSS: c_int = 0x20 as c_int;
pub const NV_STS: c_int = 0x40 as c_int;
pub const NV_RL: c_int = 0x80 as c_int;
pub const NV_KEEPREG: c_int = 0x100 as c_int;
pub const NV_NCW: c_int = 0x200 as c_int;
/// One row of [`nv_cmds`].
///
/// The flags and the argument are written as the `c_int` constants that name
/// them and narrowed here, so a row reads as the four things it is rather than
/// as four casts.
const fn cmd(cmd_char: c_int, cmd_func: nv_func_T, cmd_flags: c_int, cmd_arg: c_int) -> nv_cmd {
    nv_cmd {
        cmd_char,
        cmd_func,
        cmd_flags: cmd_flags as uint16_t,
        cmd_arg: cmd_arg as int16_t,
    }
}

static nv_cmds: GlobalCell<[nv_cmd; 188]> = GlobalCell::new([
    cmd(NUL, Some(nv_error), 0, 0),
    cmd(Ctrl_A, Some(nv_addsub), 0, 0),
    cmd(Ctrl_B, Some(nv_page), NV_STS, BACKWARD as c_int),
    cmd(Ctrl_C, Some(nv_esc), 0, true_0),
    cmd(Ctrl_D, Some(nv_halfpage), 0, 0),
    cmd(Ctrl_E, Some(nv_scroll_line), 0, true_0),
    cmd(Ctrl_F, Some(nv_page), NV_STS, FORWARD as c_int),
    cmd(Ctrl_G, Some(nv_ctrlg), 0, 0),
    cmd(Ctrl_H, Some(nv_ctrlh), 0, 0),
    cmd(Ctrl_I, Some(nv_pcmark), 0, 0),
    cmd(NL, Some(nv_down), 0, false_0),
    cmd(Ctrl_K, Some(nv_error), 0, 0),
    cmd(Ctrl_L, Some(nv_clear), 0, 0),
    cmd(CAR, Some(nv_down), 0, true_0),
    cmd(Ctrl_N, Some(nv_down), NV_STS, false_0),
    cmd(Ctrl_O, Some(nv_ctrlo), 0, 0),
    cmd(Ctrl_P, Some(nv_up), NV_STS, false_0),
    cmd(Ctrl_Q, Some(nv_visual), 0, false_0),
    cmd(Ctrl_R, Some(nv_redo_or_register), 0, 0),
    cmd(Ctrl_S, Some(nv_ignore), 0, 0),
    cmd(Ctrl_T, Some(nv_tagpop), NV_NCW, 0),
    cmd(Ctrl_U, Some(nv_halfpage), 0, 0),
    cmd(Ctrl_V, Some(nv_visual), 0, false_0),
    cmd('V' as c_int, Some(nv_visual), 0, false_0),
    cmd('v' as c_int, Some(nv_visual), 0, false_0),
    cmd(Ctrl_W, Some(nv_window), 0, 0),
    cmd(Ctrl_X, Some(nv_addsub), 0, 0),
    cmd(Ctrl_Y, Some(nv_scroll_line), 0, false_0),
    cmd(Ctrl_Z, Some(nv_suspend), 0, 0),
    cmd(ESC, Some(nv_esc), 0, false_0),
    cmd(Ctrl_BSL, Some(nv_normal), NV_NCH_ALW, 0),
    cmd(Ctrl_RSB, Some(nv_ident), NV_NCW, 0),
    cmd(Ctrl_HAT, Some(nv_hat), NV_NCW, 0),
    cmd(Ctrl__, Some(nv_error), 0, 0),
    cmd(' ' as c_int, Some(nv_right), 0, 0),
    cmd('!' as c_int, Some(nv_operator), 0, 0),
    cmd('"' as c_int, Some(nv_regname), NV_NCH_NOP | NV_KEEPREG, 0),
    cmd('#' as c_int, Some(nv_ident), 0, 0),
    cmd('$' as c_int, Some(nv_dollar), 0, 0),
    cmd('%' as c_int, Some(nv_percent), 0, 0),
    cmd('&' as c_int, Some(nv_optrans), 0, 0),
    cmd('\'' as c_int, Some(nv_gomark), NV_NCH_ALW, true_0),
    cmd('(' as c_int, Some(nv_brace), 0, BACKWARD as c_int),
    cmd(')' as c_int, Some(nv_brace), 0, FORWARD as c_int),
    cmd('*' as c_int, Some(nv_ident), 0, 0),
    cmd('+' as c_int, Some(nv_down), 0, true_0),
    cmd(',' as c_int, Some(nv_csearch), 0, true_0),
    cmd('-' as c_int, Some(nv_up), 0, true_0),
    cmd('.' as c_int, Some(nv_dot), NV_KEEPREG, 0),
    cmd('/' as c_int, Some(nv_search), 0, false_0),
    cmd('0' as c_int, Some(nv_beginline), 0, 0),
    cmd('1' as c_int, Some(nv_ignore), 0, 0),
    cmd('2' as c_int, Some(nv_ignore), 0, 0),
    cmd('3' as c_int, Some(nv_ignore), 0, 0),
    cmd('4' as c_int, Some(nv_ignore), 0, 0),
    cmd('5' as c_int, Some(nv_ignore), 0, 0),
    cmd('6' as c_int, Some(nv_ignore), 0, 0),
    cmd('7' as c_int, Some(nv_ignore), 0, 0),
    cmd('8' as c_int, Some(nv_ignore), 0, 0),
    cmd('9' as c_int, Some(nv_ignore), 0, 0),
    cmd(':' as c_int, Some(nv_colon), 0, 0),
    cmd(';' as c_int, Some(nv_csearch), 0, false_0),
    cmd('<' as c_int, Some(nv_operator), NV_RL, 0),
    cmd('=' as c_int, Some(nv_operator), 0, 0),
    cmd('>' as c_int, Some(nv_operator), NV_RL, 0),
    cmd('?' as c_int, Some(nv_search), 0, false_0),
    cmd('@' as c_int, Some(nv_at), NV_NCH_NOP, false_0),
    cmd('A' as c_int, Some(nv_edit), 0, 0),
    cmd('B' as c_int, Some(nv_bck_word), 0, 1),
    cmd('C' as c_int, Some(nv_abbrev), NV_KEEPREG, 0),
    cmd('D' as c_int, Some(nv_abbrev), NV_KEEPREG, 0),
    cmd('E' as c_int, Some(nv_wordcmd), 0, true_0),
    cmd(
        'F' as c_int,
        Some(nv_csearch),
        NV_NCH_ALW | NV_LANG,
        BACKWARD as c_int,
    ),
    cmd('G' as c_int, Some(nv_goto), 0, true_0),
    cmd('H' as c_int, Some(nv_scroll), 0, 0),
    cmd('I' as c_int, Some(nv_edit), 0, 0),
    cmd('J' as c_int, Some(nv_join), 0, 0),
    cmd('K' as c_int, Some(nv_ident), 0, 0),
    cmd('L' as c_int, Some(nv_scroll), 0, 0),
    cmd('M' as c_int, Some(nv_scroll), 0, 0),
    cmd('N' as c_int, Some(nv_next), 0, SEARCH_REV as c_int),
    cmd('O' as c_int, Some(nv_open), 0, 0),
    cmd('P' as c_int, Some(nv_put), 0, 0),
    cmd('Q' as c_int, Some(nv_regreplay), 0, 0),
    cmd('R' as c_int, Some(nv_Replace), 0, false_0),
    cmd('S' as c_int, Some(nv_subst), NV_KEEPREG, 0),
    cmd(
        'T' as c_int,
        Some(nv_csearch),
        NV_NCH_ALW | NV_LANG,
        BACKWARD as c_int,
    ),
    cmd('U' as c_int, Some(nv_Undo), 0, 0),
    cmd('W' as c_int, Some(nv_wordcmd), 0, true_0),
    cmd('X' as c_int, Some(nv_abbrev), NV_KEEPREG, 0),
    cmd('Y' as c_int, Some(nv_abbrev), NV_KEEPREG, 0),
    cmd('Z' as c_int, Some(nv_Zet), NV_NCH_NOP | NV_NCW, 0),
    cmd(
        '[' as c_int,
        Some(nv_brackets),
        NV_NCH_ALW,
        BACKWARD as c_int,
    ),
    cmd('\\' as c_int, Some(nv_error), 0, 0),
    cmd(
        ']' as c_int,
        Some(nv_brackets),
        NV_NCH_ALW,
        FORWARD as c_int,
    ),
    cmd(
        '^' as c_int,
        Some(nv_beginline),
        0,
        BL_WHITE as c_int | BL_FIX as c_int,
    ),
    cmd('_' as c_int, Some(nv_lineop), 0, 0),
    cmd('`' as c_int, Some(nv_gomark), NV_NCH_ALW, false_0),
    cmd('a' as c_int, Some(nv_edit), NV_NCH, 0),
    cmd('b' as c_int, Some(nv_bck_word), 0, 0),
    cmd('c' as c_int, Some(nv_operator), 0, 0),
    cmd('d' as c_int, Some(nv_operator), 0, 0),
    cmd('e' as c_int, Some(nv_wordcmd), 0, false_0),
    cmd(
        'f' as c_int,
        Some(nv_csearch),
        NV_NCH_ALW | NV_LANG,
        FORWARD as c_int,
    ),
    cmd('g' as c_int, Some(nv_g_cmd), NV_NCH_ALW, false_0),
    cmd('h' as c_int, Some(nv_left), NV_RL, 0),
    cmd('i' as c_int, Some(nv_edit), NV_NCH, 0),
    cmd('j' as c_int, Some(nv_down), 0, false_0),
    cmd('k' as c_int, Some(nv_up), 0, false_0),
    cmd('l' as c_int, Some(nv_right), NV_RL, 0),
    cmd('m' as c_int, Some(nv_mark), NV_NCH_NOP, 0),
    cmd('n' as c_int, Some(nv_next), 0, 0),
    cmd('o' as c_int, Some(nv_open), 0, 0),
    cmd('p' as c_int, Some(nv_put), 0, 0),
    cmd('q' as c_int, Some(nv_record), NV_NCH, 0),
    cmd('r' as c_int, Some(nv_replace), NV_NCH_NOP | NV_LANG, 0),
    cmd('s' as c_int, Some(nv_subst), NV_KEEPREG, 0),
    cmd(
        't' as c_int,
        Some(nv_csearch),
        NV_NCH_ALW | NV_LANG,
        FORWARD as c_int,
    ),
    cmd('u' as c_int, Some(nv_undo), 0, 0),
    cmd('w' as c_int, Some(nv_wordcmd), 0, false_0),
    cmd('x' as c_int, Some(nv_abbrev), NV_KEEPREG, 0),
    cmd('y' as c_int, Some(nv_operator), 0, 0),
    cmd('z' as c_int, Some(nv_zet), NV_NCH_ALW, 0),
    cmd('{' as c_int, Some(nv_findpar), 0, BACKWARD as c_int),
    cmd('|' as c_int, Some(nv_pipe), 0, 0),
    cmd('}' as c_int, Some(nv_findpar), 0, FORWARD as c_int),
    cmd('~' as c_int, Some(nv_tilde), 0, 0),
    cmd(POUND, Some(nv_ident), 0, 0),
    cmd(
        -(253 as c_int + ((KE_MOUSEUP as c_int) << 8 as c_int)),
        Some(nv_mousescroll),
        0,
        MSCR_UP as c_int,
    ),
    cmd(
        -(253 as c_int + ((KE_MOUSEDOWN as c_int) << 8 as c_int)),
        Some(nv_mousescroll),
        0,
        MSCR_DOWN as c_int,
    ),
    cmd(
        -(253 as c_int + ((KE_MOUSELEFT as c_int) << 8 as c_int)),
        Some(nv_mousescroll),
        0,
        MSCR_LEFT as c_int,
    ),
    cmd(
        -(253 as c_int + ((KE_MOUSERIGHT as c_int) << 8 as c_int)),
        Some(nv_mousescroll),
        0,
        MSCR_RIGHT as c_int,
    ),
    cmd(
        -(253 as c_int + ((KE_LEFTMOUSE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_LEFTMOUSE_NM as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_LEFTDRAG as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_LEFTRELEASE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_LEFTRELEASE_NM as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_MOUSEMOVE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_MIDDLEMOUSE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_MIDDLEDRAG as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_MIDDLERELEASE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_RIGHTMOUSE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_RIGHTDRAG as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_RIGHTRELEASE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X1MOUSE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X1DRAG as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X1RELEASE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X2MOUSE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X2DRAG as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_X2RELEASE as c_int) << 8 as c_int)),
        Some(nv_mouse),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int)),
        Some(nv_ignore),
        NV_KEEPREG,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_NOP as c_int) << 8 as c_int)),
        Some(nv_nop),
        0,
        0,
    ),
    cmd(K_INS, Some(nv_edit), 0, 0),
    cmd(
        -(253 as c_int + ((KE_KINS as c_int) << 8 as c_int)),
        Some(nv_edit),
        0,
        0,
    ),
    cmd(K_BS, Some(nv_ctrlh), 0, 0),
    cmd(K_UP, Some(nv_up), NV_SSS | NV_STS, false_0),
    cmd(
        -(253 as c_int + ((KE_S_UP as c_int) << 8 as c_int)),
        Some(nv_page),
        NV_SS,
        BACKWARD as c_int,
    ),
    cmd(K_DOWN, Some(nv_down), NV_SSS | NV_STS, false_0),
    cmd(
        -(253 as c_int + ((KE_S_DOWN as c_int) << 8 as c_int)),
        Some(nv_page),
        NV_SS,
        FORWARD as c_int,
    ),
    cmd(K_LEFT, Some(nv_left), NV_SSS | NV_STS | NV_RL, 0),
    cmd(K_S_LEFT, Some(nv_bck_word), NV_SS | NV_RL, 0),
    cmd(
        -(253 as c_int + ((KE_C_LEFT as c_int) << 8 as c_int)),
        Some(nv_bck_word),
        NV_SSS | NV_RL | NV_STS,
        1,
    ),
    cmd(K_RIGHT, Some(nv_right), NV_SSS | NV_STS | NV_RL, 0),
    cmd(K_S_RIGHT, Some(nv_wordcmd), NV_SS | NV_RL, false_0),
    cmd(
        -(253 as c_int + ((KE_C_RIGHT as c_int) << 8 as c_int)),
        Some(nv_wordcmd),
        NV_SSS | NV_RL | NV_STS,
        true_0,
    ),
    cmd(K_PAGEUP, Some(nv_page), NV_SSS | NV_STS, BACKWARD as c_int),
    cmd(K_KPAGEUP, Some(nv_page), NV_SSS | NV_STS, BACKWARD as c_int),
    cmd(K_PAGEDOWN, Some(nv_page), NV_SSS | NV_STS, FORWARD as c_int),
    cmd(
        K_KPAGEDOWN,
        Some(nv_page),
        NV_SSS | NV_STS,
        FORWARD as c_int,
    ),
    cmd(K_END, Some(nv_end), NV_SSS | NV_STS, false_0),
    cmd(K_KEND, Some(nv_end), NV_SSS | NV_STS, false_0),
    cmd(K_S_END, Some(nv_end), NV_SS, false_0),
    cmd(
        -(253 as c_int + ((KE_C_END as c_int) << 8 as c_int)),
        Some(nv_end),
        NV_SSS | NV_STS,
        true_0,
    ),
    cmd(K_HOME, Some(nv_home), NV_SSS | NV_STS, 0),
    cmd(K_KHOME, Some(nv_home), NV_SSS | NV_STS, 0),
    cmd(K_S_HOME, Some(nv_home), NV_SS, 0),
    cmd(
        -(253 as c_int + ((KE_C_HOME as c_int) << 8 as c_int)),
        Some(nv_goto),
        NV_SSS | NV_STS,
        false_0,
    ),
    cmd(K_DEL, Some(nv_abbrev), 0, 0),
    cmd(
        -(253 as c_int + ((KE_KDEL as c_int) << 8 as c_int)),
        Some(nv_abbrev),
        0,
        0,
    ),
    cmd(K_UNDO, Some(nv_kundo), 0, 0),
    cmd(K_HELP, Some(nv_help), NV_NCW, 0),
    cmd(K_F1, Some(nv_help), NV_NCW, 0),
    cmd(
        -(253 as c_int + ((KE_XF1 as c_int) << 8 as c_int)),
        Some(nv_help),
        NV_NCW,
        0,
    ),
    cmd(K_SELECT, Some(nv_select), 0, 0),
    cmd(K_PASTE_START, Some(nv_paste), NV_KEEPREG, 0),
    cmd(
        -(253 as c_int + ((KE_EVENT as c_int) << 8 as c_int)),
        Some(nv_event),
        NV_KEEPREG,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_COMMAND as c_int) << 8 as c_int)),
        Some(nv_colon),
        0,
        0,
    ),
    cmd(
        -(253 as c_int + ((KE_LUA as c_int) << 8 as c_int)),
        Some(nv_colon),
        0,
        0,
    ),
]);
pub const NV_CMDS_SIZE: usize = ::core::mem::size_of::<[nv_cmd; 188]>()
    .wrapping_div(::core::mem::size_of::<nv_cmd>())
    .wrapping_div(
        (::core::mem::size_of::<[nv_cmd; 188]>().wrapping_rem(::core::mem::size_of::<nv_cmd>())
            == 0) as c_int as usize,
    );
static nv_cmd_idx: GlobalCell<[int16_t; 188]> = GlobalCell::new([0; 188]);
static nv_max_linear: GlobalCell<c_int> = GlobalCell::new(0);
static current_oap: GlobalCell<*mut oparg_T> = GlobalCell::new(::core::ptr::null_mut::<oparg_T>());
static old_showcmd_buf: GlobalCell<[c_char; 41]> = GlobalCell::new([0; 41]);
static showcmd_is_clear: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
static showcmd_visual: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub const INT_MAX: c_int = __INT_MAX__;
pub const true_0: c_int = 1 as c_int;
pub const false_0: c_int = 0 as c_int;
pub const __INT_MAX__: c_int = 2147483647 as c_int;
