//! Command-line completion: working out what the text before the cursor
//! wants completed, finding the matches, and showing them.
//!
//! An [`Expand`] carries one completion from start to finish. It owns a copy
//! of the text it was worked out from, so the pattern is an offset into that
//! copy -- which is the same offset into the command line, the one caller
//! that edits a line in place. The command line moves its context out of
//! its own state for the length of each call in here, so user code that runs
//! meanwhile (a `customlist` function, a Lua `ui_attach` callback, a
//! backtick expression in a file name) sees no completion in progress
//! rather than one this code holds mutably.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
pub(crate) mod state;

use crate::global_cell::GlobalCell;
use crate::memory::XString;
use crate::types::{Expand, Pos, PumItem};
use core::ffi::c_int;

// The carve of the transpiled module; see each child's docs.
mod escape;
pub use self::escape::*;
mod expandone;
pub use self::expandone::*;
mod pum;
pub use self::pum::*;
mod showmatch;
pub use self::showmatch::*;
mod context;
pub use self::context::*;
mod cmdname;
pub use self::cmdname::*;
mod generate;
pub(crate) use self::generate::*;
mod fromcontext;
pub use self::fromcontext::*;
mod userfunc;
pub use self::userfunc::*;
mod wildkey;
pub(crate) use self::wildkey::*;
mod eval;
pub use self::eval::*;
mod bufpat;
pub(crate) use self::bufpat::*;
#[cfg(test)]
mod tests;

/// Not a `WILD_*` at all — `buffer.h`'s, and `expand_buf_names` reads it out
/// of the same `options` word, so it is spelled as one of them.
pub const BUF_DIFF_FILTER: WildOpts = WildOpts::from_bits(8192);

/// What [`expand_one`] should do — upstream's `WILD_*` *modes*, which are an
/// enumeration and not a flag set: exactly one is passed, and the value space
/// (1..=13) collides with [`WildOpts`]'s bits one for one.
///
/// c2rust gave both families the same `c_int`, so `expand_one(expand, s, o,
/// WILD_ALL, WILD_SILENT)` — arguments swapped — compiled. As an enum the
/// swap does not, and [`next_match`](expandone) can match exhaustively
/// instead of leaning on a `_` arm.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WildMode {
    /// Just release the previously expanded matches.
    Free,
    /// Expand, and do not keep the matches.
    ExpandFree,
    /// Expand, and keep the matches for a later `Next`/`Prev`.
    ExpandKeep,
    /// Step forward through the matches, wrapping around.
    Next,
    /// Step back through the matches, wrapping around.
    Prev,
    /// Answer every match, concatenated.
    All,
    /// Answer the longest part every match starts with.
    Longest,
    /// Answer every match and keep them.
    AllKeep,
    /// Close the popup menu and go back to the original text.
    Cancel,
    /// Apply the item selected in the popup menu.
    Apply,
    /// Move the selection a page towards the start.
    PageUp,
    /// Move the selection a page towards the end.
    PageDown,
    /// Select the item the UI named in `pum_want`.
    PumWant,
}

impl WildMode {
    /// Whether this mode only moves within a match list that already exists,
    /// rather than expanding anything.
    pub const fn navigates(self) -> bool {
        matches!(
            self,
            Self::Next | Self::Prev | Self::PageUp | Self::PageDown | Self::PumWant
        )
    }
}

crate::flag_set! {
    /// How an expansion should behave — upstream's `WILD_*` options, the
    /// `options` argument to [`expand_one`] and [`nextwild`].
    ///
    /// Distinct from the `mode` argument beside it, which is an enumeration
    /// ([`WildMode`]) whose values run 1..=13 and therefore collide with
    /// these bit values one for one. Nothing but the parameter name kept
    /// them apart while both were `int`.
    pub struct WildOpts;

    /// Answer a pattern that matched nothing as itself.
    const LIST_NOTFOUND = 1;
    /// Shorten a name under `$HOME` to `~/`.
    const HOME_REPLACE = 2;
    /// Separate the concatenated matches with newlines, not spaces.
    const USE_NL = 4;
    /// Do not beep when there is nothing to complete.
    const NO_BEEP = 8;
    /// Append a path separator to every directory answered.
    const ADD_SLASH = 16;
    /// Do not drop the matches `'wildignore'` and `'suffixes'` name.
    const KEEP_ALL = 32;
    /// Do not report a failure to the user.
    const SILENT = 64;
    /// Escape the answer for the command line it is going back into.
    const ESCAPE = 128;
    /// Match without regard to case.
    const ICASE = 256;
    /// Answer a dangling symbolic link as a match.
    const ALLLINKS = 512;
    /// Leave a trailing slash alone, whatever `'completeslash'` says.
    const IGNORE_COMPLETESLASH = 1024;
    /// Do not report a pattern that could not be expanded at all.
    const NOERROR = 2048;
    /// Order the buffer matches by when they were last used.
    const BUFLASTUSED = 4096;
    /// Leave the popup menu with nothing selected.
    const NOSELECT = 16384;
    /// `'wildmode'` says the pattern itself may be one of the answers.
    const MAY_EXPAND_PATTERN = 32768;
    /// The completion was asked for by `'wildmode'`'s function trigger.
    const FUNC_TRIGGER = 65536;
}
/// Which of the `:breakadd` family's arguments to offer: upstream's
/// `EXP_BREAKPT_*`.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum BreakptWhat {
    /// `:breakadd`: all four.
    Add,
    /// `:breakdel`: all but `expr`.
    Del,
    /// `:profdel`: the two that name something being profiled.
    ProfDel,
}

/// The wildmenu was drawn over scrolled message output.
pub(crate) const WM_SCROLLED: c_int = 2;
/// The wildmenu was drawn over a status line.
pub(crate) const WM_SHOWN: c_int = 1;

static cmd_showtail: GlobalCell<bool> = GlobalCell::new(false);
static may_expand_pattern: GlobalCell<bool> = GlobalCell::new(false);
static pre_incsearch_pos: GlobalCell<Pos> = GlobalCell::new(Pos {
    lnum: 0,
    col: 0,
    coladd: 0,
});
/// The popup menu's rows for the matches; `None` while there is no menu.
///
/// Each row's text points into the matches of the command line's
/// [`Expand`], which keeps them until [`cmdline_pum_remove`] drops the rows:
/// every path that frees the matches removes the menu first.
static compl_match_array: GlobalCell<Option<CmdlinePum>> = GlobalCell::new(None);
static compl_startcol: GlobalCell<c_int> = GlobalCell::new(0);
static compl_selected: GlobalCell<c_int> = GlobalCell::new(0);
/// The command line as it stood before the last expansion inserted a
/// match, for `:h getcompletion()`'s `cmdline_orig`. Owned: the cell frees
/// the previous copy when it takes a new one, and `None` is upstream's
/// null -- nothing has been expanded yet.
static cmdline_orig: GlobalCell<Option<XString>> = GlobalCell::new(None);
/// How much of `:filetype` has already been typed, and so which of its
/// arguments are still worth offering -- upstream's `EXP_FILETYPECMD_*`.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum FiletypeWhat {
    /// Nothing after `:filetype`.
    All,
    /// `indent` named; `plugin` is still on offer.
    Plugin,
    /// `plugin` named; `indent` is still on offer.
    Indent,
    /// Both named, so only `on`/`off` remain.
    OnOff,
}

static filetype_expand_what: GlobalCell<FiletypeWhat> = GlobalCell::new(FiletypeWhat::All);
static breakpt_expand_what: GlobalCell<BreakptWhat> = GlobalCell::new(BreakptWhat::Add);

/// A match `fuzzy_match_str` scored, numbered in the order it was found.
pub(crate) struct Scored {
    pub(crate) text: XString,
    pub(crate) score: c_int,
    pub(crate) idx: usize,
}

/// The scored matches best first, the order found breaking ties. With
/// `funcsort`, `<SNR>` functions sort to the end whatever they scored.
///
/// Callers number `idx` as they collect, so no two entries compare equal
/// and the sort needs no stability of its own.
pub(crate) fn fuzzy_sorted(mut found: Vec<Scored>, funcsort: bool) -> Vec<XString> {
    let snr = |m: &Scored| funcsort && m.text.first() == Some(&b'<');
    found.sort_by(|a, b| {
        snr(a)
            .cmp(&snr(b))
            .then(b.score.cmp(&a.score))
            .then(a.idx.cmp(&b.idx))
    });
    found.into_iter().map(|m| m.text).collect()
}
