//! Insert mode: the whole of it, less the completion machinery
//! (`insexpand.rs`) and the wrapping (`textformat.rs`).
//!
//! The parent keeps no functions.  It holds the vocabulary the children
//! share -- the `BL_*`/`INDENT_*` arguments they pass out of the
//! family, the `FO_*`/`CPO_*` option letters they test, and `InsertState`,
//! which is the *only* per-insert state that is not global -- plus the
//! nineteen `GlobalCell`s the mode runs on.
//!
//! Those cells are what makes Insert mode reentrant with everything else,
//! so here is what writes each one:
//!
//! | cell | what it means | written by |
//! | --- | --- | --- |
//! | `INSSTART_TEXTLEN` | how wide the line was when the insert started | `prompt`, `state`, `undo` |
//! | `INSSTART_BLANK_VCOL` | the column of the first blank inserted ('formatoptions' `b`) | `key`, `prompt`, `state`, `tab` |
//! | `UPDATE_INSSTART_ORIG` | whether `INSSTART_ORIG` still tracks `INSSTART` | `ctrl`, `state`, `undo` |
//! | `ARROW_USED` / `INS_NEED_UNDO` | the undo-block pair; see `undo.rs` | `ctrl`, `key`, `chars`, `state`, `undo` |
//! | `DONT_SYNC_UNDO` | `i_CTRL-G_U`: the next motion keeps the block | `motion`, `ctrl`, `key`, `state` |
//! | `LAST_INSERT` / `LAST_INSERT_SKIP` | the text `.` repeats, and the command on its front | `lastins`, `undo` |
//! | `NEW_INSERT_SKIP` | the length of that command, measured at entry | `state`, `undo` |
//! | `DID_RESTART_EDIT` | `RESTART_EDIT` as it was when `edit()` was called | `state` |
//! | `CAN_CINDENT` | whether the next key may trigger a 'cindent' re-indent | `bs`, `motion`, `ctrl`, `key`, `state`, `tab` |
//! | `REVINS_ON` / `REVINS_CHARS` / `REVINS_LEGAL` / `REVINS_SCOL` | 'revins' and how far back it may go | `ctrl`, `state`, and every key that moves |
//! | `O_LNUM` / `INS_AT_EOL` | where `i_CTRL-O` left the cursor | `state` |
//! | `REPLACE_STACK` | what Replace mode has to put back; see `replace.rs` | `replace`, `undo` |
//! | `PC_STATUS` / `PC_SCHAR` / `PC_ATTR` / `PC_ROW` / `PC_COL` | the cell `edit_putchar` wrote over | `redraw`, and the two keys that draw |
//! | `COMPL_BUSY` | a completion is running, so `edit()` must not recurse | `state` |
//!
//! The children are seamed by **what a key does**: `state` and `key` are the
//! machine and its dispatcher, `bs`/`tab`/`motion`/`ctrl`/`literal`/`chars`
//! are one family of keys each, and `replace`/`redraw`/`undo`/`prompt`/
//! `lastins`/`cursor` are the pieces they share.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::ascii::{ascii_isspace, ascii_iswhite, ascii_iswhite_nl_or_nul, ascii_isxdigit};
use crate::autocmd::defs::{AcoSave, AutoEvent};
use crate::autocmd::state::{DID_CURSORHOLD, LAST_CURSORMOVED, LAST_CURSORMOVED_WIN};
use crate::autocmd::{apply_autocmds, aucmd_prepbuf, aucmd_restbuf, has_event};
use crate::buffer::{
    buf_get_changedtick, buf_is_empty, buf_is_prompt, buf_is_quickfix, buf_meta_total, current_buf,
};
use crate::change::{
    COM_MAX_LEN, COM_MIDDLE, appended_lines_mark, change_warning, changed_bytes, del_bytes,
    del_char, get_leader_len, ins_bytes, ins_char, ins_char_bytes, ins_str, inserted_bytes,
    open_line,
};
use crate::charset::{byte2cells, char2cells, hex2nr, vim_isprintc, vim_iswordc};
use crate::cstr;
use crate::cursor::{
    char_before_cursor, check_cursor, check_cursor_col, check_visual_pos, coladvance, dec_cursor,
    gchar_cursor, get_cursor_pos_len, getviscol, inc_cursor,
};
use crate::decoration::{MT_META_INLINE, decor_conceal_line, win_lines_concealed};
use crate::digraph::{digraph_get, do_digraph, reset_digraph};
use crate::drawscreen::state::{
    CLEAR_CMDLINE, DOLLAR_VCOL, MUST_REDRAW, REDRAW_CMDLINE, REDRAW_MODE, REDRAWING_DISABLED,
};
use crate::drawscreen::{
    UPD_VALID, redraw_later, redraw_statuslines, redraw_win_line, redrawing, setcursor,
    show_cursor_info_later, showmode, skip_showmode, status_redraw_curbuf, unshowmode,
    update_screen,
};
use crate::eval::vars::{set_vim_var_string, vim_var_bytes, with_vim_var_str};
use crate::eval::{invoke_prompt_interrupt, prompt_invoke_callback};
use crate::ex_docmd::state::EX_NORMAL_BUSY;
use crate::ex_docmd::{do_cmdline_cmd, expr_map_locked};
use crate::fileio::check_timestamps;
use crate::fileio::state::{DID_CHECK_TIMESTAMPS, NEED_CHECK_TIMESTAMPS};
use crate::fold::{fold_check_close, fold_open_cursor, fold_update_after_insert};
use crate::getchar::state::{
    GOT_INT, KEY_STUFFED, KEY_TYPED, LANGMAP_MAPCHAR, MOD_MASK, REG_RECORDING, VGETC_BUSY,
    test_disable_char_avail,
};
use crate::getchar::{
    append_to_redobuff, append_to_redobuff_char, append_to_redobuff_literally,
    append_to_redobuff_number, char_avail, get_inserted, map_execute_lua, merge_mod_mask,
    paste_repeat, plain_vgetc, reset_redobuff, start_redo_ins, stop_redo_ins, stuff_empty,
    stuff_readbuf_char, stuff_readbuf_len, stuff_redo_readbuf, vgetc, vpeekc, vungetc,
};
use crate::global_cell::state_record;
use crate::grid::{
    grid_line_flush, grid_line_getchar, grid_line_put_schar, grid_line_puts, grid_line_start,
};
use crate::guard::{SANDBOX, TEXTLOCK};
use crate::highlight::state::NEED_HIGHLIGHT_CHANGED;
use crate::highlight_group::{HLF_8, highlight_changed};
use crate::indent::{
    change_indent, fix_indent, get_indent, get_sts_value, get_sw_value, inindent, ins_try_si,
    may_do_si,
};
use crate::indent_c::{cindent_on, do_c_expr_indent, in_cinkeys};
use crate::insexpand::{
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
use crate::keycodes::KS_MODIFIER;
use crate::keycodes::{K_SPECIAL, get_special_key_name, push_char_escaped};
use crate::mapping::{check_abbr, langmap_adjust_mb, map_to_exists_mode};
use crate::mark::mark_view_make;
use crate::mbyte::MB_MAXBYTES;
use crate::mbyte::{mb_adjust_cursor, utf8len_tab};
use crate::memline::{Lines, gchar_pos, ml_append_text, ml_get_len, ml_replace_buf_text};
use crate::message::state::{EMSG_ON_DISPLAY, MSG_SCROLL, MSG_SILENT};
use crate::message::{E_NOINSTEXT, E_SANDBOX, E_TEXTLOCK};
use crate::message::{emsg, msg_check_for_delay};
use crate::mouse::{
    MSCR_DOWN, MSCR_LEFT, MSCR_RIGHT, MSCR_UP, ins_mouse, ins_mousescroll, setmouse,
};
use crate::r#move::{
    adjust_skipcol, curs_columns, do_check_cursorbind, pagescroll, scrolldown_clamp,
    scrollup_clamp, set_topline, update_curswant, update_topline, validate_cursor,
    validate_cursor_col, validate_virtcol,
};
use crate::normal::{
    add_to_showcmd, add_to_showcmd_c, clear_showcmd, do_check_scrollbind, end_visual_mode,
    start_selection,
};
use crate::ops::do_join;
use crate::option::vars::{
    FDO_FLAGS, P_RI, p_ari, p_ch, p_deco, p_lrm, p_paste, p_ri, p_smd, p_sol, p_sta, p_ww,
};
use crate::option::{B_IMODE_LMAP, B_IMODE_NONE};
use crate::option::{can_bs, get_scrolloff_value, get_ve_flags, set_iminsert_global};
use crate::options::{
    OPT_BO_FLAG_BACKSPACE, OPT_BO_FLAG_COPY, OPT_BO_FLAG_CTRLG, OPT_BO_FLAG_CURSOR,
    OPT_BO_FLAG_REGISTER, OPT_FDO_FLAG_ALL, OPT_FDO_FLAG_HOR, OPT_FDO_FLAG_INSERT, OPT_VE_FLAG_ALL,
    OPT_VE_FLAG_ONEMORE,
};
use crate::os::cshim::{IS_ALNUM, gettext};
use crate::os::input::line_breakcheck;
use crate::os::libc::{int32_t, int64_t, size_t};
use crate::plines::CharsizeArg;
use crate::plines::{
    CHAR_CELLS_SPAN, charsize_nowrap, getvcol, getvcol_nolist, linetabsize_str, win_charsize,
    win_chartabsize,
};
use crate::popupmenu::state::PUM_WANT;
use crate::popupmenu::{pum_check_clear, pum_ext_want_done, pum_visible};
use crate::pos::{MAXCOL, equalpos};
use crate::register::YREG_PASTE;
use crate::register::defs::{PUT_CURSEND, PUT_FIXINDENT};
use crate::register::{
    do_put, get_expr_register, get_yank_register, insert_reg, is_literal_register, register_key,
    register_name, valid_yank_reg,
};
use crate::search::{BACKWARD, FORWARD};
use crate::spell::SPELL_REDRAW_LNUM;
use crate::state::mode::{
    AI_COL, ARROW_USED, CAN_SI, CAN_SI_BACK, DID_AI, DID_SI, EDIT_SUBMODE_EXTRA,
    END_COMMENT_PENDING, FORCE_RESTART_EDIT, INS_AT_EOL, INSSTART, INSSTART_ORIG, KM_STARTSEL,
    NEED_START_INSERTMODE, NO_ABBR, OLD_INDENT, ORIG_LINE_COUNT, REPLACE_OFFSET, RESTART_EDIT,
    RESTART_V_ISUAL_SELECT, STATE, STOP_INSERT_MODE, VR_LINES_CHANGED, WHERE_PASTE_STARTED,
};
use crate::state::{
    MODE_CMDLINE, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL, MODE_REPLACE, MODE_VREPLACE, ModeState,
    REPLACE_FLAG, VREPLACE_FLAG, may_trigger_modechanged, may_trigger_safestate, state_enter,
    state_handle_k_event, virtual_active,
};
use crate::syntax::syntax_present;
use crate::terminal::terminal_enter;
use crate::textformat::{
    auto_format, check_auto_format, comp_textwidth, fex_format, has_format_option, internal_format,
};
use crate::textobject::{bck_word, fwd_word};
use crate::types::Key;
use crate::types::{ApiString, CmdModFlags, ColNr, LineNr, OptInt, Pos, ScreenChar, Vv};
use crate::ui::state::UI_MESSAGES;
use crate::ui::{ui_cursor_shape, ui_flush, ui_has, vim_beep};
use crate::undo::U_SYNC_ONCE;
use crate::undo::{u_clearallandblockfree, u_save, u_save_cursor, u_sync};
use crate::window::{goto_tabpage, may_trigger_win_scrolled_resized};
use crate::winlayer::graph::{CMDWIN_RESULT, CMDWIN_TYPE};

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
pub(crate) use self::state::*;
pub(crate) use self::tab::*;
pub(crate) use self::undo::*;

#[derive(Debug)]
pub(crate) struct InsertState {
    pub mincol: ::core::ffi::c_int,
    pub cmdchar: Key,
    pub cmdchar_todo: Key,
    pub ins_just_started: bool,
    pub startln: bool,
    pub count: ::core::ffi::c_int,
    pub c: Key,
    pub lastc: Key,
    pub i: ::core::ffi::c_int,
    pub did_backspace: bool,
    pub line_is_white: bool,
    pub old_topline: LineNr,
    pub old_topfill: ::core::ffi::c_int,
    pub inserted_space: bool,
    pub replace_state: ::core::ffi::c_int,
    pub nomove: bool,
}
pub(crate) const NL_STR: &::core::ffi::CStr = c"\n";
pub(crate) const ESC_STR: &::core::ffi::CStr = c"\x1B";
pub(crate) const CTRL_V_STR: &::core::ffi::CStr = c"\x16";
/// Whether `i_CTRL-G_U` is holding the undoable change open across the
/// key being handled — upstream's `DONT_SYNC_UNDO`.
///
/// A three-state latch, not a tri-state boolean: `Armed` is not "unknown",
/// it is "the *next* real key is the covered one".
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum KeepUndo {
    /// Normal: a cursor motion ends the undoable change.
    No,
    /// `CTRL-G U` was pressed; the next real key is the one it covers.
    Armed,
    /// The key being handled right now is the covered one.
    Now,
}

/// What the last `edit_putchar` did to the screen cell it wrote over, and so
/// how `edit_unputchar` has to take it back.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum PutChar {
    /// Nothing was put on the screen.
    Unset,
    /// The right half of a double-width character was overwritten.
    Right,
    /// The left half of a double-width character was overwritten.
    Left,
    /// A whole cell was overwritten and `PC_SCHAR`/`PC_ATTR` hold it.
    Set,
}
state_record! {
    /// Insert mode's state between keys: upstream's `edit.c` statics, and
    /// `ins_esc`'s function-scope one. Nothing holds a borrow of it across a
    /// call: an insert runs autocommands, mappings and Lua.
    pub(crate) struct EditState in EDIT as EditField;

    COMPL_BUSY: bool = false;
    INSSTART_TEXTLEN: ColNr = ColNr::ZERO;
    INSSTART_BLANK_VCOL: ColNr = ColNr::ZERO;
    UPDATE_INSSTART_ORIG: bool = true;
    /// The text of the last insert, the redo-buffer spelling, with the
    /// command that started it on the front -- see [`lastins`].
    LAST_INSERT: ApiString = ApiString::NULL;
    LAST_INSERT_SKIP: ::core::ffi::c_int = 0;
    NEW_INSERT_SKIP: ::core::ffi::c_int = 0;
    DID_RESTART_EDIT: Key = Key::NUL;
    CAN_CINDENT: bool = false;
    REVINS_ON: bool = false;
    REVINS_CHARS: ::core::ffi::c_int = 0;
    REVINS_LEGAL: ::core::ffi::c_int = 0;
    REVINS_SCOL: ::core::ffi::c_int = 0;
    INS_NEED_UNDO: bool = false;
    DONT_SYNC_UNDO: KeepUndo = KeepUndo::No;
    O_LNUM: LineNr = LineNr::ZERO;
    /// The Replace-mode stack of overwritten bytes -- see [`replace`].
    ///
    /// A `Vec`, not klib's `kvec_t(char)`: the growth policy was the only
    /// thing the kvec provided, every operation on it is an insert or a
    /// removal in the middle, and nothing outside [`replace`] may touch it.
    REPLACE_STACK: Vec<u8> = Vec::new();
    PC_STATUS: PutChar = PutChar::Unset;
    PC_SCHAR: ScreenChar = 0;
    PC_ATTR: ::core::ffi::c_int = 0;
    PC_ROW: ::core::ffi::c_int = 0;
    PC_COL: ::core::ffi::c_int = 0;
    /// `ins_esc` took one `REDRAWING_DISABLED` level it still has to give
    /// back.
    DISABLED_REDRAW: bool = false;
}
pub(crate) const INPUT_BUFLEN: ::core::ffi::c_int = 100 as ::core::ffi::c_int;

/// `insertchar()` flags — upstream's anonymous enum in `edit.h`.
///
/// A C `int`: `insertchar` and `internal_format` take the set as one.
pub(crate) type InscharFlags = ::core::ffi::c_int;

/// force formatting
pub(crate) const INSCHAR_FORMAT: InscharFlags = 1;
/// format comments
pub(crate) const INSCHAR_DO_COM: InscharFlags = 2;
/// the character was typed just after CTRL-V
pub(crate) const INSCHAR_CTRLV: InscharFlags = 4;
/// do not use `'formatexpr'`
pub(crate) const INSCHAR_NO_FEX: InscharFlags = 8;
/// format comments with a list or second-line indent
pub(crate) const INSCHAR_COM_LIST: InscharFlags = 16;
