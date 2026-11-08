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
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::autocmd::{apply_autocmds, has_event};
use crate::buffer::{buf_is_empty, buflist_findpat, buflist_name_nr, find_buf};
use crate::buffer_updates::buf_updates_send_changes;
use crate::change::{changed_bytes, changed_lines, del_chars};
use crate::charset::transchar;
use crate::clipboard;
use crate::cursor::{coladvance_force, gchar_cursor, get_cursor_line_len, getviscol};
use crate::drawscreen::state::must_redraw;
use crate::drawscreen::{showmode, update_screen};
use crate::edit::{beginline, get_last_insert, get_last_insert_save, oneright, stuff_inserted};
use crate::eval::typval::{list_set_lock, tv_list_alloc};
use crate::eval::{eval_to_string, get_v_event, restore_v_event};
use crate::ex_cmds2::check_fname;
use crate::ex_docmd::state::{last_cmdline, new_last_cmdline};
use crate::ex_getln::cmdline_paste_str;
use crate::extmark::BCount;
use crate::extmark::{extmark_splice, extmark_splice_cols};
use crate::file_search::file_name_at_cursor;
use crate::getchar::state::{
    got_int, pending_end_reg_executing, reg_executing, reg_recorded, reg_recording,
};
use crate::getchar::{
    append_to_redobuff_char, beep_flush, get_recorded, ins_typebuf, stuff_readbuf,
    stuff_readbuf_char, stuffescaped,
};
use crate::global_cell::GlobalCell;
use crate::highlight_group::HLF_8;
use crate::indent::{get_indent, preprocs_left, set_indent};
use crate::insexpand::{ins_compl_delete, ins_compl_preinsert_effect};
use crate::keycodes::{escape_ks_bytes, unescape_ks_in};
use crate::mark::mark_adjust;
use crate::memline::{decl, ml_append_cstr, ml_append_terminated, ml_replace_owned};
use crate::memory::XString;
use crate::message::state::{msg_ext_skip_flush, redir_reg};
use crate::message::{e_noinstext, e_nolastcmd, e_noprevre, e_resulting_text_too_long};
use crate::message::{
    emsg, emsg_invreg, message_filtered, msg, msg_display_bytes, msg_ext_set_kind, msg_putchar,
    msg_str, msg_str_hl, msg_title, msgmore,
};
use crate::r#move::{changed_cline_bef_curs, invalidate_botline_win, update_topline};
use crate::ops::{adjust_cursor_eol, get_op_char};
use crate::option::get_ve_flags;
use crate::option::vars::{p_ch, p_report};
use crate::options::{kOptVeFlagAll, kOptVeFlagOnemore};
use crate::os::cshim::{gettext, ngettext};
use crate::os::input::os_breakcheck;
use crate::os::libc::{int64_t, ptrdiff_t, size_t};
use crate::os::time::os_time;
use crate::plines::CharsizeArg;
use crate::plines::{getvcol, win_charsize};
use crate::pos::MAXLNUM;
use crate::regexp::RE_SEARCH;
use crate::search::{BACKWARD, FORWARD, last_search_pat, set_last_search_pat};
use crate::state::REPLACE_FLAG;
use crate::state::mode::{State, restart_edit};
use crate::strings::escaped_with;
use crate::terminal::terminal_paste;
use crate::types::AutoEvent;
use crate::types::CAR;
use crate::types::ESC;
use crate::types::NL;
use crate::types::TAB;
use crate::types::{
    ApiString, BlockDef, CmdModFlags, ColNr, Direction, ExArg, GRegFlags, HashTab, LineNr,
    MotionType, OpArg, PUT_BLOCK_INNER, PUT_CURSEND, PUT_CURSLINE, PUT_FIXINDENT, PUT_LINE,
    PUT_LINE_FORWARD, PUT_LINE_SPLIT, Pos, PutFlags, SaveVEvent, VarLock, YankReg, kBoolVarFalse,
    kBoolVarTrue,
};
use crate::ui::state::Columns;
use crate::ui::state::kUIMessages;
use crate::ui::ui_has;
use crate::undo::{u_save, u_save_cursor};

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

pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
/// The layout of `y_regs`, and the slots with a fixed meaning: upstream's
/// anonymous enum in `register_defs.h`, as indices into the array.
pub const NUM_REGISTERS: usize = 39;
/// `"+`, the system clipboard.
pub const PLUS_REGISTER: usize = 38;
/// `"*`, the X11 primary selection.
pub const STAR_REGISTER: usize = 37;
/// How many of the slots shada saves: everything below `"*`.
pub const NUM_SAVED_REGISTERS: usize = 37;
/// `"-`, the small-delete register.
pub const DELETION_REGISTER: usize = 36;
pub const kGRegList: GRegFlags = 4;
pub const kGRegExprSrc: GRegFlags = 2;
pub const kGRegNoExpr: GRegFlags = 1;

/// What `get_yank_register()` is being asked for.  `YREG_PASTE` may query
/// the clipboard provider, `YREG_YANK` moves `y_previous`, and `YREG_PUT`
/// only reports where a paste would read from.
pub const YREG_PUT: ::core::ffi::c_int = 2;
pub const YREG_YANK: ::core::ffi::c_int = 1;
pub const YREG_PASTE: ::core::ffi::c_int = 0;

pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
static expr_line: GlobalCell<Option<XString>> = GlobalCell::new(None);
/// The register `@@` replays: the last one `@` ran, 0 for none yet.
static execreg_lastc: GlobalCell<RegName> = GlobalCell::new(0);
/// The 39 register slots, laid out as [`op_reg_index`] says; reached only
/// through [`Reg`]'s closures.
static y_regs: GlobalCell<[YankReg; NUM_REGISTERS]> =
    GlobalCell::new([YankReg::EMPTY; NUM_REGISTERS]);
/// The slot of `y_regs` that `""` reads: the last register written.
static y_previous: GlobalCell<Option<usize>> = GlobalCell::new(None);
static e_search_pattern_and_expression_register_may_not_contain_two_or_more_lines:
    &::core::ffi::CStr =
    c"E883: Search pattern and expression register may not contain two or more lines";
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
