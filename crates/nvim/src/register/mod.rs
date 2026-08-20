//! The yank registers: what is in them, and every way text gets in or out.
//!
//! The parent keeps no functions, only the process-wide state the children
//! share -- which is the point of it being one screen:
//!
//! | cell | written by |
//! | --- | --- |
//! | `y_regs` | the 39 register slots; `store`, `yank` and `contents` |
//! | `y_previous` | the last register written; `store` and `contents` |
//! | `expr_line` | the `"=` source text, kept for a repeat; `special` |
//! | `execreg_lastc` | the last `@` register, so that `@@` repeats it; `exec` |
//!
//! What each child answers:
//!
//! | file | question |
//! | --- | --- |
//! | `store` | which slot is this register, and may it be read or written |
//! | `special` | the registers whose contents are computed (`"=` `".` `"%` `":` `"/`) |
//! | `exec` | recording with `q`, replaying with `@`, CTRL-R |
//! | `yank` | copying the operator's region into a register |
//! | `put` | `p` `P` `gp` `gP` `]p` `[p` `zp` |
//! | `display` | `:registers` |
//! | `contents` | a register as text: `getreg()`, `setreg()`, the clipboard, shada |
#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::private::helpers::{cbuf_to_string, copy_string, cstr_to_string};
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::autocmd::{
    EVENT_RECORDINGENTER, EVENT_RECORDINGLEAVE, EVENT_TEXTYANKPOST, apply_autocmds, has_event,
};
use crate::buffer::{buf_is_empty, buflist_findnr, buflist_findpat, buflist_name_nr, getaltfname};
use crate::buffer_updates::buf_updates_send_changes;
use crate::change::{changed_bytes, changed_lines, del_chars};
use crate::charset::{getdigits_int, ptr2cells, skipwhite, transchar};
use crate::clipboard;
use crate::cursor::{
    coladvance_force, gchar_cursor, get_cursor_line_len, get_cursor_line_ptr, get_cursor_pos_len,
    get_cursor_pos_ptr, getviscol, getvpos,
};
use crate::drawscreen::{showmode, update_screen};
use crate::edit::{beginline, get_last_insert, get_last_insert_save, oneright, stuff_inserted};
use crate::eval::typval::{
    tv_dict_add_bool, tv_dict_add_list, tv_dict_add_str, tv_dict_set_keys_readonly, tv_list_alloc,
    tv_list_append_allocated_string, tv_list_append_string, tv_list_set_lock,
};
use crate::eval::{eval_to_string, get_v_event, restore_v_event};
use crate::ex_cmds2::check_fname;
use crate::ex_getln::{cmdline_paste_str, getcmdline};
use crate::extmark::{extmark_splice, extmark_splice_cols};
use crate::file_search::file_name_at_cursor;
use crate::fold::hasFolding;
use crate::garray::{ga_append, ga_clear, ga_concat_len, ga_init, ga_set_growsize};
use crate::getchar::{
    AppendCharToRedobuff, beep_flush, get_recorded, ins_typebuf, stuff_readbuf, stuff_readbuf_char,
    stuffescaped,
};
use crate::global_cell::GlobalCell;
use crate::highlight_group::HLF_8;
use crate::indent::{get_indent, preprocs_left, set_indent, tabstop_padding};
use crate::insexpand::{ins_compl_delete, ins_compl_preinsert_effect};
use crate::keycodes::{
    Ctrl_A, Ctrl_F, Ctrl_L, Ctrl_P, Ctrl_R, Ctrl_U, Ctrl_V, Ctrl_W, vim_strsave_escape_ks,
    vim_unescape_ks,
};
use crate::main::{
    Columns, State, VIsual_active, VIsual_mode, c_bytes, curbuf, curwin, e_nobufnr, e_noinstext,
    e_nolastcmd, e_noprevre, e_resulting_text_too_long, got_int, last_cmdline, msg_ext_skip_flush,
    must_redraw, new_last_cmdline, p_ch, p_report, p_sel, pending_end_reg_executing, redir_reg,
    reg_executing, reg_recorded, reg_recording, restart_edit,
};
use crate::mark::mark_adjust;
use crate::mbyte::{
    mb_charlen, mb_string2cells, mb_string2cells_len, mb_tolower, utf_head_off,
    utf_ptr2StrCharInfo, utf_ptr2cells_len, utf_ptr2len_len, utfc_next, utfc_ptr2len,
};
use crate::memline::{decl, ml_append, ml_get, ml_get_buf, ml_get_len, ml_replace};
use crate::memory::{
    memchrsub, memcnt, xcalloc, xfree, xmalloc, xmallocz, xmemdupz, xrealloc, xstrdup,
};
use crate::message::{
    emsg, emsg_invreg, message_filtered, msg, msg_ext_set_kind, msg_outtrans_len, msg_putchar,
    msg_puts, msg_puts_hl, msg_puts_title, msgmore,
};
use crate::r#move::{changed_cline_bef_curs, invalidate_botline_win, update_topline};
use crate::normal::find_ident_under_cursor;
use crate::ops::{adjust_cursor_eol, block_prep, charwise_block_prep, get_op_char};
use crate::option::get_ve_flags;
use crate::options::{kOptVeFlagAll, kOptVeFlagOnemore};
use crate::os::cshim::{gettext, memmove, ngettext, snprintf, strncmp};
use crate::os::input::os_breakcheck;
use crate::os::time::os_time;
use crate::plines::{getvcol, init_charsize_arg, win_charsize};
use crate::pos::{MAXCOL, MAXLNUM};
use crate::regexp::RE_SEARCH;
use crate::search::{BACKWARD, FORWARD, last_search_pat, set_last_search_pat};
use crate::state::REPLACE_FLAG;
use crate::strings::{vim_snprintf, vim_strchr, vim_strsave_escaped_ext};
use crate::terminal::terminal_paste;
use crate::types::ui::kUIMessages;
use crate::types::{
    AdditionalData, CharsizeArg, CmdModFlags, ExtmarkOp, GRegFlags, MotionType, NUL,
    PUT_BLOCK_INNER, PUT_CURSEND, PUT_CURSLINE, PUT_FIXINDENT, PUT_LINE, PUT_LINE_FORWARD,
    PUT_LINE_SPLIT, RemapValues, String_0, UndoObjectType, VAR_FIXED, bcount_t, block_def, colnr_T,
    exarg_T, garray_T, hashitem_T, hashtab_T, int64_t, kBoolVarFalse, kBoolVarTrue, linenr_T,
    oparg_T, pos_T, ptrdiff_t, save_v_event_T, size_t, ssize_t, yankreg_T,
};
use crate::ui::ui_has;
use crate::undo::{u_save, u_save_cursor};
use ::libc::{abort, atoi, memcpy, memset, strcpy, strlen};

// The carve of the transpiled module; see each child's docs.
mod contents;
mod display;
mod exec;
mod put;
mod special;
mod store;
mod yank;

pub use self::contents::*;
pub use self::display::*;
pub use self::exec::*;
pub use self::put::*;
pub use self::special::*;
pub use self::store::*;
pub use self::yank::*;

pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
/// The layout of `y_regs`, and the slots with a fixed meaning.  Upstream's
/// anonymous enum in `register_defs.h`; `c_int` because every use is an
/// index compared against or assigned to one.
pub const NUM_REGISTERS: ::core::ffi::c_int = 39;
/// `"+`, the system clipboard.
pub const PLUS_REGISTER: ::core::ffi::c_int = 38;
/// `"*`, the X11 primary selection.
pub const STAR_REGISTER: ::core::ffi::c_int = 37;
/// How many of the slots shada saves: everything below `"*`.
pub const NUM_SAVED_REGISTERS: ::core::ffi::c_int = 37;
/// `"-`, the small-delete register.
pub const DELETION_REGISTER: ::core::ffi::c_int = 36;
pub const kGRegList: GRegFlags = 4;
pub const kGRegExprSrc: GRegFlags = 2;
pub const kGRegNoExpr: GRegFlags = 1;

/// What `get_yank_register()` is being asked for.  `YREG_PASTE` may query
/// the clipboard provider, `YREG_YANK` moves `y_previous`, and `YREG_PUT`
/// only reports where a paste would read from.
pub const YREG_PUT: ::core::ffi::c_int = 2;
pub const YREG_YANK: ::core::ffi::c_int = 1;
pub const YREG_PASTE: ::core::ffi::c_int = 0;

/// `ins_typebuf` remap mode: what the inserted keys may be remapped by.
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
/// `set_indent()` flags: leave the marks alone, save for undo, this is an
/// Insert-mode indent, mark the buffer changed.
pub const SIN_NOMARK: ::core::ffi::c_int = 8;
pub const SIN_UNDO: ::core::ffi::c_int = 4;
pub const SIN_INSERT: ::core::ffi::c_int = 2;
pub const SIN_CHANGED: ::core::ffi::c_int = 1;

/// `find_ident_under_cursor()` flags: accept a non-keyword run as well as
/// an identifier.
pub const FIND_STRING: ::core::ffi::c_int = 2;
pub const FIND_IDENT: ::core::ffi::c_int = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
static expr_line: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static execreg_lastc: GlobalCell<::core::ffi::c_int> = GlobalCell::new(NUL);
/// The empty register every slot of `y_regs` starts as: upstream's
/// `static yankreg_T y_regs[NUM_REGISTERS] = { 0 }`.
const EMPTY_YANKREG: yankreg_T = yankreg_T {
    y_array: ::core::ptr::null_mut::<String_0>(),
    y_size: 0,
    y_type: kMTCharWise,
    y_width: 0,
    timestamp: 0,
    additional_data: ::core::ptr::null_mut::<AdditionalData>(),
};
static y_regs: GlobalCell<[yankreg_T; 39]> = GlobalCell::new([EMPTY_YANKREG; 39]);
static y_previous: GlobalCell<*mut yankreg_T> =
    GlobalCell::new(::core::ptr::null_mut::<yankreg_T>());
static e_search_pattern_and_expression_register_may_not_contain_two_or_more_lines:
    [::core::ffi::c_char; 79] =
    c_bytes(b"E883: Search pattern and expression register may not contain two or more lines\0");
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
