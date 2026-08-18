//! The operators: what `d`, `y`, `c`, `<`, `>`, `J`, `g~`, `r`, `I`, `A`,
//! `g@`, `!` and CTRL-A do to a region of text.
//!
//! `ops/pending.rs` is the way in: normal mode reads an operator and a motion,
//! and `do_pending_operator` turns the pair into a region and calls one of the
//! others. `ops/optype.rs` is the vocabulary that maps the keys to an `OP_*`,
//! and `ops/block.rs` the geometry three of the operators share.
//!
//! This file holds no code -- only the constants c2rust copied in from the
//! headers, and the `redo_VIsual_T` that `pending.rs` keeps one of.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::ascii::{
    ascii_isalpha, ascii_isbdigit, ascii_isdigit, ascii_isspace, ascii_iswhite, ascii_isxdigit,
};
use crate::buffer::{buf_get_changedtick, col_print};
use crate::change::{
    appended_lines_mark, changed_bytes, changed_lines, del_bytes, del_char, del_lines,
    get_last_leader_offset, get_leader_len, ins_char, ins_str, truncate_line,
};
use crate::charset::{getwhitecols, getwhitecols_curline, skipwhite, vim_str2nr};
use crate::cursor::{
    check_cursor, check_cursor_col, check_pos, coladvance, coladvance_force, dec_cursor,
    gchar_cursor, get_cursor_line_len, get_cursor_line_ptr, get_cursor_pos_ptr, getviscol,
    getviscol2, getvpos, inc_cursor,
};
use crate::drawscreen::{UPD_INVERTED, redraw_curbuf_later, update_screen};
use crate::edit::{beginline, display_dollar, edit};
use crate::eval::typval::{kCallbackNone, tv_clear, tv_dict_add_nr};
use crate::eval::{callback_call, set_ref_in_callback};
use crate::extmark::{extmark_splice, extmark_splice_cols};
use crate::fold::{deleteFold, foldCreate, foldOpenCursor, hasFolding, opFoldRange};
use crate::getchar::{
    AppendNumberToRedobuff, AppendToRedobuff, AppendToRedobuffLit, AppendToRedobuffSpec,
    CancelRedo, ResetRedobuff, beep_flush, stuffReadbuff, stuffcharReadbuff, stuffnumReadbuff,
};
use crate::global_cell::GlobalCell;
use crate::indent::{
    change_indent, fix_indent, get_expr_indent, get_indent, get_lisp_indent, get_sw_value_indent,
    inindent, may_do_si, op_reindent, preprocs_left, set_indent, tabstop_fromto,
    use_indentexpr_for_lisp,
};
use crate::indent_c::get_c_indent;
use crate::keycodes::Ctrl_V;
use crate::main::{
    IObuff, Insstart, KeyTyped, State, VIsual, VIsual_active, VIsual_mode, VIsual_reselect,
    VIsual_select, VIsual_select_reg, ai_col, bangredo, can_si, cmdmod, curbuf,
    curbuf_splice_pending, curwin, did_ai, disable_fold_update, e_invarg, e_modifiable,
    empty_string_option, finish_op, got_int, motion_force, mouse_dragging, msg_scroll,
    no_lines_msg, p_ch, p_cpo, p_fp, p_js, p_opfunc, p_report, p_ri, p_sbr, p_sel, p_shm, p_sol,
    p_sr, redo_VIsual_busy, repeat_cmdline, repeat_luaref, resel_VIsual_line_count,
    resel_VIsual_mode, resel_VIsual_vcol, restart_edit, virtual_op,
};
use crate::mark::{mark_col_adjust, mark_mb_adjustpos};
use crate::mbyte::{
    bomb_size, mb_islower, mb_isupper, mb_tolower, mb_toupper, utf_char2bytes, utf_char2cells,
    utf_char2len, utf_eat_space, utf_head_off, utf_ptr2StrCharInfo, utf_ptr2char, utf_ptr2len,
    utfc_next, utfc_ptr2len,
};
use crate::memline::{
    dec, decl, gchar_pos, inc, ml_append, ml_get, ml_get_buf_len, ml_get_buf_mut, ml_get_len,
    ml_get_pos, ml_get_pos_len, ml_replace, ml_replace_len,
};
use crate::memory::{xcalloc, xfree, xmalloc, xmallocz, xmemcpyz, xmemdupz};
use crate::message::{emsg, msg, msg_keep, msg_start, msgmore};
use crate::mouse::setmouse;
use crate::r#move::validate_virtcol;
use crate::normal::{
    clearop, clearopbeep, may_clear_cmdline, prep_redo, prep_redo_num2, restore_visual_mode,
    unadjust_for_sel,
};
use crate::option::{get_equalprg, get_fileformat, get_ve_flags, option_set_callback_func};
use crate::options::{kOptBoFlagOperator, kOptVeFlagAll, kOptVeFlagOnemore};
use crate::os::cshim::{__ctype_b_loc, gettext, memmove, ngettext};
use crate::os::input::{line_breakcheck, os_breakcheck};
use crate::plines::{
    getvcol, getvcols, getvvcol, init_charsize_arg, linetabsize_str, win_charsize,
};
use crate::pos::{MAXCOL, equalpos, lt, ltoreq};
use crate::register::{
    do_autocmd_textyankpost, get_y_register, get_yank_register, op_yank, op_yank_reg,
    shift_delete_registers, valid_yank_reg,
};
use crate::state::{MODE_INSERT, MODE_REPLACE, VREPLACE_FLAG, virtual_active};
use crate::strings::{vim_snprintf, vim_strchr};
use crate::textformat::{auto_format, has_format_option, op_format, op_formatexpr};
use crate::types::{
    CMOD_LOCKMARKS, Callback, Callback_data as C2Rust_Unnamed_5, CharsizeArg, ExtmarkOp,
    MotionType, OP_APPEND, OP_CHANGE, OP_COLON, OP_DELETE, OP_FILTER, OP_FOLD, OP_FOLDCLOSE,
    OP_FOLDCLOSEREC, OP_FOLDDEL, OP_FOLDDELREC, OP_FOLDOPEN, OP_FOLDOPENREC, OP_FORMAT, OP_FORMAT2,
    OP_FUNCTION, OP_INDENT, OP_INSERT, OP_JOIN, OP_JOIN_NS, OP_LOWER, OP_LSHIFT, OP_NOP, OP_NR_ADD,
    OP_NR_SUB, OP_REPLACE, OP_ROT13, OP_RSHIFT, OP_TILDE, OP_UPPER, OP_YANK, OpType, OptInt,
    StrCharInfo, TriState, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, bcount_t, block_def, buf_T,
    cmdarg_T, colnr_T, dict_T, int32_t, int64_t, kNone, linenr_T, oparg_T, optset_T, pos_T, size_t,
    ssize_t, typval_T, typval_vval_union, uvarnumber_T, varnumber_T, yankreg_T,
};
use crate::ui::vim_beep;
use crate::undo::{u_clearline, u_save, u_save_cursor};
use ::libc::{abort, memset, strcpy, strlen};

// The carve of the transpiled module; see each child's docs.
mod addsub;
mod block;
mod case;
mod count;
mod delete;
mod external;
mod insert;
mod join;
mod optype;
mod pending;
mod replace;
mod shift;

pub use self::addsub::*;
pub use self::block::*;
pub use self::case::*;
pub use self::count::*;
pub use self::delete::*;
pub use self::external::*;
pub use self::insert::*;
pub use self::join::*;
pub use self::optype::*;
pub use self::pending::*;
pub(crate) use self::replace::*;
pub use self::shift::*;

/// `_ISalpha` and `_ISupper` from the C library's `__ctype_b_loc` table.
///
/// `isalpha`/`isupper` are locale-dependent, which is why `do_addsub` uses
/// them rather than `ascii_isalpha`: the case of a hex digit follows the
/// user's locale.
pub const _ISalpha: ::core::ffi::c_ushort = 1024;
pub const _ISupper: ::core::ffi::c_ushort = 256;

/// Enough for any number this module formats, plus its NUL.
pub const NUMBUFLEN: ::core::ffi::c_int = 65;

/// `ExtmarkOp`: the edit is undoable, so extmarks move with it.
pub const kExtmarkUndo: ExtmarkOp = 1;

/// `vim_str2nr` flags: which bases 'nrformats' allows.
pub const STR2NR_BIN: ::core::ffi::c_int = 1;
pub const STR2NR_OCT: ::core::ffi::c_int = 2;
pub const STR2NR_HEX: ::core::ffi::c_int = 4;

/// The three region shapes an operator can be given.
pub const kMTCharWise: MotionType = 0;
pub const kMTLineWise: MotionType = 1;
pub const kMTBlockWise: MotionType = 2;

/// `cmdarg_T::retval`: normal mode must not act on what the operator left.
pub const CA_COMMAND_BUSY: ::core::ffi::c_int = 1;
/// `cmdarg_T::retval`: leave `oap->end` where the motion put it.
pub const CA_NO_ADJ_OP_END: ::core::ffi::c_int = 2;

/// `r CTRL-V <CR>` and `r CTRL-V <NL>`: the literal byte, not a line split.
pub const REPLACE_CR_NCHAR: ::core::ffi::c_int = -1;
pub const REPLACE_NL_NCHAR: ::core::ffi::c_int = -2;

/// `get_yank_register` mode: the register is about to be written.
pub const YREG_YANK: ::core::ffi::c_int = 1;

/// `change_indent`: set the indent to the given column.
pub const INDENT_SET: ::core::ffi::c_int = 1;

/// `beginline` flags: to the first non-white, to the start of the line, and
/// "fix the column even in Visual mode".
pub const BL_WHITE: ::core::ffi::c_int = 1;
pub const BL_SOL: ::core::ffi::c_int = 2;
pub const BL_FIX: ::core::ffi::c_int = 4;

/// `set_indent`: report the change through `changed_bytes`.
pub const SIN_CHANGED: ::core::ffi::c_int = 1;

/// The Visual area a `.` replays -- see `pending::REDO_VISUAL`.
#[derive(Copy, Clone)]
pub struct redo_VIsual_T {
    /// `v`, `V` or CTRL-V.
    pub rv_mode: ::core::ffi::c_int,
    /// Number of lines.
    pub rv_line_count: linenr_T,
    /// Number of columns, or the end column.
    pub rv_vcol: colnr_T,
    /// Count typed before the Visual operator.
    pub rv_count: ::core::ffi::c_int,
    /// Extra argument; `g CTRL-A` is the only user.
    pub rv_arg: ::core::ffi::c_int,
}

/// `w_valid` bits invalidated whenever a column is measured differently.
pub const VALID_WROW: ::core::ffi::c_int = 0x1;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2;
pub const VALID_VIRTCOL: ::core::ffi::c_int = 0x4;

/// `b_ml.ml_flags`: the buffer is one empty line.
pub const ML_EMPTY: ::core::ffi::c_int = 0x1;

pub const OK: ::core::ffi::c_int = 1;
pub const FAIL: ::core::ffi::c_int = 0;

/// `get_fileformat`: lines end with CR LF, so a line break is two bytes.
pub const EOL_DOS: ::core::ffi::c_int = 1;

/// 'formatoptions' letters this module reads.
pub const FO_MBYTE_JOIN: ::core::ffi::c_int = 'M' as ::core::ffi::c_int;
pub const FO_MBYTE_JOIN2: ::core::ffi::c_int = 'B' as ::core::ffi::c_int;
pub const FO_AUTO: ::core::ffi::c_int = 'a' as ::core::ffi::c_int;
pub const FO_REMOVE_COMS: ::core::ffi::c_int = 'j' as ::core::ffi::c_int;

/// 'cpoptions' letters this module reads.
pub const CPO_EMPTYREGION: ::core::ffi::c_int = 'E' as ::core::ffi::c_int;
pub const CPO_JOINCOL: ::core::ffi::c_int = 'q' as ::core::ffi::c_int;
pub const CPO_REDO: ::core::ffi::c_int = 'r' as ::core::ffi::c_int;
pub const CPO_YANK: ::core::ffi::c_int = 'y' as ::core::ffi::c_int;
pub const CPO_DOLLAR: ::core::ffi::c_int = '$' as ::core::ffi::c_int;
pub const CPO_FILTER: ::core::ffi::c_int = '!' as ::core::ffi::c_int;

/// 'comments' flag: this leader ends a three-part comment.
pub const COM_END: ::core::ffi::c_int = 'e' as ::core::ffi::c_int;

pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;

/// Size of `IObuff`, the shared message buffer.
pub const IOSIZE: ::core::ffi::c_int = 1024 + 1;

pub const true_0: ::core::ffi::c_int = 1;
pub const false_0: ::core::ffi::c_int = 0;
