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
#[inline]
unsafe extern "C" fn normal_state_init(mut s: *mut NormalState) {
    memset(
        s as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<NormalState>(),
    );
    (*s).state.check =
        Some(normal_check as unsafe extern "C" fn(*mut VimState) -> c_int) as state_check_callback;
    (*s).state.execute = Some(normal_execute as unsafe extern "C" fn(*mut VimState, c_int) -> c_int)
        as state_execute_callback;
}
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
unsafe extern "C" fn nv_compare(mut s1: *const c_void, mut s2: *const c_void) -> c_int {
    let mut c1: c_int = (*nv_cmds.ptr())[*(s1 as *const int16_t) as usize].cmd_char;
    let mut c2: c_int = (*nv_cmds.ptr())[*(s2 as *const int16_t) as usize].cmd_char;
    if c1 < 0 as c_int {
        c1 = -c1;
    }
    if c2 < 0 as c_int {
        c2 = -c2;
    }
    return if c1 == c2 {
        0 as c_int
    } else if c1 > c2 {
        1 as c_int
    } else {
        -1 as c_int
    };
}
pub unsafe extern "C" fn init_normal_cmds() {
    '_c2rust_label: {
        if ::core::mem::size_of::<[nv_cmd; 188]>()
            .wrapping_div(::core::mem::size_of::<nv_cmd>())
            .wrapping_div(
                (::core::mem::size_of::<[nv_cmd; 188]>()
                    .wrapping_rem(::core::mem::size_of::<nv_cmd>())
                    == 0) as c_int as usize,
            )
            <= 32767 as usize
        {
        } else {
            __assert_fail(
                b"NV_CMDS_SIZE <= SHRT_MAX\0".as_ptr() as *const c_char,
                b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                390 as c_uint,
                b"void init_normal_cmds(void)\0".as_ptr() as *const c_char,
            );
        }
    };
    let mut i: int16_t = 0 as int16_t;
    while (i as c_int) < NV_CMDS_SIZE as int16_t as c_int {
        (*nv_cmd_idx.ptr())[i as usize] = i;
        i += 1;
    }
    qsort(
        nv_cmd_idx.ptr() as *mut c_void,
        NV_CMDS_SIZE,
        ::core::mem::size_of::<int16_t>(),
        Some(nv_compare as unsafe extern "C" fn(*const c_void, *const c_void) -> c_int),
    );
    let mut i_0: int16_t = 0;
    i_0 = 0 as int16_t;
    while (i_0 as c_int) < NV_CMDS_SIZE as int16_t as c_int {
        if i_0 as c_int != (*nv_cmds.ptr())[(*nv_cmd_idx.ptr())[i_0 as usize] as usize].cmd_char {
            break;
        }
        i_0 += 1;
    }
    nv_max_linear.set(i_0 as c_int - 1 as c_int);
}
unsafe extern "C" fn find_command(mut cmdchar: c_int) -> c_int {
    if cmdchar >= 0x100 as c_int {
        return -1 as c_int;
    }
    if cmdchar < 0 as c_int {
        cmdchar = -cmdchar;
    }
    '_c2rust_label: {
        if nv_max_linear.get()
            < ::core::mem::size_of::<[nv_cmd; 188]>()
                .wrapping_div(::core::mem::size_of::<nv_cmd>())
                .wrapping_div(
                    (::core::mem::size_of::<[nv_cmd; 188]>()
                        .wrapping_rem(::core::mem::size_of::<nv_cmd>())
                        == 0) as c_int as usize,
                ) as c_int
        {
        } else {
            __assert_fail(
                b"nv_max_linear < (int)NV_CMDS_SIZE\0".as_ptr() as *const c_char,
                b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                428 as c_uint,
                b"int find_command(int)\0".as_ptr() as *const c_char,
            );
        }
    };
    if cmdchar <= nv_max_linear.get() {
        return (*nv_cmd_idx.ptr())[cmdchar as usize] as c_int;
    }
    let mut bot: c_int = nv_max_linear.get() + 1 as c_int;
    let mut top: c_int = NV_CMDS_SIZE.wrapping_sub(1 as usize) as c_int;
    let mut idx: c_int = -1 as c_int;
    while bot <= top {
        let mut i: c_int = (top + bot) / 2 as c_int;
        let mut c: c_int = (*nv_cmds.ptr())[(*nv_cmd_idx.ptr())[i as usize] as usize].cmd_char;
        if c < 0 as c_int {
            c = -c;
        }
        if cmdchar == c {
            idx = (*nv_cmd_idx.ptr())[i as usize] as c_int;
            break;
        } else if cmdchar > c {
            bot = i + 1 as c_int;
        } else {
            top = i - 1 as c_int;
        }
    }
    return idx;
}
unsafe extern "C" fn check_text_locked(mut oap: *mut oparg_T) -> bool {
    if !text_locked() {
        return false_0 != 0;
    }
    if !oap.is_null() {
        clearopbeep(oap);
    }
    text_locked_msg();
    return true_0 != 0;
}
pub unsafe extern "C" fn check_text_or_curbuf_locked(mut oap: *mut oparg_T) -> bool {
    if check_text_locked(oap) {
        return true_0 != 0;
    }
    if !curbuf_locked() {
        return false_0 != 0;
    }
    if !oap.is_null() {
        clearop(oap);
    }
    return true_0 != 0;
}
static current_oap: GlobalCell<*mut oparg_T> = GlobalCell::new(::core::ptr::null_mut::<oparg_T>());
pub unsafe extern "C" fn op_pending() -> bool {
    return !(!(*current_oap.ptr()).is_null()
        && !finish_op.get()
        && (*current_oap.get()).prev_opcount == 0 as c_int
        && (*current_oap.get()).prev_count0 == 0 as c_int
        && (*current_oap.get()).op_type == OP_NOP as c_int
        && (*current_oap.get()).regname == NUL);
}
pub unsafe extern "C" fn normal_enter(mut cmdwin: bool, mut noexmode: bool) {
    let mut state: NormalState = NormalState {
        state: VimState {
            check: None,
            execute: None,
        },
        command_finished: false,
        ctrl_w: false,
        need_flushbuf: false,
        set_prevcount: false,
        previous_got_int: false,
        cmdwin: false,
        noexmode: false,
        toplevel: false,
        oa: oparg_T {
            op_type: 0,
            regname: 0,
            motion_type: kMTCharWise,
            motion_force: 0,
            use_reg_one: false,
            inclusive: false,
            end_adjusted: false,
            start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            end: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            cursor_start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            line_count: 0,
            empty: false,
            is_VIsual: false,
            start_vcol: 0,
            end_vcol: 0,
            prev_opcount: 0,
            prev_count0: 0,
            excl_tr_ws: false,
        },
        ca: cmdarg_T {
            oap: ::core::ptr::null_mut::<oparg_T>(),
            prechar: 0,
            cmdchar: 0,
            nchar: 0,
            nchar_composing: [0; 32],
            nchar_len: 0,
            extra_char: 0,
            opcount: 0,
            count0: 0,
            count1: 0,
            arg: 0,
            retval: 0,
            searchbuf: ::core::ptr::null_mut::<c_char>(),
        },
        mapped_len: 0,
        old_mapped_len: 0,
        idx: 0,
        c: 0,
        old_col: 0,
        old_pos: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
    };
    normal_state_init(&raw mut state);
    let mut prev_oap: *mut oparg_T = current_oap.get();
    current_oap.set(&raw mut state.oa);
    state.cmdwin = cmdwin;
    state.noexmode = noexmode;
    state.toplevel = (!cmdwin || cmdwin_result.get() == 0 as c_int) && !noexmode;
    state_enter(&raw mut state.state);
    current_oap.set(prev_oap);
}
unsafe extern "C" fn normal_prepare(mut s: *mut NormalState) {
    memset(
        &raw mut (*s).ca as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<cmdarg_T>(),
    );
    (*s).ca.oap = &raw mut (*s).oa;
    (*s).ca.opcount = opcount.get();
    let mut c: c_int = finish_op.get() as c_int;
    finish_op.set((*s).oa.op_type != OP_NOP as c_int);
    if finish_op.get() as c_int != c {
        ui_cursor_shape();
    }
    may_trigger_modechanged();
    (*s).set_prevcount = false_0 != 0;
    if !finish_op.get() && (*s).oa.regname == 0 {
        (*s).ca.opcount = 0 as c_int;
        (*s).set_prevcount = true_0 != 0;
    }
    if (*s).oa.prev_opcount > 0 as c_int || (*s).oa.prev_count0 > 0 as c_int {
        (*s).ca.opcount = (*s).oa.prev_opcount;
        (*s).ca.count0 = (*s).oa.prev_count0;
        (*s).oa.prev_opcount = 0 as c_int;
        (*s).oa.prev_count0 = 0 as c_int;
    }
    (*s).mapped_len = typebuf_maplen();
    State.set(MODE_NORMAL_BUSY as c_int);
    if (*s).toplevel as c_int != 0 && readbuf1_empty() as c_int != 0 {
        set_vcount_ca(&raw mut (*s).ca, &raw mut (*s).set_prevcount);
    }
}
unsafe extern "C" fn normal_handle_special_visual_command(mut s: *mut NormalState) -> bool {
    if km_stopsel.get() as c_int != 0
        && (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_STS != 0
        && mod_mask.get() & MOD_MASK_SHIFT == 0
    {
        end_visual_mode();
        redraw_curbuf_later(UPD_INVERTED as c_int);
    }
    if km_startsel.get() {
        if (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_SS != 0 {
            unshift_special(&raw mut (*s).ca);
            (*s).idx = find_command((*s).ca.cmdchar);
            if (*s).idx < 0 as c_int {
                clearopbeep(&raw mut (*s).oa);
                return true_0 != 0;
            }
        } else if (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_SSS != 0
            && mod_mask.get() & MOD_MASK_SHIFT != 0
        {
            (*mod_mask.ptr()) &= !MOD_MASK_SHIFT;
        }
    }
    return false_0 != 0;
}
unsafe extern "C" fn normal_need_additional_char(mut s: *mut NormalState) -> bool {
    let mut flags: c_int = (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int;
    let mut pending_op: bool = (*s).oa.op_type != OP_NOP as c_int;
    let mut cmdchar: c_int = (*s).ca.cmdchar;
    return flags & NV_NCH != 0
        && (flags & NV_NCH_NOP == NV_NCH_NOP && !pending_op
            || flags & NV_NCH_ALW == NV_NCH_ALW
            || cmdchar == 'q' as c_int
                && !pending_op
                && reg_recording.get() == 0 as c_int
                && reg_executing.get() == 0 as c_int
            || (cmdchar == 'a' as c_int || cmdchar == 'i' as c_int)
                && (pending_op as c_int != 0 || VIsual_active.get() as c_int != 0));
}
unsafe extern "C" fn normal_need_redraw_mode_message(mut s: *mut NormalState) -> bool {
    return (p_smd.get() != 0
        && msg_silent.get() == 0 as c_int
        && (restart_edit.get() != 0 as c_int
            || VIsual_active.get() as c_int != 0
                && (*s).old_pos.lnum == (*curwin.get()).w_cursor.lnum
                && (*s).old_pos.col == (*curwin.get()).w_cursor.col)
        && (clear_cmdline.get() as c_int != 0 || redraw_cmdline.get() as c_int != 0)
        && (msg_didout.get() as c_int != 0
            || msg_didany.get() as c_int != 0 && msg_scroll.get() != 0)
        && !msg_nowait.get()
        && KeyTyped.get() as c_int != 0
        || restart_edit.get() != 0 as c_int
            && !VIsual_active.get()
            && msg_scroll.get() != 0
            && emsg_on_display.get() as c_int != 0)
        && (*s).oa.regname == 0 as c_int
        && (*s).ca.retval & CA_COMMAND_BUSY as c_int == 0
        && stuff_empty() as c_int != 0
        && typebuf_typed() != 0
        && emsg_silent.get() == 0 as c_int
        && !in_assert_fails.get()
        && !did_wait_return.get()
        && (*s).oa.op_type == OP_NOP as c_int;
}
unsafe extern "C" fn normal_redraw_mode_message(mut _s: *mut NormalState) {
    let mut save_State: c_int = State.get();
    if restart_edit.get() != 0 as c_int {
        State.set(MODE_INSERT as c_int);
    }
    if must_redraw.get() != 0 && !(*keep_msg.ptr()).is_null() && !emsg_on_display.get() {
        let mut kmsg: *mut c_char = ::core::ptr::null_mut::<c_char>();
        kmsg = keep_msg.get();
        keep_msg.set(::core::ptr::null_mut::<c_char>());
        setcursor();
        update_screen();
        keep_msg.set(kmsg);
        kmsg = xstrdup(keep_msg.get());
        msg(kmsg, keep_msg_hl_id.get());
        xfree(kmsg as *mut c_void);
    }
    setcursor();
    ui_cursor_shape();
    ui_flush();
    if msg_scroll.get() != 0 || emsg_on_display.get() as c_int != 0 {
        msg_delay(1003 as uint64_t, true_0 != 0);
    }
    msg_delay(3003 as uint64_t, false_0 != 0);
    State.set(save_State);
    msg_scroll.set(false_0);
    emsg_on_display.set(false_0 != 0);
}
unsafe extern "C" fn normal_get_additional_char(mut s: *mut NormalState) {
    let mut cp: *mut c_int = ::core::ptr::null_mut::<c_int>();
    let mut repl: bool = false_0 != 0;
    let mut lit: bool = false_0 != 0;
    let mut lang: bool = false;
    (*no_mapping.ptr()) += 1;
    (*allow_keys.ptr()) += 1;
    did_cursorhold.set(true_0 != 0);
    if (*s).ca.cmdchar == 'g' as c_int {
        (*s).ca.nchar = plain_vgetc();
        if *p_langmap.get() as c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as c_int) as c_int
                } else {
                    KeyTyped.get() as c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && (*s).ca.nchar >= 0 as c_int
        {
            if (*s).ca.nchar < 256 as c_int {
                (*s).ca.nchar = (*langmap_mapchar.ptr())[(*s).ca.nchar as usize] as c_int;
            } else {
                (*s).ca.nchar = langmap_adjust_mb((*s).ca.nchar);
            }
        }
        (*s).need_flushbuf =
            (*s).need_flushbuf as c_int | add_to_showcmd((*s).ca.nchar) as c_int != 0;
        if (*s).ca.nchar == 'r' as c_int
            || (*s).ca.nchar == '\'' as c_int
            || (*s).ca.nchar == '`' as c_int
            || (*s).ca.nchar == Ctrl_BSL
        {
            cp = &raw mut (*s).ca.extra_char;
            if (*s).ca.nchar != 'r' as c_int {
                lit = true_0 != 0;
            } else {
                repl = true_0 != 0;
            }
        } else {
            cp = ::core::ptr::null_mut::<c_int>();
        }
    } else {
        if (*s).ca.cmdchar == 'r' as c_int {
            repl = true_0 != 0;
        }
        cp = &raw mut (*s).ca.nchar;
    }
    lang =
        repl as c_int != 0 || (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_LANG != 0;
    if !cp.is_null() {
        let mut langmap_active: bool = false_0 != 0;
        if repl {
            State.set(MODE_REPLACE as c_int);
            ui_cursor_shape_no_check_conceal();
        }
        if lang as c_int != 0 && (*curbuf.get()).b_p_iminsert == B_IMODE_LMAP as OptInt {
            (*no_mapping.ptr()) -= 1;
            (*allow_keys.ptr()) -= 1;
            if repl {
                State.set(MODE_LREPLACE as c_int);
            } else {
                State.set(MODE_LANGMAP as c_int);
            }
            langmap_active = true_0 != 0;
        }
        *cp = plain_vgetc();
        if langmap_active {
            (*no_mapping.ptr()) += 1;
            (*allow_keys.ptr()) += 1;
        }
        State.set(MODE_NORMAL_BUSY as c_int);
        (*s).need_flushbuf = (*s).need_flushbuf as c_int | add_to_showcmd(*cp) as c_int != 0;
        if !lit {
            if *cp == Ctrl_K
                && ((*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_LANG != 0
                    || cp == &raw mut (*s).ca.extra_char)
                && vim_strchr(p_cpo.get(), CPO_DIGRAPH).is_null()
            {
                (*s).c = get_digraph(false_0 != 0);
                if (*s).c > 0 as c_int {
                    *cp = (*s).c;
                    del_from_showcmd(3 as c_int);
                    (*s).need_flushbuf =
                        (*s).need_flushbuf as c_int | add_to_showcmd(*cp) as c_int != 0;
                }
            }
            if *p_langmap.get() as c_int != 0
                && !lang
                && (p_lrm.get() != 0
                    || (if vgetc_busy.get() != 0 {
                        (typebuf_maplen() == 0 as c_int) as c_int
                    } else {
                        KeyTyped.get() as c_int
                    }) != 0)
                && KeyStuffed.get() == 0
                && *cp >= 0 as c_int
            {
                if *cp < 256 as c_int {
                    *cp = (*langmap_mapchar.ptr())[*cp as usize] as c_int;
                } else {
                    *cp = langmap_adjust_mb(*cp);
                }
            }
        }
        if cp == &raw mut (*s).ca.extra_char
            && (*s).ca.nchar == Ctrl_BSL
            && ((*s).ca.extra_char == Ctrl_N || (*s).ca.extra_char == Ctrl_G)
        {
            (*s).ca.cmdchar = Ctrl_BSL;
            (*s).ca.nchar = (*s).ca.extra_char;
            (*s).idx = find_command((*s).ca.cmdchar);
        } else if ((*s).ca.nchar == 'n' as c_int || (*s).ca.nchar == 'N' as c_int)
            && (*s).ca.cmdchar == 'g' as c_int
        {
            (*(*s).ca.oap).op_type = get_op_type(*cp, NUL);
        } else if *cp == Ctrl_BSL {
            let mut towait: c_int = if p_ttm.get() >= 0 as OptInt {
                p_ttm.get() as c_int
            } else {
                p_tm.get() as c_int
            };
            loop {
                (*s).c = vpeekc();
                if !((*s).c <= 0 as c_int && towait > 0 as c_int) {
                    break;
                }
                do_sleep(
                    (if towait > 50 as c_int {
                        50 as c_int
                    } else {
                        towait
                    }) as int64_t,
                    false_0 != 0,
                );
                towait -= 50 as c_int;
            }
            if (*s).c > 0 as c_int {
                (*s).c = plain_vgetc();
                if (*s).c != Ctrl_N && (*s).c != Ctrl_G {
                    vungetc((*s).c);
                } else {
                    (*s).ca.cmdchar = Ctrl_BSL;
                    (*s).ca.nchar = (*s).c;
                    (*s).idx = find_command((*s).ca.cmdchar);
                    '_c2rust_label: {
                        if (*s).idx >= 0 as c_int {
                        } else {
                            __assert_fail(
                                b"s->idx >= 0\0".as_ptr() as *const c_char,
                                b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                                827 as c_uint,
                                b"void normal_get_additional_char(NormalState *)\0".as_ptr()
                                    as *const c_char,
                            );
                        }
                    };
                }
            }
        }
        if lang {
            (*no_mapping.ptr()) -= 1;
            let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
            let mut prev_code: c_int = (*s).ca.nchar;
            loop {
                (*s).c = vpeekc();
                if !((*s).c > 0 as c_int
                    && ((*s).c >= 0x100 as c_int
                        || (*utf8len_tab.ptr())[vpeekc() as usize] as c_int > 1 as c_int))
                {
                    break;
                }
                (*s).c = plain_vgetc();
                if !utf_iscomposing(prev_code, (*s).c, &raw mut state) {
                    vungetc((*s).c);
                    break;
                } else {
                    if (*s).ca.nchar_len == 0 as c_int {
                        (*s).ca.nchar_len = utf_char2bytes(
                            (*s).ca.nchar,
                            &raw mut (*s).ca.nchar_composing as *mut c_char,
                        );
                    }
                    if (*s).ca.nchar_len + utf_char2len((*s).c)
                        < ::core::mem::size_of::<[c_char; 32]>() as c_int
                    {
                        (*s).ca.nchar_len += utf_char2bytes(
                            (*s).c,
                            (&raw mut (*s).ca.nchar_composing as *mut c_char)
                                .offset((*s).ca.nchar_len as isize),
                        );
                    }
                    prev_code = (*s).c;
                }
            }
            (*s).ca.nchar_composing[(*s).ca.nchar_len as usize] = NUL as c_char;
            (*no_mapping.ptr()) += 1;
            (*no_u_sync.ptr()) += 1;
            gotchars_ignore();
            (*no_u_sync.ptr()) -= 1;
        }
    }
    (*no_mapping.ptr()) -= 1;
    (*allow_keys.ptr()) -= 1;
}
unsafe extern "C" fn normal_invert_horizontal(mut s: *mut NormalState) {
    match (*s).ca.cmdchar {
        108 => {
            (*s).ca.cmdchar = 'h' as c_int;
        }
        K_RIGHT => {
            (*s).ca.cmdchar = K_LEFT;
        }
        K_S_RIGHT => {
            (*s).ca.cmdchar = K_S_LEFT;
        }
        -22269 => {
            (*s).ca.cmdchar = -(253 as c_int + ((KE_C_LEFT as c_int) << 8 as c_int));
        }
        104 => {
            (*s).ca.cmdchar = 'l' as c_int;
        }
        K_LEFT => {
            (*s).ca.cmdchar = K_RIGHT;
        }
        K_S_LEFT => {
            (*s).ca.cmdchar = K_S_RIGHT;
        }
        -22013 => {
            (*s).ca.cmdchar = -(253 as c_int + ((KE_C_RIGHT as c_int) << 8 as c_int));
        }
        62 => {
            (*s).ca.cmdchar = '<' as c_int;
        }
        60 => {
            (*s).ca.cmdchar = '>' as c_int;
        }
        _ => {}
    }
    (*s).idx = find_command((*s).ca.cmdchar);
}
unsafe extern "C" fn normal_get_command_count(mut s: *mut NormalState) -> bool {
    if VIsual_active.get() as c_int != 0 && VIsual_select.get() as c_int != 0 {
        return false_0 != 0;
    }
    while (*s).c >= '1' as c_int && (*s).c <= '9' as c_int
        || (*s).ca.count0 != 0 as c_int
            && ((*s).c == K_DEL
                || (*s).c == -(253 as c_int + ((KE_KDEL as c_int) << 8 as c_int))
                || (*s).c == '0' as c_int)
    {
        if (*s).c == K_DEL || (*s).c == -(253 as c_int + ((KE_KDEL as c_int) << 8 as c_int)) {
            (*s).ca.count0 /= 10 as c_int;
            del_from_showcmd(4 as c_int);
        } else if (*s).ca.count0 > 99999999 as c_int {
            (*s).ca.count0 = 999999999 as c_int;
        } else {
            (*s).ca.count0 = (*s).ca.count0 * 10 as c_int + ((*s).c - '0' as c_int);
        }
        if (*s).toplevel as c_int != 0 && readbuf1_empty() as c_int != 0 {
            set_vcount_ca(&raw mut (*s).ca, &raw mut (*s).set_prevcount);
        }
        if (*s).ctrl_w {
            (*no_mapping.ptr()) += 1;
            (*allow_keys.ptr()) += 1;
        }
        (*no_zero_mapping.ptr()) += 1;
        (*s).c = plain_vgetc();
        if *p_langmap.get() as c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as c_int) as c_int
                } else {
                    KeyTyped.get() as c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && (*s).c >= 0 as c_int
        {
            if (*s).c < 256 as c_int {
                (*s).c = (*langmap_mapchar.ptr())[(*s).c as usize] as c_int;
            } else {
                (*s).c = langmap_adjust_mb((*s).c);
            }
        }
        (*no_zero_mapping.ptr()) -= 1;
        if (*s).ctrl_w {
            (*no_mapping.ptr()) -= 1;
            (*allow_keys.ptr()) -= 1;
        }
        (*s).need_flushbuf = (*s).need_flushbuf as c_int | add_to_showcmd((*s).c) as c_int != 0;
    }
    if (*s).c == Ctrl_W && !(*s).ctrl_w && (*s).oa.op_type == OP_NOP as c_int {
        (*s).ctrl_w = true_0 != 0;
        (*s).ca.opcount = (*s).ca.count0;
        (*s).ca.count0 = 0 as c_int;
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        (*s).c = plain_vgetc();
        if *p_langmap.get() as c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as c_int) as c_int
                } else {
                    KeyTyped.get() as c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && (*s).c >= 0 as c_int
        {
            if (*s).c < 256 as c_int {
                (*s).c = (*langmap_mapchar.ptr())[(*s).c as usize] as c_int;
            } else {
                (*s).c = langmap_adjust_mb((*s).c);
            }
        }
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        (*s).need_flushbuf = (*s).need_flushbuf as c_int | add_to_showcmd((*s).c) as c_int != 0;
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn normal_finish_command(mut s: *mut NormalState) {
    let mut did_visual_op: bool = false_0 != 0;
    if !(*s).command_finished {
        if !finish_op.get()
            && (*s).oa.op_type == 0
            && ((*s).idx < 0 as c_int
                || (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_KEEPREG == 0)
        {
            clearop(&raw mut (*s).oa);
            set_reg_var(get_default_register_name());
        }
        if (*s).old_mapped_len > 0 as c_int {
            (*s).old_mapped_len = typebuf_maplen();
        }
        if (*s).ca.cmdchar != -(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int))
            && (*s).ca.cmdchar != -(253 as c_int + ((KE_MOUSEMOVE as c_int) << 8 as c_int))
        {
            did_visual_op = VIsual_active.get() as c_int != 0
                && (*s).oa.op_type != OP_NOP as c_int
                && (*s).oa.op_type != OP_COLON as c_int;
            do_pending_operator(&raw mut (*s).ca, (*s).old_col, false_0 != 0);
        }
        if normal_need_redraw_mode_message(s) {
            normal_redraw_mode_message(s);
        }
    }
    msg_nowait.set(false_0 != 0);
    if finish_op.get() as c_int != 0 || did_visual_op as c_int != 0 {
        set_reg_var(get_default_register_name());
    }
    let prev_finish_op: bool = finish_op.get();
    if (*s).oa.op_type == OP_NOP as c_int {
        finish_op.set(false_0 != 0);
        may_trigger_modechanged();
    }
    if prev_finish_op as c_int != 0
        || (*s).ca.cmdchar == 'r' as c_int
        || (*s).ca.cmdchar == 'g' as c_int && (*s).ca.nchar == 'r' as c_int
    {
        ui_cursor_shape();
    }
    if (*s).oa.op_type == OP_NOP as c_int
        && (*s).oa.regname == 0 as c_int
        && (*s).ca.cmdchar != -(253 as c_int + ((KE_EVENT as c_int) << 8 as c_int))
    {
        clear_showcmd();
    }
    checkpcmark();
    xfree((*s).ca.searchbuf as *mut c_void);
    mb_check_adjust_col(curwin.get() as *mut c_void);
    if (*curwin.get()).w_onebuf_opt.wo_scb != 0 && (*s).toplevel as c_int != 0 {
        validate_cursor(curwin.get());
        do_check_scrollbind(true_0 != 0);
    }
    if (*curwin.get()).w_onebuf_opt.wo_crb != 0 && (*s).toplevel as c_int != 0 {
        validate_cursor(curwin.get());
        do_check_cursorbind();
    }
    if (*s).oa.op_type == OP_NOP as c_int
        && (restart_edit.get() != 0 as c_int
            && !VIsual_active.get()
            && (*s).old_mapped_len == 0 as c_int
            || restart_VIsual_select.get() == 1 as c_int)
        && (*s).ca.retval & CA_COMMAND_BUSY as c_int == 0
        && stuff_empty() as c_int != 0
        && (*s).oa.regname == 0 as c_int
    {
        if restart_VIsual_select.get() == 1 as c_int {
            VIsual_select.set(true_0 != 0);
            VIsual_select_reg.set(0 as c_int);
            may_trigger_modechanged();
            showmode();
            restart_VIsual_select.set(0 as c_int);
        }
        if restart_edit.get() != 0 as c_int
            && !VIsual_active.get()
            && (*s).old_mapped_len == 0 as c_int
        {
            edit(restart_edit.get(), false_0 != 0, 1 as c_int);
        }
    }
    if restart_VIsual_select.get() == 2 as c_int {
        restart_VIsual_select.set(1 as c_int);
    }
    opcount.set((*s).ca.opcount);
}
unsafe extern "C" fn normal_execute(mut state: *mut VimState, mut key: c_int) -> c_int {
    let mut s: *mut NormalState = state as *mut NormalState;
    (*s).command_finished = false_0 != 0;
    (*s).ctrl_w = false_0 != 0;
    (*s).old_col = (*curwin.get()).w_curswant as c_int;
    (*s).c = key;
    if *p_langmap.get() as c_int != 0
        && get_real_state() != MODE_SELECT as c_int
        && (p_lrm.get() != 0
            || (if vgetc_busy.get() != 0 {
                (typebuf_maplen() == 0 as c_int) as c_int
            } else {
                KeyTyped.get() as c_int
            }) != 0)
        && KeyStuffed.get() == 0
        && (*s).c >= 0 as c_int
    {
        if (*s).c < 256 as c_int {
            (*s).c = (*langmap_mapchar.ptr())[(*s).c as usize] as c_int;
        } else {
            (*s).c = langmap_adjust_mb((*s).c);
        }
    }
    if restart_edit.get() == 0 as c_int {
        (*s).old_mapped_len = 0 as c_int;
    } else if (*s).old_mapped_len != 0
        || VIsual_active.get() as c_int != 0
            && (*s).mapped_len == 0 as c_int
            && typebuf_maplen() > 0 as c_int
    {
        (*s).old_mapped_len = typebuf_maplen();
    }
    if (*s).c == NUL {
        (*s).c = K_ZERO;
    }
    if VIsual_active.get() as c_int != 0
        && VIsual_select.get() as c_int != 0
        && (vim_isprintc((*s).c) as c_int != 0
            || (*s).c == NL
            || (*s).c == CAR
            || (*s).c == K_KENTER)
    {
        let mut len: c_int = ins_char_typebuf(vgetc_char.get(), vgetc_mod_mask.get(), true_0 != 0);
        if KeyTyped.get() {
            ungetchars(len);
        }
        if restart_edit.get() != 0 as c_int {
            (*s).c = 'd' as c_int;
        } else {
            (*s).c = 'c' as c_int;
        }
        msg_nowait.set(true_0 != 0);
        (*s).old_mapped_len = 0 as c_int;
    }
    (*s).need_flushbuf = add_to_showcmd((*s).c);
    while normal_get_command_count(s) {}
    if (*s).c == -(253 as c_int + ((KE_EVENT as c_int) << 8 as c_int)) {
        (*s).oa.prev_opcount = (*s).ca.opcount;
        (*s).oa.prev_count0 = (*s).ca.count0;
    } else if (*s).ca.opcount != 0 as c_int {
        if (*s).ca.count0 != 0 {
            if (*s).ca.opcount >= 999999999 as c_int / (*s).ca.count0 {
                (*s).ca.count0 = 999999999 as c_int;
            } else {
                (*s).ca.count0 *= (*s).ca.opcount;
            }
        } else {
            (*s).ca.count0 = (*s).ca.opcount;
        }
    }
    (*s).ca.opcount = (*s).ca.count0;
    (*s).ca.count1 = if (*s).ca.count0 == 0 as c_int {
        1 as c_int
    } else {
        (*s).ca.count0
    };
    if (*s).toplevel as c_int != 0 && readbuf1_empty() as c_int != 0 {
        set_vcount(
            (*s).ca.count0 as int64_t,
            (*s).ca.count1 as int64_t,
            (*s).set_prevcount,
        );
    }
    if (*s).ctrl_w {
        (*s).ca.nchar = (*s).c;
        (*s).ca.cmdchar = Ctrl_W;
    } else {
        (*s).ca.cmdchar = (*s).c;
    }
    (*s).idx = find_command((*s).ca.cmdchar);
    if (*s).idx < 0 as c_int {
        clearopbeep(&raw mut (*s).oa);
        (*s).command_finished = true_0 != 0;
    } else if (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_NCW != 0
        && check_text_or_curbuf_locked(&raw mut (*s).oa) as c_int != 0
    {
        (*s).command_finished = true_0 != 0;
    } else if VIsual_active.get() as c_int != 0
        && normal_handle_special_visual_command(s) as c_int != 0
    {
        (*s).command_finished = true_0 != 0;
    } else {
        if (*curwin.get()).w_onebuf_opt.wo_rl != 0
            && KeyTyped.get() as c_int != 0
            && KeyStuffed.get() == 0
            && (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_RL != 0
        {
            normal_invert_horizontal(s);
        }
        if normal_need_additional_char(s) {
            normal_get_additional_char(s);
        }
        if (*s).need_flushbuf {
            ui_flush();
        }
        if (*s).ca.cmdchar != -(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int))
            && (*s).ca.cmdchar != -(253 as c_int + ((KE_EVENT as c_int) << 8 as c_int))
        {
            did_cursorhold.set(false_0 != 0);
        }
        State.set(MODE_NORMAL as c_int);
        if (*s).ca.nchar == ESC || (*s).ca.extra_char == ESC {
            clearop(&raw mut (*s).oa);
            (*s).command_finished = true_0 != 0;
        } else {
            if (*s).ca.cmdchar != -(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int)) {
                msg_didout.set(false_0 != 0);
                msg_col.set(0 as c_int);
            }
            (*s).old_pos = (*curwin.get()).w_cursor;
            if !VIsual_active.get() && km_startsel.get() as c_int != 0 {
                if (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_SS != 0 {
                    start_selection();
                    unshift_special(&raw mut (*s).ca);
                    (*s).idx = find_command((*s).ca.cmdchar);
                    '_c2rust_label: {
                        if (*s).idx >= 0 as c_int {
                        } else {
                            __assert_fail(
                                b"s->idx >= 0\0".as_ptr() as *const c_char,
                                b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                                1239 as c_uint,
                                b"int normal_execute(VimState *, int)\0".as_ptr() as *const c_char,
                            );
                        }
                    };
                } else if (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_SSS != 0
                    && mod_mask.get() & MOD_MASK_SHIFT != 0
                {
                    start_selection();
                    (*mod_mask.ptr()) &= !MOD_MASK_SHIFT;
                }
            }
            (*s).ca.arg = (*nv_cmds.ptr())[(*s).idx as usize].cmd_arg as c_int;
            (*nv_cmds.ptr())[(*s).idx as usize]
                .cmd_func
                .expect("non-null function pointer")(&raw mut (*s).ca);
        }
    }
    normal_finish_command(s);
    return 1 as c_int;
}
unsafe extern "C" fn normal_check_stuff_buffer(mut _s: *mut NormalState) {
    if stuff_empty() {
        did_check_timestamps.set(false_0 != 0);
        if need_check_timestamps.get() {
            check_timestamps(false_0);
        }
        if need_wait_return.get() {
            wait_return(false_0);
        }
    }
}
unsafe extern "C" fn normal_check_interrupt(mut s: *mut NormalState) {
    if got_int.get() {
        if (*s).noexmode as c_int != 0
            && global_busy.get() != 0
            && !exmode_active.get()
            && (*s).previous_got_int as c_int != 0
        {
            exmode_active.set(true_0 != 0);
            State.set(MODE_NORMAL as c_int);
        } else if global_busy.get() == 0 || !exmode_active.get() {
            if !quit_more.get() {
                vgetc();
            }
            got_int.set(false_0 != 0);
        }
        (*s).previous_got_int = true_0 != 0;
    } else {
        (*s).previous_got_int = false_0 != 0;
    };
}
unsafe extern "C" fn normal_check_window_scrolled(mut _s: *mut NormalState) {
    if !finish_op.get() {
        may_trigger_win_scrolled_resized();
    }
}
unsafe extern "C" fn normal_check_cursor_moved(mut _s: *mut NormalState) {
    if !finish_op.get()
        && has_event(EVENT_CURSORMOVED) as c_int != 0
        && (last_cursormoved_win.get() != curwin.get()
            || !equalpos(last_cursormoved.get(), (*curwin.get()).w_cursor))
    {
        apply_autocmds(
            EVENT_CURSORMOVED,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        last_cursormoved_win.set(curwin.get());
        last_cursormoved.set((*curwin.get()).w_cursor);
    }
}
unsafe extern "C" fn normal_check_text_changed(mut _s: *mut NormalState) {
    if !finish_op.get()
        && has_event(EVENT_TEXTCHANGED) as c_int != 0
        && (*curbuf.get()).b_last_changedtick != buf_get_changedtick(curbuf.get())
    {
        apply_autocmds(
            EVENT_TEXTCHANGED,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        (*curbuf.get()).b_last_changedtick = buf_get_changedtick(curbuf.get());
    }
}
unsafe extern "C" fn normal_check_buffer_modified(mut _s: *mut NormalState) {
    if !finish_op.get()
        && has_event(EVENT_BUFMODIFIEDSET) as c_int != 0
        && (*curbuf.get()).b_changed_invalid as c_int == true_0
    {
        apply_autocmds(
            EVENT_BUFMODIFIEDSET,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        (*curbuf.get()).b_changed_invalid = false_0 != 0;
    }
}
unsafe extern "C" fn normal_check_safe_state(mut _s: *mut NormalState) {
    may_trigger_safestate(!op_pending() && restart_edit.get() == 0 as c_int);
}
unsafe extern "C" fn normal_check_folds(mut _s: *mut NormalState) {
    foldAdjustVisual();
    if hasAnyFolding(curwin.get()) != 0 && !char_avail() {
        foldCheckClose();
        if fdo_flags.get() & kOptFdoFlagAll as c_int as c_uint != 0 {
            foldOpenCursor();
        }
    }
}
unsafe extern "C" fn normal_redraw(mut _s: *mut NormalState) {
    update_topline(curwin.get());
    validate_cursor(curwin.get());
    show_cursor_info_later(false_0 != 0);
    if must_redraw.get() != 0 {
        update_screen();
    } else {
        redraw_statuslines();
        if redraw_cmdline.get() as c_int != 0
            || clear_cmdline.get() as c_int != 0
            || redraw_mode.get() as c_int != 0
        {
            showmode();
        }
    }
    (*curbuf.get()).b_last_used = time(::core::ptr::null_mut::<time_t>());
    if !(*keep_msg.ptr()).is_null() {
        let p: *mut c_char = xstrdup(keep_msg.get());
        msg_hist_off.set(true_0 != 0);
        msg(p, keep_msg_hl_id.get());
        msg_hist_off.set(false_0 != 0);
        xfree(p as *mut c_void);
    }
    if need_fileinfo.get() as c_int != 0 && !shortmess(SHM_FILEINFO as c_int) {
        fileinfo(false_0, true_0, false_0 != 0);
        need_fileinfo.set(false_0 != 0);
    }
    emsg_on_display.set(false_0 != 0);
    did_emsg.set(false_0);
    msg_didany.set(false_0 != 0);
    may_clear_sb_text();
    setcursor();
}
unsafe extern "C" fn normal_check(mut state: *mut VimState) -> c_int {
    let mut s: *mut NormalState = state as *mut NormalState;
    normal_check_stuff_buffer(s);
    normal_check_interrupt(s);
    if did_throw.get() as c_int != 0 && ex_normal_busy.get() == 0 {
        discard_current_exception();
    }
    if !exmode_active.get() {
        msg_scroll.set(false_0);
    }
    quit_more.set(false_0 != 0);
    state_no_longer_safe(::core::ptr::null::<c_char>());
    if skip_redraw.get() as c_int != 0 || exmode_active.get() as c_int != 0 {
        skip_redraw.set(false_0 != 0);
        setcursor();
    } else if do_redraw.get() as c_int != 0 || stuff_empty() as c_int != 0 {
        terminal_check_refresh();
        update_topline(curwin.get());
        validate_cursor(curwin.get());
        normal_check_cursor_moved(s);
        normal_check_text_changed(s);
        normal_check_window_scrolled(s);
        normal_check_buffer_modified(s);
        normal_check_safe_state(s);
        if (*curtab.get()).tp_diff_update != 0 || (*curtab.get()).tp_diff_invalid != 0 {
            ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
            (*curtab.get()).tp_diff_update = false_0;
        }
        if diff_need_scrollbind.get() {
            check_scrollbind(0 as linenr_T, 0 as c_int);
            diff_need_scrollbind.set(false_0 != 0);
        }
        normal_check_folds(s);
        normal_redraw(s);
        do_redraw.set(false_0 != 0);
        if !(*time_fd.ptr()).is_null() {
            if !(*time_fd.ptr()).is_null() {
                time_msg(
                    b"first screen update\0".as_ptr() as *const c_char,
                    ::core::ptr::null::<proftime_T>(),
                );
            }
            time_finish();
        }
        may_make_initial_scroll_size_snapshot();
    }
    may_garbage_collect.set(!(*s).cmdwin && !(*s).noexmode);
    update_curswant();
    if exmode_active.get() {
        if (*s).noexmode {
            return 0 as c_int;
        }
        do_exmode();
        return -1 as c_int;
    }
    if (*s).cmdwin as c_int != 0 && cmdwin_result.get() != 0 as c_int {
        return 0 as c_int;
    }
    normal_prepare(s);
    return 1 as c_int;
}
unsafe extern "C" fn set_vcount_ca(mut cap: *mut cmdarg_T, mut set_prevcount: *mut bool) {
    let mut count: int64_t = (*cap).count0 as int64_t;
    if (*cap).opcount != 0 as c_int {
        count = (*cap).opcount as int64_t
            * (if count == 0 as int64_t {
                1 as int64_t
            } else {
                count
            });
    }
    set_vcount(
        count,
        if count == 0 as int64_t {
            1 as int64_t
        } else {
            count
        },
        *set_prevcount,
    );
    *set_prevcount = false_0 != 0;
}
pub unsafe extern "C" fn end_visual_mode() {
    VIsual_select_exclu_adj.set(false_0 != 0);
    VIsual_active.set(false_0 != 0);
    setmouse();
    mouse_dragging.set(0 as c_int);
    (*curbuf.get()).b_visual.vi_mode = VIsual_mode.get();
    (*curbuf.get()).b_visual.vi_start = VIsual.get();
    (*curbuf.get()).b_visual.vi_end = (*curwin.get()).w_cursor;
    (*curbuf.get()).b_visual.vi_curswant = (*curwin.get()).w_curswant;
    (*curbuf.get()).b_visual_mode_eval = VIsual_mode.get();
    if !virtual_active(curwin.get()) {
        (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
    }
    may_clear_cmdline();
    adjust_cursor_eol();
    may_trigger_modechanged();
}
pub unsafe extern "C" fn reset_VIsual_and_resel() {
    if VIsual_active.get() {
        end_visual_mode();
        redraw_curbuf_later(UPD_INVERTED as c_int);
    }
    VIsual_reselect.set(false_0);
}
pub unsafe extern "C" fn reset_VIsual() {
    if VIsual_active.get() {
        end_visual_mode();
        redraw_curbuf_later(UPD_INVERTED as c_int);
        VIsual_reselect.set(false_0);
    }
}
pub unsafe extern "C" fn restore_visual_mode() {
    if VIsual_mode_orig.get() != NUL {
        (*curbuf.get()).b_visual.vi_mode = VIsual_mode_orig.get();
        VIsual_mode_orig.set(NUL);
    }
}
unsafe extern "C" fn find_is_eval_item(
    ptr: *const c_char,
    colp: *mut c_int,
    bnp: *mut c_int,
    dir: c_int,
) -> bool {
    if *ptr as c_int == ']' as c_int && dir == BACKWARD as c_int
        || *ptr as c_int == '[' as c_int && dir == FORWARD as c_int
    {
        *bnp += 1 as c_int;
    }
    if *bnp > 0 as c_int {
        if *ptr as c_int == '[' as c_int && dir == BACKWARD as c_int
            || *ptr as c_int == ']' as c_int && dir == FORWARD as c_int
        {
            *bnp -= 1 as c_int;
        }
        return true_0 != 0;
    }
    if *ptr as c_int == '.' as c_int {
        return true_0 != 0;
    }
    if *ptr.offset(
        (if dir == BACKWARD as c_int {
            0 as c_int
        } else {
            1 as c_int
        }) as isize,
    ) as c_int
        == '>' as c_int
        && *ptr.offset(
            (if dir == BACKWARD as c_int {
                -1 as c_int
            } else {
                0 as c_int
            }) as isize,
        ) as c_int
            == '-' as c_int
    {
        *colp += dir;
        return true_0 != 0;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn find_ident_under_cursor(
    mut text: *mut *mut c_char,
    mut find_type: c_int,
    mut offset: *mut c_int,
) -> size_t {
    let mut textcol: c_int = 0 as c_int;
    let mut len: size_t = find_ident_at_pos(
        curwin.get(),
        (*curwin.get()).w_cursor.lnum,
        (*curwin.get()).w_cursor.col,
        text,
        if !offset.is_null() {
            &raw mut textcol
        } else {
            ::core::ptr::null_mut::<c_int>()
        },
        find_type,
    );
    if !offset.is_null() {
        *offset = (*curwin.get()).w_cursor.col as c_int - textcol;
    }
    return len;
}
pub unsafe extern "C" fn find_ident_at_pos(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut startcol: colnr_T,
    mut text: *mut *mut c_char,
    mut textcol: *mut c_int,
    mut find_type: c_int,
) -> size_t {
    let mut col: c_int = 0 as c_int;
    let mut i: c_int = 0;
    let mut this_class: c_int = 0 as c_int;
    let mut prev_class: c_int = 0;
    let mut prevcol: c_int = 0;
    let mut bn: c_int = 0 as c_int;
    let mut ptr: *mut c_char = ml_get_buf((*wp).w_buffer, lnum);
    i = if find_type & FIND_IDENT as c_int != 0 {
        0 as c_int
    } else {
        1 as c_int
    };
    while i < 2 as c_int {
        col = startcol as c_int;
        while *ptr.offset(col as isize) as c_int != NUL {
            if find_type & FIND_EVAL as c_int != 0
                && *ptr.offset(col as isize) as c_int == ']' as c_int
            {
                break;
            }
            this_class = mb_get_class(ptr.offset(col as isize));
            if this_class != 0 as c_int && (i == 1 as c_int || this_class != 1 as c_int) {
                break;
            }
            col += utfc_ptr2len(ptr.offset(col as isize));
        }
        bn = (*ptr.offset(col as isize) as c_int == ']' as c_int) as c_int;
        if find_type & FIND_EVAL as c_int != 0 && *ptr.offset(col as isize) as c_int == ']' as c_int
        {
            this_class = mb_get_class(b"a\0".as_ptr() as *const c_char);
        } else {
            this_class = mb_get_class(ptr.offset(col as isize));
        }
        while col > 0 as c_int && this_class != 0 as c_int {
            prevcol = col
                - 1 as c_int
                - utf_head_off(ptr, ptr.offset(col as isize).offset(-(1 as c_int as isize)));
            prev_class = mb_get_class(ptr.offset(prevcol as isize));
            if this_class != prev_class
                && (i == 0 as c_int
                    || prev_class == 0 as c_int
                    || find_type & FIND_IDENT as c_int != 0)
                && (find_type & FIND_EVAL as c_int == 0
                    || prevcol == 0 as c_int
                    || !find_is_eval_item(
                        ptr.offset(prevcol as isize),
                        &raw mut prevcol,
                        &raw mut bn,
                        BACKWARD as c_int,
                    ))
            {
                break;
            }
            col = prevcol;
        }
        this_class = if this_class < 2 as c_int {
            this_class
        } else {
            2 as c_int
        };
        if find_type & FIND_STRING as c_int == 0 || this_class == 2 as c_int {
            break;
        }
        i += 1;
    }
    if *ptr.offset(col as isize) as c_int == NUL || i == 0 as c_int && this_class != 2 as c_int {
        if find_type & FIND_STRING as c_int != 0 {
            emsg(gettext(
                b"E348: No string under cursor\0".as_ptr() as *const c_char
            ));
        } else {
            emsg(gettext(&raw const e_noident as *const c_char));
        }
        return 0 as size_t;
    }
    ptr = ptr.offset(col as isize);
    *text = ptr;
    if !textcol.is_null() {
        *textcol = col;
    }
    bn = 0 as c_int;
    startcol -= col;
    col = 0 as c_int;
    this_class = mb_get_class(ptr);
    while *ptr.offset(col as isize) as c_int != NUL
        && ((if i == 0 as c_int {
            (mb_get_class(ptr.offset(col as isize)) == this_class) as c_int
        } else {
            (mb_get_class(ptr.offset(col as isize)) != 0 as c_int) as c_int
        }) != 0
            || find_type & FIND_EVAL as c_int != 0
                && col <= startcol
                && find_is_eval_item(
                    ptr.offset(col as isize),
                    &raw mut col,
                    &raw mut bn,
                    FORWARD as c_int,
                ) as c_int
                    != 0)
    {
        col += utfc_ptr2len(ptr.offset(col as isize));
    }
    '_c2rust_label: {
        if col >= 0 as c_int {
        } else {
            __assert_fail(
                b"col >= 0\0".as_ptr() as *const c_char,
                b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                1748 as c_uint,
                b"size_t find_ident_at_pos(win_T *, linenr_T, colnr_T, char **, int *, int)\0"
                    .as_ptr() as *const c_char,
            );
        }
    };
    return col as size_t;
}
unsafe extern "C" fn prep_redo_cmd(mut cap: *mut cmdarg_T) {
    prep_redo(
        (*(*cap).oap).regname,
        (*cap).count0,
        NUL,
        (*cap).cmdchar,
        NUL,
        NUL,
        NUL,
    );
    if (*cap).nchar_len > 0 as c_int {
        AppendToRedobuff(&raw mut (*cap).nchar_composing as *mut c_char);
    } else {
        AppendCharToRedobuff((*cap).nchar);
    };
}
pub unsafe extern "C" fn prep_redo(
    mut regname: c_int,
    mut num: c_int,
    mut cmd1: c_int,
    mut cmd2: c_int,
    mut cmd3: c_int,
    mut cmd4: c_int,
    mut cmd5: c_int,
) {
    prep_redo_num2(regname, num, cmd1, cmd2, 0 as c_int, cmd3, cmd4, cmd5);
}
pub unsafe extern "C" fn prep_redo_num2(
    mut regname: c_int,
    mut num1: c_int,
    mut cmd1: c_int,
    mut cmd2: c_int,
    mut num2: c_int,
    mut cmd3: c_int,
    mut cmd4: c_int,
    mut cmd5: c_int,
) {
    ResetRedobuff();
    if regname != 0 as c_int {
        AppendCharToRedobuff('"' as c_int);
        AppendCharToRedobuff(regname);
    }
    if num1 != 0 as c_int {
        AppendNumberToRedobuff(num1);
    }
    if cmd1 != NUL {
        AppendCharToRedobuff(cmd1);
    }
    if cmd2 != NUL {
        AppendCharToRedobuff(cmd2);
    }
    if num2 != 0 as c_int {
        AppendNumberToRedobuff(num2);
    }
    if cmd3 != NUL {
        AppendCharToRedobuff(cmd3);
    }
    if cmd4 != NUL {
        AppendCharToRedobuff(cmd4);
    }
    if cmd5 != NUL {
        AppendCharToRedobuff(cmd5);
    }
}
unsafe extern "C" fn checkclearop(mut oap: *mut oparg_T) -> bool {
    if (*oap).op_type == OP_NOP as c_int {
        return false_0 != 0;
    }
    clearopbeep(oap);
    return true_0 != 0;
}
unsafe extern "C" fn checkclearopq(mut oap: *mut oparg_T) -> bool {
    if (*oap).op_type == OP_NOP as c_int && !VIsual_active.get() {
        return false_0 != 0;
    }
    clearopbeep(oap);
    return true_0 != 0;
}
pub unsafe extern "C" fn clearop(mut oap: *mut oparg_T) {
    (*oap).op_type = OP_NOP as c_int;
    (*oap).regname = 0 as c_int;
    (*oap).motion_force = NUL;
    (*oap).use_reg_one = false_0 != 0;
    motion_force.set(NUL);
}
pub unsafe extern "C" fn clearopbeep(mut oap: *mut oparg_T) {
    clearop(oap);
    beep_flush();
}
unsafe extern "C" fn unshift_special(mut cap: *mut cmdarg_T) {
    match (*cap).cmdchar {
        K_S_RIGHT => {
            (*cap).cmdchar = K_RIGHT;
        }
        K_S_LEFT => {
            (*cap).cmdchar = K_LEFT;
        }
        -1277 => {
            (*cap).cmdchar = K_UP;
        }
        -1533 => {
            (*cap).cmdchar = K_DOWN;
        }
        K_S_HOME => {
            (*cap).cmdchar = K_HOME;
        }
        K_S_END => {
            (*cap).cmdchar = K_END;
        }
        _ => {}
    }
    (*cap).cmdchar = simplify_key((*cap).cmdchar, mod_mask.ptr());
}
pub unsafe extern "C" fn may_clear_cmdline() {
    if mode_displayed.get() {
        clear_cmdline.set(true_0 != 0);
    } else {
        clear_showcmd();
    };
}
static old_showcmd_buf: GlobalCell<[c_char; 41]> = GlobalCell::new([0; 41]);
static showcmd_is_clear: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
static showcmd_visual: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub unsafe extern "C" fn clear_showcmd() {
    if p_sc.get() == 0 {
        return;
    }
    if VIsual_active.get() as c_int != 0 && !char_avail() {
        let mut cursor_bot: bool = lt(VIsual.get(), (*curwin.get()).w_cursor);
        let mut lines: c_int = 0;
        let mut leftcol: colnr_T = 0;
        let mut rightcol: colnr_T = 0;
        let mut top: linenr_T = 0;
        let mut bot: linenr_T = 0;
        if cursor_bot {
            top = (*VIsual.ptr()).lnum;
            bot = (*curwin.get()).w_cursor.lnum;
        } else {
            top = (*curwin.get()).w_cursor.lnum;
            bot = (*VIsual.ptr()).lnum;
        }
        hasFolding(
            curwin.get(),
            top,
            &raw mut top,
            ::core::ptr::null_mut::<linenr_T>(),
        );
        hasFolding(
            curwin.get(),
            bot,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut bot,
        );
        lines = (bot - top + 1 as linenr_T) as c_int;
        if VIsual_mode.get() == Ctrl_V {
            let saved_sbr: *mut c_char = p_sbr.get();
            let saved_w_sbr: *mut c_char = (*curwin.get()).w_onebuf_opt.wo_sbr;
            p_sbr.set(empty_string_option.ptr() as *mut c_char);
            (*curwin.get()).w_onebuf_opt.wo_sbr = empty_string_option.ptr() as *mut c_char;
            getvcols(
                curwin.get(),
                &raw mut (*curwin.get()).w_cursor,
                VIsual.ptr(),
                &raw mut leftcol,
                &raw mut rightcol,
            );
            p_sbr.set(saved_sbr);
            (*curwin.get()).w_onebuf_opt.wo_sbr = saved_w_sbr;
            snprintf(
                showcmd_buf.ptr() as *mut c_char,
                SHOWCMD_BUFLEN as c_int as size_t,
                b"%ldx%ld\0".as_ptr() as *const c_char,
                lines as int64_t,
                rightcol as int64_t - leftcol as int64_t + 1 as int64_t,
            );
        } else if VIsual_mode.get() == 'V' as c_int
            || (*VIsual.ptr()).lnum != (*curwin.get()).w_cursor.lnum
        {
            snprintf(
                showcmd_buf.ptr() as *mut c_char,
                SHOWCMD_BUFLEN as c_int as size_t,
                b"%ld\0".as_ptr() as *const c_char,
                lines as int64_t,
            );
        } else {
            let mut s: *mut c_char = ::core::ptr::null_mut::<c_char>();
            let mut e: *mut c_char = ::core::ptr::null_mut::<c_char>();
            let mut bytes: c_int = 0 as c_int;
            let mut chars: c_int = 0 as c_int;
            if cursor_bot {
                s = ml_get_pos(VIsual.ptr());
                e = get_cursor_pos_ptr();
            } else {
                s = get_cursor_pos_ptr();
                e = ml_get_pos(VIsual.ptr());
            }
            while if *p_sel.get() as c_int != 'e' as c_int {
                (s <= e) as c_int
            } else {
                (s < e) as c_int
            } != 0
            {
                let mut l: c_int = utfc_ptr2len(s);
                if l == 0 as c_int {
                    bytes += 1;
                    chars += 1;
                    break;
                } else {
                    bytes += l;
                    chars += 1;
                    s = s.offset(l as isize);
                }
            }
            if bytes == chars {
                snprintf(
                    showcmd_buf.ptr() as *mut c_char,
                    SHOWCMD_BUFLEN as c_int as size_t,
                    b"%d\0".as_ptr() as *const c_char,
                    chars,
                );
            } else {
                snprintf(
                    showcmd_buf.ptr() as *mut c_char,
                    SHOWCMD_BUFLEN as c_int as size_t,
                    b"%d-%d\0".as_ptr() as *const c_char,
                    chars,
                    bytes,
                );
            }
        }
        let mut limit: c_int = if ui_has(kUIMessages) as c_int != 0 {
            SHOWCMD_BUFLEN as c_int - 1 as c_int
        } else {
            SHOWCMD_COLS as c_int
        };
        (*showcmd_buf.ptr())[limit as usize] = NUL as c_char;
        showcmd_visual.set(true_0 != 0);
    } else {
        (*showcmd_buf.ptr())[0 as c_int as usize] = NUL as c_char;
        showcmd_visual.set(false_0 != 0);
        if showcmd_is_clear.get() {
            return;
        }
    }
    display_showcmd();
}
pub unsafe extern "C" fn add_to_showcmd(mut c: c_int) -> bool {
    static ignore: GlobalCell<[c_int; 23]> = GlobalCell::new([
        -(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_LEFTMOUSE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_LEFTDRAG as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_LEFTRELEASE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_MOUSEMOVE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_MIDDLEMOUSE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_MIDDLEDRAG as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_MIDDLERELEASE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_RIGHTMOUSE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_RIGHTDRAG as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_RIGHTRELEASE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_MOUSEDOWN as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_MOUSEUP as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_MOUSELEFT as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_MOUSERIGHT as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_X1MOUSE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_X1DRAG as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_X1RELEASE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_X2MOUSE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_X2DRAG as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_X2RELEASE as c_int) << 8 as c_int)),
        -(253 as c_int + ((KE_EVENT as c_int) << 8 as c_int)),
        0 as c_int,
    ]);
    if p_sc.get() == 0 || msg_silent.get() != 0 as c_int || ex_normal_busy.get() != 0 {
        return false_0 != 0;
    }
    if showcmd_visual.get() {
        (*showcmd_buf.ptr())[0 as c_int as usize] = NUL as c_char;
        showcmd_visual.set(false_0 != 0);
    }
    if c < 0 as c_int {
        let mut i: c_int = 0 as c_int;
        while (*ignore.ptr())[i as usize] != 0 as c_int {
            if (*ignore.ptr())[i as usize] == c {
                return false_0 != 0;
            }
            i += 1;
        }
    }
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut mbyte_buf: [c_char; 7] = [0; 7];
    if c <= 0x7f as c_int || !vim_isprintc(c) {
        p = transchar(c);
        if *p as c_int == ' ' as c_int {
            strcpy(p, b"<20>\0".as_ptr() as *const c_char as *mut c_char);
        }
    } else {
        mbyte_buf[utf_char2bytes(c, &raw mut mbyte_buf as *mut c_char) as usize] = NUL as c_char;
        p = &raw mut mbyte_buf as *mut c_char;
    }
    let mut old_len: size_t = strlen(showcmd_buf.ptr() as *mut c_char);
    let mut extra_len: size_t = strlen(p);
    let mut limit: size_t = (if ui_has(kUIMessages) as c_int != 0 {
        SHOWCMD_BUFLEN as c_int - 1 as c_int
    } else {
        SHOWCMD_COLS as c_int
    }) as size_t;
    if old_len.wrapping_add(extra_len) > limit {
        let mut overflow: size_t = old_len.wrapping_add(extra_len).wrapping_sub(limit);
        memmove(
            showcmd_buf.ptr() as *mut c_char as *mut c_void,
            (showcmd_buf.ptr() as *mut c_char).offset(overflow as isize) as *const c_void,
            old_len.wrapping_sub(overflow).wrapping_add(1 as size_t),
        );
    }
    strcat(showcmd_buf.ptr() as *mut c_char, p);
    if char_avail() {
        return false_0 != 0;
    }
    display_showcmd();
    return true_0 != 0;
}
pub unsafe extern "C" fn add_to_showcmd_c(mut c: c_int) {
    add_to_showcmd(c);
    setcursor();
}
unsafe extern "C" fn del_from_showcmd(mut len: c_int) {
    if p_sc.get() == 0 {
        return;
    }
    let mut old_len: c_int = strlen(showcmd_buf.ptr() as *mut c_char) as c_int;
    len = if len < old_len { len } else { old_len };
    (*showcmd_buf.ptr())[(old_len - len) as usize] = NUL as c_char;
    if !char_avail() {
        display_showcmd();
    }
}
pub unsafe extern "C" fn push_showcmd() {
    if p_sc.get() != 0 {
        strcpy(
            old_showcmd_buf.ptr() as *mut c_char,
            showcmd_buf.ptr() as *mut c_char,
        );
    }
}
pub unsafe extern "C" fn pop_showcmd() {
    if p_sc.get() == 0 {
        return;
    }
    strcpy(
        showcmd_buf.ptr() as *mut c_char,
        old_showcmd_buf.ptr() as *mut c_char,
    );
    display_showcmd();
}
unsafe extern "C" fn display_showcmd() {
    showcmd_is_clear.set((*showcmd_buf.ptr())[0 as c_int as usize] as c_int == NUL);
    if *p_sloc.get() as c_int == 's' as c_int {
        if showcmd_is_clear.get() {
            (*curwin.get()).w_redr_status = true_0 != 0;
        } else {
            win_redr_status(curwin.get());
            setcursor();
        }
        return;
    }
    if *p_sloc.get() as c_int == 't' as c_int {
        if showcmd_is_clear.get() {
            redraw_tabline.set(true_0 != 0);
        } else {
            draw_tabline();
            setcursor();
        }
        return;
    }
    if ui_has(kUIMessages) {
        let mut content: Array = ARRAY_DICT_INIT;
        let mut content__items: [Object; 1] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_0 { boolean: false },
        }; 1];
        content.capacity = 1 as size_t;
        content.items = &raw mut content__items as *mut Object;
        let mut chunk: Array = ARRAY_DICT_INIT;
        let mut chunk__items: [Object; 3] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_0 { boolean: false },
        }; 3];
        chunk.capacity = 3 as size_t;
        chunk.items = &raw mut chunk__items as *mut Object;
        if !showcmd_is_clear.get() {
            let c2rust_fresh6 = chunk.size;
            chunk.size = chunk.size.wrapping_add(1);
            *chunk.items.offset(c2rust_fresh6 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_0 {
                    integer: 0 as Integer,
                },
            };
            let c2rust_fresh7 = chunk.size;
            chunk.size = chunk.size.wrapping_add(1);
            *chunk.items.offset(c2rust_fresh7 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_0 {
                    string: cstr_as_string(showcmd_buf.ptr() as *mut c_char),
                },
            };
            let c2rust_fresh8 = chunk.size;
            chunk.size = chunk.size.wrapping_add(1);
            *chunk.items.offset(c2rust_fresh8 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_0 {
                    integer: 0 as Integer,
                },
            };
            let c2rust_fresh9 = content.size;
            content.size = content.size.wrapping_add(1);
            *content.items.offset(c2rust_fresh9 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed_0 { array: chunk },
            };
        }
        ui_call_msg_showcmd(content);
        return;
    }
    if p_ch.get() == 0 as OptInt {
        return;
    }
    msg_grid_validate();
    let mut showcmd_row: c_int = Rows.get() - 1 as c_int;
    grid_line_start(msg_grid_adj.ptr(), showcmd_row);
    let mut len: c_int = 0 as c_int;
    if !showcmd_is_clear.get() {
        len = grid_line_puts(
            sc_col.get(),
            showcmd_buf.ptr() as *mut c_char,
            -1 as c_int,
            *(*hl_attr_active.ptr()).offset(HLF_MSG as c_int as isize),
        );
    }
    grid_line_puts(
        sc_col.get() + len,
        (b"          \0".as_ptr() as *const c_char as *mut c_char).offset(len as isize),
        -1 as c_int,
        *(*hl_attr_active.ptr()).offset(HLF_MSG as c_int as isize),
    );
    grid_line_flush();
}
pub unsafe extern "C" fn get_vtopline(mut wp: *mut win_T) -> c_int {
    return plines_m_win_fill(wp, 1 as linenr_T, (*wp).w_topline) - (*wp).w_topfill;
}
pub unsafe extern "C" fn do_check_scrollbind(mut check: bool) {
    static old_curwin: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
    static old_vtopline: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
    static old_buf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
    static old_leftcol: GlobalCell<colnr_T> = GlobalCell::new(0 as colnr_T);
    let mut vtopline: c_int = get_vtopline(curwin.get());
    if check as c_int != 0 && (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
        if did_syncbind.get() {
            did_syncbind.set(false_0 != 0);
        } else if curwin.get() == old_curwin.get() {
            if ((*curwin.get()).w_buffer == old_buf.get()
                || (*curwin.get()).w_onebuf_opt.wo_diff != 0)
                && (vtopline as linenr_T != old_vtopline.get()
                    || (*curwin.get()).w_leftcol != old_leftcol.get())
            {
                check_scrollbind(
                    vtopline as linenr_T - old_vtopline.get(),
                    (*curwin.get()).w_leftcol as c_int - old_leftcol.get() as c_int,
                );
            }
        } else if !vim_strchr(p_sbo.get(), 'j' as c_int).is_null() {
            check_scrollbind(
                vtopline as linenr_T - (*curwin.get()).w_scbind_pos as linenr_T,
                0 as c_int,
            );
        }
        (*curwin.get()).w_scbind_pos = vtopline;
    }
    old_curwin.set(curwin.get());
    old_vtopline.set(vtopline as linenr_T);
    old_buf.set((*curwin.get()).w_buffer);
    old_leftcol.set((*curwin.get()).w_leftcol);
}
pub unsafe extern "C" fn check_scrollbind(mut vtopline_diff: linenr_T, mut leftcol_diff: c_int) {
    let mut old_curwin: *mut win_T = curwin.get();
    let mut old_curbuf: *mut buf_T = curbuf.get();
    let mut old_VIsual_select: c_int = VIsual_select.get() as c_int;
    let mut old_VIsual_active: c_int = VIsual_active.get() as c_int;
    let mut tgt_leftcol: colnr_T = (*curwin.get()).w_leftcol;
    let mut want_ver: bool = (*old_curwin).w_onebuf_opt.wo_diff != 0
        || !vim_strchr(p_sbo.get(), 'v' as c_int).is_null() && vtopline_diff != 0 as linenr_T;
    let mut want_hor: bool = !vim_strchr(p_sbo.get(), 'h' as c_int).is_null()
        && (leftcol_diff != 0 || vtopline_diff != 0 as linenr_T);
    VIsual_active.set(false);
    VIsual_select.set(VIsual_active.get());
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        curwin.set(wp);
        curbuf.set((*curwin.get()).w_buffer);
        if !(curwin.get() == old_curwin || (*curwin.get()).w_onebuf_opt.wo_scb == 0) {
            if want_ver {
                if (*old_curwin).w_onebuf_opt.wo_diff != 0
                    && (*curwin.get()).w_onebuf_opt.wo_diff != 0
                {
                    diff_set_topline(old_curwin, curwin.get());
                } else {
                    (*curwin.get()).w_scbind_pos += vtopline_diff as c_int;
                    let mut curr_vtopline: c_int = get_vtopline(curwin.get());
                    let mut max_vtopline: c_int = curr_vtopline
                        + (*curwin.get()).w_topfill
                        + plines_m_win_fill(
                            curwin.get(),
                            (*curwin.get()).w_topline + 1 as linenr_T,
                            (*curbuf.get()).b_ml.ml_line_count,
                        );
                    let mut new_vtopline: c_int = if (if ((*curwin.get()).w_scbind_pos as linenr_T)
                        < max_vtopline as linenr_T
                    {
                        (*curwin.get()).w_scbind_pos as linenr_T
                    } else {
                        max_vtopline as linenr_T
                    }) > 1 as linenr_T
                    {
                        if ((*curwin.get()).w_scbind_pos as linenr_T) < max_vtopline as linenr_T {
                            (*curwin.get()).w_scbind_pos
                        } else {
                            max_vtopline
                        }
                    } else {
                        1 as c_int
                    };
                    let mut y: c_int = new_vtopline - curr_vtopline;
                    if y > 0 as c_int {
                        scrollup(curwin.get(), y as linenr_T, false_0 != 0);
                    } else {
                        scrolldown(curwin.get(), -(y as linenr_T), false_0);
                    }
                }
                redraw_later(curwin.get(), UPD_VALID as c_int);
                cursor_correct(curwin.get());
                (*curwin.get()).w_redr_status = true_0 != 0;
            }
            if want_hor {
                set_leftcol(tgt_leftcol);
            }
        }
        wp = (*wp).w_next;
    }
    VIsual_select.set(old_VIsual_select != 0);
    VIsual_active.set(old_VIsual_active != 0);
    curwin.set(old_curwin);
    curbuf.set(old_curbuf);
}
unsafe extern "C" fn nv_ignore(mut cap: *mut cmdarg_T) {
    (*cap).retval |= CA_COMMAND_BUSY as c_int;
}
unsafe extern "C" fn nv_nop(mut _cap: *mut cmdarg_T) {}
unsafe extern "C" fn nv_error(mut cap: *mut cmdarg_T) {
    clearopbeep((*cap).oap);
}
unsafe extern "C" fn nv_help(mut cap: *mut cmdarg_T) {
    if !checkclearopq((*cap).oap) {
        ex_help(::core::ptr::null_mut::<exarg_T>());
    }
}
unsafe extern "C" fn nv_addsub(mut cap: *mut cmdarg_T) {
    if bt_prompt(curbuf.get()) as c_int != 0 && !prompt_curpos_editable() {
        clearopbeep((*cap).oap);
    } else if !VIsual_active.get() && (*(*cap).oap).op_type == OP_NOP as c_int {
        prep_redo_cmd(cap);
        (*(*cap).oap).op_type = if (*cap).cmdchar == Ctrl_A {
            OP_NR_ADD as c_int
        } else {
            OP_NR_SUB as c_int
        };
        op_addsub((*cap).oap, (*cap).count1 as linenr_T, (*cap).arg != 0);
        (*(*cap).oap).op_type = OP_NOP as c_int;
    } else if VIsual_active.get() {
        nv_operator(cap);
    } else {
        clearop((*cap).oap);
    };
}
unsafe extern "C" fn nv_page(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    if mod_mask.get() & MOD_MASK_CTRL != 0 {
        if (*cap).arg == BACKWARD as c_int {
            goto_tabpage(-(*cap).count1);
        } else {
            goto_tabpage((*cap).count0);
        }
    } else {
        pagescroll((*cap).arg as Direction, (*cap).count1, false_0 != 0);
    };
}
unsafe extern "C" fn nv_gd(mut oap: *mut oparg_T, mut nchar: c_int, mut thisblock: c_int) {
    let mut len: size_t = 0;
    let mut ptr: *mut c_char = ::core::ptr::null_mut::<c_char>();
    len = find_ident_under_cursor(
        &raw mut ptr,
        FIND_IDENT as c_int,
        ::core::ptr::null_mut::<c_int>(),
    );
    if len == 0 as size_t
        || !find_decl(
            ptr,
            len,
            nchar == 'd' as c_int,
            thisblock != 0,
            SEARCH_START as c_int,
        )
    {
        clearopbeep(oap);
        return;
    }
    if fdo_flags.get() & kOptFdoFlagSearch as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
    if messaging() as c_int != 0 && msg_silent.get() == 0 && !shortmess(SHM_SEARCHCOUNT as c_int) {
        clear_cmdline.set(true_0 != 0);
    }
}
unsafe extern "C" fn is_ident(mut line: *const c_char, mut offset: c_int) -> bool {
    let mut incomment: bool = false_0 != 0;
    let mut instring: c_int = 0 as c_int;
    let mut prev: c_int = 0 as c_int;
    let mut i: c_int = 0 as c_int;
    while i < offset && *line.offset(i as isize) as c_int != NUL {
        if instring != 0 as c_int {
            if prev != '\\' as c_int && *line.offset(i as isize) as uint8_t as c_int == instring {
                instring = 0 as c_int;
            }
        } else if (*line.offset(i as isize) as c_int == '"' as c_int
            || *line.offset(i as isize) as c_int == '\'' as c_int)
            && !incomment
        {
            instring = *line.offset(i as isize) as uint8_t as c_int;
        } else if incomment {
            if prev == '*' as c_int && *line.offset(i as isize) as c_int == '/' as c_int {
                incomment = false_0 != 0;
            }
        } else if prev == '/' as c_int && *line.offset(i as isize) as c_int == '*' as c_int {
            incomment = true_0 != 0;
        } else if prev == '/' as c_int && *line.offset(i as isize) as c_int == '/' as c_int {
            return false_0 != 0;
        }
        prev = *line.offset(i as isize) as uint8_t as c_int;
        i += 1;
    }
    return incomment as c_int == false_0 && instring == 0 as c_int;
}
pub unsafe extern "C" fn find_decl(
    mut ptr: *mut c_char,
    mut len: size_t,
    mut locally: bool,
    mut thisblock: bool,
    mut flags_arg: c_int,
) -> bool {
    let mut par_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut found_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut t: bool = false;
    let mut retval: bool = true_0 != 0;
    let mut incll: bool = false;
    let mut searchflags: c_int = flags_arg;
    let mut patsize: size_t = len.wrapping_add(7 as size_t);
    let mut pat: *mut c_char = xmalloc(patsize) as *mut c_char;
    '_c2rust_label: {
        if patsize <= 2147483647 as c_int as size_t {
        } else {
            __assert_fail(
                b"patsize <= INT_MAX\0".as_ptr() as *const c_char,
                b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                2387 as c_uint,
                b"_Bool find_decl(char *, size_t, _Bool, _Bool, int)\0".as_ptr() as *const c_char,
            );
        }
    };
    let mut patlen: size_t = snprintf(
        pat,
        patsize,
        if vim_iswordp(ptr) as c_int != 0 {
            b"\\V\\<%.*s\\>\0".as_ptr() as *const c_char
        } else {
            b"\\V%.*s\0".as_ptr() as *const c_char
        },
        len as c_int,
        ptr,
    ) as size_t;
    let mut old_pos: pos_T = (*curwin.get()).w_cursor;
    let mut save_p_ws: bool = p_ws.get() != 0;
    let mut save_p_scs: bool = p_scs.get() != 0;
    p_ws.set(false_0);
    p_scs.set(false_0);
    if !locally
        || !findpar(
            &raw mut incll,
            BACKWARD as c_int,
            1 as c_int,
            '{' as c_int,
            false_0 != 0,
        )
    {
        setpcmark();
        (*curwin.get()).w_cursor.lnum = 1 as c_int as linenr_T;
        par_pos = (*curwin.get()).w_cursor;
    } else {
        par_pos = (*curwin.get()).w_cursor;
        while (*curwin.get()).w_cursor.lnum > 1 as linenr_T
            && *skipwhite(get_cursor_line_ptr()) as c_int != NUL
        {
            (*curwin.get()).w_cursor.lnum -= 1;
        }
    }
    (*curwin.get()).w_cursor.col = 0 as c_int as colnr_T;
    clearpos(&mut found_pos);
    loop {
        t = searchit(
            curwin.get(),
            curbuf.get(),
            &raw mut (*curwin.get()).w_cursor,
            ::core::ptr::null_mut::<pos_T>(),
            FORWARD,
            pat,
            patlen,
            1 as c_int,
            searchflags,
            RE_LAST as c_int,
            ::core::ptr::null_mut::<searchit_arg_T>(),
        ) != 0;
        if (*curwin.get()).w_cursor.lnum >= old_pos.lnum {
            t = false_0 != 0;
        }
        if thisblock as c_int != 0 && t as c_int != false_0 {
            let maxtravel: int64_t =
                (old_pos.lnum - (*curwin.get()).w_cursor.lnum + 1 as linenr_T) as int64_t;
            let mut pos: *const pos_T = findmatchlimit(
                ::core::ptr::null_mut::<oparg_T>(),
                '}' as c_int,
                FM_FORWARD as c_int,
                maxtravel,
            );
            if !pos.is_null() && (*pos).lnum < old_pos.lnum {
                (*curwin.get()).w_cursor = *pos;
                continue;
            }
        }
        if t as c_int == false_0 {
            if found_pos.lnum != 0 as linenr_T {
                (*curwin.get()).w_cursor = found_pos;
                t = true_0 != 0;
            }
            break;
        } else if get_leader_len(
            get_cursor_line_ptr(),
            ::core::ptr::null_mut::<*mut c_char>(),
            false_0 != 0,
            true_0 != 0,
        ) > 0 as c_int
        {
            (*curwin.get()).w_cursor.lnum += 1;
            (*curwin.get()).w_cursor.col = 0 as c_int as colnr_T;
        } else {
            let mut valid: bool =
                is_ident(get_cursor_line_ptr(), (*curwin.get()).w_cursor.col as c_int);
            if !valid && found_pos.lnum != 0 as linenr_T {
                (*curwin.get()).w_cursor = found_pos;
                break;
            } else {
                if valid as c_int != 0 && !locally {
                    break;
                }
                if valid as c_int != 0 && (*curwin.get()).w_cursor.lnum >= par_pos.lnum {
                    if found_pos.lnum != 0 as linenr_T {
                        (*curwin.get()).w_cursor = found_pos;
                    }
                    break;
                } else {
                    if !valid {
                        clearpos(&mut found_pos);
                    } else {
                        found_pos = (*curwin.get()).w_cursor;
                    }
                    searchflags &= !(SEARCH_START as c_int);
                }
            }
        }
    }
    if t as c_int == false_0 {
        retval = false_0 != 0;
        (*curwin.get()).w_cursor = old_pos;
    } else {
        (*curwin.get()).w_set_curswant = true_0;
        reset_search_dir();
    }
    xfree(pat as *mut c_void);
    p_ws.set(save_p_ws as c_int);
    p_scs.set(save_p_scs as c_int);
    return retval;
}
pub unsafe extern "C" fn nv_screengo(
    mut oap: *mut oparg_T,
    mut dir: c_int,
    mut dist: c_int,
    mut skip_conceal: bool,
) -> bool {
    let mut linelen: c_int = linetabsize(curwin.get(), (*curwin.get()).w_cursor.lnum);
    let mut retval: bool = true_0 != 0;
    let mut atend: bool = false_0 != 0;
    let mut col_off1: c_int = 0;
    let mut col_off2: c_int = 0;
    let mut width1: c_int = 0;
    let mut width2: c_int = 0;
    (*oap).motion_type = kMTCharWise;
    (*oap).inclusive = (*curwin.get()).w_curswant == MAXCOL as c_int;
    col_off1 = win_col_off(curwin.get());
    col_off2 = col_off1 - win_col_off2(curwin.get());
    width1 = (*curwin.get()).w_view_width - col_off1;
    width2 = (*curwin.get()).w_view_width - col_off2;
    if width2 == 0 as c_int {
        width2 = 1 as c_int;
    }
    if (*curwin.get()).w_view_width != 0 as c_int {
        let mut n: c_int = 0;
        if (*curwin.get()).w_curswant == MAXCOL as c_int {
            atend = true_0 != 0;
            validate_virtcol(curwin.get());
            if width1 <= 0 as c_int {
                (*curwin.get()).w_curswant = 0 as c_int as colnr_T;
            } else {
                (*curwin.get()).w_curswant = (width1 - 1 as c_int) as colnr_T;
                if (*curwin.get()).w_virtcol > (*curwin.get()).w_curswant {
                    (*curwin.get()).w_curswant += (((*curwin.get()).w_virtcol as c_int
                        - (*curwin.get()).w_curswant as c_int
                        - 1 as c_int)
                        / width2
                        + 1 as c_int)
                        * width2;
                }
            }
        } else {
            if linelen > width1 {
                n = ((linelen - width1 - 1 as c_int) / width2 + 1 as c_int) * width2 + width1;
            } else {
                n = width1;
            }
            (*curwin.get()).w_curswant = (if (*curwin.get()).w_curswant < n - 1 as c_int {
                (*curwin.get()).w_curswant as c_int
            } else {
                n - 1 as c_int
            }) as colnr_T;
        }
        loop {
            let c2rust_fresh10 = dist;
            dist = dist - 1;
            if c2rust_fresh10 == 0 {
                break;
            }
            if dir == BACKWARD as c_int {
                if (*curwin.get()).w_curswant >= width1
                    && !hasFolding(
                        curwin.get(),
                        (*curwin.get()).w_cursor.lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        ::core::ptr::null_mut::<linenr_T>(),
                    )
                {
                    (*curwin.get()).w_curswant -= width2;
                } else if (*curwin.get()).w_cursor.lnum <= 1 as linenr_T {
                    retval = false_0 != 0;
                    break;
                } else {
                    cursor_up_inner(curwin.get(), 1 as linenr_T, skip_conceal);
                    linelen = linetabsize(curwin.get(), (*curwin.get()).w_cursor.lnum);
                    if linelen > width1 {
                        let mut w: c_int =
                            ((linelen - width1 - 1 as c_int) / width2 + 1 as c_int) * width2;
                        '_c2rust_label: {
                            if w <= 0 as c_int
                                || (*curwin.get()).w_curswant <= 2147483647 as c_int - w
                            {
                            } else {
                                __assert_fail(
                                    b"w <= 0 || curwin->w_curswant <= INT_MAX - w\0".as_ptr()
                                        as *const c_char,
                                    b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                                    2570 as c_uint,
                                    b"_Bool nv_screengo(oparg_T *, int, int, _Bool)\0".as_ptr()
                                        as *const c_char,
                                );
                            }
                        };
                        (*curwin.get()).w_curswant += w;
                    }
                }
            } else {
                if linelen > width1 {
                    n = ((linelen - width1 - 1 as c_int) / width2 + 1 as c_int) * width2 + width1;
                } else {
                    n = width1;
                }
                if (*curwin.get()).w_curswant as c_int + width2 < n
                    && !hasFolding(
                        curwin.get(),
                        (*curwin.get()).w_cursor.lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        ::core::ptr::null_mut::<linenr_T>(),
                    )
                {
                    (*curwin.get()).w_curswant += width2;
                } else if (*curwin.get()).w_cursor.lnum
                    >= (*(*curwin.get()).w_buffer).b_ml.ml_line_count
                {
                    retval = false_0 != 0;
                    break;
                } else {
                    cursor_down_inner(curwin.get(), 1 as c_int, skip_conceal);
                    (*curwin.get()).w_curswant %= width2;
                    if (*curwin.get()).w_curswant >= width1 {
                        (*curwin.get()).w_curswant -= width2;
                    }
                    linelen = linetabsize(curwin.get(), (*curwin.get()).w_cursor.lnum);
                }
            }
        }
    }
    if virtual_active(curwin.get()) as c_int != 0 && atend as c_int != 0 {
        coladvance(curwin.get(), MAXCOL as c_int);
    } else {
        coladvance(curwin.get(), (*curwin.get()).w_curswant);
    }
    if (*curwin.get()).w_cursor.col > 0 as c_int && (*curwin.get()).w_onebuf_opt.wo_wrap != 0 {
        validate_virtcol(curwin.get());
        let mut virtcol: colnr_T = (*curwin.get()).w_virtcol;
        if virtcol > width1 && *get_showbreak_value(curwin.get()) as c_int != NUL {
            virtcol -= vim_strsize(get_showbreak_value(curwin.get()));
        }
        let mut c: c_int = utf_ptr2char(get_cursor_pos_ptr());
        if dir == FORWARD as c_int
            && virtcol < (*curwin.get()).w_curswant
            && (*curwin.get()).w_curswant <= width1
            && !vim_isprintc(c)
            && c > 255 as c_int
        {
            oneright();
        }
        if virtcol > (*curwin.get()).w_curswant
            && (if (*curwin.get()).w_curswant < width1 {
                ((*curwin.get()).w_curswant > width1 / 2 as c_int) as c_int
            } else {
                (((*curwin.get()).w_curswant as c_int - width1) % width2 > width2 / 2 as c_int)
                    as c_int
            }) != 0
        {
            (*curwin.get()).w_cursor.col -= 1;
        }
    }
    if atend {
        (*curwin.get()).w_curswant = MAXCOL as c_int as colnr_T;
    }
    adjust_skipcol();
    return retval;
}
pub unsafe extern "C" fn nv_scroll_line(mut cap: *mut cmdarg_T) {
    if !checkclearop((*cap).oap) {
        scroll_redraw((*cap).arg, (*cap).count1 as linenr_T);
    }
}
unsafe extern "C" fn nv_z_get_count(mut cap: *mut cmdarg_T, mut nchar_arg: *mut c_int) -> bool {
    let mut nchar: c_int = *nchar_arg;
    if checkclearop((*cap).oap) {
        return false_0 != 0;
    }
    let mut n: c_int = nchar - '0' as c_int;
    loop {
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        nchar = plain_vgetc();
        if *p_langmap.get() as c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as c_int) as c_int
                } else {
                    KeyTyped.get() as c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && nchar >= 0 as c_int
        {
            if nchar < 256 as c_int {
                nchar = (*langmap_mapchar.ptr())[nchar as usize] as c_int;
            } else {
                nchar = langmap_adjust_mb(nchar);
            }
        }
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        add_to_showcmd(nchar);
        if nchar == K_DEL || nchar == -(253 as c_int + ((KE_KDEL as c_int) << 8 as c_int)) {
            n /= 10 as c_int;
        } else if ascii_isdigit(nchar) {
            if crate::src::nvim::math::vim_append_digit_int(&mut n, nchar - '0' as c_int) {
                continue;
            }
            clearopbeep((*cap).oap);
            break;
        } else if nchar == CAR {
            win_setheight(n);
            break;
        } else if nchar == 'l' as c_int
            || nchar == 'h' as c_int
            || nchar == K_LEFT
            || nchar == K_RIGHT
        {
            (*cap).count1 = if n != 0 {
                n * (*cap).count1
            } else {
                (*cap).count1
            };
            *nchar_arg = nchar;
            return true_0 != 0;
        } else {
            clearopbeep((*cap).oap);
            break;
        }
    }
    (*(*cap).oap).op_type = OP_NOP as c_int;
    return false_0 != 0;
}
unsafe extern "C" fn nv_zg_zw(mut cap: *mut cmdarg_T, mut nchar: c_int) -> c_int {
    let mut undo: bool = false_0 != 0;
    if nchar == 'u' as c_int {
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        nchar = plain_vgetc();
        if *p_langmap.get() as c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as c_int) as c_int
                } else {
                    KeyTyped.get() as c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && nchar >= 0 as c_int
        {
            if nchar < 256 as c_int {
                nchar = (*langmap_mapchar.ptr())[nchar as usize] as c_int;
            } else {
                nchar = langmap_adjust_mb(nchar);
            }
        }
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        add_to_showcmd(nchar);
        if vim_strchr(b"gGwW\0".as_ptr() as *const c_char, nchar).is_null() {
            clearopbeep((*cap).oap);
            return OK;
        }
        undo = true_0 != 0;
    }
    if checkclearop((*cap).oap) {
        return OK;
    }
    let mut ptr: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut len: size_t = 0;
    if VIsual_active.get() as c_int != 0 && !get_visual_text(cap, &raw mut ptr, &raw mut len) {
        return FAIL;
    }
    if ptr.is_null() {
        let mut pos: pos_T = (*curwin.get()).w_cursor;
        (*emsg_off.ptr()) += 1;
        len = spell_move_to(
            curwin.get(),
            FORWARD as c_int,
            SMT_ALL,
            true_0 != 0,
            ::core::ptr::null_mut::<hlf_T>(),
        );
        (*emsg_off.ptr()) -= 1;
        if len != 0 as size_t && (*curwin.get()).w_cursor.col <= pos.col {
            ptr = ml_get_pos(&raw mut (*curwin.get()).w_cursor);
        }
        (*curwin.get()).w_cursor = pos;
    }
    if ptr.is_null() && {
        len = find_ident_under_cursor(
            &raw mut ptr,
            FIND_IDENT as c_int,
            ::core::ptr::null_mut::<c_int>(),
        );
        len == 0 as size_t
    } {
        return FAIL;
    }
    '_c2rust_label: {
        if len <= 2147483647 as c_int as size_t {
        } else {
            __assert_fail(
                b"len <= INT_MAX\0".as_ptr() as *const c_char,
                b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                2754 as c_uint,
                b"int nv_zg_zw(cmdarg_T *, int)\0".as_ptr() as *const c_char,
            );
        }
    };
    spell_add_word(
        ptr,
        len as c_int,
        (if nchar == 'w' as c_int || nchar == 'W' as c_int {
            SPELL_ADD_BAD as c_int
        } else {
            SPELL_ADD_GOOD as c_int
        }) as SpellAddType,
        if nchar == 'G' as c_int || nchar == 'W' as c_int {
            0 as c_int
        } else {
            (*cap).count1
        },
        undo,
    );
    return OK;
}
unsafe extern "C" fn nv_zet(mut cap: *mut cmdarg_T) {
    let mut col: colnr_T = 0;
    let mut nchar: c_int = (*cap).nchar;
    let mut old_fdl: c_int = (*curwin.get()).w_onebuf_opt.wo_fdl as c_int;
    let mut old_fen: c_int = (*curwin.get()).w_onebuf_opt.wo_fen;
    let mut siso: int64_t = get_sidescrolloff_value(curwin.get());
    if ascii_isdigit(nchar) as c_int != 0 && !nv_z_get_count(cap, &raw mut nchar) {
        return;
    }
    if (*cap).nchar != 'f' as c_int
        && (*cap).nchar != 'F' as c_int
        && !(VIsual_active.get() as c_int != 0
            && !vim_strchr(b"dcCoO\0".as_ptr() as *const c_char, (*cap).nchar).is_null())
        && (*cap).nchar != 'j' as c_int
        && (*cap).nchar != 'k' as c_int
        && checkclearop((*cap).oap) as c_int != 0
    {
        return;
    }
    if !vim_strchr(b"+\r\nt.z^-b\0".as_ptr() as *const c_char, nchar).is_null()
        && (*cap).count0 != 0
        && (*cap).count0 as linenr_T != (*curwin.get()).w_cursor.lnum
    {
        setpcmark();
        if (*cap).count0 as linenr_T > (*curbuf.get()).b_ml.ml_line_count {
            (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
        } else {
            (*curwin.get()).w_cursor.lnum = (*cap).count0 as linenr_T;
        }
        check_cursor_col(curwin.get());
    }
    's_906: {
        'c_53178: {
            'c_53195: {
                'c_55145: {
                    'c_55198: {
                        'c_53192: {
                            'c_55413: {
                                match nchar {
                                    43 => {
                                        if (*cap).count0 == 0 as c_int {
                                            validate_botline_win(curwin.get());
                                            (*curwin.get()).w_cursor.lnum = if (*curwin.get())
                                                .w_botline
                                                < (*curbuf.get()).b_ml.ml_line_count
                                            {
                                                (*curwin.get()).w_botline
                                            } else {
                                                (*curbuf.get()).b_ml.ml_line_count
                                            };
                                        }
                                        break 'c_55413;
                                    }
                                    NL | CAR | K_KENTER => {
                                        break 'c_55413;
                                    }
                                    116 => {
                                        break 'c_53178;
                                    }
                                    46 => {
                                        beginline(BL_WHITE as c_int | BL_FIX as c_int);
                                    }
                                    122 => {}
                                    94 => {
                                        if (*cap).count0 != 0 as c_int {
                                            scroll_cursor_bot(
                                                curwin.get(),
                                                0 as c_int,
                                                true_0 != 0,
                                            );
                                            (*curwin.get()).w_cursor.lnum =
                                                (*curwin.get()).w_topline;
                                        } else if (*curwin.get()).w_topline == 1 as linenr_T {
                                            (*curwin.get()).w_cursor.lnum = 1 as c_int as linenr_T;
                                        } else {
                                            (*curwin.get()).w_cursor.lnum =
                                                (*curwin.get()).w_topline - 1 as linenr_T;
                                        }
                                        break 'c_53192;
                                    }
                                    45 => {
                                        break 'c_53192;
                                    }
                                    98 => {
                                        break 'c_53195;
                                    }
                                    72 => {
                                        (*cap).count1 *= (*curwin.get()).w_view_width / 2 as c_int;
                                        break 'c_55198;
                                    }
                                    104 | K_LEFT => {
                                        break 'c_55198;
                                    }
                                    76 => {
                                        (*cap).count1 *= (*curwin.get()).w_view_width / 2 as c_int;
                                        break 'c_55145;
                                    }
                                    108 | K_RIGHT => {
                                        break 'c_55145;
                                    }
                                    115 => {
                                        if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                                            if hasFolding(
                                                curwin.get(),
                                                (*curwin.get()).w_cursor.lnum,
                                                ::core::ptr::null_mut::<linenr_T>(),
                                                ::core::ptr::null_mut::<linenr_T>(),
                                            ) {
                                                col = 0 as c_int as colnr_T;
                                            } else {
                                                getvcol(
                                                    curwin.get(),
                                                    &raw mut (*curwin.get()).w_cursor,
                                                    &raw mut col,
                                                    ::core::ptr::null_mut::<colnr_T>(),
                                                    ::core::ptr::null_mut::<colnr_T>(),
                                                );
                                            }
                                            if col as int64_t > siso {
                                                col -= siso as c_int;
                                            } else {
                                                col = 0 as c_int as colnr_T;
                                            }
                                            if (*curwin.get()).w_leftcol != col {
                                                (*curwin.get()).w_leftcol = col;
                                                redraw_later(curwin.get(), UPD_NOT_VALID as c_int);
                                            }
                                        }
                                        break 's_906;
                                    }
                                    101 => {
                                        if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                                            if hasFolding(
                                                curwin.get(),
                                                (*curwin.get()).w_cursor.lnum,
                                                ::core::ptr::null_mut::<linenr_T>(),
                                                ::core::ptr::null_mut::<linenr_T>(),
                                            ) {
                                                col = 0 as c_int as colnr_T;
                                            } else {
                                                getvcol(
                                                    curwin.get(),
                                                    &raw mut (*curwin.get()).w_cursor,
                                                    ::core::ptr::null_mut::<colnr_T>(),
                                                    ::core::ptr::null_mut::<colnr_T>(),
                                                    &raw mut col,
                                                );
                                            }
                                            let mut n: c_int = (*curwin.get()).w_view_width
                                                - win_col_off(curwin.get());
                                            if col as int64_t + siso < n as int64_t {
                                                col = 0 as c_int as colnr_T;
                                            } else if (siso - n as int64_t)
                                                < (INT_MAX - col) as int64_t
                                            {
                                                col = (col as int64_t + siso - n as int64_t
                                                    + 1 as int64_t)
                                                    as c_int
                                                    as colnr_T;
                                            } else {
                                                col = INT_MAX as colnr_T;
                                            }
                                            if (*curwin.get()).w_leftcol != col {
                                                (*curwin.get()).w_leftcol = col;
                                                redraw_later(curwin.get(), UPD_NOT_VALID as c_int);
                                            }
                                        }
                                        break 's_906;
                                    }
                                    80 | 112 => {
                                        nv_put(cap);
                                        break 's_906;
                                    }
                                    121 => {
                                        nv_operator(cap);
                                        break 's_906;
                                    }
                                    70 | 102 => {
                                        if foldManualAllowed(true_0 != 0) != 0 {
                                            (*cap).nchar = 'f' as c_int;
                                            nv_operator(cap);
                                            (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                            if nchar == 'F' as c_int
                                                && (*(*cap).oap).op_type == OP_FOLD as c_int
                                            {
                                                nv_operator(cap);
                                                finish_op.set(true_0 != 0);
                                            }
                                        } else {
                                            clearopbeep((*cap).oap);
                                        }
                                        break 's_906;
                                    }
                                    100 | 68 => {
                                        if foldManualAllowed(false_0 != 0) != 0 {
                                            if VIsual_active.get() {
                                                nv_operator(cap);
                                            } else {
                                                deleteFold(
                                                    curwin.get(),
                                                    (*curwin.get()).w_cursor.lnum,
                                                    (*curwin.get()).w_cursor.lnum,
                                                    (nchar == 'D' as c_int) as c_int,
                                                    false_0 != 0,
                                                );
                                            }
                                        }
                                        break 's_906;
                                    }
                                    69 => {
                                        if foldmethodIsManual(curwin.get()) {
                                            clearFolding(curwin.get());
                                            changed_window_setting(curwin.get());
                                        } else if foldmethodIsMarker(curwin.get()) {
                                            deleteFold(
                                                curwin.get(),
                                                1 as linenr_T,
                                                (*curbuf.get()).b_ml.ml_line_count,
                                                true_0,
                                                false_0 != 0,
                                            );
                                        } else {
                                            emsg(
                                                gettext(
                                                    b"E352: Cannot erase folds with current 'foldmethod'\0"
                                                        .as_ptr() as *const c_char,
                                                ),
                                            );
                                        }
                                        break 's_906;
                                    }
                                    110 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fen = false_0;
                                        break 's_906;
                                    }
                                    78 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        break 's_906;
                                    }
                                    105 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fen =
                                            ((*curwin.get()).w_onebuf_opt.wo_fen == 0) as c_int;
                                        break 's_906;
                                    }
                                    97 => {
                                        if hasFolding(
                                            curwin.get(),
                                            (*curwin.get()).w_cursor.lnum,
                                            ::core::ptr::null_mut::<linenr_T>(),
                                            ::core::ptr::null_mut::<linenr_T>(),
                                        ) {
                                            openFold((*curwin.get()).w_cursor, (*cap).count1);
                                        } else {
                                            closeFold((*curwin.get()).w_cursor, (*cap).count1);
                                            (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        }
                                        break 's_906;
                                    }
                                    65 => {
                                        if hasFolding(
                                            curwin.get(),
                                            (*curwin.get()).w_cursor.lnum,
                                            ::core::ptr::null_mut::<linenr_T>(),
                                            ::core::ptr::null_mut::<linenr_T>(),
                                        ) {
                                            openFoldRecurse((*curwin.get()).w_cursor);
                                        } else {
                                            closeFoldRecurse((*curwin.get()).w_cursor);
                                            (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        }
                                        break 's_906;
                                    }
                                    111 => {
                                        if VIsual_active.get() {
                                            nv_operator(cap);
                                        } else {
                                            openFold((*curwin.get()).w_cursor, (*cap).count1);
                                        }
                                        break 's_906;
                                    }
                                    79 => {
                                        if VIsual_active.get() {
                                            nv_operator(cap);
                                        } else {
                                            openFoldRecurse((*curwin.get()).w_cursor);
                                        }
                                        break 's_906;
                                    }
                                    99 => {
                                        if VIsual_active.get() {
                                            nv_operator(cap);
                                        } else {
                                            closeFold((*curwin.get()).w_cursor, (*cap).count1);
                                        }
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        break 's_906;
                                    }
                                    67 => {
                                        if VIsual_active.get() {
                                            nv_operator(cap);
                                        } else {
                                            closeFoldRecurse((*curwin.get()).w_cursor);
                                        }
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        break 's_906;
                                    }
                                    118 => {
                                        foldOpenCursor();
                                        break 's_906;
                                    }
                                    120 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        (*curwin.get()).w_foldinvalid = true_0 != 0;
                                        newFoldLevel();
                                        foldOpenCursor();
                                        break 's_906;
                                    }
                                    88 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        (*curwin.get()).w_foldinvalid = true_0 != 0;
                                        old_fdl = -1 as c_int;
                                        break 's_906;
                                    }
                                    109 => {
                                        if (*curwin.get()).w_onebuf_opt.wo_fdl > 0 as OptInt {
                                            (*curwin.get()).w_onebuf_opt.wo_fdl -=
                                                (*cap).count1 as OptInt;
                                            (*curwin.get()).w_onebuf_opt.wo_fdl = if (*curwin.get())
                                                .w_onebuf_opt
                                                .wo_fdl
                                                > 0 as OptInt
                                            {
                                                (*curwin.get()).w_onebuf_opt.wo_fdl
                                            } else {
                                                0 as OptInt
                                            };
                                        }
                                        old_fdl = -1 as c_int;
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        break 's_906;
                                    }
                                    77 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fdl = 0 as OptInt;
                                        old_fdl = -1 as c_int;
                                        (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
                                        break 's_906;
                                    }
                                    114 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fdl +=
                                            (*cap).count1 as OptInt;
                                        let mut d: c_int = getDeepestNesting(curwin.get());
                                        (*curwin.get()).w_onebuf_opt.wo_fdl =
                                            if (*curwin.get()).w_onebuf_opt.wo_fdl < d as OptInt {
                                                (*curwin.get()).w_onebuf_opt.wo_fdl
                                            } else {
                                                d as OptInt
                                            };
                                        break 's_906;
                                    }
                                    82 => {
                                        (*curwin.get()).w_onebuf_opt.wo_fdl =
                                            getDeepestNesting(curwin.get()) as OptInt;
                                        old_fdl = -1 as c_int;
                                        break 's_906;
                                    }
                                    106 | 107 => {
                                        if foldMoveTo(
                                            true_0 != 0,
                                            if nchar == 'j' as c_int {
                                                FORWARD as c_int
                                            } else {
                                                BACKWARD as c_int
                                            },
                                            (*cap).count1,
                                        ) == false_0
                                        {
                                            clearopbeep((*cap).oap);
                                        }
                                        break 's_906;
                                    }
                                    117 | 103 | 119 | 71 | 87 => {
                                        if nv_zg_zw(cap, nchar) == FAIL {
                                            return;
                                        }
                                        break 's_906;
                                    }
                                    61 => {
                                        if !checkclearop((*cap).oap) {
                                            spell_suggest((*cap).count0);
                                        }
                                        break 's_906;
                                    }
                                    _ => {
                                        clearopbeep((*cap).oap);
                                        break 's_906;
                                    }
                                }
                                scroll_cursor_halfway(curwin.get(), true_0 != 0, false_0 != 0);
                                redraw_later(curwin.get(), UPD_VALID as c_int);
                                set_fraction(curwin.get());
                                break 's_906;
                            }
                            beginline(BL_WHITE as c_int | BL_FIX as c_int);
                            break 'c_53178;
                        }
                        beginline(BL_WHITE as c_int | BL_FIX as c_int);
                        break 'c_53195;
                    }
                    if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                        set_leftcol(if (*cap).count1 > (*curwin.get()).w_leftcol {
                            0 as colnr_T
                        } else {
                            (*curwin.get()).w_leftcol - (*cap).count1
                        });
                    }
                    break 's_906;
                }
                if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                    set_leftcol((*curwin.get()).w_leftcol + (*cap).count1);
                }
                break 's_906;
            }
            scroll_cursor_bot(curwin.get(), 0 as c_int, true_0 != 0);
            redraw_later(curwin.get(), UPD_VALID as c_int);
            set_fraction(curwin.get());
            break 's_906;
        }
        scroll_cursor_top(curwin.get(), 0 as c_int, true_0);
        redraw_later(curwin.get(), UPD_VALID as c_int);
        set_fraction(curwin.get());
    }
    if old_fen != (*curwin.get()).w_onebuf_opt.wo_fen {
        if foldmethodIsDiff(curwin.get()) as c_int != 0 && (*curwin.get()).w_onebuf_opt.wo_scb != 0
        {
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                if wp != curwin.get()
                    && foldmethodIsDiff(wp) as c_int != 0
                    && (*wp).w_onebuf_opt.wo_scb != 0
                {
                    (*wp).w_onebuf_opt.wo_fen = (*curwin.get()).w_onebuf_opt.wo_fen;
                    changed_window_setting(wp);
                }
                wp = (*wp).w_next;
            }
        }
        changed_window_setting(curwin.get());
    }
    if old_fdl as OptInt != (*curwin.get()).w_onebuf_opt.wo_fdl {
        newFoldLevel();
    }
}
unsafe extern "C" fn nv_regreplay(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    loop {
        let c2rust_fresh11 = (*cap).count1;
        (*cap).count1 = (*cap).count1 - 1;
        if !(c2rust_fresh11 != 0 && !got_int.get()) {
            break;
        }
        if do_execreg(reg_recorded.get(), false_0, false_0, false_0) == false_0 {
            clearopbeep((*cap).oap);
            break;
        } else {
            line_breakcheck();
        }
    }
}
unsafe extern "C" fn nv_colon(mut cap: *mut cmdarg_T) {
    let mut cmd_result: bool = false;
    let mut is_cmdkey: bool =
        (*cap).cmdchar == -(253 as c_int + ((KE_COMMAND as c_int) << 8 as c_int));
    let mut is_lua: bool = (*cap).cmdchar == -(253 as c_int + ((KE_LUA as c_int) << 8 as c_int));
    if VIsual_active.get() as c_int != 0 && !is_cmdkey && !is_lua {
        nv_operator(cap);
        return;
    }
    if (*(*cap).oap).op_type != OP_NOP as c_int {
        (*(*cap).oap).motion_type = kMTCharWise;
        (*(*cap).oap).inclusive = false_0 != 0;
    } else if (*cap).count0 != 0 && !is_cmdkey && !is_lua {
        stuffcharReadbuff('.' as c_int);
        if (*cap).count0 > 1 as c_int {
            stuffReadbuff(b",.+\0".as_ptr() as *const c_char);
            stuffnumReadbuff((*cap).count0 - 1 as c_int);
        }
    }
    if KeyTyped.get() {
        msg_ext_set_trigger(b"typed_cmd\0".as_ptr() as *const c_char);
        compute_cmdrow();
    }
    if is_lua {
        cmd_result = map_execute_lua(true_0 != 0, false_0 != 0);
    } else {
        cmd_result = do_cmdline(
            ::core::ptr::null_mut::<c_char>(),
            if is_cmdkey as c_int != 0 {
                Some(
                    getcmdkeycmd
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                )
            } else {
                Some(
                    getexline
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                )
            },
            NULL,
            if (*(*cap).oap).op_type != OP_NOP as c_int {
                DOCMD_KEEPLINE as c_int
            } else {
                0 as c_int
            },
        ) != 0;
    }
    msg_ext_set_trigger(b"\0".as_ptr() as *const c_char);
    if cmd_result as c_int == false_0 {
        clearop((*cap).oap);
    } else if (*(*cap).oap).op_type != OP_NOP as c_int
        && ((*(*cap).oap).start.lnum > (*curbuf.get()).b_ml.ml_line_count
            || (*(*cap).oap).start.col > ml_get_len((*(*cap).oap).start.lnum)
            || did_emsg.get() != 0)
    {
        clearopbeep((*cap).oap);
    }
}
unsafe extern "C" fn nv_ctrlg(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() {
        VIsual_select.set(!VIsual_select.get());
        may_trigger_modechanged();
        showmode();
    } else if !checkclearop((*cap).oap) {
        fileinfo((*cap).count0, false_0, true_0 != 0);
    }
}
unsafe extern "C" fn nv_ctrlh(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() as c_int != 0 && VIsual_select.get() as c_int != 0 {
        (*cap).cmdchar = 'x' as c_int;
        v_visop(cap);
    } else {
        nv_left(cap);
    };
}
unsafe extern "C" fn nv_clear(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    syn_stack_free_all((*curwin.get()).w_s);
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        (*(*wp).w_s).b_syn_slow = false_0 != 0;
        wp = (*wp).w_next;
    }
    redraw_later(curwin.get(), UPD_CLEAR as c_int);
}
unsafe extern "C" fn nv_ctrlo(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() as c_int != 0 && VIsual_select.get() as c_int != 0 {
        VIsual_select.set(false_0 != 0);
        may_trigger_modechanged();
        showmode();
        restart_VIsual_select.set(2 as c_int);
    } else {
        (*cap).count1 = -(*cap).count1;
        nv_pcmark(cap);
    };
}
unsafe extern "C" fn nv_hat(mut cap: *mut cmdarg_T) {
    if !checkclearopq((*cap).oap) {
        buflist_getfile(
            (*cap).count0,
            0 as linenr_T,
            GETF_SETMARK as c_int | GETF_ALT as c_int,
            false_0,
        );
    }
}
unsafe extern "C" fn nv_Zet(mut cap: *mut cmdarg_T) {
    if checkclearopq((*cap).oap) {
        return;
    }
    match (*cap).nchar {
        90 => {
            do_cmdline_cmd(b"x\0".as_ptr() as *const c_char);
        }
        81 => {
            do_cmdline_cmd(b"q!\0".as_ptr() as *const c_char);
        }
        82 => {
            if (*cap).count0 >= 1 as c_int {
                do_cmdline_cmd(b"restart +qall!\0".as_ptr() as *const c_char);
            } else {
                do_cmdline_cmd(b"restart\0".as_ptr() as *const c_char);
            }
        }
        _ => {
            clearopbeep((*cap).oap);
        }
    };
}
pub unsafe extern "C" fn do_nv_ident(mut c1: c_int, mut c2: c_int) {
    let mut oa: oparg_T = oparg_T {
        op_type: 0,
        regname: 0,
        motion_type: kMTCharWise,
        motion_force: 0,
        use_reg_one: false,
        inclusive: false,
        end_adjusted: false,
        start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        end: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cursor_start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        line_count: 0,
        empty: false,
        is_VIsual: false,
        start_vcol: 0,
        end_vcol: 0,
        prev_opcount: 0,
        prev_count0: 0,
        excl_tr_ws: false,
    };
    let mut ca: cmdarg_T = cmdarg_T {
        oap: ::core::ptr::null_mut::<oparg_T>(),
        prechar: 0,
        cmdchar: 0,
        nchar: 0,
        nchar_composing: [0; 32],
        nchar_len: 0,
        extra_char: 0,
        opcount: 0,
        count0: 0,
        count1: 0,
        arg: 0,
        retval: 0,
        searchbuf: ::core::ptr::null_mut::<c_char>(),
    };
    clear_oparg(&raw mut oa);
    memset(
        &raw mut ca as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<cmdarg_T>(),
    );
    ca.oap = &raw mut oa;
    ca.cmdchar = c1;
    ca.nchar = c2;
    nv_ident(&raw mut ca);
}
unsafe extern "C" fn nv_K_getcmd(
    mut cap: *mut cmdarg_T,
    mut kp: *mut c_char,
    mut kp_help: bool,
    mut kp_ex: bool,
    mut ptr_arg: *mut *mut c_char,
    mut n: size_t,
    mut buf: *mut c_char,
    mut bufsize: size_t,
    mut buflen: *mut size_t,
) -> size_t {
    if kp_help {
        strcpy(buf, b"help! \0".as_ptr() as *const c_char as *mut c_char);
        *buflen = ::core::mem::size_of::<[c_char; 7]>().wrapping_sub(1 as usize) as size_t;
        return n;
    }
    if kp_ex {
        *buflen = 0 as size_t;
        *buflen = snprintf(buf, bufsize, b"%s \0".as_ptr() as *const c_char, kp) as size_t;
        if (*cap).count0 != 0 as c_int {
            *buflen = (*buflen).wrapping_add(snprintf(
                buf.offset(*buflen as isize),
                bufsize.wrapping_sub(*buflen),
                b"%ld \0".as_ptr() as *const c_char,
                (*cap).count0 as int64_t,
            ) as size_t);
        }
        return n;
    }
    let mut ptr: *mut c_char = *ptr_arg;
    while *ptr as c_int == '-' as c_int && n > 0 as size_t {
        ptr = ptr.offset(1);
        n = n.wrapping_sub(1);
    }
    if n == 0 as size_t {
        emsg(gettext(&raw const e_noident as *const c_char));
        xfree(buf as *mut c_void);
        *ptr_arg = ptr;
        return 0 as size_t;
    }
    let mut isman: bool = strcmp(kp, b"man\0".as_ptr() as *const c_char) == 0 as c_int;
    let mut isman_s: bool = strcmp(kp, b"man -s\0".as_ptr() as *const c_char) == 0 as c_int;
    if (*cap).count0 != 0 as c_int && !(isman as c_int != 0 || isman_s as c_int != 0) {
        *buflen = snprintf(
            buf,
            bufsize,
            b".,.+%ld\0".as_ptr() as *const c_char,
            ((*cap).count0 - 1 as c_int) as int64_t,
        ) as size_t;
    }
    do_cmdline_cmd(b"tabnew\0".as_ptr() as *const c_char);
    *buflen = (*buflen).wrapping_add(snprintf(
        buf.offset(*buflen as isize),
        bufsize.wrapping_sub(*buflen),
        b"terminal \0".as_ptr() as *const c_char,
    ) as size_t);
    if (*cap).count0 == 0 as c_int && isman_s as c_int != 0 {
        *buflen = (*buflen).wrapping_add(snprintf(
            buf.offset(*buflen as isize),
            bufsize.wrapping_sub(*buflen),
            b"man \0".as_ptr() as *const c_char,
        ) as size_t);
    } else {
        *buflen = (*buflen).wrapping_add(snprintf(
            buf.offset(*buflen as isize),
            bufsize.wrapping_sub(*buflen),
            b"%s \0".as_ptr() as *const c_char,
            kp,
        ) as size_t);
    }
    if (*cap).count0 != 0 as c_int && (isman as c_int != 0 || isman_s as c_int != 0) {
        *buflen = (*buflen).wrapping_add(snprintf(
            buf.offset(*buflen as isize),
            bufsize.wrapping_sub(*buflen),
            b"%ld \0".as_ptr() as *const c_char,
            (*cap).count0 as int64_t,
        ) as size_t);
    }
    *ptr_arg = ptr;
    return n;
}
unsafe extern "C" fn nv_ident(mut cap: *mut cmdarg_T) {
    let mut ptr: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut n: size_t = 0 as size_t;
    let mut cmdchar: c_int = 0;
    let mut g_cmd: bool = false;
    let mut tag_cmd: bool = false_0 != 0;
    if (*cap).cmdchar == 'g' as c_int {
        cmdchar = (*cap).nchar;
        g_cmd = true_0 != 0;
    } else {
        cmdchar = (*cap).cmdchar;
        g_cmd = false_0 != 0;
    }
    if cmdchar == POUND {
        cmdchar = '#' as c_int;
    }
    let mut visual_sel: bool = false_0 != 0;
    if cmdchar == ']' as c_int || cmdchar == Ctrl_RSB || cmdchar == 'K' as c_int {
        if VIsual_active.get() as c_int != 0
            && get_visual_text(cap, &raw mut ptr, &raw mut n) as c_int == false_0
        {
            return;
        }
        visual_sel = !ptr.is_null();
        if checkclearopq((*cap).oap) {
            return;
        }
    }
    let mut ident_offset: c_int = 0 as c_int;
    if ptr.is_null() && {
        n = find_ident_under_cursor(
            &raw mut ptr,
            if cmdchar == '*' as c_int || cmdchar == '#' as c_int {
                FIND_IDENT as c_int | FIND_STRING as c_int
            } else {
                FIND_IDENT as c_int
            },
            &raw mut ident_offset,
        );
        n == 0 as size_t
    } {
        clearop((*cap).oap);
        return;
    }
    let mut kp: *mut c_char = if *(*curbuf.get()).b_p_kp as c_int == NUL {
        p_kp.get()
    } else {
        (*curbuf.get()).b_p_kp
    };
    let mut kp_helpbang: bool = strequal(kp, b":help!\0".as_ptr() as *const c_char);
    let mut kp_help: bool = kp_helpbang as c_int != 0
        || *kp as c_int == NUL
        || strequal(kp, b":he\0".as_ptr() as *const c_char) as c_int != 0
        || strequal(kp, b":help\0".as_ptr() as *const c_char) as c_int != 0;
    if kp_help as c_int != 0 && !kp_helpbang && *skipwhite(ptr) as c_int == NUL {
        emsg(gettext(&raw const e_noident as *const c_char));
        return;
    }
    let mut kp_ex: bool = *kp as c_int == ':' as c_int;
    let mut bufsize: size_t = n
        .wrapping_mul(2 as size_t)
        .wrapping_add(30 as size_t)
        .wrapping_add(strlen(kp));
    let mut buf: *mut c_char = xmalloc(bufsize) as *mut c_char;
    *buf.offset(0 as c_int as isize) = NUL as c_char;
    let mut buflen: size_t = 0 as size_t;
    match cmdchar {
        42 | 35 => {
            setpcmark();
            (*curwin.get()).w_cursor.col = ptr.offset_from(get_cursor_line_ptr()) as colnr_T;
            if !g_cmd && vim_iswordp(ptr) as c_int != 0 {
                strcpy(buf, b"\\<\0".as_ptr() as *const c_char as *mut c_char);
                buflen = ::core::mem::size_of::<[c_char; 3]>().wrapping_sub(1 as usize) as size_t;
            }
            no_smartcase.set(true_0 != 0);
        }
        75 => {
            n = nv_K_getcmd(
                cap,
                kp,
                kp_help,
                kp_ex,
                &raw mut ptr,
                n,
                buf,
                bufsize,
                &raw mut buflen,
            );
            if n == 0 as size_t {
                return;
            }
        }
        93 => {
            tag_cmd = true_0 != 0;
            strcpy(buf, b"tselect \0".as_ptr() as *const c_char as *mut c_char);
            buflen = ::core::mem::size_of::<[c_char; 9]>().wrapping_sub(1 as usize) as size_t;
        }
        _ => {
            tag_cmd = true_0 != 0;
            if (*curbuf.get()).b_help {
                strcpy(buf, b"help! \0".as_ptr() as *const c_char as *mut c_char);
                buflen = ::core::mem::size_of::<[c_char; 7]>().wrapping_sub(1 as usize) as size_t;
            } else if g_cmd {
                strcpy(buf, b"tjump \0".as_ptr() as *const c_char as *mut c_char);
                buflen = ::core::mem::size_of::<[c_char; 7]>().wrapping_sub(1 as usize) as size_t;
            } else if (*cap).count0 == 0 as c_int {
                strcpy(buf, b"tag \0".as_ptr() as *const c_char as *mut c_char);
                buflen = ::core::mem::size_of::<[c_char; 5]>().wrapping_sub(1 as usize) as size_t;
            } else {
                buflen = snprintf(
                    buf,
                    bufsize,
                    b":%ldtag \0".as_ptr() as *const c_char,
                    (*cap).count0 as int64_t,
                ) as size_t;
            }
        }
    }
    if cmdchar == 'K' as c_int && kp_helpbang as c_int != 0 && !visual_sel {
        strcpy(buf, b"help!\0".as_ptr() as *const c_char as *mut c_char);
        buflen = ::core::mem::size_of::<[c_char; 6]>().wrapping_sub(1 as usize) as size_t;
    } else if cmdchar == 'K' as c_int && !kp_help {
        ptr = xstrnsave(ptr, n);
        if kp_ex {
            p = vim_strsave_fnameescape(ptr, VSE_NONE as c_int);
        } else {
            p = vim_strsave_shellescape(ptr, true_0 != 0, true_0 != 0);
        }
        xfree(ptr as *mut c_void);
        let mut plen: size_t = strlen(p);
        let mut newbuf: *mut c_char = xrealloc(
            buf as *mut c_void,
            buflen.wrapping_add(plen).wrapping_add(1 as size_t),
        ) as *mut c_char;
        buf = newbuf;
        strcpy(buf.offset(buflen as isize), p);
        buflen = buflen.wrapping_add(plen);
        xfree(p as *mut c_void);
    } else {
        let mut aux_ptr: *mut c_char = ::core::ptr::null_mut::<c_char>();
        if cmdchar == '*' as c_int {
            aux_ptr = (if magic_isset() as c_int != 0 {
                b"/.*~[^$\\\0".as_ptr() as *const c_char
            } else {
                b"/^$\\\0".as_ptr() as *const c_char
            }) as *mut c_char;
        } else if cmdchar == '#' as c_int {
            aux_ptr = (if magic_isset() as c_int != 0 {
                b"/?.*~[^$\\\0".as_ptr() as *const c_char
            } else {
                b"/?^$\\\0".as_ptr() as *const c_char
            }) as *mut c_char;
        } else if tag_cmd {
            if strcmp((*curbuf.get()).b_p_ft, b"help\0".as_ptr() as *const c_char) == 0 as c_int {
                aux_ptr = b"\0".as_ptr() as *const c_char as *mut c_char;
            } else {
                aux_ptr = b"\\|\"\n[\0".as_ptr() as *const c_char as *mut c_char;
            }
        } else {
            aux_ptr = b"\\|\"\n*?[\0".as_ptr() as *const c_char as *mut c_char;
        }
        p = buf.offset(buflen as isize);
        loop {
            let c2rust_fresh0 = n;
            n = n.wrapping_sub(1);
            if c2rust_fresh0 <= 0 as size_t {
                break;
            }
            if !vim_strchr(aux_ptr, *ptr as uint8_t as c_int).is_null() {
                let c2rust_fresh1 = p;
                p = p.offset(1);
                *c2rust_fresh1 = '\\' as c_char;
            }
            let len: size_t = (utfc_ptr2len(ptr) - 1 as c_int) as size_t;
            let mut i: size_t = 0 as size_t;
            while i < len && n > 0 as size_t {
                let c2rust_fresh2 = ptr;
                ptr = ptr.offset(1);
                let c2rust_fresh3 = p;
                p = p.offset(1);
                *c2rust_fresh3 = *c2rust_fresh2;
                i = i.wrapping_add(1);
                n = n.wrapping_sub(1);
            }
            let c2rust_fresh4 = ptr;
            ptr = ptr.offset(1);
            let c2rust_fresh5 = p;
            p = p.offset(1);
            *c2rust_fresh5 = *c2rust_fresh4;
        }
        *p = NUL as c_char;
        buflen = p.offset_from(buf) as size_t;
    }
    if cmdchar == '*' as c_int || cmdchar == '#' as c_int {
        if !g_cmd && vim_iswordp(mb_prevptr(get_cursor_line_ptr(), ptr)) as c_int != 0 {
            strcpy(
                buf.offset(buflen as isize),
                b"\\>\0".as_ptr() as *const c_char as *mut c_char,
            );
            buflen = (buflen as c_ulong).wrapping_add(
                ::core::mem::size_of::<[c_char; 3]>().wrapping_sub(1 as usize) as c_ulong,
            ) as size_t;
        }
        init_history();
        add_to_history(
            HIST_SEARCH as c_int,
            ::core::slice::from_raw_parts(buf as *const u8, buflen as usize),
            true_0 != 0,
            NUL as u8,
        );
        normal_search(
            cap,
            if cmdchar == '*' as c_int {
                '/' as c_int
            } else {
                '?' as c_int
            },
            buf,
            buflen,
            0 as c_int,
            ::core::ptr::null_mut::<c_int>(),
        );
    } else {
        g_tag_at_cursor.set(true_0 != 0);
        do_cmdline_cmd(buf);
        g_tag_at_cursor.set(false_0 != 0);
        if cmdchar == 'K' as c_int && !kp_ex && !kp_help {
            restart_edit.set('i' as c_int);
            add_map(
                b"<esc>\0".as_ptr() as *const c_char as *mut c_char,
                b"<Cmd>bdelete!<CR>\0".as_ptr() as *const c_char as *mut c_char,
                MODE_TERMINAL as c_int,
                true_0 != 0,
            );
        }
    }
    xfree(buf as *mut c_void);
}
pub unsafe extern "C" fn get_visual_text(
    mut cap: *mut cmdarg_T,
    mut pp: *mut *mut c_char,
    mut lenp: *mut size_t,
) -> bool {
    if VIsual_mode.get() != 'V' as c_int {
        unadjust_for_sel();
    }
    if (*VIsual.ptr()).lnum != (*curwin.get()).w_cursor.lnum {
        if !cap.is_null() {
            clearopbeep((*cap).oap);
        }
        return false_0 != 0;
    }
    if VIsual_mode.get() == 'V' as c_int {
        *pp = get_cursor_line_ptr();
        *lenp = get_cursor_line_len() as size_t;
    } else {
        if lt((*curwin.get()).w_cursor, VIsual.get()) {
            *pp = ml_get_pos(&raw mut (*curwin.get()).w_cursor);
            *lenp = ((*VIsual.ptr()).col as size_t)
                .wrapping_sub((*curwin.get()).w_cursor.col as size_t)
                .wrapping_add(1 as size_t);
        } else {
            *pp = ml_get_pos(VIsual.ptr());
            *lenp = ((*curwin.get()).w_cursor.col as size_t)
                .wrapping_sub((*VIsual.ptr()).col as size_t)
                .wrapping_add(1 as size_t);
        }
        if **pp as c_int == NUL {
            *lenp = 0 as size_t;
        }
        if *lenp > 0 as size_t {
            *lenp = (*lenp).wrapping_add(
                (utfc_ptr2len((*pp).offset((*lenp).wrapping_sub(1 as size_t) as isize))
                    - 1 as c_int) as size_t,
            );
        }
    }
    reset_VIsual_and_resel();
    return true_0 != 0;
}
unsafe extern "C" fn nv_tagpop(mut cap: *mut cmdarg_T) {
    if !checkclearopq((*cap).oap) {
        do_tag(
            b"\0".as_ptr() as *const c_char as *mut c_char,
            DT_POP as c_int,
            (*cap).count1,
            false_0,
            true_0 != 0,
        );
    }
}
unsafe extern "C" fn nv_scroll(mut cap: *mut cmdarg_T) {
    let mut n: c_int = 0;
    let mut lnum: linenr_T = 0;
    (*(*cap).oap).motion_type = kMTLineWise;
    setpcmark();
    if (*cap).cmdchar == 'L' as c_int {
        validate_botline_win(curwin.get());
        (*curwin.get()).w_cursor.lnum = (*curwin.get()).w_botline - 1 as linenr_T;
        if (*cap).count1 as linenr_T - 1 as linenr_T >= (*curwin.get()).w_cursor.lnum {
            (*curwin.get()).w_cursor.lnum = 1 as c_int as linenr_T;
        } else if win_lines_concealed(curwin.get()) {
            n = (*cap).count1 - 1 as c_int;
            while n > 0 as c_int && (*curwin.get()).w_cursor.lnum > (*curwin.get()).w_topline {
                hasFolding(
                    curwin.get(),
                    (*curwin.get()).w_cursor.lnum,
                    &raw mut (*curwin.get()).w_cursor.lnum,
                    ::core::ptr::null_mut::<linenr_T>(),
                );
                n += decor_conceal_line(
                    curwin.get(),
                    (*curwin.get()).w_cursor.lnum as c_int,
                    true_0 != 0,
                ) as c_int;
                if (*curwin.get()).w_cursor.lnum > (*curwin.get()).w_topline {
                    (*curwin.get()).w_cursor.lnum -= 1;
                }
                n -= 1;
            }
        } else {
            (*curwin.get()).w_cursor.lnum =
                ((*curwin.get()).w_cursor.lnum as c_int - ((*cap).count1 - 1 as c_int)) as linenr_T;
        }
    } else {
        if (*cap).cmdchar == 'M' as c_int {
            let mut used: c_int = 0 as c_int;
            used -=
                win_get_fill(curwin.get(), (*curwin.get()).w_topline) - (*curwin.get()).w_topfill;
            validate_botline_win(curwin.get());
            let mut half: c_int = ((*curwin.get()).w_view_height - (*curwin.get()).w_empty_rows
                + 1 as c_int)
                / 2 as c_int;
            n = 0 as c_int;
            while ((*curwin.get()).w_topline + n as linenr_T) < (*curbuf.get()).b_ml.ml_line_count {
                if n > 0 as c_int
                    && used
                        + win_get_fill(curwin.get(), (*curwin.get()).w_topline + n as linenr_T)
                            / 2 as c_int
                        >= half
                {
                    n -= 1;
                    break;
                } else {
                    used += plines_win(
                        curwin.get(),
                        (*curwin.get()).w_topline + n as linenr_T,
                        true_0 != 0,
                    );
                    if used >= half {
                        break;
                    }
                    if hasFolding(
                        curwin.get(),
                        (*curwin.get()).w_topline + n as linenr_T,
                        ::core::ptr::null_mut::<linenr_T>(),
                        &raw mut lnum,
                    ) {
                        n = (lnum - (*curwin.get()).w_topline) as c_int;
                    }
                    n += 1;
                }
            }
            if n > 0 as c_int && used > (*curwin.get()).w_view_height {
                n -= 1;
            }
        } else {
            n = (*cap).count1 - 1 as c_int;
            if win_lines_concealed(curwin.get()) {
                lnum = (*curwin.get()).w_topline;
                while (decor_conceal_line(curwin.get(), lnum as c_int - 1 as c_int, true_0 != 0)
                    as c_int
                    != 0
                    || {
                        let c2rust_fresh12 = n;
                        n = n - 1;
                        c2rust_fresh12 > 0 as c_int
                    })
                    && lnum < (*curwin.get()).w_botline - 1 as linenr_T
                {
                    hasFolding(
                        curwin.get(),
                        lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        &raw mut lnum,
                    );
                    lnum += 1;
                }
                n = (lnum - (*curwin.get()).w_topline) as c_int;
            }
        }
        (*curwin.get()).w_cursor.lnum =
            if ((*curwin.get()).w_topline + n as linenr_T) < (*curbuf.get()).b_ml.ml_line_count {
                (*curwin.get()).w_topline + n as linenr_T
            } else {
                (*curbuf.get()).b_ml.ml_line_count
            };
    }
    if (*(*cap).oap).op_type == OP_NOP as c_int {
        cursor_correct(curwin.get());
    }
    beginline(BL_SOL as c_int | BL_FIX as c_int);
}
unsafe extern "C" fn nv_right(mut cap: *mut cmdarg_T) {
    let mut n: c_int = 0;
    if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0 {
        if mod_mask.get() & MOD_MASK_CTRL != 0 {
            (*cap).arg = true_0;
        }
        nv_wordcmd(cap);
        return;
    }
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    let mut past_line: bool =
        VIsual_active.get() as c_int != 0 && *p_sel.get() as c_int != 'o' as c_int;
    if virtual_active(curwin.get()) {
        past_line = false_0 != 0;
    }
    n = (*cap).count1;
    while n > 0 as c_int {
        if !past_line && oneright() == false_0
            || past_line as c_int != 0 && *get_cursor_pos_ptr() as c_int == NUL
        {
            if ((*cap).cmdchar == ' ' as c_int && !vim_strchr(p_ww.get(), 's' as c_int).is_null()
                || (*cap).cmdchar == 'l' as c_int
                    && !vim_strchr(p_ww.get(), 'l' as c_int).is_null()
                || (*cap).cmdchar == K_RIGHT && !vim_strchr(p_ww.get(), '>' as c_int).is_null())
                && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
            {
                if (*(*cap).oap).op_type != OP_NOP as c_int
                    && !(*(*cap).oap).inclusive
                    && !(*ml_get((*curwin.get()).w_cursor.lnum) as c_int == NUL)
                {
                    (*(*cap).oap).inclusive = true_0 != 0;
                } else {
                    (*curwin.get()).w_cursor.lnum += 1;
                    (*curwin.get()).w_cursor.col = 0 as c_int as colnr_T;
                    (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
                    (*curwin.get()).w_set_curswant = true_0;
                    (*(*cap).oap).inclusive = false_0 != 0;
                }
            } else {
                if (*(*cap).oap).op_type == OP_NOP as c_int {
                    if n == (*cap).count1 {
                        beep_flush();
                    }
                } else if !(*ml_get((*curwin.get()).w_cursor.lnum) as c_int == NUL) {
                    (*(*cap).oap).inclusive = true_0 != 0;
                }
                break;
            }
        } else if past_line {
            (*curwin.get()).w_set_curswant = true_0;
            if virtual_active(curwin.get()) {
                oneright();
            } else {
                (*curwin.get()).w_cursor.col += utfc_ptr2len(get_cursor_pos_ptr());
            }
        }
        n -= 1;
    }
    if n != (*cap).count1
        && fdo_flags.get() & kOptFdoFlagHor as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_left(mut cap: *mut cmdarg_T) {
    let mut n: c_int = 0;
    if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0 {
        if mod_mask.get() & MOD_MASK_CTRL != 0 {
            (*cap).arg = 1 as c_int;
        }
        nv_bck_word(cap);
        return;
    }
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    n = (*cap).count1;
    while n > 0 as c_int {
        if oneleft() == false_0 {
            if (((*cap).cmdchar == K_BS || (*cap).cmdchar == Ctrl_H)
                && !vim_strchr(p_ww.get(), 'b' as c_int).is_null()
                || (*cap).cmdchar == 'h' as c_int
                    && !vim_strchr(p_ww.get(), 'h' as c_int).is_null()
                || (*cap).cmdchar == K_LEFT && !vim_strchr(p_ww.get(), '<' as c_int).is_null())
                && (*curwin.get()).w_cursor.lnum > 1 as linenr_T
            {
                (*curwin.get()).w_cursor.lnum -= 1;
                coladvance(curwin.get(), MAXCOL as c_int);
                (*curwin.get()).w_set_curswant = true_0;
                if ((*(*cap).oap).op_type == OP_DELETE as c_int
                    || (*(*cap).oap).op_type == OP_CHANGE as c_int)
                    && !(*ml_get((*curwin.get()).w_cursor.lnum) as c_int == NUL)
                {
                    let mut cp: *mut c_char = get_cursor_pos_ptr();
                    if *cp as c_int != NUL {
                        (*curwin.get()).w_cursor.col += utfc_ptr2len(cp);
                    }
                    (*cap).retval |= CA_NO_ADJ_OP_END as c_int;
                }
            } else {
                if (*(*cap).oap).op_type == OP_NOP as c_int && n == (*cap).count1 {
                    beep_flush();
                }
                break;
            }
        }
        n -= 1;
    }
    if n != (*cap).count1
        && fdo_flags.get() & kOptFdoFlagHor as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_up(mut cap: *mut cmdarg_T) {
    if mod_mask.get() & MOD_MASK_SHIFT != 0 {
        (*cap).arg = BACKWARD as c_int;
        nv_page(cap);
        return;
    }
    (*(*cap).oap).motion_type = kMTLineWise;
    if cursor_up(
        (*cap).count1 as linenr_T,
        (*(*cap).oap).op_type == OP_NOP as c_int,
    ) == false_0
    {
        clearopbeep((*cap).oap);
    } else if (*cap).arg != 0 {
        beginline(BL_WHITE as c_int | BL_FIX as c_int);
    }
}
unsafe extern "C" fn nv_down(mut cap: *mut cmdarg_T) {
    if mod_mask.get() & MOD_MASK_SHIFT != 0 {
        (*cap).arg = FORWARD as c_int;
        nv_page(cap);
    } else if bt_quickfix(curbuf.get()) as c_int != 0 && (*cap).cmdchar == CAR {
        qf_view_result(false_0 != 0);
    } else if cmdwin_type.get() != 0 as c_int && (*cap).cmdchar == CAR {
        cmdwin_result.set(CAR);
    } else if bt_prompt(curbuf.get()) as c_int != 0
        && (*cap).cmdchar == CAR
        && (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_ml.ml_line_count
    {
        prompt_invoke_callback();
        if restart_edit.get() == 0 as c_int {
            restart_edit.set('a' as c_int);
        }
    } else {
        (*(*cap).oap).motion_type = kMTLineWise;
        if cursor_down((*cap).count1, (*(*cap).oap).op_type == OP_NOP as c_int) == false_0 {
            clearopbeep((*cap).oap);
        } else if (*cap).arg != 0 {
            beginline(BL_WHITE as c_int | BL_FIX as c_int);
        }
    };
}
unsafe extern "C" fn nv_gotofile(mut cap: *mut cmdarg_T) {
    let mut lnum: linenr_T = -1 as linenr_T;
    if check_text_or_curbuf_locked((*cap).oap) {
        return;
    }
    if !check_can_set_curbuf_disabled() {
        return;
    }
    let mut ptr: *mut c_char = grab_file_name((*cap).count1, &raw mut lnum);
    if !ptr.is_null() {
        if curbufIsChanged() as c_int != 0
            && (*curbuf.get()).b_nwindows <= 1 as c_int
            && !buf_hide(curbuf.get())
        {
            autowrite(curbuf.get(), false_0 != 0);
        }
        setpcmark();
        if do_ecmd(
            0 as c_int,
            ptr,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<exarg_T>(),
            ECMD_LAST as c_int as linenr_T,
            if buf_hide(curbuf.get()) as c_int != 0 {
                ECMD_HIDE as c_int
            } else {
                0 as c_int
            },
            curwin.get(),
        ) == OK
            && (*cap).nchar == 'F' as c_int
            && lnum >= 0 as linenr_T
        {
            (*curwin.get()).w_cursor.lnum = lnum;
            check_cursor_lnum(curwin.get());
            beginline(BL_SOL as c_int | BL_FIX as c_int);
        }
        xfree(ptr as *mut c_void);
    } else {
        clearop((*cap).oap);
    };
}
unsafe extern "C" fn nv_end(mut cap: *mut cmdarg_T) {
    if (*cap).arg != 0 || mod_mask.get() & MOD_MASK_CTRL != 0 {
        (*cap).arg = true_0;
        nv_goto(cap);
        (*cap).count1 = 1 as c_int;
    }
    nv_dollar(cap);
}
unsafe extern "C" fn nv_dollar(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = true_0 != 0;
    if !virtual_active(curwin.get())
        || gchar_cursor() != NUL
        || (*(*cap).oap).op_type == OP_NOP as c_int
    {
        (*curwin.get()).w_curswant = MAXCOL as c_int as colnr_T;
    }
    if cursor_down(
        (*cap).count1 - 1 as c_int,
        (*(*cap).oap).op_type == OP_NOP as c_int,
    ) == false_0
    {
        clearopbeep((*cap).oap);
    } else if fdo_flags.get() & kOptFdoFlagHor as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_search(mut cap: *mut cmdarg_T) {
    let mut oap: *mut oparg_T = (*cap).oap;
    let mut save_cursor: pos_T = (*curwin.get()).w_cursor;
    if (*cap).cmdchar == '?' as c_int && (*(*cap).oap).op_type == OP_ROT13 as c_int {
        (*cap).cmdchar = 'g' as c_int;
        (*cap).nchar = '?' as c_int;
        nv_operator(cap);
        return;
    }
    (*cap).searchbuf = getcmdline((*cap).cmdchar, (*cap).count1, 0 as c_int, true_0 != 0);
    if (*cap).searchbuf.is_null() {
        clearop(oap);
        return;
    }
    normal_search(
        cap,
        (*cap).cmdchar,
        (*cap).searchbuf,
        strlen((*cap).searchbuf),
        if (*cap).arg != 0 || !equalpos(save_cursor, (*curwin.get()).w_cursor) {
            0 as c_int
        } else {
            SEARCH_MARK as c_int
        },
        ::core::ptr::null_mut::<c_int>(),
    );
}
unsafe extern "C" fn nv_next(mut cap: *mut cmdarg_T) {
    let mut old: pos_T = (*curwin.get()).w_cursor;
    let mut wrapped: c_int = false_0;
    let mut i: c_int = normal_search(
        cap,
        0 as c_int,
        ::core::ptr::null_mut::<c_char>(),
        0 as size_t,
        SEARCH_MARK as c_int | (*cap).arg,
        &raw mut wrapped,
    );
    if i == 1 as c_int && wrapped == 0 && equalpos(old, (*curwin.get()).w_cursor) as c_int != 0 {
        (*cap).count1 += 1 as c_int;
        normal_search(
            cap,
            0 as c_int,
            ::core::ptr::null_mut::<c_char>(),
            0 as size_t,
            SEARCH_MARK as c_int | (*cap).arg,
            ::core::ptr::null_mut::<c_int>(),
        );
        (*cap).count1 -= 1 as c_int;
    }
    if i > 0 as c_int
        && p_hls.get() != 0
        && !no_hlsearch.get()
        && win_hl_attr(curwin.get(), HLF_LC as c_int) != win_hl_attr(curwin.get(), HLF_L as c_int)
    {
        redraw_later(curwin.get(), UPD_SOME_VALID as c_int);
    }
}
unsafe extern "C" fn normal_search(
    mut cap: *mut cmdarg_T,
    mut dir: c_int,
    mut pat: *mut c_char,
    mut patlen: size_t,
    mut opt: c_int,
    mut wrapped: *mut c_int,
) -> c_int {
    let mut sia: searchit_arg_T = searchit_arg_T {
        sa_stop_lnum: 0,
        sa_tm: ::core::ptr::null_mut::<proftime_T>(),
        sa_timed_out: 0,
        sa_wrapped: 0,
    };
    let prev_cursor: pos_T = (*curwin.get()).w_cursor;
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    (*(*cap).oap).use_reg_one = true_0 != 0;
    (*curwin.get()).w_set_curswant = true_0;
    memset(
        &raw mut sia as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<searchit_arg_T>(),
    );
    let mut i: c_int = do_search(
        (*cap).oap,
        dir,
        dir,
        pat,
        patlen,
        (*cap).count1,
        opt | SEARCH_OPT as c_int | SEARCH_ECHO as c_int | SEARCH_MSG as c_int,
        &raw mut sia,
    );
    if !wrapped.is_null() {
        *wrapped = sia.sa_wrapped;
    }
    if i == 0 as c_int {
        clearop((*cap).oap);
    } else {
        if i == 2 as c_int {
            (*(*cap).oap).motion_type = kMTLineWise;
        }
        (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
        if (*(*cap).oap).op_type == OP_NOP as c_int
            && fdo_flags.get() & kOptFdoFlagSearch as c_int as c_uint != 0
            && KeyTyped.get() as c_int != 0
        {
            foldOpenCursor();
        }
    }
    if !equalpos((*curwin.get()).w_cursor, prev_cursor)
        && p_hls.get() != 0
        && !no_hlsearch.get()
        && win_hl_attr(curwin.get(), HLF_LC as c_int) != win_hl_attr(curwin.get(), HLF_L as c_int)
    {
        redraw_later(curwin.get(), UPD_SOME_VALID as c_int);
    }
    check_cursor(curwin.get());
    return i;
}
unsafe extern "C" fn nv_csearch(mut cap: *mut cmdarg_T) {
    let mut cursor_dec: bool = false_0 != 0;
    if *p_sel.get() as c_int == 'e' as c_int
        && VIsual_active.get() as c_int != 0
        && VIsual_mode.get() == 'v' as c_int
        && VIsual_select_exclu_adj.get() as c_int != 0
    {
        unadjust_for_sel();
        cursor_dec = true_0 != 0;
    }
    let mut t_cmd: bool = (*cap).cmdchar == 't' as c_int || (*cap).cmdchar == 'T' as c_int;
    (*(*cap).oap).motion_type = kMTCharWise;
    if (*cap).nchar < 0 as c_int || searchc(cap, t_cmd) == false_0 {
        clearopbeep((*cap).oap);
        if cursor_dec {
            adjust_for_sel(cap);
        }
        return;
    }
    (*curwin.get()).w_set_curswant = true_0;
    if gchar_cursor() == TAB
        && virtual_active(curwin.get()) as c_int != 0
        && (*cap).arg == FORWARD as c_int
        && (t_cmd as c_int != 0 || (*(*cap).oap).op_type != OP_NOP as c_int)
    {
        let mut scol: colnr_T = 0;
        let mut ecol: colnr_T = 0;
        getvcol(
            curwin.get(),
            &raw mut (*curwin.get()).w_cursor,
            &raw mut scol,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut ecol,
        );
        (*curwin.get()).w_cursor.coladd = ecol - scol;
    } else {
        (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
    }
    adjust_for_sel(cap);
    if fdo_flags.get() & kOptFdoFlagHor as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_bracket_block(mut cap: *mut cmdarg_T, mut old_pos: *const pos_T) {
    let mut new_pos: pos_T = pos_T {
        lnum: 0 as linenr_T,
        col: 0 as colnr_T,
        coladd: 0 as colnr_T,
    };
    let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut prev_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut n: c_int = 0;
    let mut findc: c_int = 0;
    if (*cap).nchar == '*' as c_int {
        (*cap).nchar = '/' as c_int;
    }
    prev_pos.lnum = 0 as c_int as linenr_T;
    if (*cap).nchar == 'm' as c_int || (*cap).nchar == 'M' as c_int {
        if (*cap).cmdchar == '[' as c_int {
            findc = '{' as c_int;
        } else {
            findc = '}' as c_int;
        }
        n = 9999 as c_int;
    } else {
        findc = (*cap).nchar;
        n = (*cap).count1;
    }
    while n > 0 as c_int {
        pos = findmatchlimit(
            (*cap).oap,
            findc,
            if (*cap).cmdchar == '[' as c_int {
                FM_BACKWARD as c_int
            } else {
                FM_FORWARD as c_int
            },
            0 as int64_t,
        );
        if pos.is_null() {
            if new_pos.lnum == 0 as linenr_T {
                if (*cap).nchar != 'm' as c_int && (*cap).nchar != 'M' as c_int {
                    clearopbeep((*cap).oap);
                }
            } else {
                pos = &raw mut new_pos;
            }
            break;
        } else {
            prev_pos = new_pos;
            (*curwin.get()).w_cursor = *pos;
            new_pos = *pos;
            n -= 1;
        }
    }
    (*curwin.get()).w_cursor = *old_pos;
    if (*cap).nchar == 'm' as c_int || (*cap).nchar == 'M' as c_int {
        let mut c: c_int = 0;
        let mut norm: bool =
            (findc == '{' as c_int) as c_int == ((*cap).nchar == 'm' as c_int) as c_int;
        n = (*cap).count1;
        if prev_pos.lnum != 0 as linenr_T {
            pos = &raw mut prev_pos;
            (*curwin.get()).w_cursor = prev_pos;
            if norm {
                n -= 1;
            }
        } else {
            pos = ::core::ptr::null_mut::<pos_T>();
        }
        while n > 0 as c_int {
            loop {
                if (if findc == '{' as c_int {
                    dec_cursor()
                } else {
                    inc_cursor()
                }) < 0 as c_int
                {
                    if pos.is_null() {
                        clearopbeep((*cap).oap);
                    }
                    n = 0 as c_int;
                    break;
                } else {
                    c = gchar_cursor();
                    if !(c == '{' as c_int || c == '}' as c_int) {
                        continue;
                    }
                    if c == findc && norm as c_int != 0 || n == 1 as c_int && !norm {
                        new_pos = (*curwin.get()).w_cursor;
                        pos = &raw mut new_pos;
                        n = 0 as c_int;
                    } else if new_pos.lnum == 0 as linenr_T {
                        new_pos = (*curwin.get()).w_cursor;
                        pos = &raw mut new_pos;
                    } else {
                        pos = findmatchlimit(
                            (*cap).oap,
                            findc,
                            if (*cap).cmdchar == '[' as c_int {
                                FM_BACKWARD as c_int
                            } else {
                                FM_FORWARD as c_int
                            },
                            0 as int64_t,
                        );
                        if pos.is_null() {
                            n = 0 as c_int;
                        } else {
                            (*curwin.get()).w_cursor = *pos;
                        }
                    }
                    break;
                }
            }
            n -= 1;
        }
        (*curwin.get()).w_cursor = *old_pos;
        if pos.is_null() && new_pos.lnum != 0 as linenr_T {
            clearopbeep((*cap).oap);
        }
    }
    if !pos.is_null() {
        setpcmark();
        (*curwin.get()).w_cursor = *pos;
        (*curwin.get()).w_set_curswant = true_0;
        if fdo_flags.get() & kOptFdoFlagBlock as c_int as c_uint != 0
            && KeyTyped.get() as c_int != 0
            && (*(*cap).oap).op_type == OP_NOP as c_int
        {
            foldOpenCursor();
        }
    }
}
unsafe extern "C" fn nv_brackets(mut cap: *mut cmdarg_T) {
    let mut flag: c_int = 0;
    let mut n: c_int = 0;
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    let mut old_pos: pos_T = (*curwin.get()).w_cursor;
    (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
    if (*cap).nchar == 'f' as c_int {
        nv_gotofile(cap);
    } else if !vim_strchr(b"iI\tdD\x04\0".as_ptr() as *const c_char, (*cap).nchar).is_null() {
        let mut ptr: *mut c_char = ::core::ptr::null_mut::<c_char>();
        let mut len: size_t = 0;
        len = find_ident_under_cursor(
            &raw mut ptr,
            FIND_IDENT as c_int,
            ::core::ptr::null_mut::<c_int>(),
        );
        if len == 0 as size_t {
            clearop((*cap).oap);
        } else {
            ptr = xmemdupz(ptr as *const c_void, len) as *mut c_char;
            find_pattern_in_path(
                ptr,
                kDirectionNotSet,
                len,
                true_0 != 0,
                if (*cap).count0 == 0 as c_int {
                    (*(*__ctype_b_loc()).offset((*cap).nchar as isize) as c_int
                        & _ISupper as c_int as c_ushort as c_int
                        == 0) as c_int
                } else {
                    false_0
                } != 0,
                if (*cap).nchar & 0xf as c_int == 'd' as c_int & 0xf as c_int {
                    FIND_DEFINE as c_int
                } else {
                    FIND_ANY as c_int
                },
                (*cap).count1,
                if *(*__ctype_b_loc()).offset((*cap).nchar as isize) as c_int
                    & _ISupper as c_int as c_ushort as c_int
                    != 0
                {
                    ACTION_SHOW_ALL as c_int
                } else if *(*__ctype_b_loc()).offset((*cap).nchar as isize) as c_int
                    & _ISlower as c_int as c_ushort as c_int
                    != 0
                {
                    ACTION_SHOW as c_int
                } else {
                    ACTION_GOTO as c_int
                },
                if (*cap).cmdchar == ']' as c_int {
                    (*curwin.get()).w_cursor.lnum + 1 as linenr_T
                } else {
                    1 as linenr_T
                },
                MAXLNUM as c_int as linenr_T,
                false_0 != 0,
                false_0 != 0,
            );
            xfree(ptr as *mut c_void);
            (*curwin.get()).w_set_curswant = true_0;
        }
    } else if (*cap).cmdchar == '[' as c_int
        && !vim_strchr(b"{(*/#mM\0".as_ptr() as *const c_char, (*cap).nchar).is_null()
        || (*cap).cmdchar == ']' as c_int
            && !vim_strchr(b"})*/#mM\0".as_ptr() as *const c_char, (*cap).nchar).is_null()
    {
        nv_bracket_block(cap, &raw mut old_pos);
    } else if (*cap).nchar == '[' as c_int || (*cap).nchar == ']' as c_int {
        if (*cap).nchar == (*cap).cmdchar {
            flag = '{' as c_int;
        } else {
            flag = '}' as c_int;
        }
        (*curwin.get()).w_set_curswant = true_0;
        if !findpar(
            &raw mut (*(*cap).oap).inclusive,
            (*cap).arg,
            (*cap).count1,
            flag,
            (*(*cap).oap).op_type != OP_NOP as c_int
                && (*cap).arg == FORWARD as c_int
                && flag == '{' as c_int,
        ) {
            clearopbeep((*cap).oap);
        } else {
            if (*(*cap).oap).op_type == OP_NOP as c_int {
                beginline(BL_WHITE as c_int | BL_FIX as c_int);
            }
            if fdo_flags.get() & kOptFdoFlagBlock as c_int as c_uint != 0
                && KeyTyped.get() as c_int != 0
                && (*(*cap).oap).op_type == OP_NOP as c_int
            {
                foldOpenCursor();
            }
        }
    } else if (*cap).nchar == 'p' as c_int || (*cap).nchar == 'P' as c_int {
        nv_put_opt(cap, true_0 != 0);
    } else if (*cap).nchar == '\'' as c_int || (*cap).nchar == '`' as c_int {
        let mut fm: *mut fmark_T = pos_to_mark(
            curbuf.get(),
            ::core::ptr::null_mut::<fmark_T>(),
            (*curwin.get()).w_cursor,
        );
        '_c2rust_label: {
            if !fm.is_null() {
            } else {
                __assert_fail(
                    b"fm != NULL\0".as_ptr() as *const c_char,
                    b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                    4311 as c_uint,
                    b"void nv_brackets(cmdarg_T *)\0".as_ptr() as *const c_char,
                );
            }
        };
        let mut prev_fm: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
        n = (*cap).count1;
        while n > 0 as c_int {
            prev_fm = fm;
            fm = getnextmark(
                &raw mut (*fm).mark,
                if (*cap).cmdchar == '[' as c_int {
                    BACKWARD as c_int
                } else {
                    FORWARD as c_int
                },
                ((*cap).nchar == '\'' as c_int) as c_int,
            );
            if fm.is_null() {
                break;
            }
            n -= 1;
        }
        if fm.is_null() {
            fm = prev_fm;
        }
        let mut flags: MarkMove = kMarkContext;
        flags = (flags as c_uint
            | (if (*cap).nchar == '\'' as c_int {
                kMarkBeginLine as c_int
            } else {
                0 as c_int
            }) as c_uint) as MarkMove;
        nv_mark_move_to(cap, flags, fm);
    } else if (*cap).nchar >= -(253 as c_int + ((KE_RIGHTRELEASE as c_int) << 8 as c_int))
        && (*cap).nchar <= -(253 as c_int + ((KE_LEFTMOUSE as c_int) << 8 as c_int))
    {
        do_mouse(
            (*cap).oap,
            (*cap).nchar,
            if (*cap).cmdchar == ']' as c_int {
                FORWARD as c_int
            } else {
                BACKWARD as c_int
            },
            (*cap).count1,
            PUT_FIXINDENT as c_int != 0,
        );
    } else if (*cap).nchar == 'z' as c_int {
        if foldMoveTo(
            false_0 != 0,
            if (*cap).cmdchar == ']' as c_int {
                FORWARD as c_int
            } else {
                BACKWARD as c_int
            },
            (*cap).count1,
        ) == false_0
        {
            clearopbeep((*cap).oap);
        }
    } else if (*cap).nchar == 'c' as c_int {
        if diff_move_to(
            if (*cap).cmdchar == ']' as c_int {
                FORWARD as c_int
            } else {
                BACKWARD as c_int
            },
            (*cap).count1,
        ) == false_0
        {
            clearopbeep((*cap).oap);
        }
    } else if (*cap).nchar == 'r' as c_int
        || (*cap).nchar == 's' as c_int
        || (*cap).nchar == 'S' as c_int
    {
        setpcmark();
        n = 0 as c_int;
        while n < (*cap).count1 {
            if spell_move_to(
                curwin.get(),
                if (*cap).cmdchar == ']' as c_int {
                    FORWARD as c_int
                } else {
                    BACKWARD as c_int
                },
                (if (*cap).nchar == 's' as c_int {
                    SMT_ALL as c_int
                } else {
                    if (*cap).nchar == 'r' as c_int {
                        SMT_RARE as c_int
                    } else {
                        SMT_BAD as c_int
                    }
                }) as smt_T,
                false_0 != 0,
                ::core::ptr::null_mut::<hlf_T>(),
            ) == 0 as size_t
            {
                clearopbeep((*cap).oap);
                break;
            } else {
                (*curwin.get()).w_set_curswant = true_0;
                n += 1;
            }
        }
        if (*(*cap).oap).op_type == OP_NOP as c_int
            && fdo_flags.get() & kOptFdoFlagSearch as c_int as c_uint != 0
            && KeyTyped.get() as c_int != 0
        {
            foldOpenCursor();
        }
    } else {
        clearopbeep((*cap).oap);
    };
}
unsafe extern "C" fn nv_percent(mut cap: *mut cmdarg_T) {
    let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    (*(*cap).oap).inclusive = true_0 != 0;
    if (*cap).count0 != 0 {
        if (*cap).count0 > 100 as c_int {
            clearopbeep((*cap).oap);
        } else {
            (*(*cap).oap).motion_type = kMTLineWise;
            setpcmark();
            if (*curbuf.get()).b_ml.ml_line_count >= 21474836 as linenr_T {
                (*curwin.get()).w_cursor.lnum =
                    ((*curbuf.get()).b_ml.ml_line_count + 99 as linenr_T) / 100 as linenr_T
                        * (*cap).count0 as linenr_T;
            } else {
                (*curwin.get()).w_cursor.lnum = ((*curbuf.get()).b_ml.ml_line_count
                    * (*cap).count0 as linenr_T
                    + 99 as linenr_T)
                    / 100 as linenr_T;
            }
            (*curwin.get()).w_cursor.lnum = if (if (*curwin.get()).w_cursor.lnum > 1 as linenr_T {
                (*curwin.get()).w_cursor.lnum
            } else {
                1 as linenr_T
            }) < (*curbuf.get()).b_ml.ml_line_count
            {
                if (*curwin.get()).w_cursor.lnum > 1 as linenr_T {
                    (*curwin.get()).w_cursor.lnum
                } else {
                    1 as linenr_T
                }
            } else {
                (*curbuf.get()).b_ml.ml_line_count
            };
            beginline(BL_SOL as c_int | BL_FIX as c_int);
        }
    } else {
        let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        (*(*cap).oap).motion_type = kMTCharWise;
        (*(*cap).oap).use_reg_one = true_0 != 0;
        pos = findmatch((*cap).oap, NUL);
        if pos.is_null() {
            clearopbeep((*cap).oap);
        } else {
            setpcmark();
            (*curwin.get()).w_cursor = *pos;
            (*curwin.get()).w_set_curswant = true_0;
            (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
            adjust_for_sel(cap);
        }
    }
    if (*(*cap).oap).op_type == OP_NOP as c_int
        && lnum != (*curwin.get()).w_cursor.lnum
        && fdo_flags.get() & kOptFdoFlagPercent as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_brace(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).use_reg_one = true_0 != 0;
    (*(*cap).oap).inclusive = false_0 != 0;
    (*curwin.get()).w_set_curswant = true_0;
    if findsent((*cap).arg as Direction, (*cap).count1) == FAIL {
        clearopbeep((*cap).oap);
        return;
    }
    adjust_cursor((*cap).oap);
    (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
    if fdo_flags.get() & kOptFdoFlagBlock as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_mark(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    if setmark((*cap).nchar) == false_0 {
        clearopbeep((*cap).oap);
    }
}
unsafe extern "C" fn nv_findpar(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    (*(*cap).oap).use_reg_one = true_0 != 0;
    (*curwin.get()).w_set_curswant = true_0;
    if !findpar(
        &raw mut (*(*cap).oap).inclusive,
        (*cap).arg,
        (*cap).count1,
        NUL,
        false_0 != 0,
    ) {
        clearopbeep((*cap).oap);
        return;
    }
    (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
    if fdo_flags.get() & kOptFdoFlagBlock as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_undo(mut cap: *mut cmdarg_T) {
    if (*(*cap).oap).op_type == OP_LOWER as c_int || VIsual_active.get() as c_int != 0 {
        (*cap).cmdchar = 'g' as c_int;
        (*cap).nchar = 'u' as c_int;
        nv_operator(cap);
    } else {
        nv_kundo(cap);
    };
}
unsafe extern "C" fn nv_kundo(mut cap: *mut cmdarg_T) {
    if checkclearopq((*cap).oap) {
        return;
    }
    u_undo((*cap).count1);
    (*curwin.get()).w_set_curswant = true_0;
}
unsafe extern "C" fn nv_replace(mut cap: *mut cmdarg_T) {
    let mut had_ctrl_v: c_int = 0;
    if checkclearop((*cap).oap) {
        return;
    }
    if bt_prompt(curbuf.get()) as c_int != 0 && !prompt_curpos_editable() {
        clearopbeep((*cap).oap);
        return;
    }
    if (*cap).nchar == Ctrl_V || (*cap).nchar == Ctrl_Q {
        had_ctrl_v = Ctrl_V;
        (*cap).nchar = get_literal(false_0 != 0);
        if (*cap).nchar > DEL {
            had_ctrl_v = NUL;
        }
    } else {
        had_ctrl_v = NUL;
    }
    if (*cap).nchar < 0 as c_int {
        clearopbeep((*cap).oap);
        return;
    }
    if VIsual_active.get() {
        if got_int.get() {
            got_int.set(false_0 != 0);
        }
        if had_ctrl_v != 0 {
            if (*cap).nchar == CAR {
                (*cap).nchar = REPLACE_CR_NCHAR as c_int;
            } else if (*cap).nchar == NL {
                (*cap).nchar = REPLACE_NL_NCHAR as c_int;
            }
        }
        nv_operator(cap);
        return;
    }
    if virtual_active(curwin.get()) {
        if u_save_cursor() == false_0 {
            return;
        }
        if gchar_cursor() == NUL {
            coladvance_force(getviscol() + (*cap).count1);
            '_c2rust_label: {
                if (*cap).count1 <= 2147483647 as c_int {
                } else {
                    __assert_fail(
                        b"cap->count1 <= INT_MAX\0".as_ptr() as *const c_char,
                        b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                        4553 as c_uint,
                        b"void nv_replace(cmdarg_T *)\0".as_ptr() as *const c_char,
                    );
                }
            };
            (*curwin.get()).w_cursor.col -= (*cap).count1;
        } else if gchar_cursor() == TAB {
            coladvance_force(getviscol());
        }
    }
    if (get_cursor_pos_len() as size_t) < (*cap).count1 as c_uint as size_t
        || mb_charlen(get_cursor_pos_ptr()) < (*cap).count1
    {
        clearopbeep((*cap).oap);
        return;
    }
    if had_ctrl_v != Ctrl_V
        && (*cap).nchar == '\t' as c_int
        && ((*curbuf.get()).b_p_et != 0 || p_sta.get() != 0)
    {
        stuffnumReadbuff((*cap).count1);
        stuffcharReadbuff('R' as c_int);
        stuffcharReadbuff('\t' as c_int);
        stuffcharReadbuff(ESC);
        return;
    }
    if u_save_cursor() == false_0 {
        return;
    }
    if had_ctrl_v != Ctrl_V && ((*cap).nchar == '\r' as c_int || (*cap).nchar == '\n' as c_int) {
        del_chars((*cap).count1, false_0);
        stuffcharReadbuff('\r' as c_int);
        stuffcharReadbuff(ESC);
        invoke_edit(cap, true_0, 'r' as c_int, false_0);
    } else {
        prep_redo(
            (*(*cap).oap).regname,
            (*cap).count1,
            NUL,
            'r' as c_int,
            NUL,
            had_ctrl_v,
            0 as c_int,
        );
        (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
        let old_State: c_int = State.get();
        if (*cap).nchar_len > 0 as c_int {
            AppendToRedobuff(&raw mut (*cap).nchar_composing as *mut c_char);
        } else {
            AppendCharToRedobuff((*cap).nchar);
        }
        let mut n: c_int = (*cap).count1;
        while n > 0 as c_int {
            State.set(MODE_REPLACE as c_int);
            if (*cap).nchar == Ctrl_E || (*cap).nchar == Ctrl_Y {
                let mut c: c_int = ins_copychar(
                    (*curwin.get()).w_cursor.lnum
                        + (if (*cap).nchar == Ctrl_Y {
                            -1 as linenr_T
                        } else {
                            1 as linenr_T
                        }),
                );
                if c != NUL {
                    ins_char(c);
                } else {
                    (*curwin.get()).w_cursor.col += 1;
                }
            } else if (*cap).nchar_len != 0 {
                ins_char_bytes(
                    &raw mut (*cap).nchar_composing as *mut c_char,
                    (*cap).nchar_len as size_t,
                );
            } else {
                ins_char((*cap).nchar);
            }
            State.set(old_State);
            n -= 1;
        }
        (*curwin.get()).w_cursor.col -= 1;
        mb_adjust_cursor();
        (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
        (*curwin.get()).w_set_curswant = true_0;
        set_last_insert((*cap).nchar);
    }
    foldUpdateAfterInsert();
}
unsafe extern "C" fn v_swap_corners(mut cmdchar: c_int) {
    let mut left: colnr_T = 0;
    let mut right: colnr_T = 0;
    if cmdchar == 'O' as c_int && VIsual_mode.get() == Ctrl_V {
        let mut old_cursor: pos_T = (*curwin.get()).w_cursor;
        getvcols(
            curwin.get(),
            &raw mut old_cursor,
            VIsual.ptr(),
            &raw mut left,
            &raw mut right,
        );
        (*curwin.get()).w_cursor.lnum = (*VIsual.ptr()).lnum;
        coladvance(curwin.get(), left);
        VIsual.set((*curwin.get()).w_cursor);
        (*curwin.get()).w_cursor.lnum = old_cursor.lnum;
        (*curwin.get()).w_curswant = right;
        if old_cursor.lnum >= (*VIsual.ptr()).lnum && *p_sel.get() as c_int == 'e' as c_int {
            (*curwin.get()).w_curswant += 1;
        }
        coladvance(curwin.get(), (*curwin.get()).w_curswant);
        if (*curwin.get()).w_cursor.col == old_cursor.col
            && (!virtual_active(curwin.get())
                || (*curwin.get()).w_cursor.coladd == old_cursor.coladd)
        {
            (*curwin.get()).w_cursor.lnum = (*VIsual.ptr()).lnum;
            if old_cursor.lnum <= (*VIsual.ptr()).lnum && *p_sel.get() as c_int == 'e' as c_int {
                right += 1;
            }
            coladvance(curwin.get(), right);
            VIsual.set((*curwin.get()).w_cursor);
            (*curwin.get()).w_cursor.lnum = old_cursor.lnum;
            coladvance(curwin.get(), left);
            (*curwin.get()).w_curswant = left;
        }
    } else {
        let mut old_cursor_0: pos_T = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor = VIsual.get();
        VIsual.set(old_cursor_0);
        (*curwin.get()).w_set_curswant = true_0;
    };
}
unsafe extern "C" fn nv_Replace(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() {
        (*cap).cmdchar = 'c' as c_int;
        (*cap).nchar = NUL;
        VIsual_mode_orig.set(VIsual_mode.get());
        VIsual_mode.set('V' as c_int);
        nv_operator(cap);
        return;
    }
    if checkclearopq((*cap).oap) {
        return;
    }
    if (*curbuf.get()).b_p_ma == 0 {
        emsg(gettext(&raw const e_modifiable as *const c_char));
    } else {
        if virtual_active(curwin.get()) {
            coladvance(curwin.get(), getviscol());
        }
        invoke_edit(
            cap,
            false_0,
            if (*cap).arg != 0 {
                'V' as c_int
            } else {
                'R' as c_int
            },
            false_0,
        );
    };
}
unsafe extern "C" fn nv_vreplace(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() {
        (*cap).cmdchar = 'r' as c_int;
        (*cap).nchar = (*cap).extra_char;
        nv_replace(cap);
        return;
    }
    if checkclearopq((*cap).oap) {
        return;
    }
    if (*curbuf.get()).b_p_ma == 0 {
        emsg(gettext(&raw const e_modifiable as *const c_char));
    } else {
        if (*cap).extra_char == Ctrl_V || (*cap).extra_char == Ctrl_Q {
            (*cap).extra_char = get_literal(false_0 != 0);
        }
        if (*cap).extra_char < ' ' as c_int {
            stuffcharReadbuff(Ctrl_V);
        }
        stuffcharReadbuff((*cap).extra_char);
        stuffcharReadbuff(ESC);
        if virtual_active(curwin.get()) {
            coladvance(curwin.get(), getviscol());
        }
        invoke_edit(cap, true_0, 'v' as c_int, false_0);
    };
}
unsafe extern "C" fn n_swapchar(mut cap: *mut cmdarg_T) {
    let mut did_change: bool = false_0 != 0;
    if checkclearopq((*cap).oap) {
        return;
    }
    if *ml_get((*curwin.get()).w_cursor.lnum) as c_int == NUL
        && vim_strchr(p_ww.get(), '~' as c_int).is_null()
    {
        clearopbeep((*cap).oap);
        return;
    }
    prep_redo_cmd(cap);
    if u_save_cursor() == false_0 {
        return;
    }
    let mut startpos: pos_T = (*curwin.get()).w_cursor;
    let mut n: c_int = (*cap).count1;
    while n > 0 as c_int {
        did_change = did_change as c_int
            | swapchar((*(*cap).oap).op_type, &raw mut (*curwin.get()).w_cursor) as c_int
            != 0;
        inc_cursor();
        if gchar_cursor() == NUL {
            if !(!vim_strchr(p_ww.get(), '~' as c_int).is_null()
                && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count)
            {
                break;
            }
            (*curwin.get()).w_cursor.lnum += 1;
            (*curwin.get()).w_cursor.col = 0 as c_int as colnr_T;
            if n > 1 as c_int {
                if u_savesub((*curwin.get()).w_cursor.lnum) == false_0 {
                    break;
                }
                u_clearline(curbuf.get());
            }
        }
        n -= 1;
    }
    check_cursor(curwin.get());
    (*curwin.get()).w_set_curswant = true_0;
    if did_change {
        changed_lines(
            curbuf.get(),
            startpos.lnum,
            startpos.col,
            (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
            0 as linenr_T,
            true_0 != 0,
        );
        (*curbuf.get()).b_op_start = startpos;
        (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
        if (*curbuf.get()).b_op_end.col > 0 as c_int {
            (*curbuf.get()).b_op_end.col -= 1;
        }
    }
}
unsafe extern "C" fn nv_mark_move_to(
    mut cap: *mut cmdarg_T,
    mut flags: MarkMove,
    mut fm: *mut fmark_T,
) -> MarkMoveRes {
    let mut res: MarkMoveRes = mark_move_to(fm, flags);
    if res as c_uint & kMarkMoveFailed as c_int as c_uint != 0 {
        clearop((*cap).oap);
    }
    (*(*cap).oap).motion_type = (if flags as c_uint & kMarkBeginLine as c_int as c_uint != 0 {
        kMTLineWise as c_int
    } else {
        kMTCharWise as c_int
    }) as MotionType;
    if (*cap).cmdchar == '`' as c_int {
        (*(*cap).oap).use_reg_one = true_0 != 0;
    }
    (*(*cap).oap).inclusive = false_0 != 0;
    (*curwin.get()).w_set_curswant = true_0;
    return res;
}
unsafe extern "C" fn v_visop(mut cap: *mut cmdarg_T) {
    static trans: GlobalCell<[c_char; 17]> = GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 17], [c_char; 17]>(*b"YyDdCcxdXdAAIIrr\0")
    });
    if *(*__ctype_b_loc()).offset((*cap).cmdchar as isize) as c_int
        & _ISupper as c_int as c_ushort as c_int
        != 0
    {
        if VIsual_mode.get() != Ctrl_V {
            VIsual_mode_orig.set(VIsual_mode.get());
            VIsual_mode.set('V' as c_int);
        } else if (*cap).cmdchar == 'C' as c_int || (*cap).cmdchar == 'D' as c_int {
            (*curwin.get()).w_curswant = MAXCOL as c_int as colnr_T;
        }
    }
    (*cap).cmdchar = *vim_strchr(trans.ptr() as *mut c_char, (*cap).cmdchar)
        .offset(1 as c_int as isize) as uint8_t as c_int;
    nv_operator(cap);
}
unsafe extern "C" fn nv_subst(mut cap: *mut cmdarg_T) {
    if bt_prompt(curbuf.get()) as c_int != 0 && !prompt_curpos_editable() {
        clearopbeep((*cap).oap);
        return;
    }
    if VIsual_active.get() {
        if (*cap).cmdchar == 'S' as c_int {
            VIsual_mode_orig.set(VIsual_mode.get());
            VIsual_mode.set('V' as c_int);
        }
        (*cap).cmdchar = 'c' as c_int;
        nv_operator(cap);
    } else {
        nv_optrans(cap);
    };
}
unsafe extern "C" fn nv_abbrev(mut cap: *mut cmdarg_T) {
    if (*cap).cmdchar == K_DEL
        || (*cap).cmdchar == -(253 as c_int + ((KE_KDEL as c_int) << 8 as c_int))
    {
        (*cap).cmdchar = 'x' as c_int;
    }
    if VIsual_active.get() {
        v_visop(cap);
    } else {
        nv_optrans(cap);
    };
}
unsafe extern "C" fn nv_optrans(mut cap: *mut cmdarg_T) {
    static ar: GlobalCell<[*const c_char; 8]> = GlobalCell::new([
        b"dl\0".as_ptr() as *const c_char,
        b"dh\0".as_ptr() as *const c_char,
        b"d$\0".as_ptr() as *const c_char,
        b"c$\0".as_ptr() as *const c_char,
        b"cl\0".as_ptr() as *const c_char,
        b"cc\0".as_ptr() as *const c_char,
        b"yy\0".as_ptr() as *const c_char,
        b":s\r\0".as_ptr() as *const c_char,
    ]);
    static str: GlobalCell<*const c_char> =
        GlobalCell::new(b"xXDCsSY&\0".as_ptr() as *const c_char);
    if !checkclearopq((*cap).oap) {
        if (*cap).count0 != 0 {
            stuffnumReadbuff((*cap).count0);
        }
        stuffReadbuff(
            (*ar.ptr())[strchr(str.get(), (*cap).cmdchar as c_char as c_int).offset_from(str.get())
                as usize] as *const c_char,
        );
    }
    (*cap).opcount = 0 as c_int;
}
unsafe extern "C" fn nv_gomark(mut cap: *mut cmdarg_T) {
    let mut name: c_int = 0;
    let mut flags: MarkMove = (if jop_flags.get() & kOptJopFlagView as c_int as c_uint != 0 {
        kMarkSetView as c_int
    } else {
        0 as c_int
    }) as MarkMove;
    if (*(*cap).oap).op_type != OP_NOP as c_int {
        flags = 0 as MarkMove;
    }
    let mut move_res: MarkMoveRes = 0 as MarkMoveRes;
    let old_KeyTyped: bool = KeyTyped.get();
    if (*cap).cmdchar == 'g' as c_int {
        name = (*cap).extra_char;
        flags = (flags as c_uint | KMarkNoContext as c_int as c_uint) as MarkMove;
    } else {
        name = (*cap).nchar;
        flags = (flags as c_uint | kMarkContext as c_int as c_uint) as MarkMove;
    }
    flags = (flags as c_uint
        | (if (*cap).arg != 0 {
            kMarkBeginLine as c_int
        } else {
            0 as c_int
        }) as c_uint) as MarkMove;
    flags = (flags as c_uint
        | (if (*cap).count0 != 0 {
            kMarkSetView as c_int
        } else {
            0 as c_int
        }) as c_uint) as MarkMove;
    let mut fm: *mut fmark_T = mark_get(
        curbuf.get(),
        curwin.get(),
        ::core::ptr::null_mut::<fmark_T>(),
        kMarkAll,
        name,
    );
    move_res = nv_mark_move_to(cap, flags, fm);
    if !virtual_active(curwin.get()) {
        (*curwin.get()).w_cursor.coladd = 0 as c_int as colnr_T;
    }
    if (*(*cap).oap).op_type == OP_NOP as c_int
        && move_res as c_uint & kMarkMoveSuccess as c_int as c_uint != 0
        && (move_res as c_uint & kMarkSwitchedBuf as c_int as c_uint != 0
            || move_res as c_uint & kMarkChangedCursor as c_int as c_uint != 0)
        && fdo_flags.get() & kOptFdoFlagMark as c_int as c_uint != 0
        && old_KeyTyped as c_int != 0
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_pcmark(mut cap: *mut cmdarg_T) {
    let mut fm: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
    let mut flags: MarkMove = (if jop_flags.get() & kOptJopFlagView as c_int as c_uint != 0 {
        kMarkSetView as c_int
    } else {
        0 as c_int
    }) as MarkMove;
    let mut move_res: MarkMoveRes = 0 as MarkMoveRes;
    let old_KeyTyped: bool = KeyTyped.get();
    if checkclearopq((*cap).oap) {
        return;
    }
    if (*cap).cmdchar == TAB && mod_mask.get() == MOD_MASK_CTRL {
        if !goto_tabpage_lastused() {
            clearopbeep((*cap).oap);
        }
        return;
    }
    if (*cap).cmdchar == 'g' as c_int {
        fm = get_changelist(curbuf.get(), curwin.get(), (*cap).count1);
    } else {
        fm = get_jumplist(curwin.get(), (*cap).count1);
        flags = (flags as c_uint | (KMarkNoContext as c_int | kMarkJumpList as c_int) as c_uint)
            as MarkMove;
    }
    if !fm.is_null() {
        move_res = nv_mark_move_to(cap, flags, fm);
    } else if (*cap).cmdchar == 'g' as c_int {
        if (*curbuf.get()).b_changelistlen == 0 as c_int {
            emsg(gettext(e_changelist_is_empty.as_ptr()));
        } else if (*cap).count1 < 0 as c_int {
            emsg(gettext(
                b"E662: At start of changelist\0".as_ptr() as *const c_char
            ));
        } else {
            emsg(gettext(
                b"E663: At end of changelist\0".as_ptr() as *const c_char
            ));
        }
    } else {
        clearopbeep((*cap).oap);
    }
    if (*(*cap).oap).op_type == OP_NOP as c_int
        && (move_res as c_uint & kMarkSwitchedBuf as c_int as c_uint != 0
            || move_res as c_uint & kMarkChangedLine as c_int as c_uint != 0)
        && fdo_flags.get() & kOptFdoFlagMark as c_int as c_uint != 0
        && old_KeyTyped as c_int != 0
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_regname(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    if (*cap).nchar == '=' as c_int {
        (*cap).nchar = get_expr_register();
    }
    if (*cap).nchar != NUL && valid_yank_reg((*cap).nchar, false_0 != 0) as c_int != 0 {
        (*(*cap).oap).regname = (*cap).nchar;
        (*cap).opcount = (*cap).count0;
        set_reg_var((*(*cap).oap).regname);
    } else {
        clearopbeep((*cap).oap);
    };
}
unsafe extern "C" fn nv_visual(mut cap: *mut cmdarg_T) {
    if (*cap).cmdchar == Ctrl_Q {
        (*cap).cmdchar = Ctrl_V;
    }
    if (*(*cap).oap).op_type != OP_NOP as c_int {
        (*(*cap).oap).motion_force = (*cap).cmdchar;
        motion_force.set((*(*cap).oap).motion_force);
        finish_op.set(false_0 != 0);
        return;
    }
    VIsual_select.set((*cap).arg != 0);
    if VIsual_active.get() {
        if VIsual_mode.get() == (*cap).cmdchar {
            end_visual_mode();
        } else {
            VIsual_mode.set((*cap).cmdchar);
            showmode();
            may_trigger_modechanged();
        }
        redraw_curbuf_later(UPD_INVERTED as c_int);
    } else if (*cap).count0 > 0 as c_int && resel_VIsual_mode.get() != NUL {
        VIsual.set((*curwin.get()).w_cursor);
        VIsual_active.set(true_0 != 0);
        VIsual_reselect.set(true_0);
        if (*cap).arg == 0 {
            may_start_select('c' as c_int);
        }
        setmouse();
        if p_smd.get() != 0 && msg_silent.get() == 0 as c_int {
            redraw_cmdline.set(true_0 != 0);
        }
        if resel_VIsual_mode.get() != 'v' as c_int || resel_VIsual_line_count.get() > 1 as linenr_T
        {
            (*curwin.get()).w_cursor.lnum = ((*curwin.get()).w_cursor.lnum as c_int
                + (resel_VIsual_line_count.get() * (*cap).count0 as linenr_T - 1 as linenr_T)
                    as c_int) as linenr_T;
            check_cursor(curwin.get());
        }
        VIsual_mode.set(resel_VIsual_mode.get());
        if VIsual_mode.get() == 'v' as c_int {
            if resel_VIsual_line_count.get() <= 1 as linenr_T {
                update_curswant_force();
                '_c2rust_label: {
                    if (*cap).count0 >= -2147483647 as c_int - 1 as c_int
                        && (*cap).count0 <= 2147483647 as c_int
                    {
                    } else {
                        __assert_fail(
                            b"cap->count0 >= INT_MIN && cap->count0 <= INT_MAX\0".as_ptr()
                                as *const c_char,
                            b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                            5057 as c_uint,
                            b"void nv_visual(cmdarg_T *)\0".as_ptr() as *const c_char,
                        );
                    }
                };
                (*curwin.get()).w_curswant += resel_VIsual_vcol.get() as c_int * (*cap).count0;
                if *p_sel.get() as c_int != 'e' as c_int {
                    (*curwin.get()).w_curswant -= 1;
                }
            } else {
                (*curwin.get()).w_curswant = resel_VIsual_vcol.get();
            }
            coladvance(curwin.get(), (*curwin.get()).w_curswant);
        }
        if resel_VIsual_vcol.get() == MAXCOL as c_int {
            (*curwin.get()).w_curswant = MAXCOL as c_int as colnr_T;
            coladvance(curwin.get(), MAXCOL as c_int);
        } else if VIsual_mode.get() == Ctrl_V {
            let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
            (*curwin.get()).w_cursor.lnum = (*VIsual.ptr()).lnum;
            update_curswant_force();
            '_c2rust_label_0: {
                if (*cap).count0 >= -2147483647 as c_int - 1 as c_int
                    && (*cap).count0 <= 2147483647 as c_int
                {
                } else {
                    __assert_fail(
                        b"cap->count0 >= INT_MIN && cap->count0 <= INT_MAX\0".as_ptr()
                            as *const c_char,
                        b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                        5075 as c_uint,
                        b"void nv_visual(cmdarg_T *)\0".as_ptr() as *const c_char,
                    );
                }
            };
            (*curwin.get()).w_curswant +=
                resel_VIsual_vcol.get() as c_int * (*cap).count0 - 1 as c_int;
            (*curwin.get()).w_cursor.lnum = lnum;
            if *p_sel.get() as c_int == 'e' as c_int {
                (*curwin.get()).w_curswant += 1;
            }
            coladvance(curwin.get(), (*curwin.get()).w_curswant);
        } else {
            (*curwin.get()).w_set_curswant = true_0;
        }
        redraw_curbuf_later(UPD_INVERTED as c_int);
    } else {
        if (*cap).arg == 0 {
            may_start_select('c' as c_int);
        }
        n_start_visual_mode((*cap).cmdchar);
        if VIsual_mode.get() != 'V' as c_int && *p_sel.get() as c_int == 'e' as c_int {
            (*cap).count1 += 1;
        } else {
            VIsual_select_exclu_adj.set(false_0 != 0);
        }
        if (*cap).count0 > 0 as c_int && {
            (*cap).count1 -= 1;
            (*cap).count1 > 0 as c_int
        } {
            if VIsual_mode.get() == 'v' as c_int || VIsual_mode.get() == Ctrl_V {
                nv_right(cap);
            } else if VIsual_mode.get() == 'V' as c_int {
                nv_down(cap);
            }
        }
    };
}
pub unsafe extern "C" fn start_selection() {
    may_start_select('k' as c_int);
    n_start_visual_mode('v' as c_int);
}
pub unsafe extern "C" fn may_start_select(mut c: c_int) {
    VIsual_select.set(
        (c == 'o' as c_int || stuff_empty() as c_int != 0 && typebuf_typed() != 0)
            && !vim_strchr(p_slm.get(), c).is_null(),
    );
}
unsafe extern "C" fn n_start_visual_mode(mut c: c_int) {
    VIsual_mode.set(c);
    VIsual_active.set(true_0 != 0);
    VIsual_reselect.set(true_0);
    if c == Ctrl_V
        && get_ve_flags(curwin.get()) & kOptVeFlagBlock as c_int as c_uint != 0
        && gchar_cursor() == TAB
    {
        validate_virtcol(curwin.get());
        coladvance(curwin.get(), (*curwin.get()).w_virtcol);
    }
    VIsual.set((*curwin.get()).w_cursor);
    foldAdjustVisual();
    may_trigger_modechanged();
    setmouse();
    conceal_check_cursor_line();
    if p_smd.get() != 0 && msg_silent.get() == 0 as c_int {
        redraw_cmdline.set(true_0 != 0);
    }
    if (*curwin.get()).w_redr_type < UPD_INVERTED as c_int {
        (*curwin.get()).w_old_cursor_lnum = (*curwin.get()).w_cursor.lnum;
        (*curwin.get()).w_old_visual_lnum = (*curwin.get()).w_cursor.lnum;
    }
    redraw_curbuf_later(UPD_VALID as c_int);
}
unsafe extern "C" fn nv_window(mut cap: *mut cmdarg_T) {
    if (*cap).nchar == ':' as c_int {
        (*cap).cmdchar = ':' as c_int;
        (*cap).nchar = NUL;
        nv_colon(cap);
    } else if !checkclearop((*cap).oap) {
        do_window((*cap).nchar, (*cap).count0, NUL);
    }
}
unsafe extern "C" fn nv_suspend(mut cap: *mut cmdarg_T) {
    clearop((*cap).oap);
    if VIsual_active.get() {
        end_visual_mode();
    }
    do_cmdline_cmd(b"st\0".as_ptr() as *const c_char);
}
unsafe extern "C" fn nv_gv_cmd(mut cap: *mut cmdarg_T) {
    if (*curbuf.get()).b_visual.vi_start.lnum == 0 as linenr_T
        || (*curbuf.get()).b_visual.vi_start.lnum > (*curbuf.get()).b_ml.ml_line_count
        || (*curbuf.get()).b_visual.vi_end.lnum == 0 as linenr_T
    {
        beep_flush();
        return;
    }
    let mut tpos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    if VIsual_active.get() {
        let mut i: c_int = VIsual_mode.get();
        VIsual_mode.set((*curbuf.get()).b_visual.vi_mode);
        (*curbuf.get()).b_visual.vi_mode = i;
        (*curbuf.get()).b_visual_mode_eval = i;
        i = (*curwin.get()).w_curswant as c_int;
        (*curwin.get()).w_curswant = (*curbuf.get()).b_visual.vi_curswant;
        (*curbuf.get()).b_visual.vi_curswant = i as colnr_T;
        tpos = (*curbuf.get()).b_visual.vi_end;
        (*curbuf.get()).b_visual.vi_end = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor = (*curbuf.get()).b_visual.vi_start;
        (*curbuf.get()).b_visual.vi_start = VIsual.get();
    } else {
        VIsual_mode.set((*curbuf.get()).b_visual.vi_mode);
        (*curwin.get()).w_curswant = (*curbuf.get()).b_visual.vi_curswant;
        tpos = (*curbuf.get()).b_visual.vi_end;
        (*curwin.get()).w_cursor = (*curbuf.get()).b_visual.vi_start;
    }
    VIsual_active.set(true_0 != 0);
    VIsual_reselect.set(true_0);
    check_cursor(curwin.get());
    VIsual.set((*curwin.get()).w_cursor);
    (*curwin.get()).w_cursor = tpos;
    check_cursor(curwin.get());
    update_topline(curwin.get());
    if (*cap).arg != 0 {
        VIsual_select.set(true_0 != 0);
        VIsual_select_reg.set(0 as c_int);
    } else {
        may_start_select('c' as c_int);
    }
    setmouse();
    redraw_curbuf_later(UPD_INVERTED as c_int);
    showmode();
}
pub unsafe extern "C" fn nv_g_home_m_cmd(mut cap: *mut cmdarg_T) {
    let mut i: c_int = 0;
    let flag: bool = (*cap).nchar == '^' as c_int;
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    if (*curwin.get()).w_onebuf_opt.wo_wrap != 0 && (*curwin.get()).w_view_width != 0 as c_int {
        let mut width1: c_int = (*curwin.get()).w_view_width - win_col_off(curwin.get());
        let mut width2: c_int = width1 + win_col_off2(curwin.get());
        validate_virtcol(curwin.get());
        i = 0 as c_int;
        if (*curwin.get()).w_virtcol >= width1 && width2 > 0 as c_int {
            i = ((*curwin.get()).w_virtcol as c_int - width1) / width2 * width2 + width1;
        }
        if (*curwin.get()).w_skipcol > 0 as c_int
            && (*curwin.get()).w_cursor.lnum == (*curwin.get()).w_topline
        {
            let mut overlap: c_int =
                sms_marker_overlap(curwin.get(), (*curwin.get()).w_view_width - width2);
            if overlap > 0 as c_int && i == (*curwin.get()).w_skipcol {
                i += overlap;
            }
        }
    } else {
        i = (*curwin.get()).w_leftcol as c_int;
    }
    if (*cap).nchar == 'm' as c_int {
        i += ((*curwin.get()).w_view_width - win_col_off(curwin.get())
            + (if (*curwin.get()).w_onebuf_opt.wo_wrap != 0 && i > 0 as c_int {
                win_col_off2(curwin.get())
            } else {
                0 as c_int
            }))
            / 2 as c_int;
    }
    coladvance(curwin.get(), i);
    if flag {
        loop {
            i = gchar_cursor();
            if !(ascii_iswhite(i) as c_int != 0 && oneright() == OK) {
                break;
            }
        }
        (*curwin.get()).w_valid &= !VALID_WCOL;
    }
    (*curwin.get()).w_set_curswant = true_0;
    if hasAnyFolding(curwin.get()) != 0 {
        validate_cheight(curwin.get());
        if (*curwin.get()).w_cline_folded {
            update_curswant_force();
        }
    }
    adjust_skipcol();
}
unsafe extern "C" fn nv_g_underscore_cmd(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = true_0 != 0;
    (*curwin.get()).w_curswant = MAXCOL as c_int as colnr_T;
    if cursor_down(
        (*cap).count1 - 1 as c_int,
        (*(*cap).oap).op_type == OP_NOP as c_int,
    ) == false_0
    {
        clearopbeep((*cap).oap);
        return;
    }
    let mut ptr: *mut c_char = get_cursor_line_ptr();
    if (*curwin.get()).w_cursor.col > 0 as c_int
        && *ptr.offset((*curwin.get()).w_cursor.col as isize) as c_int == NUL
    {
        (*curwin.get()).w_cursor.col -= 1;
    }
    while (*curwin.get()).w_cursor.col > 0 as c_int
        && ascii_iswhite(*ptr.offset((*curwin.get()).w_cursor.col as isize) as c_int) as c_int != 0
    {
        (*curwin.get()).w_cursor.col -= 1;
    }
    (*curwin.get()).w_set_curswant = true_0;
    adjust_for_sel(cap);
}
unsafe extern "C" fn nv_g_dollar_cmd(mut cap: *mut cmdarg_T) {
    let mut oap: *mut oparg_T = (*cap).oap;
    let mut i: c_int = 0;
    let mut col_off: c_int = win_col_off(curwin.get());
    let flag: bool = (*cap).nchar == K_END || (*cap).nchar == K_KEND;
    (*oap).motion_type = kMTCharWise;
    (*oap).inclusive = true_0 != 0;
    if (*curwin.get()).w_onebuf_opt.wo_wrap != 0 && (*curwin.get()).w_view_width != 0 as c_int {
        (*curwin.get()).w_curswant = MAXCOL as c_int as colnr_T;
        if (*cap).count1 == 1 as c_int {
            let mut width1: c_int = (*curwin.get()).w_view_width - col_off;
            let mut width2: c_int = width1 + win_col_off2(curwin.get());
            validate_virtcol(curwin.get());
            i = width1 - 1 as c_int;
            if (*curwin.get()).w_virtcol >= width1 {
                i += (((*curwin.get()).w_virtcol as c_int - width1) / width2 + 1 as c_int) * width2;
            }
            coladvance(curwin.get(), i);
            update_curswant_force();
            if (*curwin.get()).w_cursor.col > 0 as c_int
                && (*curwin.get()).w_onebuf_opt.wo_wrap != 0
            {
                if (*curwin.get()).w_virtcol > i {
                    (*curwin.get()).w_cursor.col -= 1;
                }
            }
        } else if nv_screengo(
            oap,
            FORWARD as c_int,
            (*cap).count1 - 1 as c_int,
            false_0 != 0,
        ) as c_int
            == false_0
        {
            clearopbeep(oap);
        }
    } else {
        if (*cap).count1 > 1 as c_int {
            cursor_down((*cap).count1 - 1 as c_int, false_0 != 0);
        }
        i = (*curwin.get()).w_leftcol as c_int + (*curwin.get()).w_view_width
            - col_off
            - 1 as c_int;
        coladvance(curwin.get(), i);
        if (*curwin.get()).w_cursor.col > 0 as c_int
            && utf_ptr2cells(get_cursor_pos_ptr()) > 1 as c_int
        {
            let mut vcol: colnr_T = 0;
            getvvcol(
                curwin.get(),
                &raw mut (*curwin.get()).w_cursor,
                ::core::ptr::null_mut::<colnr_T>(),
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut vcol,
            );
            if vcol >= (*curwin.get()).w_leftcol as c_int + (*curwin.get()).w_view_width - col_off {
                (*curwin.get()).w_cursor.col -= 1;
            }
        }
        update_curswant_force();
    }
    if flag {
        loop {
            i = gchar_cursor();
            if !(ascii_iswhite_or_nul(i) as c_int != 0 && oneleft() == OK) {
                break;
            }
        }
        (*curwin.get()).w_valid &= !VALID_WCOL;
    }
}
unsafe extern "C" fn nv_gi_cmd(mut cap: *mut cmdarg_T) {
    if (*curbuf.get()).b_last_insert.mark.lnum != 0 as linenr_T {
        (*curwin.get()).w_cursor = (*curbuf.get()).b_last_insert.mark;
        check_cursor_lnum(curwin.get());
        let mut i: c_int = get_cursor_line_len();
        if (*curwin.get()).w_cursor.col > i {
            if virtual_active(curwin.get()) {
                (*curwin.get()).w_cursor.coladd += (*curwin.get()).w_cursor.col as c_int - i;
            }
            (*curwin.get()).w_cursor.col = i as colnr_T;
        }
    }
    (*cap).cmdchar = 'i' as c_int;
    nv_edit(cap);
}
unsafe extern "C" fn nv_g_cmd(mut cap: *mut cmdarg_T) {
    let mut oap: *mut oparg_T = (*cap).oap;
    let mut i: c_int = 0;
    's_650: {
        'c_40473: {
            'c_36907: {
                match (*cap).nchar {
                    Ctrl_A | Ctrl_X => {
                        if VIsual_active.get() {
                            (*cap).arg = true_0;
                            (*cap).cmdchar = (*cap).nchar;
                            (*cap).nchar = NUL;
                            nv_addsub(cap);
                        } else {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    82 => {
                        (*cap).arg = true_0;
                        nv_Replace(cap);
                        break 's_650;
                    }
                    114 => {
                        nv_vreplace(cap);
                        break 's_650;
                    }
                    38 => {
                        do_cmdline_cmd(b"%s//~/&\0".as_ptr() as *const c_char);
                        break 's_650;
                    }
                    118 => {
                        nv_gv_cmd(cap);
                        break 's_650;
                    }
                    86 => {
                        VIsual_reselect.set(false_0);
                        break 's_650;
                    }
                    K_BS => {
                        (*cap).nchar = Ctrl_H;
                    }
                    104 | 72 | Ctrl_H => {}
                    78 | 110 => {
                        if current_search((*cap).count1, (*cap).nchar == 'n' as c_int) == 0 {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    106 | K_DOWN => {
                        if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                            (*oap).motion_type = kMTLineWise;
                            i = cursor_down((*cap).count1, (*oap).op_type == OP_NOP as c_int);
                        } else {
                            i = nv_screengo(oap, FORWARD as c_int, (*cap).count1, false_0 != 0)
                                as c_int;
                        }
                        if i == 0 {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    107 | K_UP => {
                        if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
                            (*oap).motion_type = kMTLineWise;
                            i = cursor_up(
                                (*cap).count1 as linenr_T,
                                (*oap).op_type == OP_NOP as c_int,
                            );
                        } else {
                            i = nv_screengo(oap, BACKWARD as c_int, (*cap).count1, false_0 != 0)
                                as c_int;
                        }
                        if i == 0 {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    74 => {
                        nv_join(cap);
                        break 's_650;
                    }
                    94 | 48 | 109 | K_HOME | K_KHOME => {
                        nv_g_home_m_cmd(cap);
                        break 's_650;
                    }
                    77 => {
                        (*oap).motion_type = kMTCharWise;
                        (*oap).inclusive = false_0 != 0;
                        i = linetabsize(curwin.get(), (*curwin.get()).w_cursor.lnum);
                        if (*cap).count0 > 0 as c_int && (*cap).count0 <= 100 as c_int {
                            coladvance(curwin.get(), i * (*cap).count0 / 100 as c_int);
                        } else {
                            coladvance(curwin.get(), i / 2 as c_int);
                        }
                        (*curwin.get()).w_set_curswant = true_0;
                        break 's_650;
                    }
                    95 => {
                        nv_g_underscore_cmd(cap);
                        break 's_650;
                    }
                    36 | K_END | K_KEND => {
                        nv_g_dollar_cmd(cap);
                        break 's_650;
                    }
                    42 | 35 | POUND | Ctrl_RSB | 93 => {
                        nv_ident(cap);
                        break 's_650;
                    }
                    101 | 69 => {
                        (*oap).motion_type = kMTCharWise;
                        (*curwin.get()).w_set_curswant = true_0;
                        (*oap).inclusive = true_0 != 0;
                        if bckend_word((*cap).count1, (*cap).nchar == 'E' as c_int, false_0 != 0)
                            == false_0
                        {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    Ctrl_G => {
                        cursor_pos_info(::core::ptr::null_mut::<dict_T>());
                        break 's_650;
                    }
                    105 => {
                        nv_gi_cmd(cap);
                        break 's_650;
                    }
                    73 => {
                        beginline(0 as c_int);
                        if !checkclearopq(oap) {
                            invoke_edit(cap, false_0, 'g' as c_int, false_0);
                        }
                        break 's_650;
                    }
                    102 | 70 => {
                        nv_gotofile(cap);
                        break 's_650;
                    }
                    39 => {
                        (*cap).arg = true_0;
                        break 'c_36907;
                    }
                    96 => {
                        break 'c_36907;
                    }
                    115 => {
                        do_sleep(((*cap).count1 * 1000 as c_int) as int64_t, false_0 != 0);
                        break 's_650;
                    }
                    97 => {
                        do_ascii(::core::ptr::null_mut::<exarg_T>());
                        break 's_650;
                    }
                    56 => {
                        if (*cap).count0 == 8 as c_int {
                            utf_find_illegal();
                        } else {
                            show_utf8();
                        }
                        break 's_650;
                    }
                    60 => {
                        show_sb_text();
                        break 's_650;
                    }
                    103 => {
                        (*cap).arg = false_0;
                        nv_goto(cap);
                        break 's_650;
                    }
                    113 | 119 => {
                        (*oap).cursor_start = (*curwin.get()).w_cursor;
                        break 'c_40473;
                    }
                    126 | 117 | 85 | 63 | 64 => {
                        break 'c_40473;
                    }
                    100 | 68 => {
                        nv_gd(oap, (*cap).nchar, (*cap).count0);
                        break 's_650;
                    }
                    -12285 | -12541 | -12797 | -11517 | -11773 | -12029 | -25853 | -13053
                    | -13309 | -13565 | -23037 | -23293 | -23549 | -23805 | -24061 | -24317 => {
                        mod_mask.set(MOD_MASK_CTRL);
                        do_mouse(oap, (*cap).nchar, BACKWARD as c_int, (*cap).count1, false);
                        break 's_650;
                    }
                    -13821 => {
                        break 's_650;
                    }
                    112 | 80 => {
                        nv_put(cap);
                        break 's_650;
                    }
                    111 => {
                        (*oap).inclusive = false_0 != 0;
                        goto_byte((*cap).count0);
                        break 's_650;
                    }
                    81 => {
                        if !check_text_locked((*cap).oap) && !checkclearopq(oap) {
                            do_exmode();
                        }
                        break 's_650;
                    }
                    44 => {
                        nv_pcmark(cap);
                        break 's_650;
                    }
                    59 => {
                        (*cap).count1 = -(*cap).count1;
                        nv_pcmark(cap);
                        break 's_650;
                    }
                    116 => {
                        if !checkclearop(oap) {
                            goto_tabpage((*cap).count0);
                        }
                        break 's_650;
                    }
                    84 => {
                        if !checkclearop(oap) {
                            goto_tabpage(-(*cap).count1);
                        }
                        break 's_650;
                    }
                    TAB => {
                        if !checkclearop(oap) && !goto_tabpage_lastused() {
                            clearopbeep(oap);
                        }
                        break 's_650;
                    }
                    43 | 45 => {
                        if !checkclearopq(oap) {
                            undo_time(
                                if (*cap).nchar == '-' as c_int {
                                    -(*cap).count1
                                } else {
                                    (*cap).count1
                                },
                                false_0 != 0,
                                false_0 != 0,
                                false_0 != 0,
                            );
                        }
                        break 's_650;
                    }
                    _ => {
                        clearopbeep(oap);
                        break 's_650;
                    }
                }
                (*cap).cmdchar = (*cap).nchar + ('v' as c_int - 'h' as c_int);
                (*cap).arg = true_0;
                nv_visual(cap);
                break 's_650;
            }
            nv_gomark(cap);
            break 's_650;
        }
        nv_operator(cap);
    };
}
unsafe extern "C" fn n_opencmd(mut cap: *mut cmdarg_T) {
    if checkclearopq((*cap).oap) {
        return;
    }
    if (*cap).cmdchar == 'O' as c_int {
        hasFolding(
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            &raw mut (*curwin.get()).w_cursor.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
        );
    } else {
        hasFolding(
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut (*curwin.get()).w_cursor.lnum,
        );
    }
    (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());
    if u_save(
        (*curwin.get()).w_cursor.lnum
            - (if (*cap).cmdchar == 'O' as c_int {
                1 as linenr_T
            } else {
                0 as linenr_T
            }),
        (*curwin.get()).w_cursor.lnum
            + (if (*cap).cmdchar == 'o' as c_int {
                1 as linenr_T
            } else {
                0 as linenr_T
            }),
    ) != 0
        && open_line(
            if (*cap).cmdchar == 'O' as c_int {
                BACKWARD as c_int
            } else {
                FORWARD as c_int
            },
            if has_format_option(FO_OPEN_COMS) as c_int != 0 {
                OPENLINE_DO_COM as c_int
            } else {
                0 as c_int
            },
            0 as c_int,
            ::core::ptr::null_mut::<bool>(),
        ) as c_int
            != 0
    {
        if win_cursorline_standout(curwin.get()) {
            (*curwin.get()).w_valid &= !VALID_CROW;
        }
        invoke_edit(cap, false_0, (*cap).cmdchar, true_0);
    }
}
unsafe extern "C" fn nv_dot(mut cap: *mut cmdarg_T) {
    if checkclearopq((*cap).oap) {
        return;
    }
    if start_redo(
        (*cap).count0,
        restart_edit.get() != 0 as c_int && !arrow_used.get(),
    ) == false_0
    {
        clearopbeep((*cap).oap);
    }
}
unsafe extern "C" fn nv_redo_or_register(mut cap: *mut cmdarg_T) {
    if VIsual_select.get() as c_int != 0 && VIsual_active.get() as c_int != 0 {
        (*no_mapping.ptr()) += 1;
        let mut reg: c_int = plain_vgetc();
        if *p_langmap.get() as c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as c_int) as c_int
                } else {
                    KeyTyped.get() as c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && reg >= 0 as c_int
        {
            if reg < 256 as c_int {
                reg = (*langmap_mapchar.ptr())[reg as usize] as c_int;
            } else {
                reg = langmap_adjust_mb(reg);
            }
        }
        (*no_mapping.ptr()) -= 1;
        if reg == '"' as c_int {
            reg = 0 as c_int;
        }
        VIsual_select_reg.set(if valid_yank_reg(reg, true_0 != 0) as c_int != 0 {
            reg
        } else {
            0 as c_int
        });
        return;
    }
    if checkclearopq((*cap).oap) {
        return;
    }
    u_redo((*cap).count1);
    (*curwin.get()).w_set_curswant = true_0;
}
unsafe extern "C" fn nv_Undo(mut cap: *mut cmdarg_T) {
    if (*(*cap).oap).op_type == OP_UPPER as c_int || VIsual_active.get() as c_int != 0 {
        (*cap).cmdchar = 'g' as c_int;
        (*cap).nchar = 'U' as c_int;
        nv_operator(cap);
        return;
    }
    if checkclearopq((*cap).oap) {
        return;
    }
    u_undoline();
    (*curwin.get()).w_set_curswant = true_0;
}
unsafe extern "C" fn nv_tilde(mut cap: *mut cmdarg_T) {
    if p_to.get() == 0 && !VIsual_active.get() && (*(*cap).oap).op_type != OP_TILDE as c_int {
        if bt_prompt(curbuf.get()) as c_int != 0 && !prompt_curpos_editable() {
            clearopbeep((*cap).oap);
            return;
        }
        n_swapchar(cap);
    } else {
        nv_operator(cap);
    };
}
unsafe extern "C" fn nv_operator(mut cap: *mut cmdarg_T) {
    let mut op_type: c_int = get_op_type((*cap).cmdchar, (*cap).nchar);
    if bt_prompt(curbuf.get()) as c_int != 0
        && op_is_change(op_type) != 0
        && !prompt_curpos_editable()
    {
        clearopbeep((*cap).oap);
        return;
    }
    if op_type == (*(*cap).oap).op_type {
        nv_lineop(cap);
    } else if !checkclearop((*cap).oap) {
        (*(*cap).oap).start = (*curwin.get()).w_cursor;
        (*(*cap).oap).op_type = op_type;
        set_op_var(op_type);
    }
}
unsafe extern "C" fn set_op_var(mut optype: c_int) {
    if optype == OP_NOP as c_int {
        set_vim_var_string(VV_OP, ::core::ptr::null::<c_char>(), 0 as ptrdiff_t);
    } else {
        let mut opchars: [c_char; 3] = [0; 3];
        let mut opchar0: c_int = get_op_char(optype);
        '_c2rust_label: {
            if opchar0 >= 0 as c_int && opchar0 <= 127 as c_int * 2 as c_int + 1 as c_int {
            } else {
                __assert_fail(
                    b"opchar0 >= 0 && opchar0 <= UCHAR_MAX\0".as_ptr() as *const c_char,
                    b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                    5876 as c_uint,
                    b"void set_op_var(int)\0".as_ptr() as *const c_char,
                );
            }
        };
        opchars[0 as c_int as usize] = opchar0 as c_char;
        let mut opchar1: c_int = get_extra_op_char(optype);
        '_c2rust_label_0: {
            if opchar1 >= 0 as c_int && opchar1 <= 127 as c_int * 2 as c_int + 1 as c_int {
            } else {
                __assert_fail(
                    b"opchar1 >= 0 && opchar1 <= UCHAR_MAX\0".as_ptr() as *const c_char,
                    b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                    5880 as c_uint,
                    b"void set_op_var(int)\0".as_ptr() as *const c_char,
                );
            }
        };
        opchars[1 as c_int as usize] = opchar1 as c_char;
        opchars[2 as c_int as usize] = NUL as c_char;
        set_vim_var_string(VV_OP, &raw mut opchars as *mut c_char, 2 as ptrdiff_t);
    };
}
unsafe extern "C" fn nv_lineop(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTLineWise;
    if cursor_down(
        (*cap).count1 - 1 as c_int,
        (*(*cap).oap).op_type == OP_NOP as c_int,
    ) == false_0
    {
        clearopbeep((*cap).oap);
    } else if (*(*cap).oap).op_type == OP_DELETE as c_int
        && (*(*cap).oap).motion_force != 'v' as c_int
        && (*(*cap).oap).motion_force != Ctrl_V
        || (*(*cap).oap).op_type == OP_LSHIFT as c_int
        || (*(*cap).oap).op_type == OP_RSHIFT as c_int
    {
        beginline(BL_SOL as c_int | BL_FIX as c_int);
    } else if (*(*cap).oap).op_type != OP_YANK as c_int {
        beginline(BL_WHITE as c_int | BL_FIX as c_int);
    }
}
unsafe extern "C" fn nv_home(mut cap: *mut cmdarg_T) {
    if mod_mask.get() & MOD_MASK_CTRL != 0 {
        nv_goto(cap);
    } else {
        (*cap).count0 = 1 as c_int;
        nv_pipe(cap);
    }
    ins_at_eol.set(false_0 != 0);
}
unsafe extern "C" fn nv_pipe(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    beginline(0 as c_int);
    if (*cap).count0 > 0 as c_int {
        coladvance(curwin.get(), (*cap).count0 - 1 as c_int);
        (*curwin.get()).w_curswant = (*cap).count0 - 1 as c_int;
    } else {
        (*curwin.get()).w_curswant = 0 as c_int as colnr_T;
    }
    (*curwin.get()).w_set_curswant = false_0;
}
unsafe extern "C" fn nv_bck_word(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    (*curwin.get()).w_set_curswant = true_0;
    if bck_word((*cap).count1, (*cap).arg != 0, false_0 != 0) == false_0 {
        clearopbeep((*cap).oap);
    } else if fdo_flags.get() & kOptFdoFlagHor as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_wordcmd(mut cap: *mut cmdarg_T) {
    let mut n: c_int = 0;
    let mut word_end: bool = false;
    let mut flag: bool = false_0 != 0;
    let mut startpos: pos_T = (*curwin.get()).w_cursor;
    if (*cap).cmdchar == 'e' as c_int || (*cap).cmdchar == 'E' as c_int {
        word_end = true_0 != 0;
    } else {
        word_end = false_0 != 0;
    }
    (*(*cap).oap).inclusive = word_end;
    if !word_end && (*(*cap).oap).op_type == OP_CHANGE as c_int {
        n = gchar_cursor();
        if n != NUL && !ascii_iswhite(n) {
            if !vim_strchr(p_cpo.get(), CPO_CHANGEW).is_null() {
                (*(*cap).oap).inclusive = true_0 != 0;
                word_end = true_0 != 0;
            }
            flag = true_0 != 0;
        }
    }
    (*(*cap).oap).motion_type = kMTCharWise;
    (*curwin.get()).w_set_curswant = true_0;
    if word_end {
        n = end_word((*cap).count1, (*cap).arg != 0, flag, false_0 != 0);
    } else {
        n = fwd_word(
            (*cap).count1,
            (*cap).arg != 0,
            (*(*cap).oap).op_type != OP_NOP as c_int,
        );
    }
    if lt(startpos, (*curwin.get()).w_cursor) {
        adjust_cursor((*cap).oap);
    }
    if n == false_0 && (*(*cap).oap).op_type == OP_NOP as c_int {
        clearopbeep((*cap).oap);
    } else {
        adjust_for_sel(cap);
        if fdo_flags.get() & kOptFdoFlagHor as c_int as c_uint != 0
            && KeyTyped.get() as c_int != 0
            && (*(*cap).oap).op_type == OP_NOP as c_int
        {
            foldOpenCursor();
        }
    };
}
unsafe extern "C" fn adjust_cursor(mut oap: *mut oparg_T) {
    if (*curwin.get()).w_cursor.col > 0 as c_int
        && gchar_cursor() == NUL
        && (!VIsual_active.get() || *p_sel.get() as c_int == 'o' as c_int)
        && !virtual_active(curwin.get())
        && get_ve_flags(curwin.get()) & kOptVeFlagOnemore as c_int as c_uint == 0 as c_uint
    {
        (*curwin.get()).w_cursor.col -= 1;
        mb_adjust_cursor();
        (*oap).inclusive = true_0 != 0;
    }
}
unsafe extern "C" fn nv_beginline(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTCharWise;
    (*(*cap).oap).inclusive = false_0 != 0;
    beginline((*cap).arg);
    if fdo_flags.get() & kOptFdoFlagHor as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
    ins_at_eol.set(false_0 != 0);
}
unsafe extern "C" fn adjust_for_sel(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() as c_int != 0
        && (*(*cap).oap).inclusive as c_int != 0
        && *p_sel.get() as c_int == 'e' as c_int
        && gchar_cursor() != NUL
        && lt(VIsual.get(), (*curwin.get()).w_cursor) as c_int != 0
    {
        inc_cursor();
        (*(*cap).oap).inclusive = false_0 != 0;
        VIsual_select_exclu_adj.set(true_0 != 0);
    }
}
pub unsafe extern "C" fn unadjust_for_sel() -> bool {
    if *p_sel.get() as c_int == 'e' as c_int && !equalpos(VIsual.get(), (*curwin.get()).w_cursor) {
        return unadjust_for_sel_inner(
            if lt(VIsual.get(), (*curwin.get()).w_cursor) as c_int != 0 {
                &raw mut (*curwin.get()).w_cursor
            } else {
                VIsual.ptr()
            },
        );
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn unadjust_for_sel_inner(mut pp: *mut pos_T) -> bool {
    VIsual_select_exclu_adj.set(false_0 != 0);
    if (*pp).coladd > 0 as c_int {
        (*pp).coladd -= 1;
    } else if (*pp).col > 0 as c_int {
        (*pp).col -= 1;
        mark_mb_adjustpos(curbuf.get(), pp);
        if virtual_active(curwin.get()) {
            let mut cs: colnr_T = 0;
            let mut ce: colnr_T = 0;
            getvcol(
                curwin.get(),
                pp,
                &raw mut cs,
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut ce,
            );
            (*pp).coladd = ce - cs;
        }
    } else if (*pp).lnum > 1 as linenr_T {
        (*pp).lnum -= 1;
        (*pp).col = ml_get_len((*pp).lnum);
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn nv_select(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() {
        VIsual_select.set(true_0 != 0);
        VIsual_select_reg.set(0 as c_int);
    } else if VIsual_reselect.get() != 0 {
        (*cap).nchar = 'v' as c_int;
        (*cap).arg = true_0;
        nv_g_cmd(cap);
    }
}
unsafe extern "C" fn nv_goto(mut cap: *mut cmdarg_T) {
    let mut lnum: linenr_T = 0;
    if (*cap).arg != 0 {
        lnum = (*curbuf.get()).b_ml.ml_line_count;
    } else {
        lnum = 1 as c_int as linenr_T;
    }
    (*(*cap).oap).motion_type = kMTLineWise;
    setpcmark();
    if (*cap).count0 != 0 as c_int {
        lnum = (*cap).count0 as linenr_T;
    }
    lnum = if (if lnum > 1 as linenr_T {
        lnum
    } else {
        1 as linenr_T
    }) < (*curbuf.get()).b_ml.ml_line_count
    {
        if lnum > 1 as linenr_T {
            lnum
        } else {
            1 as linenr_T
        }
    } else {
        (*curbuf.get()).b_ml.ml_line_count
    };
    (*curwin.get()).w_cursor.lnum = lnum;
    beginline(BL_SOL as c_int | BL_FIX as c_int);
    if fdo_flags.get() & kOptFdoFlagJump as c_int as c_uint != 0
        && KeyTyped.get() as c_int != 0
        && (*(*cap).oap).op_type == OP_NOP as c_int
    {
        foldOpenCursor();
    }
}
unsafe extern "C" fn nv_normal(mut cap: *mut cmdarg_T) {
    if (*cap).nchar == Ctrl_N || (*cap).nchar == Ctrl_G {
        clearop((*cap).oap);
        if restart_edit.get() != 0 as c_int && mode_displayed.get() as c_int != 0 {
            clear_cmdline.set(true_0 != 0);
        }
        restart_edit.set(0 as c_int);
        if cmdwin_type.get() != 0 as c_int {
            cmdwin_result.set(Ctrl_C);
        }
        if VIsual_active.get() {
            end_visual_mode();
            redraw_curbuf_later(UPD_INVERTED as c_int);
        }
    } else {
        clearopbeep((*cap).oap);
    };
}
unsafe extern "C" fn nv_esc(mut cap: *mut cmdarg_T) {
    let mut no_reason: bool = (*(*cap).oap).op_type == OP_NOP as c_int
        && (*cap).opcount == 0 as c_int
        && (*cap).count0 == 0 as c_int
        && (*(*cap).oap).regname == 0 as c_int;
    if (*cap).arg != 0 {
        if restart_edit.get() == 0 as c_int
            && cmdwin_type.get() == 0 as c_int
            && !VIsual_active.get()
            && no_reason as c_int != 0
        {
            if anyBufIsChanged() {
                msg(
                    gettext(
                        b"Type  :qa!  and press <Enter> to abandon all changes and exit Nvim\0"
                            .as_ptr() as *const c_char,
                    ),
                    0 as c_int,
                );
            } else {
                msg(
                    gettext(
                        b"Type  :qa  and press <Enter> to exit Nvim\0".as_ptr() as *const c_char
                    ),
                    0 as c_int,
                );
            }
        }
        if restart_edit.get() != 0 as c_int {
            redraw_mode.set(true_0 != 0);
        }
        restart_edit.set(0 as c_int);
        if cmdwin_type.get() != 0 as c_int {
            cmdwin_result.set(-(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int)));
            got_int.set(false_0 != 0);
            return;
        }
    } else if cmdwin_type.get() != 0 as c_int
        && ex_normal_busy.get() != 0
        && typebuf_was_empty.get() as c_int != 0
    {
        cmdwin_result.set(-(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int)));
        return;
    }
    if VIsual_active.get() {
        end_visual_mode();
        check_cursor_col(curwin.get());
        (*curwin.get()).w_set_curswant = true_0;
        redraw_curbuf_later(UPD_INVERTED as c_int);
    } else if no_reason {
        vim_beep(kOptBoFlagEsc as c_int as c_uint);
    }
    clearop((*cap).oap);
}
pub unsafe extern "C" fn set_cursor_for_append_to_line() {
    (*curwin.get()).w_set_curswant = true_0;
    if get_ve_flags(curwin.get()) == kOptVeFlagAll as c_int as c_uint {
        let save_State: c_int = State.get();
        State.set(MODE_INSERT as c_int);
        coladvance(curwin.get(), MAXCOL as c_int);
        State.set(save_State);
    } else {
        (*curwin.get()).w_cursor.col += strlen(get_cursor_pos_ptr()) as colnr_T;
    };
}
unsafe extern "C" fn nv_edit(mut cap: *mut cmdarg_T) {
    if (*cap).cmdchar == K_INS
        || (*cap).cmdchar == -(253 as c_int + ((KE_KINS as c_int) << 8 as c_int))
    {
        (*cap).cmdchar = 'i' as c_int;
    }
    if VIsual_active.get() as c_int != 0
        && ((*cap).cmdchar == 'A' as c_int || (*cap).cmdchar == 'I' as c_int)
    {
        v_visop(cap);
    } else if ((*cap).cmdchar == 'a' as c_int || (*cap).cmdchar == 'i' as c_int)
        && ((*(*cap).oap).op_type != OP_NOP as c_int || VIsual_active.get() as c_int != 0)
    {
        nv_object(cap);
    } else if (*curbuf.get()).b_p_ma == 0 && (*curbuf.get()).terminal.is_null() {
        emsg(gettext(&raw const e_modifiable as *const c_char));
        clearop((*cap).oap);
    } else if !checkclearopq((*cap).oap) {
        match (*cap).cmdchar {
            65 => {
                set_cursor_for_append_to_line();
            }
            73 => {
                beginline(BL_WHITE as c_int);
            }
            97 => {
                if virtual_active(curwin.get()) as c_int != 0
                    && ((*curwin.get()).w_cursor.coladd > 0 as c_int
                        || *get_cursor_pos_ptr() as c_int == NUL
                        || *get_cursor_pos_ptr() as c_int == TAB)
                {
                    (*curwin.get()).w_cursor.coladd += 1;
                } else if *get_cursor_pos_ptr() as c_int != NUL {
                    inc_cursor();
                }
            }
            _ => {}
        }
        if (*curwin.get()).w_cursor.coladd != 0 && (*cap).cmdchar != 'A' as c_int {
            let mut save_State: c_int = State.get();
            State.set(MODE_INSERT as c_int);
            coladvance(curwin.get(), getviscol());
            State.set(save_State);
        }
        invoke_edit(cap, false_0, (*cap).cmdchar, false_0);
    }
}
unsafe extern "C" fn invoke_edit(
    mut cap: *mut cmdarg_T,
    mut repl: c_int,
    mut cmd: c_int,
    mut startln: c_int,
) {
    let mut restart_edit_save: c_int = 0 as c_int;
    if repl != 0 || !stuff_empty() {
        restart_edit_save = restart_edit.get();
    } else {
        restart_edit_save = 0 as c_int;
    }
    restart_edit.set(0 as c_int);
    if (*cap).cmdchar != 'O' as c_int && (*cap).cmdchar != 'o' as c_int {
        (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());
    }
    if edit(cmd, startln != 0, (*cap).count1) {
        (*cap).retval |= CA_COMMAND_BUSY as c_int;
    }
    if restart_edit.get() == 0 as c_int {
        restart_edit.set(restart_edit_save);
    }
}
unsafe extern "C" fn nv_object(mut cap: *mut cmdarg_T) {
    let mut flag: bool = false;
    let mut include: bool = false;
    if (*cap).cmdchar == 'i' as c_int {
        include = false_0 != 0;
    } else {
        include = true_0 != 0;
    }
    let mut mps_save: *mut c_char = (*curbuf.get()).b_p_mps;
    (*curbuf.get()).b_p_mps = b"(:),{:},[:],<:>\0".as_ptr() as *const c_char as *mut c_char;
    match (*cap).nchar {
        119 => {
            flag = current_word((*cap).oap, (*cap).count1, include, false_0 != 0) != 0;
        }
        87 => {
            flag = current_word((*cap).oap, (*cap).count1, include, true_0 != 0) != 0;
        }
        98 | 40 | 41 => {
            flag = current_block(
                (*cap).oap,
                (*cap).count1,
                include,
                '(' as c_int,
                ')' as c_int,
            ) != 0;
        }
        66 | 123 | 125 => {
            flag = current_block(
                (*cap).oap,
                (*cap).count1,
                include,
                '{' as c_int,
                '}' as c_int,
            ) != 0;
        }
        91 | 93 => {
            flag = current_block(
                (*cap).oap,
                (*cap).count1,
                include,
                '[' as c_int,
                ']' as c_int,
            ) != 0;
        }
        60 | 62 => {
            flag = current_block(
                (*cap).oap,
                (*cap).count1,
                include,
                '<' as c_int,
                '>' as c_int,
            ) != 0;
        }
        116 => {
            (*cap).retval |= CA_NO_ADJ_OP_END as c_int;
            flag = current_tagblock((*cap).oap, (*cap).count1, include) != 0;
        }
        112 => {
            flag = current_par((*cap).oap, (*cap).count1, include, 'p' as c_int) != 0;
        }
        115 => {
            flag = current_sent((*cap).oap, (*cap).count1, include) != 0;
        }
        34 | 39 | 96 => {
            flag = current_quote((*cap).oap, (*cap).count1, include, (*cap).nchar);
        }
        _ => {
            flag = false_0 != 0;
        }
    }
    (*curbuf.get()).b_p_mps = mps_save;
    if !flag {
        clearopbeep((*cap).oap);
    }
    adjust_cursor_col();
    (*curwin.get()).w_set_curswant = true_0;
}
unsafe extern "C" fn nv_record(mut cap: *mut cmdarg_T) {
    if (*(*cap).oap).op_type == OP_FORMAT as c_int {
        (*cap).cmdchar = 'g' as c_int;
        (*cap).nchar = 'q' as c_int;
        nv_operator(cap);
        return;
    }
    if checkclearop((*cap).oap) {
        return;
    }
    if (*cap).nchar == ':' as c_int || (*cap).nchar == '/' as c_int || (*cap).nchar == '?' as c_int
    {
        if cmdwin_type.get() != 0 as c_int {
            emsg(gettext(e_cmdline_window_already_open.as_ptr()));
            return;
        }
        stuffcharReadbuff((*cap).nchar);
        stuffcharReadbuff(-(253 as c_int + ((KE_CMDWIN as c_int) << 8 as c_int)));
    } else if reg_executing.get() == 0 as c_int && do_record((*cap).nchar) == FAIL {
        clearopbeep((*cap).oap);
    }
}
unsafe extern "C" fn nv_at(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    if (*cap).nchar == '=' as c_int {
        if get_expr_register() == NUL {
            return;
        }
    }
    loop {
        let c2rust_fresh13 = (*cap).count1;
        (*cap).count1 = (*cap).count1 - 1;
        if !(c2rust_fresh13 != 0 && !got_int.get()) {
            break;
        }
        if do_execreg((*cap).nchar, false_0, false_0, false_0) == false_0 {
            clearopbeep((*cap).oap);
            break;
        } else {
            line_breakcheck();
        }
    }
}
unsafe extern "C" fn nv_halfpage(mut cap: *mut cmdarg_T) {
    if !checkclearop((*cap).oap) {
        pagescroll(
            (if (*cap).cmdchar == Ctrl_D {
                FORWARD as c_int
            } else {
                BACKWARD as c_int
            }) as Direction,
            (*cap).count0,
            true_0 != 0,
        );
    }
}
unsafe extern "C" fn nv_join(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() {
        nv_operator(cap);
        return;
    }
    if checkclearop((*cap).oap) {
        return;
    }
    (*cap).count0 = if (*cap).count0 > 2 as c_int {
        (*cap).count0
    } else {
        2 as c_int
    };
    if (*curwin.get()).w_cursor.lnum + (*cap).count0 as linenr_T - 1 as linenr_T
        > (*curbuf.get()).b_ml.ml_line_count
    {
        if (*cap).count0 <= 2 as c_int {
            clearopbeep((*cap).oap);
            return;
        }
        (*cap).count0 = ((*curbuf.get()).b_ml.ml_line_count - (*curwin.get()).w_cursor.lnum
            + 1 as linenr_T) as c_int;
    }
    prep_redo(
        (*(*cap).oap).regname,
        (*cap).count0,
        NUL,
        (*cap).cmdchar,
        NUL,
        NUL,
        (*cap).nchar,
    );
    do_join(
        (*cap).count0 as size_t,
        (*cap).nchar == NUL,
        true_0 != 0,
        true_0 != 0,
        true_0 != 0,
    );
}
unsafe extern "C" fn nv_put(mut cap: *mut cmdarg_T) {
    nv_put_opt(cap, false_0 != 0);
}
unsafe extern "C" fn nv_put_opt(mut cap: *mut cmdarg_T, mut fix_indent: bool) {
    let mut savereg: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
    let mut empty: bool = false_0 != 0;
    let mut was_visual: bool = false_0 != 0;
    let mut dir: c_int = 0;
    let mut flags: c_int = 0 as c_int;
    let save_fen: c_int = (*curwin.get()).w_onebuf_opt.wo_fen;
    if (*(*cap).oap).op_type != OP_NOP as c_int {
        if (*(*cap).oap).op_type == OP_DELETE as c_int && (*cap).cmdchar == 'p' as c_int {
            clearop((*cap).oap);
            '_c2rust_label: {
                if (*cap).opcount >= 0 as c_int {
                } else {
                    __assert_fail(
                        b"cap->opcount >= 0\0".as_ptr() as *const c_char,
                        b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                        6502 as c_uint,
                        b"void nv_put_opt(cmdarg_T *, _Bool)\0".as_ptr() as *const c_char,
                    );
                }
            };
            nv_diffgetput(true_0 != 0, (*cap).opcount as size_t);
        } else {
            clearopbeep((*cap).oap);
        }
        return;
    }
    if bt_prompt(curbuf.get()) as c_int != 0 && !prompt_curpos_editable() {
        if (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_prompt_start.mark.lnum {
            (*curwin.get()).w_cursor.col = (*curbuf.get()).b_prompt_start.mark.col;
            (*cap).cmdchar = 'P' as c_int;
        } else {
            clearopbeep((*cap).oap);
            return;
        }
    }
    if fix_indent {
        dir = if (*cap).cmdchar == ']' as c_int && (*cap).nchar == 'p' as c_int {
            FORWARD as c_int
        } else {
            BACKWARD as c_int
        };
        flags |= PUT_FIXINDENT as c_int;
    } else {
        dir = if (*cap).cmdchar == 'P' as c_int
            || ((*cap).cmdchar == 'g' as c_int || (*cap).cmdchar == 'z' as c_int)
                && (*cap).nchar == 'P' as c_int
        {
            BACKWARD as c_int
        } else {
            FORWARD as c_int
        };
    }
    prep_redo_cmd(cap);
    if (*cap).cmdchar == 'g' as c_int {
        flags |= PUT_CURSEND as c_int;
    } else if (*cap).cmdchar == 'z' as c_int {
        flags |= PUT_BLOCK_INNER as c_int;
    }
    if VIsual_active.get() {
        was_visual = true_0 != 0;
        let mut regname: c_int = (*(*cap).oap).regname;
        let mut keep_registers: bool = (*cap).cmdchar == 'P' as c_int;
        let mut clipoverwrite: bool = (regname == '+' as c_int || regname == '*' as c_int)
            && cb_flags.get()
                & (kOptCbFlagUnnamed as c_int | kOptCbFlagUnnamedplus as c_int) as c_uint
                != 0;
        if regname == 0 as c_int
            || regname == '"' as c_int
            || clipoverwrite as c_int != 0
            || ascii_isdigit(regname) as c_int != 0
            || regname == '-' as c_int
        {
            savereg = copy_register(regname);
        }
        (*curwin.get()).w_onebuf_opt.wo_fen = false_0;
        if !VIsual_active.get() || VIsual_mode.get() == 'V' as c_int || regname != '.' as c_int {
            (*cap).cmdchar = 'd' as c_int;
            (*cap).nchar = NUL;
            (*(*cap).oap).regname = if keep_registers as c_int != 0 {
                '_' as c_int
            } else {
                NUL
            };
            (*msg_silent.ptr()) += 1;
            nv_operator(cap);
            do_pending_operator(cap, 0 as c_int, false_0 != 0);
            empty = (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0;
            (*msg_silent.ptr()) -= 1;
            (*(*cap).oap).regname = regname;
        }
        if VIsual_mode.get() == 'V' as c_int {
            flags |= PUT_LINE as c_int;
        } else if VIsual_mode.get() == 'v' as c_int {
            flags |= PUT_LINE_SPLIT as c_int;
        }
        if VIsual_mode.get() == Ctrl_V && dir == FORWARD as c_int {
            flags |= PUT_LINE_FORWARD as c_int;
        }
        dir = BACKWARD as c_int;
        if VIsual_mode.get() != 'V' as c_int
            && (*curwin.get()).w_cursor.col < (*curbuf.get()).b_op_start.col
            || VIsual_mode.get() == 'V' as c_int
                && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_op_start.lnum
        {
            dir = FORWARD as c_int;
        }
        VIsual_active.set(true_0 != 0);
    }
    do_put((*(*cap).oap).regname, savereg, dir, (*cap).count1, flags);
    if !savereg.is_null() {
        free_register(savereg);
        xfree(savereg as *mut c_void);
    }
    if was_visual {
        if save_fen != 0 {
            (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
        }
        (*curbuf.get()).b_visual.vi_start = (*curbuf.get()).b_op_start;
        (*curbuf.get()).b_visual.vi_end = (*curbuf.get()).b_op_end;
        if *p_sel.get() as c_int == 'e' as c_int {
            inc(&raw mut (*curbuf.get()).b_visual.vi_end);
        }
    }
    if empty as c_int != 0 && *ml_get((*curbuf.get()).b_ml.ml_line_count) as c_int == NUL {
        ml_delete_flags((*curbuf.get()).b_ml.ml_line_count, ML_DEL_MESSAGE as c_int);
        deleted_lines(
            (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T,
            1 as linenr_T,
        );
        if (*curwin.get()).w_cursor.lnum > (*curbuf.get()).b_ml.ml_line_count {
            (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
            coladvance(curwin.get(), MAXCOL as c_int);
        }
    }
    auto_format(false_0 != 0, true_0 != 0);
}
unsafe extern "C" fn nv_open(mut cap: *mut cmdarg_T) {
    if (*(*cap).oap).op_type == OP_DELETE as c_int && (*cap).cmdchar == 'o' as c_int {
        clearop((*cap).oap);
        '_c2rust_label: {
            if (*cap).opcount >= 0 as c_int {
            } else {
                __assert_fail(
                    b"cap->opcount >= 0\0".as_ptr() as *const c_char,
                    b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                    6645 as c_uint,
                    b"void nv_open(cmdarg_T *)\0".as_ptr() as *const c_char,
                );
            }
        };
        nv_diffgetput(false_0 != 0, (*cap).opcount as size_t);
    } else if VIsual_active.get() {
        v_swap_corners((*cap).cmdchar);
    } else if bt_prompt(curbuf.get()) as c_int != 0
        && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_prompt_start.mark.lnum
    {
        clearopbeep((*cap).oap);
    } else {
        n_opencmd(cap);
    };
}
unsafe extern "C" fn nv_paste(mut cap: *mut cmdarg_T) {
    paste_repeat((*cap).count1);
}
unsafe extern "C" fn nv_event(mut cap: *mut cmdarg_T) {
    may_garbage_collect.set(false_0 != 0);
    let mut may_restart: bool =
        restart_edit.get() != 0 as c_int || restart_VIsual_select.get() != 0 as c_int;
    state_handle_k_event();
    finish_op.set(false_0 != 0);
    if may_restart {
        (*cap).retval |= CA_COMMAND_BUSY as c_int;
    }
}
pub unsafe extern "C" fn normal_cmd(mut oap: *mut oparg_T, mut toplevel: bool) {
    let mut s: NormalState = NormalState {
        state: VimState {
            check: None,
            execute: None,
        },
        command_finished: false,
        ctrl_w: false,
        need_flushbuf: false,
        set_prevcount: false,
        previous_got_int: false,
        cmdwin: false,
        noexmode: false,
        toplevel: false,
        oa: oparg_T {
            op_type: 0,
            regname: 0,
            motion_type: kMTCharWise,
            motion_force: 0,
            use_reg_one: false,
            inclusive: false,
            end_adjusted: false,
            start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            end: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            cursor_start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            line_count: 0,
            empty: false,
            is_VIsual: false,
            start_vcol: 0,
            end_vcol: 0,
            prev_opcount: 0,
            prev_count0: 0,
            excl_tr_ws: false,
        },
        ca: cmdarg_T {
            oap: ::core::ptr::null_mut::<oparg_T>(),
            prechar: 0,
            cmdchar: 0,
            nchar: 0,
            nchar_composing: [0; 32],
            nchar_len: 0,
            extra_char: 0,
            opcount: 0,
            count0: 0,
            count1: 0,
            arg: 0,
            retval: 0,
            searchbuf: ::core::ptr::null_mut::<c_char>(),
        },
        mapped_len: 0,
        old_mapped_len: 0,
        idx: 0,
        c: 0,
        old_col: 0,
        old_pos: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
    };
    normal_state_init(&raw mut s);
    s.toplevel = toplevel;
    s.oa = *oap;
    normal_prepare(&raw mut s);
    normal_execute(&raw mut s.state, safe_vgetc());
    *oap = s.oa;
}
pub const INT_MAX: c_int = __INT_MAX__;
pub const true_0: c_int = 1 as c_int;
pub const false_0: c_int = 0 as c_int;
pub const __INT_MAX__: c_int = 2147483647 as c_int;
