//! The yank registers: what is in them, and every way text gets in or out.
//!
//! The parent keeps no functions, only the process-wide state the children
//! share -- which is the point of it being one screen:
//!
//! | cell | written by |
//! | --- | --- |
//! | `Y_REGS` | the 39 register slots; `store`, `yank` and `contents` |
//! | `Y_PREVIOUS` | the last register written; `store` and `contents` |
//! | `EXPR_LINE` | the `"=` source text, kept for a repeat; `special` |
//! | `EXECREG_LASTC` | the last `@` register, so that `@@` repeats it; `exec` |
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

use crate::ascii::CAR;
use crate::ascii::ESC;
use crate::ascii::NL;
use crate::ascii::TAB;
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::autocmd::defs::AutoEvent;
use crate::autocmd::{apply_autocmds, has_event};
use crate::buffer::{buf_is_empty, buflist_findpat, buflist_name_nr, find_buf};
use crate::buffer_updates::buf_updates_send_changes;
use crate::change::{changed_bytes, changed_lines, del_chars};
use crate::charset::transchar;
use crate::clipboard;
use crate::cursor::{coladvance_force, gchar_cursor, get_cursor_line_len, getviscol};
use crate::drawscreen::state::MUST_REDRAW;
use crate::drawscreen::{showmode, update_screen};
use crate::edit::{beginline, get_last_insert, get_last_insert_save, oneright, stuff_inserted};
use crate::eval::typval::{list_set_lock, tv_list_alloc};
use crate::eval::{eval_to_string, get_v_event, restore_v_event};
use crate::ex_cmds2::check_fname;
use crate::ex_docmd::state::{LAST_CMDLINE, NEW_LAST_CMDLINE};
use crate::ex_getln::cmdline_paste_str;
use crate::extmark::BCount;
use crate::extmark::{extmark_splice, extmark_splice_cols};
use crate::file_search::file_name_at_cursor;
use crate::getchar::state::{
    GOT_INT, PENDING_END_REG_EXECUTING, REG_EXECUTING, REG_RECORDED, REG_RECORDING,
};
use crate::getchar::{
    append_to_redobuff_char, beep_flush, get_recorded, ins_typebuf, stuff_readbuf,
    stuff_readbuf_char, stuffescaped,
};
use crate::global_cell::GlobalCell;
use crate::hashtab::HashTab;
use crate::highlight_group::HLF_8;
use crate::indent::{get_indent, preprocs_left, set_indent};
use crate::insexpand::{ins_compl_delete, ins_compl_preinsert_effect};
use crate::keycodes::{escape_ks_bytes, unescape_ks_in};
use crate::mark::mark_adjust;
use crate::memline::{decl, ml_append_cstr, ml_append_terminated, ml_replace_owned};
use crate::memory::XString;
use crate::message::state::{MSG_EXT_SKIP_FLUSH, REDIR_REG};
use crate::message::{E_NOINSTEXT, E_NOLASTCMD, E_NOPREVRE, E_RESULTING_TEXT_TOO_LONG};
use crate::message::{
    emsg, emsg_invreg, message_filtered, msg, msg_display_bytes, msg_ext_set_kind, msg_putchar,
    msg_str, msg_str_hl, msg_title, msgmore,
};
use crate::r#move::{changed_cline_bef_curs, invalidate_botline_win, update_topline};
use crate::normal::defs::{MotionType, OpArg};
use crate::ops::{adjust_cursor_eol, get_op_char};
use crate::option::get_ve_flags;
use crate::option::vars::{p_ch, p_report};
use crate::options::{OPT_VE_FLAG_ALL, OPT_VE_FLAG_ONEMORE};
use crate::os::cshim::{gettext, ngettext};
use crate::os::input::os_breakcheck;
use crate::os::libc::{int64_t, ptrdiff_t, size_t};
use crate::os::time::os_time;
use crate::plines::CharsizeArg;
use crate::plines::{getvcol, win_charsize};
use crate::pos::MAXLNUM;
use crate::regexp::RE_SEARCH;
use crate::register::defs::{
    BlockDef, GRegFlags, PUT_BLOCK_INNER, PUT_CURSEND, PUT_CURSLINE, PUT_FIXINDENT, PUT_LINE,
    PUT_LINE_FORWARD, PUT_LINE_SPLIT, PutFlags, YankReg,
};
use crate::search::{BACKWARD, FORWARD, last_search_pat, set_last_search_pat};
use crate::state::REPLACE_FLAG;
use crate::state::mode::{RESTART_EDIT, STATE};
use crate::strings::escaped_with;
use crate::terminal::terminal_paste;
use crate::types::{
    ApiString, BOOL_VAR_FALSE, BOOL_VAR_TRUE, CmdModFlags, ColNr, Direction, ExArg, LineNr, Pos,
    SaveVEvent, VarLock,
};
use crate::ui::state::COLUMNS;
use crate::ui::state::UI_MESSAGES;
use crate::ui::ui_has;
use crate::undo::{u_save, u_save_cursor};

// The carve of the transpiled module; see each child's docs.
mod contents;
pub mod defs;
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

pub const MT_UNKNOWN: MotionType = -1;
pub const MT_BLOCK_WISE: MotionType = 2;
pub const MT_LINE_WISE: MotionType = 1;
pub const MT_CHAR_WISE: MotionType = 0;
/// The layout of `Y_REGS`, and the slots with a fixed meaning: upstream's
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
pub const G_REG_LIST: GRegFlags = 4;
pub const G_REG_EXPR_SRC: GRegFlags = 2;
pub const G_REG_NO_EXPR: GRegFlags = 1;

/// What `get_yank_register()` is being asked for.  `YREG_PASTE` may query
/// the clipboard provider, `YREG_YANK` moves `Y_PREVIOUS`, and `YREG_PUT`
/// only reports where a paste would read from.
pub const YREG_PUT: ::core::ffi::c_int = 2;
pub const YREG_YANK: ::core::ffi::c_int = 1;
pub const YREG_PASTE: ::core::ffi::c_int = 0;

static EXPR_LINE: GlobalCell<Option<XString>> = GlobalCell::new(None);
/// The register `@@` replays: the last one `@` ran, 0 for none yet.
static EXECREG_LASTC: GlobalCell<RegName> = GlobalCell::new(0);
/// The 39 register slots, laid out as [`op_reg_index`] says; reached only
/// through [`Reg`]'s closures.
static Y_REGS: GlobalCell<[YankReg; NUM_REGISTERS]> =
    GlobalCell::new([YankReg::EMPTY; NUM_REGISTERS]);
/// The slot of `Y_REGS` that `""` reads: the last register written.
static Y_PREVIOUS: GlobalCell<Option<usize>> = GlobalCell::new(None);
static E_SEARCH_PATTERN_AND_EXPRESSION_REGISTER_MAY_NOT_CONTAIN_TWO_OR_MORE_LINES:
    &::core::ffi::CStr =
    c"E883: Search pattern and expression register may not contain two or more lines";
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
