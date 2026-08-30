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
#![deny(unsafe_op_in_unsafe_fn)]

use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::autocmd::{EVENT_FILECHANGEDRO, apply_autocmds};
use crate::buffer::{
    buf_inc_changedtick, buf_is_dontwrite, buf_is_prompt, buf_meta_total, current_buf,
};
use crate::buffer_updates::buf_updates_send_changes;
use crate::charset::{getdigits_int, getwhitecols_curline, ptr2cells, skipwhite, vim_strnsize};
use crate::cursor::{
    check_cursor_lnum, check_visual_pos, coladvance_force, get_cursor_line_len,
    get_cursor_line_ptr, get_cursor_pos_ptr, getviscol,
};
use crate::decoration::{kMTMetaInline, kMTMetaLines};
use crate::diff::{diff_internal, diff_lnum_win, diff_update_line};
use crate::drawscreen::{
    UPD_NOT_VALID, UPD_VALID, redraw_buf_status_later, redraw_win_line, set_must_redraw, showmode,
};
use crate::edit::{prompt_text, replace_push, replace_push_nul, truncate_spaces};
use crate::eval::vars::set_vim_var_string;
use crate::extmark::{extmark_adjust, extmark_splice, extmark_splice_cols};
use crate::fold::{find_wl_entry, fold_update, has_folding_win};
use crate::highlight_group::HLF_W;
use crate::indent::{
    copy_indent, fixthisline, get_indent, get_lisp_indent, get_sw_value, indent_size_ts, may_do_si,
    set_indent, use_indentexpr_for_lisp,
};
use crate::indent_c::{cin_is_cinword, do_c_expr_indent, in_cinkeys};
use crate::insexpand::ins_compl_active;
use crate::main::{
    Insstart, Rows, State, ai_col, autocmd_busy, can_si, can_si_back, curbuf,
    curbuf_splice_pending, curwin, did_ai, did_si, emsg_silent, end_comment_pending,
    highlight_match, in_assert_fails, last_cursormoved, last_cursormoved_win, msg_col, msg_row,
    msg_scroll, msg_silent, need_maketitle, need_wait_return, orig_line_count, p_deco, p_paste,
    p_ri, p_sm, p_sr, redraw_cmdline, redraw_not_allowed, redraw_tabline, restart_edit,
    search_hl_has_cursor_lnum, silent_mode, vr_lines_changed,
};
use crate::mark::{free_fmark, mark_adjust, mark_col_adjust, mark_view_make};
use crate::mbyte::{
    mb_adjust_cursor, utf_char2bytes, utf_composinglike, utf_head_off, utf_iscomposing_first,
    utf_ptr2char, utf_ptr2len, utfc_ptr2len, utfc_ptr2len_len,
};
use crate::memline::{
    ml_add_deleted_len, ml_append, ml_delete_flags, ml_get, ml_get_buf, ml_get_len,
    ml_line_alloced, ml_open_file, ml_replace, ml_setflags,
};
use crate::memory::{xfree, xmalloc, xmallocz, xmemcpyz, xstrdup};
use crate::message::{
    msg_clr_eos, msg_delay, msg_end, msg_ext_set_kind, msg_puts_hl, msg_source, msg_start,
    wait_return,
};
use crate::r#move::{
    approximate_botline_win, changed_cline_bef_curs, changed_line_abv_curs_win,
    invalidate_botline_win, set_topline, sms_marker_overlap,
};
use crate::option::{copy_option_part, get_ve_flags};
use crate::options::kOptVeFlagOnemore;
use crate::os::time::os_time;
use crate::plines::{getvcol, linetabsize_eol, win_chartabsize};
use crate::pos::{MAXCOL, MAXLNUM};
use crate::search::{BACKWARD, FORWARD, check_linecomment, findmatch, linewhite, showmatch};
use crate::spell::spell_check_window;
use crate::state::{MODE_INSERT, REPLACE_FLAG, VREPLACE_FLAG, virtual_active};
use crate::strings::{concat_str, vim_strchr, xstrnsave};
use crate::textformat::{comp_textwidth, has_format_option};
use crate::types::ui::kUIMessages;
use crate::types::{
    CmdModFlags, ExtmarkOp, GraphemeState, Vv, bcount_t, buf_T, colnr_T, fmark_T, fmarkv_T,
    int64_t, linenr_T, pos_T, size_t, ssize_t,
};
use crate::ui::{ui_active, ui_has};
use crate::undo::{curbuf_is_changed, u_clearline, u_save_cursor, u_savedel};
use ::libc::strcat;

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

pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;

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

/// `set_indent()` flags: do not move the marks, and this indent is being
/// set from Insert mode.
pub const SIN_NOMARK: ::core::ffi::c_int = 8;
pub const SIN_INSERT: ::core::ffi::c_int = 2;

/// `ml_delete_flags()`: report "N fewer lines".
pub const ML_DEL_MESSAGE: ::core::ffi::c_int = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const JUMPLISTSIZE: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
pub const COM_NEST: ::core::ffi::c_int = 'n' as ::core::ffi::c_int;
pub const COM_BLANK: ::core::ffi::c_int = 'b' as ::core::ffi::c_int;
pub const COM_START: ::core::ffi::c_int = 's' as ::core::ffi::c_int;
pub const COM_MIDDLE: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const COM_END: ::core::ffi::c_int = 'e' as ::core::ffi::c_int;
pub const COM_AUTO_END: ::core::ffi::c_int = 'x' as ::core::ffi::c_int;
pub const COM_FIRST: ::core::ffi::c_int = 'f' as ::core::ffi::c_int;
pub const COM_LEFT: ::core::ffi::c_int = 'l' as ::core::ffi::c_int;
pub const COM_RIGHT: ::core::ffi::c_int = 'r' as ::core::ffi::c_int;
pub const COM_NOBACK: ::core::ffi::c_int = 'O' as ::core::ffi::c_int;
pub const COM_MAX_LEN: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
pub const GRAPHEME_STATE_INIT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
