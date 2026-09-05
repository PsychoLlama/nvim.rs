//! Where the editor is, and what it is in the middle of.
//!
//! [`State`] is the mode word the rest of the tree branches on; the names
//! around it are the parts of a modal edit that outlive a single keystroke —
//! the operator being built (`finish_op`, `opcount`, `motion_force`), the
//! Visual selection that can be reselected with `gv` (`VIsual_*`,
//! `resel_VIsual_*`), and the insert session's own bookkeeping (`Insstart`,
//! `did_ai`, the `can_si` smart-indent flags, the `edit_submode` text the
//! ruler shows).
//!
//! They are here rather than in [`edit`] or [`normal`] because they are what
//! those two modes say to *each other* and to everything that has to know
//! whether an edit is in progress: `restart_edit` is set by Normal mode and
//! read by the insert loop, `Insstart` is written by the insert loop and read
//! by the operators, and `State` is read by nearly everything.
//!
//! [`State`]: self::State
//! [`edit`]: crate::edit
//! [`normal`]: crate::normal
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

use super::MODE_NORMAL;
use crate::cstr::c_bytes;
use crate::global_cell::GlobalCell;
use crate::highlight_group::HLF_NONE;
use crate::normal::VisualMode;
use crate::types::{ColNr, Hlf, LineNr, Pos};
use core::ffi::{c_char, c_int};

pub(crate) static edit_submode: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static edit_submode_pre: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static edit_submode_extra: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static edit_submode_highl: GlobalCell<Hlf> = GlobalCell::new(HLF_NONE);
pub(crate) static VIsual_select_reg: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static VIsual_select_exclu_adj: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static restart_VIsual_select: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static VIsual_reselect: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static redo_VIsual_busy: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static resel_VIsual_mode: GlobalCell<VisualMode> = GlobalCell::new(VisualMode::NONE);
pub(crate) static resel_VIsual_line_count: GlobalCell<LineNr> = GlobalCell::new(0);
pub(crate) static resel_VIsual_vcol: GlobalCell<ColNr> = GlobalCell::new(0);
pub(crate) static where_paste_started: GlobalCell<Pos> = GlobalCell::new(Pos {
    lnum: 0,
    col: 0,
    coladd: 0,
});
pub(crate) static did_ai: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static ai_col: GlobalCell<ColNr> = GlobalCell::new(0 as ColNr);
pub(crate) static end_comment_pending: GlobalCell<c_int> = GlobalCell::new('\0' as c_int);
pub(crate) static did_syncbind: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static did_si: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static can_si: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static can_si_back: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static old_indent: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static saved_cursor: GlobalCell<Pos> = GlobalCell::new(Pos {
    lnum: 0 as LineNr,
    col: 0 as ColNr,
    coladd: 0 as ColNr,
});
pub(crate) static Insstart: GlobalCell<Pos> = GlobalCell::new(Pos {
    lnum: 0,
    col: 0,
    coladd: 0,
});
pub(crate) static Insstart_orig: GlobalCell<Pos> = GlobalCell::new(Pos {
    lnum: 0,
    col: 0,
    coladd: 0,
});
pub(crate) static orig_line_count: GlobalCell<LineNr> = GlobalCell::new(0 as LineNr);
pub(crate) static vr_lines_changed: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static State: GlobalCell<c_int> = GlobalCell::new(MODE_NORMAL);
pub(crate) static finish_op: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static opcount: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static motion_force: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static exmode_active: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static pending_exmode_active: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static force_restart_edit: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static restart_edit: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static arrow_used: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static ins_at_eol: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static no_abbr: GlobalCell<bool> = GlobalCell::new(true);
pub(crate) static stop_insert_mode: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static need_start_insertmode: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static last_mode: GlobalCell<[c_char; 4]> = GlobalCell::new(c_bytes(b"n\0\0\0"));
pub(crate) static replace_offset: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static km_stopsel: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static km_startsel: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static virtual_op: GlobalCell<Option<bool>> = GlobalCell::new(None);
