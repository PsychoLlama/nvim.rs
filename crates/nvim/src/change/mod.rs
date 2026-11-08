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

use crate::ascii::TAB;
use crate::ascii::ascii_iswhite;
use crate::autocmd::defs::AutoEvent;
use crate::autocmd::state::{autocmd_busy, last_cursormoved, last_cursormoved_win};
use crate::buffer::{
    buf_inc_changedtick, buf_is_dontwrite, buf_is_prompt, buf_meta_total, current_buf,
};
use crate::buffer_updates::buf_updates_send_changes;
use crate::charset::getwhitecols_curline;
use crate::cursor::{check_cursor_lnum, check_visual_pos, coladvance_force, getviscol};
use crate::decoration::{kMTMetaInline, kMTMetaLines};
use crate::diff::{diff_internal, diff_lnum_win, diff_update_line};
use crate::drawscreen::find_wl_entry;
use crate::drawscreen::state::{
    need_maketitle, redraw_cmdline, redraw_not_allowed, redraw_tabline, search_hl_has_cursor_lnum,
};
use crate::drawscreen::{
    UPD_NOT_VALID, UPD_VALID, redraw_buf_status_later, redraw_win_line, set_must_redraw, showmode,
};
use crate::edit::{replace_push, replace_push_nul, truncate_spaces};
use crate::eval::vars::set_vim_var_string;
use crate::extmark::BCount;
use crate::extmark::curbuf_splice_pending;
use crate::extmark::{extmark_adjust, extmark_splice, extmark_splice_cols};
use crate::fold::{fold_update, has_folding_win};
use crate::highlight_group::HLF_W;
use crate::indent::{
    copy_indent, fixthisline, get_indent, get_sw_value, may_do_si, set_indent,
    use_indentexpr_for_lisp,
};
use crate::indent_c::{do_c_expr_indent, in_cinkeys, starts_with_cinword};
use crate::insexpand::ins_compl_active;
use crate::mark::defs::FileMarkView;
use crate::mark::{fresh_mark, mark_adjust, mark_col_adjust, mark_view_make};
use crate::mbyte::GraphemeState;
use crate::mbyte::{mb_adjust_cursor, utf_iscomposing_first};
use crate::memline::{
    ml_add_deleted_len, ml_delete_flags, ml_line_alloced, ml_open_file, ml_replace_buf_text,
    ml_replace_owned, ml_setflags,
};
use crate::message::state::{
    emsg_silent, in_assert_fails, msg_col, msg_row, msg_scroll, msg_silent, need_wait_return,
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
use crate::options::kOptVeFlagOnemore;
use crate::os::libc::int64_t;
use crate::plines::{linetabsize_eol, win_chartabsize};
use crate::pos::MAXLNUM;
use crate::search::state::highlight_match;
use crate::search::{BACKWARD, FORWARD, check_linecomment, findmatch, linewhite, showmatch};
use crate::spell::spell_check_window;
use crate::startup::silent_mode;
use crate::state::mode::{
    Insstart, State, ai_col, can_si, can_si_back, did_ai, did_si, end_comment_pending,
    orig_line_count, restart_edit, vr_lines_changed,
};
use crate::state::{MODE_INSERT, REPLACE_FLAG, VREPLACE_FLAG, virtual_active};
use crate::strings::concat;
use crate::textformat::{comp_textwidth, has_format_option};
use crate::types::{CmdModFlags, ColNr, LineNr, Pos, Vv};
use crate::ui::state::Rows;
use crate::ui::state::kUIMessages;
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

/// The pseudo-keys `in_cinkeys()` matches `o`/`O` against, compared to a
/// `c_int` `keytyped`.
pub const KEY_OPEN_BACK: ::core::ffi::c_int = 258;
pub const KEY_OPEN_FORW: ::core::ffi::c_int = 257;

/// `ml_delete_flags()`: report "N fewer lines".
pub const ML_DEL_MESSAGE: ::core::ffi::c_int = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
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
pub const GRAPHEME_STATE_INIT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
