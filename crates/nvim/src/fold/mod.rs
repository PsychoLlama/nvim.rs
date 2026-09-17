//! Folding.
//!
//! The toplevel folds of a window live in its `w_folds` growarray. Each of
//! them can hold an array of second-level folds in `fd_nested`, and so on:
//! every level is the same `GArray` of [`Fold`], so the whole tree is
//! reached through [`list`]'s [`FoldList`] and [`FoldRef`] handles.
//!
//! A fold's `fd_top` is relative to its parent, which is what makes
//! inserting and deleting lines cheap — only the folds on the changed line's
//! own path need adjusting.
//!
//! The per-'foldmethod' level computations live in [`level`], the manual
//! fold commands and the open/closed state in [`open_close`], the
//! `{{{`/`}}}` marker handling in [`marker`], the fold-text rendering in
//! [`text`], line-number bookkeeping in [`adjust`], `:mkview` output in
//! [`session`], and the Vimscript builtins in [`builtins`].

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::diff::state::need_diff_redraw;
use crate::drawscreen::UPD_NOT_VALID;
use crate::garray::{ga_clear, ga_grow, ga_init};
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::memory::xfree;
use crate::plines::plines_win_nofold;
use crate::state::mode::State;
use crate::types::TAB;
use crate::types::*;
use crate::winlayer::Win;
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::{ptr, slice};

pub mod adjust;
mod builtins;
mod level;
mod list;
mod marker;
mod open_close;
mod session;
mod text;

pub use adjust::{
    fold_adjust_cursor, fold_adjust_visual, fold_mark_adjust, fold_move_range, fold_move_to,
};
pub use builtins::{f_foldclosed, f_foldclosedend, f_foldlevel, f_foldtext, f_foldtextresult};
pub use open_close::{
    close_fold, close_fold_recurse, delete_fold, fold_check_close, fold_create,
    fold_manual_allowed, fold_open_cursor, new_fold_level, op_fold_range, open_fold,
    open_fold_recurse,
};
pub use session::put_folds;
pub use text::get_foldtext;

use crate::pos::MAXLNUM;
use crate::state::MODE_INSERT;

use crate::optionstr::LocalOptStr;
use level::fold_update_computed;
use list::{FLine, FoldList, FoldRef};
use open_close::check_closed;

pub(crate) static disable_fold_update: GlobalCell<c_int> = GlobalCell::new(0 as c_int);

pub const VIRTTEXT_EMPTY: VirtText = VirtText {
    size: 0,
    capacity: 0,
    items: ptr::null_mut(),
};
pub const kExtmarkUndo: ExtmarkOp = 1;

/// The deepest nesting 'foldnestmax' will accept.
pub const MAX_LEVEL: c_int = 20;
/// Size of the buffer `get_foldtext` renders the default fold text into.
pub const FOLD_TEXT_LEN: c_uint = 51;

/// `Fold::fd_flags` — whether a fold is drawn open, closed, or takes its
/// state from 'foldlevel'.
pub const FD_OPEN: c_int = 0;
pub const FD_CLOSED: c_int = 1;
pub const FD_LEVEL: c_int = 2;

/// What `set_manual_fold` and friends did, so the caller knows whether to
/// keep looking for a fold to act on.
pub const DONE_NOTHING: c_int = 0;
pub const DONE_ACTION: c_int = 1;
pub const DONE_FOLD: c_int = 2;

/// The `amount` `mark_adjust` passes to mean "these lines are gone".
const LINES_DELETED: LineNr = MAXLNUM;

/// One of the six per-'foldmethod' level computations in [`level`].
pub(in crate::fold) type LevelGetter = Option<unsafe fn(FLine) -> ()>;

#[derive(Copy, Clone)]
pub struct Fold {
    /// First line of the fold; relative to the parent for a nested fold.
    pub fd_top: LineNr,
    /// Number of lines in the fold.
    pub fd_len: LineNr,
    /// The folds nested inside this one.
    pub fd_nested: GArray,
    /// `FD_OPEN`, `FD_CLOSED` or `FD_LEVEL`.
    pub fd_flags: c_char,
    /// Whether the fold is smaller than 'foldminlines'. `None` means "not
    /// worked out yet", and applies to the nested folds too.
    pub fd_small: Option<bool>,
}

/// What the per-'foldmethod' level computations are handed, and what they
/// answer in.
pub struct FoldLine {
    pub wp: *mut Window,
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
static foldstartmarkerlen: GlobalCell<size_t> = GlobalCell::new(0);
static foldendmarker: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());
static foldendmarkerlen: GlobalCell<size_t> = GlobalCell::new(0);

/// A window's toplevel fold list.
fn window_folds(mut window: Win) -> FoldList {
    // SAFETY: `w_folds` is initialised by `fold_init_win` when the window is
    // allocated and lives as long as the window does.
    unsafe { FoldList::new(&raw mut window.w_folds) }
}

/// Copy the folding state from window `wp_from` to window `wp_to`.
pub fn copy_folding_state(mut wp_from: Win, mut wp_to: Win) {
    wp_to.w_fold_manual = wp_from.w_fold_manual;
    wp_to.w_foldinvalid = wp_from.w_foldinvalid;
    // SAFETY: the caller's promise about `wp_to`; both fold lists live inside
    // their own window.
    unsafe { clone_fold_list(&raw mut wp_from.w_folds, &raw mut wp_to.w_folds) };
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
}

/// Returns true if there may be folded lines in window "win".
///
/// A window on screen always shows a buffer, which is what reading
/// `terminal` through [`Win::buffer`] leans on -- as the C does.
pub fn has_any_folding(win: Win) -> c_int {
    (win.buffer().terminal.is_null()
        && win.w_onebuf_opt.wo_fen != 0
        && (!foldmethod_is_manual(win) || win.w_folds.ga_len > 0)) as c_int
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
    if cache {
        let x = find_wl_entry(win, lnum);
        if x >= 0 {
            // SAFETY: `find_wl_entry` only ever answers with an index of a
            // valid `w_lines` entry.
            let entry = unsafe { &*win.w_lines.offset(x as isize) };
            first = entry.wl_lnum;
            last = entry.wl_foldend;
            had_folded = entry.wl_folded;
        }
    }
    let mut lnum_rel = lnum;
    let mut level = 0;
    let mut low_level = 0;
    let mut maybe_small = false;
    let mut use_level = false;
    if first == 0 {
        let mut folds = window_folds(win);
        // Walk down the tree until a level says "closed here", accumulating
        // each fold's `fd_top` — which is relative to its parent — into the
        // absolute `first`.
        while let Ok(i) = folds.find(lnum_rel) {
            let fold = folds.at(i);
            if lnum_rel == fold.top() && low_level == 0 {
                low_level = level + 1;
            }
            first += fold.top();
            last += fold.top();
            had_folded = check_closed(
                win,
                fold,
                &mut use_level,
                level,
                &mut maybe_small,
                lnum - lnum_rel,
            );
            if had_folded {
                last += fold.len() - 1;
                break;
            }
            folds = fold.nested();
            lnum_rel -= fold.top();
            level += 1;
        }
    }
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
    // SAFETY: a live window's `w_folds` is a live fold list.
    unsafe { delete_fold_recurse(&raw mut win.w_folds) };
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
    let folds = window_folds(window);
    if !folds.is_empty() {
        // Every fold that starts inside the changed range may have changed
        // size, so its 'foldminlines' answer has to be worked out again.
        let (start, end) = (top.min(bot), top.max(bot));
        let first = match folds.find(start) {
            Ok(i) | Err(i) => i,
        };
        for fold in (first..folds.len())
            .map(|i| folds.at(i))
            .take_while(|fold| fold.top() <= end)
        {
            fold.set_small(None);
        }
    }
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

/// Init the fold info in a new window.
pub fn fold_init_win(mut new_win: Win) {
    // SAFETY: the caller's promise. This is the call that makes `w_folds` a
    // fold list, i.e. the one every `FoldList::new` leans on.
    unsafe { ga_init(&raw mut new_win.w_folds, size_of::<Fold>() as c_int, 10) };
}

/// Find an entry in the win->w_lines[] array for buffer line "lnum".
/// Only valid entries are considered (for entries where wl_valid is false the
/// line number can be wrong).
///
/// Returns index of entry or -1 if not found.
///
pub fn find_wl_entry(win: Win, lnum: LineNr) -> c_int {
    let valid = win.w_lines_valid;
    // SAFETY: a live window's `w_lines` holds at least `w_lines_valid`
    // initialised entries, and is only null while that count is zero.
    let lines = unsafe {
        if valid <= 0 {
            &[][..]
        } else {
            slice::from_raw_parts(win.w_lines, valid as usize)
        }
    };
    for (i, entry) in lines.iter().enumerate() {
        if entry.wl_valid {
            if lnum < entry.wl_lnum {
                return -1;
            }
            if lnum <= entry.wl_foldend {
                return i as c_int;
            }
        }
    }
    -1
}

/// Will "clone" (i.e deep copy) a GArray of folds.
///
/// # Safety
/// `from` must be a live fold list; `to` must be writable and is
/// re-initialised, so anything it held is leaked.
pub unsafe fn clone_fold_list(from: *mut GArray, to: *mut GArray) {
    // SAFETY: the caller's promise; `ga_init` is what makes `to` a fold list.
    let (src, dst) = unsafe {
        ga_init(to, (*from).ga_itemsize, (*from).ga_growsize);
        (FoldList::new(from), FoldList::new(to))
    };
    if src.is_empty() {
        return;
    }
    // SAFETY: `to` is a fold list, as of the `ga_init` above.
    unsafe { ga_grow(to, src.len()) };
    for (i, fold) in src.folds().enumerate() {
        let copy = dst.at(i as c_int);
        copy.set_top(fold.top());
        copy.set_len(fold.len());
        copy.set_flags(fold.flags());
        copy.set_small(fold.small());
        // SAFETY: the source fold's nested list is live, and the destination
        // entry is the zeroed storage `ga_grow` just handed out.
        unsafe { clone_fold_list(fold.nested().gap(), copy.nested().gap()) };
        // Grown one at a time so an interrupted clone still frees cleanly.
        dst.set_len(dst.len() + 1);
    }
}

/// Returns fold level at line number "lnum" in window "wp".
///
fn fold_level_win(window: Win, lnum: LineNr) -> c_int {
    let mut folds = window_folds(window);
    let mut lnum_rel = lnum;
    let mut level = 0;
    while let Ok(i) = folds.find(lnum_rel) {
        let fold = folds.at(i);
        lnum_rel -= fold.top();
        level += 1;
        folds = fold.nested();
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
///
/// # Safety
/// `idx` must name an entry of `folds`.
unsafe fn delete_fold_entry(folds: FoldList, idx: c_int, recursive: bool) {
    let fold = folds.at(idx);
    if recursive || fold.nested().is_empty() {
        // SAFETY: `fd_nested` of a live fold is a live fold list.
        unsafe { delete_fold_recurse(fold.nested().gap()) };
        folds.set_len(folds.len() - 1);
        if idx < folds.len() {
            // SAFETY: the entries above `idx` slide down over the hole, and
            // `ga_len` has already been shortened by the one that went.
            unsafe {
                ptr::copy(
                    fold.entry().add(1),
                    fold.entry(),
                    (folds.len() - idx) as usize,
                )
            };
        }
        return;
    }
    // Not recursive and it has children: the children take its place, so the
    // list grows by `moved - 1`.
    let moved = fold.nested().len();
    // SAFETY: a live fold list.
    unsafe { ga_grow(folds.gap(), moved - 1) };
    // `ga_grow` may have moved the storage, so re-derive the entry.
    let fold = folds.at(idx);
    let nested = fold.nested();
    let children = nested.at(0).entry();
    for child in nested.folds() {
        // The children were relative to the fold that is going away.
        child.set_top(child.top() + fold.top());
        if fold.is(FD_LEVEL) {
            child.set_flags(FD_LEVEL);
        }
        if fold.small().is_none() {
            child.set_small(None);
        }
    }
    // SAFETY: both moves stay inside the array `ga_grow` just sized for
    // `moved - 1` more entries; the second reads the nested array, which is a
    // separate allocation.
    if idx + 1 < folds.len() {
        unsafe {
            ptr::copy(
                fold.entry().add(1),
                fold.entry().add(moved as usize),
                (folds.len() - (idx + 1)) as usize,
            )
        };
    }
    unsafe { ptr::copy(children, fold.entry(), moved as usize) };
    unsafe { xfree(children as *mut c_void) };
    folds.set_len(folds.len() + moved - 1);
}

/// Free `gap` and every fold nested inside it.
///
/// # Safety
/// `gap` must be a live fold list. It is left empty, not freed.
pub unsafe fn delete_fold_recurse(gap: *mut GArray) {
    // SAFETY: the caller's promise.
    let folds = unsafe { FoldList::new(gap) };
    if folds.has_data() {
        for fold in folds.folds() {
            // SAFETY: `fd_nested` of a live fold is a live fold list.
            unsafe { delete_fold_recurse(fold.nested().gap()) };
        }
    }
    // SAFETY: the caller's promise.
    unsafe { ga_clear(gap) };
}

/// Get the lowest 'foldlevel' value that makes the deepest nested fold in
/// window `window`.
///
pub fn deepest_fold_nesting(window: Win) -> c_int {
    checkupdate(window);
    deepest_nesting_of(window_folds(window))
}

/// How many levels of fold `folds` holds, counting itself.
fn deepest_nesting_of(folds: FoldList) -> c_int {
    folds
        .folds()
        .map(|fold| deepest_nesting_of(fold.nested()) + 1)
        .max()
        .unwrap_or(0)
}

/// Work out `fold`'s `fd_small` — whether it is shorter than 'foldminlines',
/// and so should be drawn open however it is flagged.
///
/// This is the one answer in `fold/` that is not a function of the tree:
/// 'foldminlines' counts *screen* lines, so `plines_win_nofold` makes it
/// depend on the window's width and 'wrap' too.
///
/// `lnum_off` — offset for fold->top()
fn check_small(window: Win, fold: FoldRef, lnum_off: LineNr) {
    if fold.small().is_some() {
        return;
    }
    forget_small_flags(fold.nested());
    let foldminlines = window.w_onebuf_opt.wo_fml;
    if fold.len() as OptInt > foldminlines {
        fold.set_small(Some(false));
        return;
    }
    let mut count = 0;
    for n in 0..fold.len() {
        count += plines_win_nofold(window, fold.top() + lnum_off + n);
        if count as OptInt > foldminlines {
            fold.set_small(Some(false));
            return;
        }
    }
    fold.set_small(Some(true));
}

/// Forget the `fd_small` answer for every fold in `folds`, so the next
/// `check_small` works it out again.
fn forget_small_flags(folds: FoldList) {
    for fold in folds.folds() {
        fold.set_small(None);
    }
}
