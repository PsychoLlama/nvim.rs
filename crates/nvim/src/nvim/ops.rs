#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::ascii::{
    ascii_isbdigit, ascii_isdigit, ascii_isspace, ascii_iswhite, ascii_isxdigit,
};
use crate::src::nvim::buffer::buf_get_changedtick;
use crate::src::nvim::buffer::col_print;
use crate::src::nvim::change::{
    appended_lines_mark, changed_bytes, changed_lines, del_bytes, del_char, del_lines,
    get_last_leader_offset, get_leader_len, ins_char, ins_str, truncate_line,
};
use crate::src::nvim::charset::{getwhitecols, getwhitecols_curline, skipwhite, vim_str2nr};
use crate::src::nvim::cursor::{
    check_cursor, check_cursor_col, check_pos, coladvance, coladvance_force, dec_cursor,
    gchar_cursor, get_cursor_line_len, get_cursor_line_ptr, get_cursor_pos_ptr, getviscol,
    getviscol2, getvpos, inc_cursor,
};
use crate::src::nvim::drawscreen::{UPD_INVERTED, redraw_curbuf_later, update_screen};
use crate::src::nvim::edit::{beginline, display_dollar, edit};
use crate::src::nvim::eval::typval::{kCallbackNone, tv_clear, tv_dict_add_nr};
use crate::src::nvim::eval::{callback_call, set_ref_in_callback};
use crate::src::nvim::extmark::{extmark_splice, extmark_splice_cols};
use crate::src::nvim::fold::{deleteFold, foldCreate, foldOpenCursor, hasFolding, opFoldRange};
use crate::src::nvim::getchar::{
    AppendNumberToRedobuff, AppendToRedobuff, AppendToRedobuffLit, AppendToRedobuffSpec,
    CancelRedo, ResetRedobuff, beep_flush, stuffReadbuff, stuffcharReadbuff, stuffnumReadbuff,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::indent::{
    change_indent, fix_indent, get_expr_indent, get_indent, get_lisp_indent, get_sw_value_indent,
    inindent, may_do_si, op_reindent, preprocs_left, set_indent, tabstop_fromto,
    use_indentexpr_for_lisp,
};
use crate::src::nvim::indent_c::get_c_indent;
use crate::src::nvim::keycodes::{Ctrl_V, KE_COMMAND, KE_LUA};
use crate::src::nvim::main::{
    IObuff, Insstart, KeyTyped, State, VIsual, VIsual_active, VIsual_mode, VIsual_reselect,
    VIsual_select, VIsual_select_reg, ai_col, bangredo, can_si, cmdmod, curbuf,
    curbuf_splice_pending, curwin, did_ai, disable_fold_update, e_invarg, e_modifiable,
    empty_string_option, finish_op, got_int, motion_force, mouse_dragging, msg_scroll,
    no_lines_msg, p_ch, p_cpo, p_fp, p_js, p_opfunc, p_report, p_ri, p_sbr, p_sel, p_shm, p_sol,
    p_sr, redo_VIsual_busy, repeat_cmdline, repeat_luaref, resel_VIsual_line_count,
    resel_VIsual_mode, resel_VIsual_vcol, restart_edit, virtual_op,
};
use crate::src::nvim::mark::{mark_col_adjust, mark_mb_adjustpos};
use crate::src::nvim::mbyte::{
    bomb_size, mb_islower, mb_isupper, mb_tolower, mb_toupper, utf_char2bytes, utf_char2cells,
    utf_char2len, utf_eat_space, utf_head_off, utf_ptr2StrCharInfo, utf_ptr2char, utf_ptr2len,
    utfc_next, utfc_ptr2len,
};
use crate::src::nvim::memline::{
    dec, decl, gchar_pos, inc, ml_append, ml_get, ml_get_buf_len, ml_get_buf_mut, ml_get_len,
    ml_get_pos, ml_get_pos_len, ml_replace, ml_replace_len,
};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xmallocz, xmemcpyz, xmemdupz};
use crate::src::nvim::message::{emsg, msg, msg_keep, msg_start, msgmore, smsg};
use crate::src::nvim::mouse::setmouse;
use crate::src::nvim::r#move::validate_virtcol;
use crate::src::nvim::normal::{
    clearop, clearopbeep, may_clear_cmdline, prep_redo, prep_redo_num2, restore_visual_mode,
    unadjust_for_sel,
};
use crate::src::nvim::option::{
    get_equalprg, get_fileformat, get_ve_flags, option_set_callback_func,
};
use crate::src::nvim::options::{kOptBoFlagOperator, kOptVeFlagAll, kOptVeFlagOnemore};
use crate::src::nvim::os::input::{line_breakcheck, os_breakcheck};
use crate::src::nvim::os::libc::{
    __ctype_b_loc, abort, gettext, memmove, memset, ngettext, strcpy, strlen,
};
use crate::src::nvim::plines::linetabsize_str;
use crate::src::nvim::plines::{getvcol, getvcols, getvvcol, init_charsize_arg, win_charsize};
use crate::src::nvim::pos::{MAXCOL, equalpos, lt, ltoreq};
use crate::src::nvim::register::{
    do_autocmd_textyankpost, get_y_register, get_yank_register, op_yank, op_yank_reg,
    shift_delete_registers, valid_yank_reg,
};
use crate::src::nvim::state::{MODE_INSERT, MODE_REPLACE, VREPLACE_FLAG, virtual_active};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr};
use crate::src::nvim::textformat::{auto_format, has_format_option, op_format, op_formatexpr};
use crate::src::nvim::types::{
    CMOD_LOCKMARKS, Callback, Callback_data as C2Rust_Unnamed_5, CharsizeArg, ExtmarkOp,
    MotionType, OP_APPEND, OP_CHANGE, OP_COLON, OP_DELETE, OP_FILTER, OP_FOLD, OP_FOLDCLOSE,
    OP_FOLDCLOSEREC, OP_FOLDDEL, OP_FOLDDELREC, OP_FOLDOPEN, OP_FOLDOPENREC, OP_FORMAT,
    OP_FUNCTION, OP_INDENT, OP_INSERT, OP_JOIN, OP_LOWER, OP_LSHIFT, OP_NOP, OP_NR_SUB, OP_REPLACE,
    OP_ROT13, OP_RSHIFT, OP_UPPER, OP_YANK, OpType, OptInt, StrCharInfo, TriState, VAR_STRING,
    VAR_UNKNOWN, VAR_UNLOCKED, bcount_t, block_def, buf_T, cmdarg_T, colnr_T, dict_T, int32_t,
    int64_t, kNone, linenr_T, oparg_T, optset_T, pos_T, size_t, ssize_t, typval_T,
    typval_vval_union, uint8_t, uvarnumber_T, varnumber_T, yankreg_T,
};
use crate::src::nvim::ui::vim_beep;
use crate::src::nvim::undo::{u_clearline, u_save, u_save_cursor};

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

pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISalpha: C2Rust_Unnamed = 1024;
pub const _ISupper: C2Rust_Unnamed = 256;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const NUMBUFLEN: C2Rust_Unnamed_16 = 65;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const STR2NR_HEX: C2Rust_Unnamed_20 = 4;
pub const STR2NR_OCT: C2Rust_Unnamed_20 = 2;
pub const STR2NR_BIN: C2Rust_Unnamed_20 = 1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const CA_NO_ADJ_OP_END: C2Rust_Unnamed_21 = 2;
pub const CA_COMMAND_BUSY: C2Rust_Unnamed_21 = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_int;
pub const REPLACE_NL_NCHAR: C2Rust_Unnamed_22 = -2;
pub const REPLACE_CR_NCHAR: C2Rust_Unnamed_22 = -1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const YREG_YANK: C2Rust_Unnamed_23 = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const INDENT_SET: C2Rust_Unnamed_25 = 1;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_26 = 4;
pub const BL_SOL: C2Rust_Unnamed_26 = 2;
pub const BL_WHITE: C2Rust_Unnamed_26 = 1;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const SIN_CHANGED: C2Rust_Unnamed_28 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct redo_VIsual_T {
    pub rv_mode: ::core::ffi::c_int,
    pub rv_line_count: linenr_T,
    pub rv_vcol: colnr_T,
    pub rv_count: ::core::ffi::c_int,
    pub rv_arg: ::core::ffi::c_int,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const VALID_WROW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VALID_VIRTCOL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EOL_DOS: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FO_MBYTE_JOIN: ::core::ffi::c_int = 'M' as ::core::ffi::c_int;
pub const FO_MBYTE_JOIN2: ::core::ffi::c_int = 'B' as ::core::ffi::c_int;
pub const FO_AUTO: ::core::ffi::c_int = 'a' as ::core::ffi::c_int;
pub const FO_REMOVE_COMS: ::core::ffi::c_int = 'j' as ::core::ffi::c_int;
pub const CPO_EMPTYREGION: ::core::ffi::c_int = 'E' as ::core::ffi::c_int;
pub const CPO_JOINCOL: ::core::ffi::c_int = 'q' as ::core::ffi::c_int;
pub const CPO_REDO: ::core::ffi::c_int = 'r' as ::core::ffi::c_int;
pub const CPO_YANK: ::core::ffi::c_int = 'y' as ::core::ffi::c_int;
pub const CPO_DOLLAR: ::core::ffi::c_int = '$' as ::core::ffi::c_int;
pub const CPO_FILTER: ::core::ffi::c_int = '!' as ::core::ffi::c_int;
pub const COM_END: ::core::ffi::c_int = 'e' as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const NL_STR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"\n\0") };
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const OPF_LINES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const OPF_CHANGE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
