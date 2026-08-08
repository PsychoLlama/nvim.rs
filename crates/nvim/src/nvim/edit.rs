#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::ascii::{
    ascii_isdigit, ascii_isspace, ascii_iswhite, ascii_iswhite_nl_or_nul, ascii_isxdigit,
};
use crate::src::nvim::autocmd::{
    EVENT_BUFMODIFIEDSET, EVENT_CURSORMOVEDI, EVENT_INSERTCHANGE, EVENT_INSERTCHARPRE,
    EVENT_INSERTENTER, EVENT_INSERTLEAVE, EVENT_INSERTLEAVEPRE, EVENT_TEXTCHANGEDI,
    EVENT_TEXTCHANGEDP, apply_autocmds, aucmd_prepbuf, aucmd_restbuf, has_event,
};
use crate::src::nvim::buffer::{bt_prompt, bt_quickfix, buf_is_empty};
use crate::src::nvim::buffer::{buf_get_changedtick, buf_meta_total};
use crate::src::nvim::change::{
    appended_lines_mark, change_warning, changed_bytes, del_bytes, del_char, get_leader_len,
    ins_bytes_len, ins_char, ins_char_bytes, ins_str, inserted_bytes, open_line,
};
use crate::src::nvim::charset::{
    byte2cells, char2cells, hex2nr, ptr2cells, skipwhite, vim_isprintc, vim_iswordc,
};
use crate::src::nvim::cursor::{
    char_before_cursor, check_cursor, check_cursor_col, check_visual_pos, coladvance, dec_cursor,
    gchar_cursor, get_cursor_line_len, get_cursor_line_ptr, get_cursor_pos_len, get_cursor_pos_ptr,
    getviscol, inc_cursor,
};
use crate::src::nvim::decoration::{decor_conceal_line, kMTMetaInline, win_lines_concealed};
use crate::src::nvim::digraph::{digraph_get, do_digraph};
use crate::src::nvim::drawscreen::{
    UPD_VALID, redraw_later, redraw_statuslines, redrawWinline, redrawing, setcursor,
    show_cursor_info_later, showmode, skip_showmode, status_redraw_curbuf, unshowmode,
    update_screen,
};
use crate::src::nvim::eval::vars::{get_vim_var_str, set_vim_var_string};
use crate::src::nvim::eval::{invoke_prompt_interrupt, prompt_invoke_callback};
use crate::src::nvim::ex_docmd::{do_cmdline, do_cmdline_cmd, expr_map_locked};
use crate::src::nvim::fileio::check_timestamps;
use crate::src::nvim::fold::{
    foldCheckClose, foldOpenCursor, foldUpdateAfterInsert, hasFolding, hasFoldingWin,
};
use crate::src::nvim::getchar::{
    AppendCharToRedobuff, AppendNumberToRedobuff, AppendToRedobuff, AppendToRedobuffLit,
    ResetRedobuff, char_avail, get_inserted, getcmdkeycmd, map_execute_lua, merge_modifiers,
    paste_repeat, plain_vgetc, start_redo_ins, stop_redo_ins, stuff_empty, stuffReadbuffLen,
    stuffRedoReadbuff, stuffcharReadbuff, typebuf_maplen, vgetc, vpeekc, vungetc,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{
    grid_line_flush, grid_line_getchar, grid_line_put_schar, grid_line_puts, grid_line_start,
};
use crate::src::nvim::highlight_group::{HLF_8, highlight_changed};
use crate::src::nvim::indent::{
    change_indent, fix_indent, get_indent, get_sts_value, get_sw_value, inindent, ins_try_si,
    may_do_si, tabstop_at, tabstop_count, tabstop_first, tabstop_padding, tabstop_start,
};
use crate::src::nvim::indent_c::{cindent_on, do_c_expr_indent, in_cinkeys};
use crate::src::nvim::insexpand::{
    check_compl_option, compl_status_clear, compl_status_local, ctrl_x_mode_cmdline,
    ctrl_x_mode_dictionary, ctrl_x_mode_files, ctrl_x_mode_function, ctrl_x_mode_line_or_eval,
    ctrl_x_mode_none, ctrl_x_mode_normal, ctrl_x_mode_omni, ctrl_x_mode_path_defines,
    ctrl_x_mode_path_patterns, ctrl_x_mode_register, ctrl_x_mode_scroll, ctrl_x_mode_spell,
    ctrl_x_mode_tags, ctrl_x_mode_thesaurus, ctrl_x_mode_whole_line, ins_compl_accept_char,
    ins_compl_active, ins_compl_addfrommatch, ins_compl_addleader, ins_compl_bs, ins_compl_cancel,
    ins_compl_clear, ins_compl_col, ins_compl_delete, ins_compl_enable_autocomplete,
    ins_compl_enter_selects, ins_compl_has_autocomplete, ins_compl_has_shown_match,
    ins_compl_init_get_longest, ins_compl_insert, ins_compl_is_match_selected,
    ins_compl_long_shown_match, ins_compl_preinsert_effect, ins_compl_preinsert_longest,
    ins_compl_prep, ins_compl_used_match, ins_compl_win_active, ins_complete, ins_ctrl_x,
    pum_wanted,
};
use crate::src::nvim::keycodes::{
    Ctrl__, Ctrl_A, Ctrl_BSL, Ctrl_C, Ctrl_D, Ctrl_E, Ctrl_F, Ctrl_G, Ctrl_H, Ctrl_HAT, Ctrl_J,
    Ctrl_K, Ctrl_L, Ctrl_N, Ctrl_O, Ctrl_P, Ctrl_Q, Ctrl_R, Ctrl_RSB, Ctrl_S, Ctrl_T, Ctrl_U,
    Ctrl_V, Ctrl_W, Ctrl_X, Ctrl_Y, Ctrl_Z, K_BS, K_COMMAND, K_DEL, K_DOWN, K_END, K_F1, K_HELP,
    K_HOME, K_INS, K_KDEL, K_KEND, K_KENTER, K_KHOME, K_KINS, K_KPAGEDOWN, K_KPAGEUP, K_LEFT,
    K_LEFTDRAG, K_LEFTMOUSE, K_LEFTMOUSE_NM, K_LEFTRELEASE, K_LEFTRELEASE_NM, K_LUA, K_MIDDLEDRAG,
    K_MIDDLEMOUSE, K_MIDDLERELEASE, K_MOUSEDOWN, K_MOUSELEFT, K_MOUSEMOVE, K_MOUSERIGHT, K_MOUSEUP,
    K_PAGEDOWN, K_PAGEUP, K_PASTE_START, K_RIGHT, K_RIGHTDRAG, K_RIGHTMOUSE, K_RIGHTRELEASE,
    K_S_DOWN, K_S_END, K_S_HOME, K_S_LEFT, K_S_RIGHT, K_S_TAB, K_S_UP, K_SELECT, K_SPECIAL, K_UP,
    K_X1DRAG, K_X1MOUSE, K_X1RELEASE, K_X2DRAG, K_X2MOUSE, K_X2RELEASE, K_XF1, K_ZERO, KE_C_LEFT,
    KE_C_RIGHT, KE_EVENT, KE_IGNORE, KE_NOP, add_char2buf, get_special_key_name,
};
use crate::src::nvim::main::{
    Insstart, Insstart_orig, KeyStuffed, KeyTyped, RedrawingDisabled, State, VIsual_active, ai_col,
    allow_keys, arrow_used, can_si, can_si_back, clear_cmdline, cmdmod, cmdwin_result, cmdwin_type,
    curbuf, curwin, default_grid, did_ai, did_check_timestamps, did_cursorhold, did_si,
    disable_fold_update, dollar_vcol, e_noinstext, e_sandbox, e_textlock, edit_submode_extra,
    emsg_on_display, end_comment_pending, ex_normal_busy, fdo_flags, first_tabpage,
    force_restart_edit, got_int, hl_attr_active, ins_at_eol, km_startsel, langmap_mapchar,
    last_cursormoved, last_cursormoved_win, mod_mask, msg_scroll, msg_silent, must_redraw,
    need_check_timestamps, need_highlight_changed, need_start_insertmode, no_abbr, no_mapping,
    no_u_sync, old_indent, orig_line_count, p_ari, p_ch, p_cpo, p_deco, p_langmap, p_lrm, p_paste,
    p_ri, p_smd, p_sol, p_sta, p_ww, pum_want, redraw_cmdline, redraw_mode, reg_recording,
    replace_offset, restart_VIsual_select, restart_edit, sandbox, spell_redraw_lnum,
    stop_insert_mode, test_disable_char_avail, textlock, u_sync_once, vgetc_busy, vr_lines_changed,
    where_paste_started,
};
use crate::src::nvim::mapping::{check_abbr, langmap_adjust_mb, map_to_exists_mode};
use crate::src::nvim::mark::{free_fmark, mark_view_make};
use crate::src::nvim::mbyte::{
    mb_adjust_cursor, mb_get_class, utf_char2bytes, utf_char2len, utf_composinglike, utf_head_off,
    utf_ptr2StrCharInfo, utf_ptr2char, utf_ptr2len, utf8len_tab, utfc_next, utfc_ptr2len,
};
use crate::src::nvim::memline::{gchar_pos, ml_append, ml_get, ml_get_buf, ml_get_len, ml_replace};
use crate::src::nvim::memory::{strnequal, xfree, xmalloc, xmemdupz, xrealloc, xstrdup};
use crate::src::nvim::message::{emsg, msg_check_for_delay};
use crate::src::nvim::mouse::{ins_mouse, ins_mousescroll, setmouse};
use crate::src::nvim::r#move::{
    adjust_skipcol, curs_columns, do_check_cursorbind, pagescroll, scrolldown_clamp,
    scrollup_clamp, set_topline, update_curswant, update_topline, validate_cursor,
    validate_cursor_col, validate_virtcol,
};
use crate::src::nvim::normal::{
    add_to_showcmd, add_to_showcmd_c, clear_showcmd, do_check_scrollbind, end_visual_mode,
    start_selection,
};
use crate::src::nvim::ops::do_join;
use crate::src::nvim::option::{
    can_bs, copy_option_part, get_scrolloff_value, get_ve_flags, set_iminsert_global,
};
use crate::src::nvim::options::{
    kOptBoFlagBackspace, kOptBoFlagCopy, kOptBoFlagCtrlg, kOptBoFlagCursor, kOptBoFlagRegister,
    kOptFdoFlagAll, kOptFdoFlagHor, kOptFdoFlagInsert, kOptVeFlagAll, kOptVeFlagOnemore,
};
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{__ctype_b_loc, gettext, memcpy, memmove, memset, strcmp, strlen};
use crate::src::nvim::os::time::os_time;
use crate::src::nvim::plines::linetabsize_str;
use crate::src::nvim::plines::{
    charsize_nowrap, getvcol, getvcol_nolist, init_charsize_arg, win_charsize, win_chartabsize,
};
use crate::src::nvim::popupmenu::{pum_check_clear, pum_visible};
use crate::src::nvim::pos::{MAXCOL, equalpos};
use crate::src::nvim::register::is_literal_register;
use crate::src::nvim::register::{
    do_put, get_expr_register, get_yank_register, insert_reg, valid_yank_reg,
};
use crate::src::nvim::search::{BACKWARD, FORWARD};
use crate::src::nvim::state::{
    MODE_CMDLINE, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL, MODE_REPLACE, MODE_VREPLACE,
    REPLACE_FLAG, VREPLACE_FLAG, may_trigger_modechanged, may_trigger_safestate, state_enter,
    state_handle_k_event, virtual_active,
};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr, xstrnsave};
use crate::src::nvim::syntax::syntax_present;
use crate::src::nvim::terminal::terminal_enter;
use crate::src::nvim::textformat::{
    auto_format, check_auto_format, comp_textwidth, fex_format, has_format_option, internal_format,
};
use crate::src::nvim::textobject::{bck_word, fwd_word};
use crate::src::nvim::types::ui::kUIMessages;
use crate::src::nvim::types::{
    BS_EOL, BS_INDENT, BS_NOSTOP, BS_START, CMOD_KEEPJUMPS, CharsizeArg, CharsizeKind,
    GraphemeState, INSCHAR_CTRLV, INSCHAR_FORMAT, INSCHAR_NO_FEX, MB_MAXBYTES, OptInt, PUT_CURSEND,
    PUT_FIXINDENT, StrCharInfo, String_0, TriState, VV_CHAR, VV_INSERTMODE, VimState, aco_save_T,
    buf_T, cmdarg_T, colnr_T, event_T, int32_t, int64_t, kFalse, kNone, kTrue, linenr_T, pos_T,
    ptrdiff_t, schar_T, size_t, ssize_t, state_check_callback, state_execute_callback, uint8_t,
    uint32_t, varnumber_T, win_T,
};
use crate::src::nvim::ui::{ui_cursor_shape, ui_flush, ui_has, vim_beep};
use crate::src::nvim::undo::{u_clearallandblockfree, u_save, u_save_cursor, u_sync};
use crate::src::nvim::window::{goto_tabpage, may_trigger_win_scrolled_resized};

// The carve of the transpiled module; see each child's docs.
mod bs;
mod chars;
mod ctrl;
mod cursor;
mod key;
mod lastins;
mod literal;
mod motion;
mod prompt;
mod redraw;
mod replace;
mod state;
mod tab;
mod undo;

pub(crate) use self::bs::*;
pub(crate) use self::chars::*;
pub(crate) use self::ctrl::*;
pub(crate) use self::cursor::*;
pub(crate) use self::key::*;
pub(crate) use self::lastins::*;
pub(crate) use self::literal::*;
pub(crate) use self::motion::*;
pub(crate) use self::prompt::*;
pub(crate) use self::redraw::*;
pub(crate) use self::replace::*;
pub use self::state::*;
pub use self::tab::*;
pub(crate) use self::undo::*;

/// The alphanumeric bit of the C library's `__ctype_b_loc()` table, the one
/// `isalnum()` reads.  Locale-dependent by construction.
pub const _ISalnum: ::core::ffi::c_ushort = 8;
pub const OPENLINE_DO_COM: ::core::ffi::c_int = 2;
pub const INDENT_DEC: ::core::ffi::c_int = 3;
pub const INDENT_INC: ::core::ffi::c_int = 2;
pub const INDENT_SET: ::core::ffi::c_int = 1;
pub const BL_FIX: ::core::ffi::c_int = 4;
pub const BL_SOL: ::core::ffi::c_int = 2;
pub const BL_WHITE: ::core::ffi::c_int = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct InsertState {
    pub state: VimState,
    pub ca: *mut cmdarg_T,
    pub mincol: ::core::ffi::c_int,
    pub cmdchar: ::core::ffi::c_int,
    pub cmdchar_todo: ::core::ffi::c_int,
    pub ins_just_started: bool,
    pub startln: ::core::ffi::c_int,
    pub count: ::core::ffi::c_int,
    pub c: ::core::ffi::c_int,
    pub lastc: ::core::ffi::c_int,
    pub i: ::core::ffi::c_int,
    pub did_backspace: bool,
    pub line_is_white: bool,
    pub old_topline: linenr_T,
    pub old_topfill: ::core::ffi::c_int,
    pub inserted_space: ::core::ffi::c_int,
    pub replaceState: ::core::ffi::c_int,
    pub did_restart_edit: ::core::ffi::c_int,
    pub nomove: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ReplaceStack {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_char,
}
pub const MSCR_RIGHT: ::core::ffi::c_int = -2;
pub const MSCR_LEFT: ::core::ffi::c_int = -1;
pub const MSCR_UP: ::core::ffi::c_int = 1;
pub const MSCR_DOWN: ::core::ffi::c_int = 0;
pub const BACKSPACE_LINE: ::core::ffi::c_int = 4;
pub const BACKSPACE_CHAR: ::core::ffi::c_int = 1;
pub const BACKSPACE_WORD_NOT_SPACE: ::core::ffi::c_int = 3;
pub const BACKSPACE_WORD: ::core::ffi::c_int = 2;
pub const YREG_PASTE: ::core::ffi::c_int = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const REPLACE_STACK_EMPTY: ReplaceStack = ReplaceStack {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
pub const VALID_WROW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VALID_VIRTCOL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const B_IMODE_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const B_IMODE_LMAP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const ML_LINE_DIRTY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const ML_ALLOCATED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const NL_STR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"\n\0") };
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const ESC_STR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"\x1B\0") };
pub const DEL: ::core::ffi::c_int = 0x7f as ::core::ffi::c_int;
pub const CTRL_V_STR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"\x16\0") };
pub const FO_RET_COMS: ::core::ffi::c_int = 'r' as ::core::ffi::c_int;
pub const FO_INS_LONG: ::core::ffi::c_int = 'l' as ::core::ffi::c_int;
pub const FO_INS_BLANK: ::core::ffi::c_int = 'b' as ::core::ffi::c_int;
pub const FO_WHITE_PAR: ::core::ffi::c_int = 'w' as ::core::ffi::c_int;
pub const FO_AUTO: ::core::ffi::c_int = 'a' as ::core::ffi::c_int;
pub const CPO_INDENT: ::core::ffi::c_int = 'I' as ::core::ffi::c_int;
pub const CPO_LISTWM: ::core::ffi::c_int = 'L' as ::core::ffi::c_int;
pub const CPO_BACKSPACE: ::core::ffi::c_int = 'v' as ::core::ffi::c_int;
pub const CPO_REPLCNT: ::core::ffi::c_int = 'X' as ::core::ffi::c_int;
pub const COM_MIDDLE: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const COM_MAX_LEN: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
static compl_busy: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static Insstart_textlen: GlobalCell<colnr_T> = GlobalCell::new(0);
static Insstart_blank_vcol: GlobalCell<colnr_T> = GlobalCell::new(0);
static update_Insstart_orig: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
static last_insert: GlobalCell<String_0> = GlobalCell::new(String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
});
static last_insert_skip: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static new_insert_skip: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static did_restart_edit: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static can_cindent: GlobalCell<bool> = GlobalCell::new(false);
static revins_on: GlobalCell<bool> = GlobalCell::new(false);
static revins_chars: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static revins_legal: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static revins_scol: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static ins_need_undo: GlobalCell<bool> = GlobalCell::new(false);
static dont_sync_undo: GlobalCell<TriState> = GlobalCell::new(kFalse);
static o_lnum: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
static replace_stack: GlobalCell<ReplaceStack> = GlobalCell::new(REPLACE_STACK_EMPTY);
/// What the last `edit_putchar` did to the screen cell it wrote over, and so
/// how `edit_unputchar` has to take it back.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PutChar {
    /// Nothing was put on the screen.
    Unset,
    /// The right half of a double-width character was overwritten.
    Right,
    /// The left half of a double-width character was overwritten.
    Left,
    /// A whole cell was overwritten and `pc_schar`/`pc_attr` hold it.
    Set,
}
static pc_status: GlobalCell<PutChar> = GlobalCell::new(PutChar::Unset);
static pc_schar: GlobalCell<schar_T> = GlobalCell::new(0);
static pc_attr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pc_row: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pc_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub const INPUT_BUFLEN: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
pub const ABBR_OFF: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
pub const MOD_MASK_SHIFT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOD_MASK_CTRL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MOD_MASK_CMD: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const NULL_STRING: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
