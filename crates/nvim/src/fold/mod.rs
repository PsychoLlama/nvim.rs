//! Folding.
//!
//! The toplevel folds of a window live in its `w_folds` vector. Each of them
//! can hold a vector of second-level folds in `nested`, and so on: every
//! level is a `Vec<Fold>` sorted by `top`, searched with [`list::find`].
//!
//! A fold's `top` is relative to its parent, which is what makes inserting
//! and deleting lines cheap — only the folds on the changed line's own path
//! need adjusting.
//!
//! # Reaching the tree
//!
//! The tree is a field of the window record, reached through the [`Win`]
//! handle. Its pure operations — the line-number arithmetic in [`adjust`],
//! the inserts, splits and merges the update makes — take `&mut Vec<Fold>`
//! and touch nothing else, so a caller lends them the field for the length
//! of the call (`Win::with_folds`). Everything that also has to ask the
//! window, the buffer or the user something — a fold's screen-line count for
//! 'foldminlines', 'foldexpr', the syntax and diff level getters — runs
//! *between* such loans, and the walk holds positions (indices, a depth, a
//! line) rather than a reference across it, re-deriving its place afterwards.
//! That is what lets a 'foldexpr' that deletes every fold (`zE` after
//! switching to "manual") end the update instead of writing into freed
//! storage.
//!
//! The per-'foldmethod' level computations live in [`level`], the manual
//! fold commands and the open/closed state in [`open_close`], the
//! `{{{`/`}}}` marker handling in [`marker`], the fold-text rendering in
//! [`text`], line-number bookkeeping in [`adjust`], `:mkview` output in
//! [`session`], and the Vimscript builtins in [`builtins`].

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::diff::state::need_diff_redraw;
use crate::drawscreen::{UPD_NOT_VALID, find_wl_entry};
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::plines::plines_win_nofold;
use crate::state::mode::State;
use crate::types::TAB;
use crate::types::*;
use crate::winlayer::Win;
use core::ffi::{c_int, c_uint};

pub mod adjust;
mod builtins;
mod level;
mod list;
mod marker;
mod open_close;
mod session;
mod text;

pub use adjust::{
    adjust_fold_list, fold_adjust_cursor, fold_adjust_visual, fold_mark_adjust, fold_move_range,
    fold_move_to,
};
pub use builtins::{f_foldclosed, f_foldclosedend, f_foldlevel, f_foldtext, f_foldtextresult};
pub use open_close::{
    close_fold, close_fold_recurse, delete_fold, fold_check_close, fold_create,
    fold_manual_allowed, fold_open_cursor, new_fold_level, op_fold_range, open_fold,
    open_fold_recurse,
};
pub use session::put_folds;
pub(crate) use text::{FoldText, get_foldtext};

use crate::pos::MAXLNUM;
use crate::state::MODE_INSERT;

use crate::optionstr::OptStringRef;
use level::fold_update_computed;
use list::{find, find_at_or_below, fold_on_line};
use open_close::check_closed;

pub(crate) static disable_fold_update: GlobalCell<c_int> = GlobalCell::new(0 as c_int);

pub const VIRTTEXT_EMPTY: VirtText = VirtText::new();
pub const kExtmarkUndo: ExtmarkOp = 1;

/// The deepest nesting 'foldnestmax' will accept.
pub const MAX_LEVEL: c_int = 20;
/// Size of the buffer `get_foldtext` renders the default fold text into.
pub const FOLD_TEXT_LEN: c_uint = 51;

/// What `set_manual_fold` and friends did, so the caller knows whether to
/// keep looking for a fold to act on.
pub const DONE_NOTHING: c_int = 0;
pub const DONE_ACTION: c_int = 1;
pub const DONE_FOLD: c_int = 2;

/// The `amount` `mark_adjust` passes to mean "these lines are gone".
const LINES_DELETED: LineNr = MAXLNUM;

/// Whether a fold is drawn open or closed, or takes its state from
/// 'foldlevel' — upstream's `FD_OPEN`, `FD_CLOSED` and `FD_LEVEL`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FoldState {
    Open,
    Closed,
    Level,
}

#[derive(Clone, Debug)]
pub struct Fold {
    /// First line of the fold; relative to the parent for a nested fold.
    pub top: LineNr,
    /// Number of lines in the fold.
    pub len: LineNr,
    /// The folds nested inside this one.
    pub nested: Vec<Fold>,
    /// Drawn open, closed, or as 'foldlevel' says.
    pub state: FoldState,
    /// Whether the fold is smaller than 'foldminlines'. `None` means "not
    /// worked out yet", and applies to the nested folds too.
    pub small: Option<bool>,
}

impl Fold {
    /// A fold over `top..top + len`, open, with nothing inside.
    pub fn new(top: LineNr, len: LineNr) -> Self {
        Self {
            top,
            len,
            nested: Vec::new(),
            state: FoldState::Open,
            small: None,
        }
    }

    /// The fold's last line, in the same frame as [`Fold::top`].
    pub fn last(&self) -> LineNr {
        self.top + self.len - 1
    }

    /// Whether the fold is drawn as `state` says.
    pub fn is(&self, state: FoldState) -> bool {
        self.state == state
    }
}

/// What the per-'foldmethod' level computations are handed, and what they
/// answer in.
pub struct FoldLine {
    /// The window whose folds are being computed, taken live by
    /// `fold_update`; the computation is one call.
    pub win: Win,
    /// Current line number.
    pub lnum: LineNr,
    /// Offset between `lnum` and the real line number.
    pub off: LineNr,
    /// Line number used by `fold_update_computed_recurse`.
    pub lnum_save: LineNr,
    /// Current level; -1 for undefined.
    pub lvl: c_int,
    /// Level to use for the next line.
    pub lvl_next: c_int,
    /// Number of folds forced to start at this line.
    pub start: c_int,
    /// Level of the fold forced to end below this line.
    pub end: c_int,
    /// Level of the fold forced to end above this line — the previous line's
    /// `end`.
    pub had_end: c_int,
}

/// Set when the folds changed and the window needs redrawing.
static fold_changed: GlobalCell<bool> = GlobalCell::new(false);
static e_nofold: &::core::ffi::CStr = c"E490: No fold found";

/// While the folds are being updated, the lines between `invalid_top` and
/// `invalid_bot` have an undefined fold level. Only meaningful for the window
/// currently being updated.
static invalid_top: GlobalCell<LineNr> = GlobalCell::new(0);
static invalid_bot: GlobalCell<LineNr> = GlobalCell::new(0);

/// With 'foldexpr' we sometimes ask for the level of the *next* line, which
/// calls `foldlevel()` for the current one — which has not been stored yet.
/// The previous line's level is parked here to break that cycle;
/// `prev_lnum` is zero when there is nothing to offer.
static prev_lnum: GlobalCell<LineNr> = GlobalCell::new(0);
static prev_lnum_lvl: GlobalCell<c_int> = GlobalCell::new(-1);

/// 'foldmarker' split into its two halves, refreshed by `parse_marker`.
static foldstartmarkerlen: GlobalCell<usize> = GlobalCell::new(0);
static foldendmarkerlen: GlobalCell<usize> = GlobalCell::new(0);

/// Copy the folding state from window `wp_from` to window `wp_to`.
pub fn copy_folding_state(wp_from: Win, mut wp_to: Win) {
    wp_to.w_fold_manual = wp_from.w_fold_manual;
    wp_to.w_foldinvalid = wp_from.w_foldinvalid;
    // Cloned before the assignment, so the two windows may be one.
    let folds = wp_from.w_folds.clone();
    wp_to.w_folds = folds;
}

/// The fold questions a caller asks *of a window*, as [`Win`] methods.
///
/// They live here rather than in `winlayer` for the reason that module's docs
/// give: a family hangs its own projections off `impl Win`, so the shared
/// module stays the minimum. Each is one call to [`has_folding`] with the
/// out-parameters read back.
impl Win {
    /// First line of the fold containing `lnum`, if there is one.
    #[inline(always)]
    pub fn fold_first(self, lnum: LineNr) -> Option<LineNr> {
        let mut first = lnum;
        // `firstp` is written only when the answer is true, so the seed
        // survives a line that is in no fold.
        let folded = has_folding(self, lnum, Some(&mut first), None);
        folded.then_some(first)
    }

    /// Last line of the fold containing `lnum`, or `lnum` when it is in none.
    #[inline(always)]
    pub fn fold_last(self, lnum: LineNr) -> LineNr {
        let mut last = lnum;
        // `lastp` is written only when folded.
        has_folding(self, lnum, None, Some(&mut last));
        last
    }

    /// Last line of the fold containing `lnum`, `None` when it is in none --
    /// [`Win::fold_first`]'s partner at the other end.
    #[inline(always)]
    pub fn fold_end(self, lnum: LineNr) -> Option<LineNr> {
        let (folded, _, last) = self.fold_span(lnum);
        folded.then_some(last)
    }

    /// The whole fold containing `lnum`: whether there is one, and its first
    /// and last line (both `lnum` when there is not).
    #[inline(always)]
    pub fn fold_span(self, lnum: LineNr) -> (bool, LineNr, LineNr) {
        let (mut first, mut last) = (lnum, lnum);
        // Both out-params are written only when folded.
        let folded = has_folding(self, lnum, Some(&mut first), Some(&mut last));
        (folded, first, last)
    }

    #[inline(always)]
    pub fn has_any_folding(self) -> bool {
        has_any_folding(self) != 0
    }

    /// Lend the window's toplevel fold list to `f`.
    ///
    /// The loan is the field and nothing else: `f` must not reach the window
    /// (or anything that reads its folds) while it holds the list, which is
    /// why the closures handed to this are the pure tree operations and why
    /// every option they need is read before the call.
    #[inline(always)]
    fn with_folds<R>(self, f: impl FnOnce(&mut Vec<Fold>) -> R) -> R {
        let mut window = self;
        f(&mut window.w_folds)
    }
}

/// Returns true if there may be folded lines in window "win".
///
/// A window on screen always shows a buffer, which is what reading
/// `terminal` through [`Win::buffer`] leans on -- as the C does.
pub fn has_any_folding(win: Win) -> c_int {
    (win.buffer().terminal.is_null()
        && win.w_onebuf_opt.wo_fen != 0
        && (!foldmethod_is_manual(win) || !win.w_folds.is_empty())) as c_int
}

/// When returning true, `firstp` and `lastp` are set to the first and last
/// lnum of the sequence of folded lines (each skipped when `None`).
///
/// Returns true if line "lnum" in window "win" is part of a closed fold.
pub fn has_folding(
    win: Win,
    lnum: LineNr,
    firstp: Option<&mut LineNr>,
    lastp: Option<&mut LineNr>,
) -> bool {
    has_folding_win(win, lnum, firstp, lastp, true, None)
}

/// Where [`has_folding_win`]'s walk down the tree has got to.
struct ClosedWalk {
    /// `lnum` relative to the list the walk is in.
    lnum_rel: LineNr,
    /// Depth of that list; 0 is the window's toplevel.
    level: c_int,
    /// The shallowest level whose fold starts at `lnum`, plus one; 0 for none.
    low_level: c_int,
    use_level: bool,
    maybe_small: bool,
    /// The absolute first and last line of the fold the walk stands in.
    first: LineNr,
    last: LineNr,
}

/// What a stretch of [`ClosedWalk::descend`] ended on.
enum WalkEnd {
    /// The walk ran out of folds containing the line, or found it closed
    /// (`closed`).
    Done { closed: bool },
    /// A closed fold whose 'foldminlines' answer needs its screen lines
    /// counted, which needs the window: the walk lets go of the tree first.
    /// `top` is relative to the list the walk is in, as [`Fold::top`] is.
    CountRows { top: LineNr, len: LineNr },
}

impl ClosedWalk {
    /// Walk down from depth `self.level` towards `lnum` until a fold says
    /// "closed here", accumulating each fold's `top` — relative to its parent
    /// — into the absolute `first`/`last`.
    #[inline(always)]
    fn descend(
        &mut self,
        root: &mut [Fold],
        lnum: LineNr,
        foldlevel: OptInt,
        foldminlines: OptInt,
    ) -> WalkEnd {
        let mut folds = root;
        if self.level > 0 {
            // Resuming after a row count: find the list again.
            let mut lnum_rel = lnum;
            for _ in 0..self.level {
                let Ok(i) = find(folds, lnum_rel) else {
                    return WalkEnd::Done { closed: false };
                };
                lnum_rel -= folds[i].top;
                folds = &mut folds[i].nested;
            }
        }
        while let Ok(i) = find(folds, self.lnum_rel) {
            let fold = &mut folds[i];
            if self.lnum_rel == fold.top && self.low_level == 0 {
                self.low_level = self.level + 1;
            }
            self.first += fold.top;
            self.last += fold.top;
            let verdict = check_closed(
                fold,
                &mut self.use_level,
                self.level,
                &mut self.maybe_small,
                foldlevel,
                foldminlines,
            );
            match verdict {
                Closed::Yes => {
                    self.last += fold.len - 1;
                    return WalkEnd::Done { closed: true };
                }
                Closed::CountRows => {
                    return WalkEnd::CountRows {
                        top: fold.top,
                        len: fold.len,
                    };
                }
                Closed::No => {}
            }
            self.lnum_rel -= fold.top;
            self.level += 1;
            folds = &mut fold.nested;
        }
        WalkEnd::Done { closed: false }
    }
}

/// Search folds starting at lnum
/// `lnum` — first line to search
/// `firstp` — first line of fold containing lnum
/// `lastp` — last line with a fold
/// `cache` — when true: use cached values of window
/// `infop` — where to store fold info
///
/// Returns true if range contains folds
///
pub fn has_folding_win(
    win: Win,
    lnum: LineNr,
    firstp: Option<&mut LineNr>,
    lastp: Option<&mut LineNr>,
    cache: bool,
    mut infop: Option<&mut FoldInfo>,
) -> bool {
    checkupdate(win);
    if has_any_folding(win) == 0 {
        if let Some(info) = infop.as_mut() {
            info.fi_level = 0;
        }
        return false;
    }
    let mut had_folded = false;
    let mut first: LineNr = 0;
    let mut last: LineNr = 0;
    if cache && let Some(entry) = find_wl_entry(win, lnum) {
        first = entry.wl_lnum;
        last = entry.wl_foldend;
        had_folded = entry.wl_folded;
    }
    let mut walk = ClosedWalk {
        lnum_rel: lnum,
        level: 0,
        low_level: 0,
        use_level: false,
        maybe_small: false,
        first,
        last,
    };
    if first == 0 {
        let (foldlevel, foldminlines) = (win.w_onebuf_opt.wo_fdl, win.w_onebuf_opt.wo_fml);
        loop {
            let end = win.with_folds(|root| walk.descend(root, lnum, foldlevel, foldminlines));
            match end {
                WalkEnd::Done { closed } => {
                    had_folded = closed;
                    break;
                }
                WalkEnd::CountRows { top, len } => {
                    let lnum_off = lnum - walk.lnum_rel;
                    let small = fits_in_rows(win, top + lnum_off, len, foldminlines);
                    let level = usize::try_from(walk.level).unwrap_or(0);
                    win.with_folds(|root| {
                        if let Some(fold) = fold_on_line(root, lnum, level) {
                            fold.small = Some(small);
                        }
                    });
                    if !small {
                        walk.last += len - 1;
                        had_folded = true;
                        break;
                    }
                    // 'foldminlines' vetoes it: look further down.
                    walk.lnum_rel -= top;
                    walk.level += 1;
                }
            }
        }
        first = walk.first;
        last = walk.last;
    }
    let ClosedWalk {
        lnum_rel,
        level,
        low_level,
        ..
    } = walk;
    if !had_folded {
        if let Some(info) = infop.as_mut() {
            info.fi_level = level;
            info.fi_lnum = lnum - lnum_rel;
            info.fi_low_level = if low_level == 0 { level } else { low_level };
        }
        return false;
    }
    last = last.min(win.buffer().b_ml.ml_line_count);
    if let Some(lastp) = lastp {
        *lastp = last;
    }
    if let Some(firstp) = firstp {
        *firstp = first;
    }
    if let Some(info) = infop.as_mut() {
        info.fi_level = level + 1;
        info.fi_lnum = first;
        info.fi_low_level = if low_level == 0 { level + 1 } else { low_level };
    }
    true
}

/// Returns fold level at line number "lnum" in the current window.
fn fold_level(lnum: LineNr) -> c_int {
    let win = Win::current();
    if invalid_top.get() == 0 {
        checkupdate(win);
    } else if lnum == prev_lnum.get() && prev_lnum_lvl.get() >= 0 {
        return prev_lnum_lvl.get();
    } else if lnum >= invalid_top.get() && lnum <= invalid_bot.get() {
        return -1;
    }
    if has_any_folding(win) == 0 {
        return 0;
    }
    fold_level_win(win, lnum)
}

/// Low level function to check if a line is folded.  Doesn't use any caching.
///
/// Returns true if line is folded or,
///          false if line is not folded.
///
pub fn line_folded(win: Win, lnum: LineNr) -> bool {
    fold_info(win, lnum).fi_lines != 0
}

/// Count the number of lines that are folded at line number "lnum".
/// Normally "lnum" is the first line of a possible fold, and the returned
/// number is the number of lines in the fold.
/// Doesn't use caching from the displayed window.
///
/// Returns with the fold level info.
///         fi_lines = number of folded lines from "lnum",
///                    or 0 if line is not folded.
///
pub fn fold_info(win: Win, lnum: LineNr) -> FoldInfo {
    let mut info = FoldInfo {
        fi_lnum: 0,
        fi_level: 0,
        fi_low_level: 0,
        fi_lines: 0,
    };
    let mut last: LineNr = 0;
    let folded = has_folding_win(win, lnum, None, Some(&mut last), false, Some(&mut info));
    info.fi_lines = if folded { last - lnum + 1 } else { 0 };
    info
}

/// Whether 'foldmethod' in `window` is non-empty and carries `c` at index `at`.
///
/// The six predicates below are each one such test: the six values
/// ("manual", "indent", "expr", "marker", "syntax", "diff") are told apart by
/// a single byte in the first three, which is what upstream leans on too.
///
fn foldmethod_byte_is(window: Win, at: usize, c: u8) -> bool {
    // Every legal 'foldmethod' is at least four bytes long, so a non-empty
    // value always has a byte at `at <= 3`.
    window.w_onebuf_opt.wo_fdm.bytes().get(at) == Some(&c)
}

/// Returns true if 'foldmethod' is "manual"
pub fn foldmethod_is_manual(window: Win) -> bool {
    foldmethod_byte_is(window, 3, b'u')
}

/// Returns true if 'foldmethod' is "indent"
pub fn foldmethod_is_indent(window: Win) -> bool {
    foldmethod_byte_is(window, 0, b'i')
}

/// Returns true if 'foldmethod' is "expr"
pub fn foldmethod_is_expr(window: Win) -> bool {
    foldmethod_byte_is(window, 1, b'x')
}

/// Returns true if 'foldmethod' is "marker"
pub fn foldmethod_is_marker(window: Win) -> bool {
    foldmethod_byte_is(window, 2, b'r')
}

/// Returns true if 'foldmethod' is "syntax"
pub fn foldmethod_is_syntax(window: Win) -> bool {
    foldmethod_byte_is(window, 0, b's')
}

/// Returns true if 'foldmethod' is "diff"
pub fn foldmethod_is_diff(window: Win) -> bool {
    foldmethod_byte_is(window, 0, b'd')
}

/// Remove all folding for window "win".
///
pub fn clear_folding(mut win: Win) {
    win.w_folds = Vec::new();
    win.w_foldinvalid = false;
}

/// Update folds for changes in the buffer of a window.
/// Note that inserted/deleted lines must have already been taken care of by
/// calling fold_mark_adjust().
/// The changes in lines from top to bot (inclusive).
///
pub fn fold_update(window: Win, top: LineNr, bot: LineNr) {
    if disable_fold_update.get() != 0
        || State.get() & MODE_INSERT != 0 && !foldmethod_is_indent(window)
    {
        return;
    }
    if need_diff_redraw.get() {
        return;
    }
    window.with_folds(|folds| {
        // Every fold that starts inside the changed range may have changed
        // size, so its 'foldminlines' answer has to be worked out again.
        let (start, end) = (top.min(bot), top.max(bot));
        let first = find_at_or_below(folds, start);
        for fold in folds[first..].iter_mut().take_while(|fold| fold.top <= end) {
            fold.small = None;
        }
    });
    if foldmethod_is_indent(window)
        || foldmethod_is_expr(window)
        || foldmethod_is_marker(window)
        || foldmethod_is_diff(window)
        || foldmethod_is_syntax(window)
    {
        // `fold_update_computed` runs 'foldexpr', which the user can
        // interrupt; a CTRL-C there must not leak out into whatever the
        // caller was doing.
        let save_got_int = got_int.get();
        got_int.set(false);
        fold_update_computed(window, top, bot);
        got_int.set(got_int.get() | save_got_int);
    }
}

/// Updates folds when leaving insert-mode.
pub fn fold_update_after_insert() {
    let win = Win::current();
    if foldmethod_is_manual(win) || foldmethod_is_syntax(win) || foldmethod_is_expr(win) {
        return;
    }
    fold_update_all(win);
    fold_open_cursor();
}

/// Update all lines in a window for folding.
/// Used when a fold setting changes or after reloading the buffer.
/// The actual updating is postponed until fold info is used, to avoid doing
/// every time a setting is changed or a syntax item is added.
///
pub fn fold_update_all(mut win: Win) {
    win.w_foldinvalid = true;
    win.redraw_later(UPD_NOT_VALID);
}

/// Returns fold level at line number "lnum" in window "wp".
///
fn fold_level_win(window: Win, lnum: LineNr) -> c_int {
    let mut folds: &[Fold] = &window.w_folds;
    let mut lnum_rel = lnum;
    let mut level = 0;
    while let Ok(i) = find(folds, lnum_rel) {
        let fold = &folds[i];
        lnum_rel -= fold.top;
        level += 1;
        folds = &fold.nested;
    }
    level
}

/// Check if the folds in window "wp" are invalid and update them if needed.
///
fn checkupdate(mut window: Win) {
    if !window.w_foldinvalid {
        return;
    }
    fold_update(window, 1, MAXLNUM);
    window.w_foldinvalid = false;
}

/// Delete fold "idx" from fold list "folds".
///
/// `recursive` — when true, also delete all the folds contained in it.
///                   when false, contained folds are moved one level up.
fn delete_fold_entry(folds: &mut Vec<Fold>, idx: usize, recursive: bool) {
    let mut fold = folds.remove(idx);
    if recursive || fold.nested.is_empty() {
        return;
    }
    // Not recursive and it has children: the children take its place.
    let mut children = core::mem::take(&mut fold.nested);
    for child in &mut children {
        // The children were relative to the fold that is going away.
        child.top += fold.top;
        if fold.is(FoldState::Level) {
            child.state = FoldState::Level;
        }
        if fold.small.is_none() {
            child.small = None;
        }
    }
    folds.splice(idx..idx, children);
}

/// Get the lowest 'foldlevel' value that makes the deepest nested fold in
/// window `window`.
///
pub fn deepest_fold_nesting(window: Win) -> c_int {
    checkupdate(window);
    deepest_nesting_of(&window.w_folds)
}

/// How many levels of fold `folds` holds, counting itself.
fn deepest_nesting_of(folds: &[Fold]) -> c_int {
    folds
        .iter()
        .map(|fold| deepest_nesting_of(&fold.nested) + 1)
        .max()
        .unwrap_or(0)
}

/// Whether the `len` lines from `first` take no more than `foldminlines`
/// screen lines in `window` — the half of a fold's `small` answer that is
/// not a function of the tree: 'foldminlines' counts *screen* lines, so
/// `plines_win_nofold` makes it depend on the window's width and 'wrap' too.
fn fits_in_rows(window: Win, first: LineNr, len: LineNr, foldminlines: OptInt) -> bool {
    let mut count = 0;
    for n in 0..len {
        count += plines_win_nofold(window, first + n);
        if OptInt::from(count) > foldminlines {
            return false;
        }
    }
    true
}

/// Forget the `small` answer for every fold in `folds`, so the next
/// [`check_closed`] works it out again.
fn forget_small_flags(folds: &mut [Fold]) {
    for fold in folds {
        fold.small = None;
    }
}

/// [`check_closed`]'s answer.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Closed {
    No,
    Yes,
    /// Closed by its flags, but whether 'foldminlines' vetoes it depends on
    /// how many screen lines it takes: the caller counts them with
    /// [`fits_in_rows`] and stores the answer in the fold's `small`.
    CountRows,
}
