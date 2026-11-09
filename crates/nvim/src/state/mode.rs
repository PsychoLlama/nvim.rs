//! Where the editor is, and what it is in the middle of.
//!
//! [`State`] is the mode word the rest of the tree branches on; the names
//! around it are the parts of a modal edit that outlive a single keystroke —
//! the operator being built (`FINISH_OP`, `opcount`, `MOTION_FORCE`), the
//! Visual selection that can be reselected with `gv` (`VIsual_*`,
//! `resel_VIsual_*`), and the insert session's own bookkeeping (`INSSTART`,
//! `DID_AI`, the `CAN_SI` smart-indent flags, the `EDIT_SUBMODE` text the
//! ruler shows).
//!
//! They are here rather than in [`edit`] or [`normal`] because they are what
//! those two modes say to *each other* and to everything that has to know
//! whether an edit is in progress: `RESTART_EDIT` is set by Normal mode and
//! read by the insert loop, `INSSTART` is written by the insert loop and read
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
//! The three `EDIT_SUBMODE*` texts are the mode line's while a CTRL-X
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

use super::MODE_NORMAL;
use crate::global_cell::state_record;
use crate::highlight::Hlf;
use crate::highlight_group::HLF_NONE;
use crate::memory::XString;
use crate::normal::VisualMode;
use crate::types::{ColNr, Key, LineNr, Pos};
use core::ffi::{CStr, c_int};

/// The origin, which every position here starts at.
const POS_ZERO: Pos = Pos {
    lnum: LineNr::ZERO,
    col: ColNr::ZERO,
    coladd: 0,
};

state_record! {
    /// Where the editor is, and what it is in the middle of. See the
    /// [module docs](self).
    pub(crate) struct ModeState in MODE as ModeField;

    // -- the mode --
    /// The mode word: `MODE_NORMAL`, `MODE_INSERT`, ... with flags.
    pub(crate) STATE: c_int = MODE_NORMAL;
    /// The mode `mode()` reported last, for `ModeChanged`.
    pub(crate) LAST_MODE: [u8; 4] = *b"n\0\0\0";
    /// In Ex mode (`Q`/`gQ`).
    pub(crate) EXMODE_ACTIVE: bool = false;
    /// Enter Ex mode when the current command is done.
    pub(crate) PENDING_EXMODE_ACTIVE: bool = false;

    // -- the operator being built --
    /// An operator is pending: the next motion finishes it.
    pub(crate) FINISH_OP: bool = false;
    /// The count typed before the operator.
    pub(crate) OPCOUNT: c_int = 0;
    /// `v`, `V` or CTRL-V typed after the operator, forcing its motion type.
    pub(crate) MOTION_FORCE: Key = Key::NUL;
    /// 'virtualedit' forced for one operator: `None` follows the option.
    pub(crate) VIRTUAL_OP: Option<bool> = None;

    // -- Visual mode --
    /// The register Select mode puts replaced text in.
    pub(crate) V_ISUAL_SELECT_REG: crate::register::RegName = crate::register::RegName::UNNAMED;
    /// An exclusive Select-mode selection was adjusted by one.
    pub(crate) V_ISUAL_SELECT_EXCLU_ADJ: bool = false;
    /// Go back to Select mode when the CTRL-O command is done.
    pub(crate) RESTART_V_ISUAL_SELECT: c_int = 0;
    /// Restart the selection after a Select-mode mapping or menu.
    pub(crate) V_ISUAL_RESELECT: c_int = 0;
    /// Redoing a Visual-mode operator.
    pub(crate) REDO_V_ISUAL_BUSY: bool = false;
    /// The previous Visual area, for `gv` and `.`.
    pub(crate) RESEL_V_ISUAL_MODE: VisualMode = VisualMode::NONE;
    pub(crate) RESEL_V_ISUAL_LINE_COUNT: i32 = 0;
    pub(crate) RESEL_V_ISUAL_VCOL: ColNr = ColNr::ZERO;
    /// 'keymodel' has `startsel`: a shifted special key starts a selection.
    pub(crate) KM_STARTSEL: bool = false;
    /// 'keymodel' has `stopsel`: an unshifted special key stops it.
    pub(crate) KM_STOPSEL: bool = false;

    // -- the insert session --
    /// Where the insert started.
    pub(crate) INSSTART: Pos = POS_ZERO;
    /// `INSSTART` as it was when the insert started; `INSSTART` moves.
    pub(crate) INSSTART_ORIG: Pos = POS_ZERO;
    /// Where 'paste' was switched on during the insert.
    pub(crate) WHERE_PASTE_STARTED: Pos = POS_ZERO;
    /// Restart the insert when the current command is done: `i`, `a`, `R`, `V`.
    pub(crate) RESTART_EDIT: Key = Key::NUL;
    /// Honour `RESTART_EDIT` even where a finished command would drop it
    /// (`:normal` into a terminal buffer).
    pub(crate) FORCE_RESTART_EDIT: bool = false;
    /// A cursor key was used in Insert mode: the insert is broken in two.
    pub(crate) ARROW_USED: bool = false;
    /// Put the cursor past the end of the line when the insert restarts
    /// after CTRL-O.
    pub(crate) INS_AT_EOL: bool = false;
    /// Don't check for abbreviations.
    pub(crate) NO_ABBR: bool = true;
    /// Leave Insert mode at the next chance.
    pub(crate) STOP_INSERT_MODE: bool = false;
    /// Start Insert mode at the next chance.
    pub(crate) NEED_START_INSERTMODE: bool = false;
    /// The line count when `gR` started.
    pub(crate) ORIG_LINE_COUNT: LineNr = LineNr::ZERO;
    /// Lines changed by `gR` so far.
    pub(crate) VR_LINES_CHANGED: c_int = 0;
    /// The cursor before text was formatted.
    pub(crate) SAVED_CURSOR: Pos = POS_ZERO;
    /// The offset `replace_push` inserts at.
    pub(crate) REPLACE_OFFSET: c_int = 0;
    /// A scroll already synced the 'scrollbind' windows; the next check
    /// skips its own pass.
    pub(crate) DID_SYNCBIND: bool = false;

    // -- autoindent and smartindent --
    /// Autoindent was done; undone if nothing is typed.
    pub(crate) DID_AI: bool = false;
    /// The column the autoindent reached.
    pub(crate) AI_COL: ColNr = ColNr::ZERO;
    /// The last byte of the end of a three-part comment leader just
    /// inserted, which typed closes the comment.
    pub(crate) END_COMMENT_PENDING: Option<u8> = None;
    /// Smartindent was done.
    pub(crate) DID_SI: bool = false;
    /// Smartindent may indent after this line.
    pub(crate) CAN_SI: bool = false;
    /// Smartindent may dedent a `}`.
    pub(crate) CAN_SI_BACK: bool = false;
    /// The indent `^ CTRL-D` takes away for one line.
    pub(crate) OLD_INDENT: c_int = 0;

    // -- the CTRL-X submode's mode line --
    /// The submode's name, as `-- Keyword completion (^N^P)`.
    pub(crate) EDIT_SUBMODE: Option<&'static CStr> = None;
    /// What comes before it: ` Adding`.
    pub(crate) EDIT_SUBMODE_PRE: Option<&'static CStr> = None;
    /// What comes after it: `match 1 of 4`, `Back at original`.
    pub(crate) EDIT_SUBMODE_EXTRA: Option<XString> = None;
    /// The highlight `EDIT_SUBMODE_EXTRA` is drawn in.
    pub(crate) EDIT_SUBMODE_HIGHL: Hlf = HLF_NONE;
}
