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
//! # One record
//!
//! Upstream declares each of these as its own `EXTERN` in `globals.h`. They
//! are one [`ModeState`] here, behind one cell, and every name below is a
//! `const` selector into it rather than a cell of its own: a reader still
//! writes `State.get()`, and nothing can take the address of one. See
//! [`state_record`](crate::global_cell::state_record) for the access rules.
//!
//! The three `edit_submode*` texts are the mode line's while a CTRL-X
//! submode runs. Upstream holds `char *`s to translated literals and to a
//! static buffer `ins_compl_show_statusmsg` formats into; the first two are
//! always literals here (`&'static CStr`), and the third owns its text.
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
use crate::global_cell::state_record;
use crate::highlight_group::HLF_NONE;
use crate::memory::XString;
use crate::normal::VisualMode;
use crate::types::{ColNr, Hlf, LineNr, Pos};
use core::ffi::{CStr, c_char, c_int};

/// The origin, which every position here starts at.
const POS_ZERO: Pos = Pos {
    lnum: 0,
    col: 0,
    coladd: 0,
};

state_record! {
    /// Where the editor is, and what it is in the middle of. See the
    /// [module docs](self).
    pub(crate) struct ModeState in MODE as ModeField;

    // -- the mode --
    /// The mode word: `MODE_NORMAL`, `MODE_INSERT`, ... with flags.
    pub(crate) State: c_int = MODE_NORMAL;
    /// The mode `mode()` reported last, for `ModeChanged`.
    pub(crate) last_mode: [c_char; 4] = c_bytes(b"n\0\0\0");
    /// In Ex mode (`Q`/`gQ`).
    pub(crate) exmode_active: bool = false;
    /// Enter Ex mode when the current command is done.
    pub(crate) pending_exmode_active: bool = false;

    // -- the operator being built --
    /// An operator is pending: the next motion finishes it.
    pub(crate) finish_op: bool = false;
    /// The count typed before the operator.
    pub(crate) opcount: c_int = 0;
    /// `v`, `V` or CTRL-V typed after the operator, forcing its motion type.
    pub(crate) motion_force: c_int = 0;
    /// 'virtualedit' forced for one operator: `None` follows the option.
    pub(crate) virtual_op: Option<bool> = None;

    // -- Visual mode --
    /// The register Select mode puts replaced text in.
    pub(crate) VIsual_select_reg: c_int = 0;
    /// An exclusive Select-mode selection was adjusted by one.
    pub(crate) VIsual_select_exclu_adj: bool = false;
    /// Go back to Select mode when the CTRL-O command is done.
    pub(crate) restart_VIsual_select: c_int = 0;
    /// Restart the selection after a Select-mode mapping or menu.
    pub(crate) VIsual_reselect: c_int = 0;
    /// Redoing a Visual-mode operator.
    pub(crate) redo_VIsual_busy: bool = false;
    /// The previous Visual area, for `gv` and `.`.
    pub(crate) resel_VIsual_mode: VisualMode = VisualMode::NONE;
    pub(crate) resel_VIsual_line_count: LineNr = 0;
    pub(crate) resel_VIsual_vcol: ColNr = 0;
    /// 'keymodel' has `startsel`: a shifted special key starts a selection.
    pub(crate) km_startsel: bool = false;
    /// 'keymodel' has `stopsel`: an unshifted special key stops it.
    pub(crate) km_stopsel: bool = false;

    // -- the insert session --
    /// Where the insert started.
    pub(crate) Insstart: Pos = POS_ZERO;
    /// `Insstart` as it was when the insert started; `Insstart` moves.
    pub(crate) Insstart_orig: Pos = POS_ZERO;
    /// Where 'paste' was switched on during the insert.
    pub(crate) where_paste_started: Pos = POS_ZERO;
    /// Restart the insert when the current command is done: `i`, `a`, `R`, `V`.
    pub(crate) restart_edit: c_int = 0;
    /// Honour `restart_edit` even where a finished command would drop it
    /// (`:normal` into a terminal buffer).
    pub(crate) force_restart_edit: bool = false;
    /// A cursor key was used in Insert mode: the insert is broken in two.
    pub(crate) arrow_used: bool = false;
    /// Put the cursor past the end of the line when the insert restarts
    /// after CTRL-O.
    pub(crate) ins_at_eol: bool = false;
    /// Don't check for abbreviations.
    pub(crate) no_abbr: bool = true;
    /// Leave Insert mode at the next chance.
    pub(crate) stop_insert_mode: bool = false;
    /// Start Insert mode at the next chance.
    pub(crate) need_start_insertmode: bool = false;
    /// The line count when `gR` started.
    pub(crate) orig_line_count: LineNr = 0;
    /// Lines changed by `gR` so far.
    pub(crate) vr_lines_changed: c_int = 0;
    /// The cursor before text was formatted.
    pub(crate) saved_cursor: Pos = POS_ZERO;
    /// The offset `replace_push` inserts at.
    pub(crate) replace_offset: c_int = 0;
    /// A scroll already synced the 'scrollbind' windows; the next check
    /// skips its own pass.
    pub(crate) did_syncbind: bool = false;

    // -- autoindent and smartindent --
    /// Autoindent was done; undone if nothing is typed.
    pub(crate) did_ai: bool = false;
    /// The column the autoindent reached.
    pub(crate) ai_col: ColNr = 0;
    /// The end character of a three-part comment leader just inserted.
    pub(crate) end_comment_pending: c_int = 0;
    /// Smartindent was done.
    pub(crate) did_si: bool = false;
    /// Smartindent may indent after this line.
    pub(crate) can_si: bool = false;
    /// Smartindent may dedent a `}`.
    pub(crate) can_si_back: bool = false;
    /// The indent `^ CTRL-D` takes away for one line.
    pub(crate) old_indent: c_int = 0;

    // -- the CTRL-X submode's mode line --
    /// The submode's name, as `-- Keyword completion (^N^P)`.
    pub(crate) edit_submode: Option<&'static CStr> = None;
    /// What comes before it: ` Adding`.
    pub(crate) edit_submode_pre: Option<&'static CStr> = None;
    /// What comes after it: `match 1 of 4`, `Back at original`.
    pub(crate) edit_submode_extra: Option<XString> = None;
    /// The highlight `edit_submode_extra` is drawn in.
    pub(crate) edit_submode_highl: Hlf = HLF_NONE;
}
