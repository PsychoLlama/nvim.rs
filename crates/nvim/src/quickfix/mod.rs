//! The quickfix and location list stacks.
//!
//! A [`QfStack`] holds up to `'chistory'` [`QfList`]s; a list holds its
//! [`QfEntry`]s in order. There is one quickfix stack for the editor and one
//! location list stack per window that asks for one, all owned by the table
//! in `stack`, which is also where the module's rules for holding a stack
//! across user code are written down. This file holds the glue — the
//! constant families and the shared statics — and the children hold the
//! work:
//!
//! - Building a list: `efm` compiles `'errorformat'`, `parse` applies it to
//!   a line, `read` drives the whole read, `list` and `entry` own the
//!   entries, `stack` owns the stacks.
//! - The commands that build one: `cmds` (`:cfile`/`:cbuffer`/`:cexpr`),
//!   `make` (`:make`/`:grep`), `vimgrep` with `dummy`, and `helpgrep`.
//! - Using a list: `navigate` picks an entry, `jump` and `switchbuf` go
//!   there, `display` and `window` with `fill` show it.
//! - Vimscript: `getprops`, `setprops` and the `eval` bridges.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::ascii::PATHSEP;
use crate::ascii::{TAB, ascii_iswhite};
use crate::autocmd::defs::AutoEvent;
use crate::autocmd::{
    au_event_disable, au_event_restore, block_autocmds, fire_autocmds_for, unblock_autocmds,
};
use crate::buffer::{
    BCO_ENTER, BCO_NOHELP, BLN_DUMMY, BLN_NOOPT, DOBUF_UNLOAD, DOBUF_WIPE, GETF_SETMARK,
    GETF_SWITCH, buf_is_help, buf_is_normal, buf_is_quickfix, buflist_getfile, close_buffer,
    current_buf, do_modelines, find_buf, no_write_message, wipe_buffer,
};
use crate::change::changed_lines;
use crate::charset::{getdigits_int_at, vim_isprintc};
use crate::cursor::{check_cursor, coladvance};
use crate::drawscreen::state::{CMDLINE_ROW, MUST_REDRAW};
use crate::drawscreen::{
    UPD_NOT_VALID, UPD_VALID, redraw_buf_later, redraw_curbuf_later, update_screen,
};
use crate::edit::beginline;
use crate::eval::funcs::builtin::EvalFuncData;
use crate::eval::mark_root;
use crate::eval::typval::{
    KeyTaken, dict_get_bool, dict_get_number, dict_get_tv, list_len, tv_copy, tv_dict_alloc,
    tv_dict_alloc_lock, tv_dict_alloc_ret, tv_get_number_chk, tv_list_alloc, tv_list_alloc_ret,
};
use crate::eval::typval::{ListRef, TV_INITIAL_VALUE};
use crate::eval::window::{find_win_by_nr_or_id, win_by_id};
use crate::ex_cmds::{VGR_FUZZY, VGR_GLOBAL, VGR_NOJUMP, append_redir, check_secure};
use crate::ex_cmds2::{autowrite_all, can_abandon};
use crate::ex_docmd::state::CMDMOD;
use crate::ex_docmd::{do_cmdline_cmd, ex_cd, is_loclist_cmd};
use crate::ex_eval::aborting;
use crate::extmark::{EXTMARK_NO_UNDO, extmark_splice};
use crate::fileio::shorten_fnames;
use crate::fold::{fold_open_cursor, fold_update_all};
use crate::fuzzy::{FUZZY_MATCH_MAX_LEN, fuzzy_match};
use crate::getchar::state::{GOT_INT, KEY_TYPED};
use crate::global_cell::GlobalCell;
use crate::highlight_group::syn_name2id;
use crate::mark::setpcmark;
use crate::memline::{check_need_swap, ml_delete, ml_get_buf_len, ml_open};
use crate::memory::ThinCString;
use crate::message::state::{MSG_COL, MSG_DIDOUT, MSG_NOWAIT, MSG_SCROLL, MSG_SCROLLED};
use crate::message::{
    E_AU_RECURSIVE, E_BUFFER_IS_NOT_LOADED, E_DICTREQ, E_INVALPAT, E_INVARG, E_INVRANGE, E_LISTREQ,
    E_LOCLIST, E_NO_ERRORS, E_NOMATCH, E_NOPREVRE, E_NOTMP, E_STRING_REQUIRED,
    E_WINFIXBUF_CANNOT_GO_TO_BUFFER,
};
use crate::message::{
    emsg, internal_error, message_filtered, msg, msg_clr_eos, msg_display, msg_ext_set_kind,
    msg_prt_line, msg_start, msg_str, msg_str_hl,
};
use crate::message_fmt::{emsg_text, msg_bytes};
use crate::r#move::update_topline;
use crate::normal::reset_visual_and_resel;
use crate::ops::get_region_bytecount;
use crate::option::defs::{OptSet, OptVal};
use crate::option::vars::{
    FDO_FLAGS, P_ENC, P_MLS, P_SWB, SWB_FLAGS, p_ch, p_chi, p_hh, p_ic, p_mef, p_mls,
};
use crate::option::{buf_copy_options, set_option_direct, set_option_value_give_err, shortmess};
use crate::options::{
    OPT_BUFHIDDEN, OPT_BUFTYPE, OPT_ERRORFILE, OPT_FDO_FLAG_QUICKFIX, OPT_FILETYPE, OPT_FOLDMETHOD,
    OPT_SWAPFILE, OPT_SWB_FLAG_USELAST, OPT_SWB_FLAG_USETAB,
};
use crate::os::cshim::gettext;
use crate::os::env::process_id;
use crate::os::fs::os_remove;
use crate::os::input::{line_breakcheck, os_breakcheck};
use crate::os::libc::{ptrdiff_t, size_t};
use crate::path::{path_fnamecmp, path_is_absolute, vim_is_abs_name};
use crate::pos::MAXLNUM;
use crate::quickfix::model::{
    DirStack, QFLT_INTERNAL, QFLT_LOCATION, QFLT_QUICKFIX, QfEntry, QfList, QfListType, QfStack,
};
use crate::regexp::{RegMMatch, RegMatch};
use crate::search::{BACKWARD, BACKWARD_FILE, FORWARD, FORWARD_FILE};
use crate::state::mode::RESTART_EDIT;
use crate::tag::CMDBUFFSIZE;
use crate::tr;
use crate::types::Failed;
use crate::types::{
    Callback, ColNr, Dict, Direction, ExArg, LineNr, List, OptInt, Pos, ScriptId, TypVal,
    VarNumber, VarType,
};
use crate::ui::state::COLUMNS;
use crate::ui::ui_flush;
use crate::undo::u_clearallandblockfree;
use crate::window::{
    check_can_set_curbuf_forceit, check_lnums, goto_tabpage_win, win_close, win_enter, win_goto,
    win_setheight, win_split, win_valid,
};
use crate::winlayer::graph::{FIRSTWIN, LASTWIN, PREVWIN};
use core::ffi::{CStr, c_int, c_uint};

// The carve of the transpiled module; see each child's docs.
mod efm;
pub(crate) use self::efm::EfmError;
pub(crate) use self::efm::*;
mod parse;
pub(crate) use self::parse::*;
mod read;
pub(crate) use self::read::*;
mod stack;
pub use self::stack::*;
pub(crate) use crate::winlayer::{Buf, Win};
mod list;
pub(crate) use self::list::*;
mod entry;
pub use self::entry::*;
mod switchbuf;
pub(crate) use self::switchbuf::*;
mod jump;
pub use self::jump::*;
mod display;
pub use self::display::*;
mod window;
pub use self::window::*;
mod fill;
pub(crate) use self::fill::*;
mod navigate;
pub use self::navigate::*;
mod vimgrep;
pub use self::vimgrep::*;
mod dummy;
pub(crate) use self::dummy::*;
mod make;
pub use self::make::*;
mod helpgrep;
pub use self::helpgrep::*;
mod cmds;
pub use self::cmds::*;
mod getprops;
pub(crate) use self::getprops::*;
mod setprops;
pub(crate) use self::setprops::*;
mod eval;
pub use self::eval::*;
pub mod model;
#[cfg(test)]
mod tests;

/// The names `:ls` and the status line give the two list buffers. Upstream
/// keeps them in `char *` globals; nothing has ever written to either.
pub(crate) static MSG_LOCLIST: &CStr = c"[Location List]";
pub(crate) static MSG_QFLIST: &CStr = c"[Quickfix List]";

/// Why a quickfix list could not be read, or a `getqflist()`/`setqflist()`
/// request carried out.
///
/// Most of these Vimscript sees only as the function's `-1`: upstream says
/// nothing more, and [`QfError::report`] puts nothing out for them. The
/// variants are for the code: `qf_*` used to be a chain of functions whose
/// only vocabulary was `FAIL`, so the reason a request was refused stopped
/// existing the moment it was returned. The ones that do carry a message
/// -- an errorfile that would not open, a bad `'errorformat'`, a read
/// error -- are reported once, by [`qf_init_ext`], before the list buffer
/// is updated (as upstream reported them).
#[derive(Clone, PartialEq, Eq, Debug, thiserror::Error)]
pub(crate) enum QfError {
    /// The request named a list that is not on the stack: an `nr` or `id`
    /// that matches nothing, or no stack at all.
    #[error("no such quickfix list")]
    NoSuchList,
    /// A key of the `what` dictionary was given a value of a type it cannot
    /// take — an `idx` that is not a Number, a `title` that is not a String,
    /// `lines` that is not a List.
    #[error("a value of the wrong type")]
    BadValue,
    /// `lines` could not be turned into entries with the `'errorformat'` in
    /// force. Said by the read that failed, if by anyone.
    #[error("the lines could not be read")]
    Unparsable,
    /// The `what` dictionary named nothing this understands, so there was
    /// nothing to do. `setqflist()` reports that as a failure.
    #[error("nothing to set")]
    NothingToSet,
    /// `'secure'` or the sandbox forbids setting a callback. The message has
    /// already been reported.
    #[error("not allowed here")]
    Forbidden,
    /// A key could not be written into the answer dictionary. Unreachable in
    /// practice — the answer is built fresh and each key is written once —
    /// but the dictionary layer is entitled to refuse and this is where its
    /// refusal lands.
    #[error("a key could not be written")]
    KeyTaken,
    /// The errorfile would not open.
    #[error("{}", tr!("E40: Can't open errorfile {}", msg_bytes(.0.as_bytes())))]
    CannotOpen(ThinCString),
    /// A part of `'errorformat'` that does not compile.
    #[error(transparent)]
    Format(#[from] EfmError),
    /// The errorfile could not be read to its end.
    #[error("{}", tr!("E47: Error while reading errorfile"))]
    ReadError,
    /// A message already out: the regexp engine's for a format's pattern,
    /// or a `%D` with no directory (E379).
    #[error(transparent)]
    Reported(#[from] Failed),
}

// Two words, so a `Result` carrying one is no wider than it has to be.
const _: () = assert!(size_of::<QfError>() <= 16);

impl QfError {
    /// Puts the message out, for the variants that have one Vimscript sees.
    pub(crate) fn report(&self) {
        if matches!(
            self,
            Self::CannotOpen(_) | Self::Format(_) | Self::ReadError
        ) {
            emsg_text(self.to_string());
        }
    }
}

impl From<KeyTaken> for QfError {
    fn from(_: KeyTaken) -> Self {
        QfError::KeyTaken
    }
}

pub const QF_WINHEIGHT: c_uint = 10;
crate::flag_set! {
    /// Which properties `getqflist({what})` was asked for -- one bit per key
    /// of the `what` dictionary, and the same set of keys in the same order
    /// on the way out.
    pub struct GetListProps;

    const TITLE = 1;
    const ITEMS = 2;
    const NR = 4;
    const WINID = 8;
    const CONTEXT = 16;
    const ID = 32;
    const IDX = 64;
    const SIZE = 128;
    const TICK = 256;
    /// A location list's owning window. Answered for a location list only,
    /// which is why [`Self::ALL`] has to have it taken back out again for a
    /// quickfix list.
    const FILEWINID = 512;
    const QFBUFNR = 1024;
    const QFTF = 2048;

    /// What `{'all': 1}` asks for: every bit above.
    const ALL = 4095;
}
pub const INVALID_QFIDX: c_int = -1;
/// Messages more than one child reports.
pub(crate) const E_NO_MORE_ITEMS: &CStr = c"E553: No more items";
pub(crate) const E_QUICKFIX_LIST_CHANGED: &CStr = c"E925: Current quickfix list was changed";
pub(crate) const E_LOCATION_LIST_CHANGED: &CStr = c"E926: Current location list was changed";
/// The parsed global `'quickfixtextfunc'`. A list-local one lives in
/// [`QfList::text_func`].
static QFTF_CB: GlobalCell<Callback> = GlobalCell::new(Callback::None);
static QF_FILE_HL_ID: GlobalCell<c_int> = GlobalCell::new(0);
static QF_SEP_HL_ID: GlobalCell<c_int> = GlobalCell::new(0);
static QF_LINE_HL_ID: GlobalCell<c_int> = GlobalCell::new(0);
/// Which of a buffer's `b_has_qf_entry` bits an entry sets: one for the
/// quickfix stack, one for any location list.
pub const BUF_HAS_QF_ENTRY: c_int = 1;
pub const BUF_HAS_LL_ENTRY: c_int = 2;
