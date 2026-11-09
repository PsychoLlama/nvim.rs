//! Changing the text of a buffer, and telling the rest of the editor about it.
//!
//! The parent keeps no functions: only the flag vocabulary the children
//! share, so `OPENLINE_*` and the `COM_*` letters of 'comments' are one
//! screen.  What each child answers:
//!
//! | file | question |
//! | --- | --- |
//! | `flag` | is the buffer modified, and was it modified *by the user* |
//! | `splice` | which lines moved, and who has to be told |
//! | `text` | insert and delete bytes, characters and lines |
//! | `open_line` | open a new line below or above, with its indent and leader |
//! | `leader` | how long is the 'comments' leader on this line |
//!
//! `open_line` is reentrant with `textformat.rs`: it calls `internal_format`
//! when 'textwidth' asks for a wrap, and `internal_format` calls it back to
//! break the line.  Its `did_do_comment` out-parameter is how the second
//! half of a broken line is stopped from starting a *new* comment leader.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::ascii::{TAB, ascii_iswhite};
use crate::autocmd::defs::AutoEvent;
use crate::autocmd::state::{AUTOCMD_BUSY, LAST_CURSORMOVED, LAST_CURSORMOVED_WIN};
use crate::buffer::{
    buf_inc_changedtick, buf_is_dontwrite, buf_is_prompt, buf_meta_total, current_buf,
};
use crate::buffer_updates::buf_updates_send_changes;
use crate::charset::getwhitecols_curline;
use crate::cursor::{check_cursor_lnum, check_visual_pos, coladvance_force, getviscol};
use crate::decoration::{MT_META_INLINE, MT_META_LINES};
use crate::diff::{diff_internal, diff_lnum_win, diff_update_line};
use crate::drawscreen::find_wl_entry;
use crate::drawscreen::state::{
    NEED_MAKETITLE, REDRAW_CMDLINE, REDRAW_NOT_ALLOWED, REDRAW_TABLINE, SEARCH_HL_HAS_CURSOR_LNUM,
};
use crate::drawscreen::{
    UPD_NOT_VALID, UPD_VALID, redraw_buf_status_later, redraw_win_line, set_must_redraw, showmode,
};
use crate::edit::{replace_push, replace_push_nul, truncate_spaces};
use crate::eval::vars::set_vim_var_string;
use crate::extmark::{BCount, CURBUF_SPLICE_PENDING};
use crate::extmark::{extmark_adjust, extmark_splice, extmark_splice_cols};
use crate::fold::fold_update;
use crate::highlight_group::HLF_W;
use crate::indent::{
    copy_indent, fixthisline, get_indent, get_sw_value, may_do_si, set_indent,
    use_indentexpr_for_lisp,
};
use crate::indent_c::{KEY_OPEN_BACK, KEY_OPEN_FORW};
use crate::indent_c::{do_c_expr_indent, in_cinkeys, starts_with_cinword};
use crate::insexpand::ins_compl_active;
use crate::mark::defs::FileMarkView;
use crate::mark::{mark_adjust, mark_col_adjust, mark_view_make};
use crate::mbyte::{GRAPHEME_STATE_INIT, GraphemeState};
use crate::mbyte::{mb_adjust_cursor, utf_iscomposing_first};
use crate::memline::ML_DEL_MESSAGE;
use crate::memline::{
    ml_add_deleted_len, ml_delete_flags, ml_line_alloced, ml_open_file, ml_replace_buf_text,
    ml_replace_owned, ml_setflags,
};
use crate::message::state::{
    EMSG_SILENT, IN_ASSERT_FAILS, MSG_COL, MSG_ROW, MSG_SCROLL, MSG_SILENT, NEED_WAIT_RETURN,
};
use crate::message::{
    msg_clr_eos, msg_delay, msg_end, msg_ext_set_kind, msg_source, msg_start, msg_str_hl,
    wait_return,
};
use crate::r#move::{
    approximate_botline_win, changed_cline_bef_curs, changed_line_abv_curs_win,
    invalidate_botline_win, set_topline, sms_marker_overlap,
};
use crate::option::get_ve_flags;
use crate::option::vars::{p_deco, p_paste, p_ri, p_sm, p_sr};
use crate::options::OPT_VE_FLAG_ONEMORE;
use crate::os::libc::int64_t;
use crate::plines::{linetabsize_eol, win_chartabsize};
use crate::pos::MAXLNUM;
use crate::search::state::HIGHLIGHT_MATCH;
use crate::search::{BACKWARD, FORWARD, check_linecomment, findmatch, linewhite, showmatch};
use crate::spell::spell_check_window;
use crate::startup::SILENT_MODE;
use crate::state::mode::{
    AI_COL, CAN_SI, CAN_SI_BACK, DID_AI, DID_SI, END_COMMENT_PENDING, INSSTART, ORIG_LINE_COUNT,
    RESTART_EDIT, STATE, VR_LINES_CHANGED,
};
use crate::state::{MODE_INSERT, REPLACE_FLAG, VREPLACE_FLAG, virtual_active};
use crate::strings::concat;
use crate::textformat::{comp_textwidth, has_format_option};
use crate::types::{CmdModFlags, ColNr, LineNr, Pos, Vv};
use crate::ui::state::{ROWS, UI_MESSAGES};
use crate::ui::{ui_active, ui_has};
use crate::undo::{curbuf_is_changed, u_clearline, u_save_cursor, u_savedel};

// The carve of the transpiled module; see each child's docs.
mod flag;
mod leader;
mod open_line;
mod splice;
mod text;

pub use self::flag::*;
pub use self::leader::*;
pub use self::open_line::*;
pub use self::splice::*;
pub use self::text::*;

/// `open_line()`'s `flags` argument -- upstream's anonymous enum in
/// `change.h`.  Every one is passed in a `c_int` parameter, so `c_int` is
/// the type that avoids a cast at each use.
pub const OPENLINE_FORCE_INDENT: ::core::ffi::c_int = 64;
/// Format the new line with 'textwidth'.
pub const OPENLINE_FORMAT: ::core::ffi::c_int = 32;
/// `second_indent` is a comment-leader length, not an indent.
pub const OPENLINE_COM_LIST: ::core::ffi::c_int = 16;
/// Move marks that were on the old line to the new one.
pub const OPENLINE_MARKFIX: ::core::ffi::c_int = 8;
/// Keep trailing white space on the old line.
pub const OPENLINE_KEEPTRAIL: ::core::ffi::c_int = 4;
/// Copy the 'comments' leader onto the new line.
pub const OPENLINE_DO_COM: ::core::ffi::c_int = 2;
/// Delete the white space the split left behind.
pub const OPENLINE_DELSPACES: ::core::ffi::c_int = 1;

pub const COM_NEST: u8 = b'n';
pub const COM_BLANK: u8 = b'b';
pub const COM_START: u8 = b's';
pub const COM_MIDDLE: u8 = b'm';
pub const COM_END: u8 = b'e';
pub const COM_AUTO_END: u8 = b'x';
pub const COM_FIRST: u8 = b'f';
pub const COM_LEFT: u8 = b'l';
pub const COM_RIGHT: u8 = b'r';
pub const COM_NOBACK: u8 = b'O';
pub const COM_MAX_LEN: usize = 50;
