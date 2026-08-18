//! Folding.
//!
//! The toplevel folds of a window live in its `w_folds` growarray. Each of
//! them can hold an array of second-level folds in `fd_nested`, and so on:
//! every level is the same `garray_T` of [`fold_T`], so the whole tree is
//! reached through [`folds`] and friends.
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

use crate::drawscreen::{UPD_NOT_VALID, redraw_later};
use crate::garray::{ga_clear, ga_grow, ga_init};
use crate::global_cell::GlobalCell;
use crate::main::{State, curwin, disable_fold_update, got_int, need_diff_redraw};
use crate::memory::xfree;
use crate::os::cshim::memmove;
use crate::plines::plines_win_nofold;
use crate::types::*;
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;

pub mod adjust;
mod builtins;
mod level;
mod marker;
mod open_close;
mod session;
mod text;

pub use adjust::{foldAdjustCursor, foldAdjustVisual, foldMarkAdjust, foldMoveRange, foldMoveTo};
pub use builtins::{f_foldclosed, f_foldclosedend, f_foldlevel, f_foldtext, f_foldtextresult};
pub use open_close::{
    closeFold, closeFoldRecurse, deleteFold, foldCheckClose, foldCreate, foldManualAllowed,
    foldOpenCursor, newFoldLevel, opFoldRange, openFold, openFoldRecurse,
};
pub use session::put_folds;
pub use text::get_foldtext;

use crate::pos::MAXLNUM;
use crate::state::MODE_INSERT;
use crate::types::{kFalse, kNone, kTrue};
use level::foldUpdateIEMS;
use open_close::check_closed;

pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
pub const NUL: c_int = 0;
pub const TAB: c_int = '\t' as c_int;
pub const VIRTTEXT_EMPTY: VirtText = VirtText {
    size: 0,
    capacity: 0,
    items: ptr::null_mut(),
};
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kWinOptFoldtext: c_int = 22;

/// The deepest nesting 'foldnestmax' will accept.
pub const MAX_LEVEL: c_int = 20;
/// Size of the buffer `get_foldtext` renders the default fold text into.
pub const FOLD_TEXT_LEN: c_uint = 51;

/// `fold_T::fd_flags` — whether a fold is drawn open, closed, or takes its
/// state from 'foldlevel'.
pub const FD_OPEN: c_uint = 0;
pub const FD_CLOSED: c_uint = 1;
pub const FD_LEVEL: c_uint = 2;

/// What `setManualFold` and friends did, so the caller knows whether to
/// keep looking for a fold to act on.
pub const DONE_NOTHING: c_int = 0;
pub const DONE_ACTION: c_int = 1;
pub const DONE_FOLD: c_int = 2;

/// One of the six per-'foldmethod' level computations in this file.
pub type LevelGetter = Option<unsafe fn(*mut fline_T) -> ()>;

#[derive(Copy, Clone)]
pub struct fold_T {
    /// First line of the fold; relative to the parent for a nested fold.
    pub fd_top: linenr_T,
    /// Number of lines in the fold.
    pub fd_len: linenr_T,
    /// The folds nested inside this one.
    pub fd_nested: garray_T,
    /// `FD_OPEN`, `FD_CLOSED` or `FD_LEVEL`.
    pub fd_flags: c_char,
    /// Whether the fold is smaller than 'foldminlines'. `kNone` means "not
    /// worked out yet", and applies to the nested folds too.
    pub fd_small: TriState,
}

/// What the per-'foldmethod' level computations are handed, and what they
/// answer in.
#[derive(Copy, Clone)]
pub struct fline_T {
    pub wp: *mut win_T,
    /// Current line number.
    pub lnum: linenr_T,
    /// Offset between `lnum` and the real line number.
    pub off: linenr_T,
    /// Line number used by `foldUpdateIEMSRecurse`.
    pub lnum_save: linenr_T,
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

/// The `fold_T` array a `garray_T` holds. Every fold list in the tree — a
/// window's top level and each fold's `fd_nested` — is an untyped growable
/// array, so this cast is how the folds are reached.
fn folds(gap: &garray_T) -> *mut fold_T {
    gap.ga_data.cast()
}

/// The `i`th fold of `gap`. `i == gap.ga_len` yields the one-past-the-end
/// pointer the walks in this file compare against.
fn fold_at(gap: &garray_T, i: c_int) -> *mut fold_T {
    folds(gap).wrapping_offset(i as isize)
}

/// One past `gap`'s last fold.
fn folds_end(gap: &garray_T) -> *mut fold_T {
    fold_at(gap, gap.ga_len)
}

/// Where `fp` sits in `gap`.
fn fold_index(gap: &garray_T, fp: *const fold_T) -> c_int {
    let bytes = fp.addr() as isize - folds(gap).addr() as isize;
    (bytes / size_of::<fold_T>() as isize) as c_int
}

/// Set when the folds changed and the window needs redrawing.
static fold_changed: GlobalCell<bool> = GlobalCell::new(false);
static e_nofold: GlobalCell<*const c_char> = GlobalCell::new(c"E490: No fold found".as_ptr());

/// While the folds are being updated, the lines between `invalid_top` and
/// `invalid_bot` have an undefined fold level. Only meaningful for the window
/// currently being updated.
static invalid_top: GlobalCell<linenr_T> = GlobalCell::new(0);
static invalid_bot: GlobalCell<linenr_T> = GlobalCell::new(0);

/// With 'foldexpr' we sometimes ask for the level of the *next* line, which
/// calls `foldlevel()` for the current one — which has not been stored yet.
/// The previous line's level is parked here to break that cycle;
/// `prev_lnum` is zero when there is nothing to offer.
static prev_lnum: GlobalCell<linenr_T> = GlobalCell::new(0);
static prev_lnum_lvl: GlobalCell<c_int> = GlobalCell::new(-1);

/// 'foldmarker' split into its two halves, refreshed by `parseMarker`.
static foldstartmarkerlen: GlobalCell<size_t> = GlobalCell::new(0);
static foldendmarker: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());
static foldendmarkerlen: GlobalCell<size_t> = GlobalCell::new(0);
/// Copy that folding state from window "wp_from" to window "wp_to".
pub unsafe fn copyFoldingState(mut wp_from: *mut win_T, mut wp_to: *mut win_T) {
    (*wp_to).w_fold_manual = (*wp_from).w_fold_manual;
    (*wp_to).w_foldinvalid = (*wp_from).w_foldinvalid;
    cloneFoldGrowArray(&raw mut (*wp_from).w_folds, &raw mut (*wp_to).w_folds);
}
/// Returns true if there may be folded lines in window "win".
pub unsafe fn hasAnyFolding(mut win: *mut win_T) -> c_int {
    return ((*(*win).w_buffer).terminal.is_null()
        && (*win).w_onebuf_opt.wo_fen != 0
        && (!foldmethodIsManual(win) || !((*win).w_folds.ga_len <= 0))) as c_int;
}
/// When returning true, *firstp and *lastp are set to the first and last
/// lnum of the sequence of folded lines (skipped when NULL).
///
/// Returns true if line "lnum" in window "win" is part of a closed fold.
pub unsafe fn hasFolding(
    mut win: *mut win_T,
    mut lnum: linenr_T,
    mut firstp: *mut linenr_T,
    mut lastp: *mut linenr_T,
) -> bool {
    return hasFoldingWin(win, lnum, firstp, lastp, true, ptr::null_mut());
}
/// Search folds starting at lnum
/// `lnum` — first line to search
/// `first` — first line of fold containing lnum
/// `lastp` — last line with a fold
/// `cache` — when true: use cached values of window
/// `infop` — where to store fold info
///
/// Returns true if range contains folds
pub unsafe fn hasFoldingWin(
    win: *mut win_T,
    lnum: linenr_T,
    firstp: *mut linenr_T,
    lastp: *mut linenr_T,
    cache: bool,
    infop: *mut foldinfo_T,
) -> bool {
    checkupdate(win);
    if hasAnyFolding(win) == 0 {
        if !infop.is_null() {
            (*infop).fi_level = 0;
        }
        return false;
    }
    let mut had_folded: bool = false;
    let mut first: linenr_T = 0;
    let mut last: linenr_T = 0;
    if cache {
        let x: c_int = find_wl_entry(win, lnum);
        if x >= 0 {
            first = (*(*win).w_lines.offset(x as isize)).wl_lnum;
            last = (*(*win).w_lines.offset(x as isize)).wl_foldend;
            had_folded = (*(*win).w_lines.offset(x as isize)).wl_folded;
        }
    }
    let mut lnum_rel: linenr_T = lnum;
    let mut level: c_int = 0;
    let mut low_level: c_int = 0;
    let mut fp: *mut fold_T = ptr::null_mut();
    let mut maybe_small: bool = false;
    let mut use_level: bool = false;
    if first == 0 {
        let mut gap: *mut garray_T = &raw mut (*win).w_folds;
        while foldFind(gap, lnum_rel, &raw mut fp) {
            if lnum_rel == (*fp).fd_top && low_level == 0 {
                low_level = level + 1;
            }
            first += (*fp).fd_top;
            last += (*fp).fd_top;
            had_folded = check_closed(
                win,
                fp,
                &raw mut use_level,
                level,
                &raw mut maybe_small,
                lnum - lnum_rel,
            );
            if had_folded {
                last = (last as c_int + ((*fp).fd_len - 1) as c_int) as linenr_T;
                break;
            } else {
                gap = &raw mut (*fp).fd_nested;
                lnum_rel -= (*fp).fd_top;
                level += 1;
            }
        }
    }
    if !had_folded {
        if !infop.is_null() {
            (*infop).fi_level = level;
            (*infop).fi_lnum = lnum - lnum_rel;
            (*infop).fi_low_level = if low_level == 0 { level } else { low_level };
        }
        return false;
    }
    last = if last < (*(*win).w_buffer).b_ml.ml_line_count {
        last
    } else {
        (*(*win).w_buffer).b_ml.ml_line_count
    };
    if !lastp.is_null() {
        *lastp = last;
    }
    if !firstp.is_null() {
        *firstp = first;
    }
    if !infop.is_null() {
        (*infop).fi_level = level + 1;
        (*infop).fi_lnum = first;
        (*infop).fi_low_level = if low_level == 0 { level + 1 } else { low_level };
    }
    return true;
}
/// Returns fold level at line number "lnum" in the current window.
unsafe fn foldLevel(mut lnum: linenr_T) -> c_int {
    if invalid_top.get() == 0 {
        checkupdate(curwin.get());
    } else if lnum == prev_lnum.get() && prev_lnum_lvl.get() >= 0 {
        return prev_lnum_lvl.get();
    } else if lnum >= invalid_top.get() && lnum <= invalid_bot.get() {
        return -1;
    }
    if hasAnyFolding(curwin.get()) == 0 {
        return 0;
    }
    return foldLevelWin(curwin.get(), lnum);
}
/// Low level function to check if a line is folded.  Doesn't use any caching.
///
/// Returns true if line is folded or,
///          false if line is not folded.
pub unsafe fn lineFolded(win: *mut win_T, lnum: linenr_T) -> bool {
    return fold_info(win, lnum).fi_lines != 0;
}
///
/// Count the number of lines that are folded at line number "lnum".
/// Normally "lnum" is the first line of a possible fold, and the returned
/// number is the number of lines in the fold.
/// Doesn't use caching from the displayed window.
///
/// Returns with the fold level info.
///         fi_lines = number of folded lines from "lnum",
///                    or 0 if line is not folded.
pub unsafe fn fold_info(mut win: *mut win_T, mut lnum: linenr_T) -> foldinfo_T {
    let mut info: foldinfo_T = foldinfo_T {
        fi_lnum: 0,
        fi_level: 0,
        fi_low_level: 0,
        fi_lines: 0,
    };
    let mut last: linenr_T = 0;
    if hasFoldingWin(
        win,
        lnum,
        ptr::null_mut(),
        &raw mut last,
        false,
        &raw mut info,
    ) {
        info.fi_lines = last - lnum + 1;
    } else {
        info.fi_lines = 0;
    }
    return info;
}
/// Returns true if 'foldmethod' is "manual"
pub unsafe fn foldmethodIsManual(mut wp: *mut win_T) -> bool {
    return *(*wp).w_onebuf_opt.wo_fdm.offset(0) as c_int != NUL
        && *(*wp).w_onebuf_opt.wo_fdm.offset(3) as c_int == 'u' as c_int;
}
/// Returns true if 'foldmethod' is "indent"
pub unsafe fn foldmethodIsIndent(mut wp: *mut win_T) -> bool {
    return *(*wp).w_onebuf_opt.wo_fdm.offset(0) as c_int == 'i' as c_int;
}
/// Returns true if 'foldmethod' is "expr"
pub unsafe fn foldmethodIsExpr(mut wp: *mut win_T) -> bool {
    return *(*wp).w_onebuf_opt.wo_fdm.offset(0) as c_int != NUL
        && *(*wp).w_onebuf_opt.wo_fdm.offset(1) as c_int == 'x' as c_int;
}
/// Returns true if 'foldmethod' is "marker"
pub unsafe fn foldmethodIsMarker(mut wp: *mut win_T) -> bool {
    return *(*wp).w_onebuf_opt.wo_fdm.offset(0) as c_int != NUL
        && *(*wp).w_onebuf_opt.wo_fdm.offset(2) as c_int == 'r' as c_int;
}
/// Returns true if 'foldmethod' is "syntax"
pub unsafe fn foldmethodIsSyntax(mut wp: *mut win_T) -> bool {
    return *(*wp).w_onebuf_opt.wo_fdm.offset(0) as c_int == 's' as c_int;
}
/// Returns true if 'foldmethod' is "diff"
pub unsafe fn foldmethodIsDiff(mut wp: *mut win_T) -> bool {
    return *(*wp).w_onebuf_opt.wo_fdm.offset(0) as c_int == 'd' as c_int;
}
/// Remove all folding for window "win".
pub unsafe fn clearFolding(mut win: *mut win_T) {
    deleteFoldRecurse(&raw mut (*win).w_folds);
    (*win).w_foldinvalid = false;
}
/// Update folds for changes in the buffer of a window.
/// Note that inserted/deleted lines must have already been taken care of by
/// calling foldMarkAdjust().
/// The changes in lines from top to bot (inclusive).
pub unsafe fn foldUpdate(mut wp: *mut win_T, mut top: linenr_T, mut bot: linenr_T) {
    if disable_fold_update.get() != 0 || State.get() & MODE_INSERT != 0 && !foldmethodIsIndent(wp) {
        return;
    }
    if need_diff_redraw.get() {
        return;
    }
    if (*wp).w_folds.ga_len > 0 {
        let mut maybe_small_start: linenr_T = if top < bot { top } else { bot };
        let mut maybe_small_end: linenr_T = if top > bot { top } else { bot };
        let mut fp: *mut fold_T = ptr::null_mut();
        foldFind(&raw mut (*wp).w_folds, maybe_small_start, &raw mut fp);
        while fp < folds_end(&(*wp).w_folds) && (*fp).fd_top <= maybe_small_end {
            (*fp).fd_small = kNone;
            fp = fp.offset(1);
        }
    }
    if foldmethodIsIndent(wp)
        || foldmethodIsExpr(wp)
        || foldmethodIsMarker(wp)
        || foldmethodIsDiff(wp)
        || foldmethodIsSyntax(wp)
    {
        let mut save_got_int: c_int = got_int.get() as c_int;
        got_int.set(false);
        foldUpdateIEMS(wp, top, bot);
        got_int.set(got_int.get() as c_int | save_got_int != 0);
    }
}
/// Updates folds when leaving insert-mode.
pub unsafe fn foldUpdateAfterInsert() {
    if foldmethodIsManual(curwin.get())
        || foldmethodIsSyntax(curwin.get())
        || foldmethodIsExpr(curwin.get())
    {
        return;
    }
    foldUpdateAll(curwin.get());
    foldOpenCursor();
}
/// Update all lines in a window for folding.
/// Used when a fold setting changes or after reloading the buffer.
/// The actual updating is postponed until fold info is used, to avoid doing
/// every time a setting is changed or a syntax item is added.
pub unsafe fn foldUpdateAll(mut win: *mut win_T) {
    (*win).w_foldinvalid = true;
    redraw_later(win, UPD_NOT_VALID);
}
/// Init the fold info in a new window.
pub unsafe fn foldInitWin(mut new_win: *mut win_T) {
    ga_init(
        &raw mut (*new_win).w_folds,
        size_of::<fold_T>() as c_int,
        10,
    );
}
/// Find an entry in the win->w_lines[] array for buffer line "lnum".
/// Only valid entries are considered (for entries where wl_valid is false the
/// line number can be wrong).
///
/// Returns index of entry or -1 if not found.
pub unsafe fn find_wl_entry(mut win: *mut win_T, mut lnum: linenr_T) -> c_int {
    let mut i: c_int = 0;
    while i < (*win).w_lines_valid {
        if (*(*win).w_lines.offset(i as isize)).wl_valid {
            if lnum < (*(*win).w_lines.offset(i as isize)).wl_lnum {
                return -1;
            }
            if lnum <= (*(*win).w_lines.offset(i as isize)).wl_foldend {
                return i;
            }
        }
        i += 1;
    }
    return -1;
}
/// Will "clone" (i.e deep copy) a garray_T of folds.
pub unsafe fn cloneFoldGrowArray(mut from: *mut garray_T, mut to: *mut garray_T) {
    ga_init(to, (*from).ga_itemsize, (*from).ga_growsize);
    if (*from).ga_len <= 0 {
        return;
    }
    ga_grow(to, (*from).ga_len);
    let mut from_p: *mut fold_T = folds(&*from);
    let mut to_p: *mut fold_T = folds(&*to);
    let mut i: c_int = 0;
    while i < (*from).ga_len {
        (*to_p).fd_top = (*from_p).fd_top;
        (*to_p).fd_len = (*from_p).fd_len;
        (*to_p).fd_flags = (*from_p).fd_flags;
        (*to_p).fd_small = (*from_p).fd_small;
        cloneFoldGrowArray(&raw mut (*from_p).fd_nested, &raw mut (*to_p).fd_nested);
        (*to).ga_len += 1;
        from_p = from_p.offset(1);
        to_p = to_p.offset(1);
        i += 1;
    }
}
/// Search for line "lnum" in folds of growarray "gap".
/// Set "*fpp" to the fold struct for the fold that contains "lnum" or
/// the first fold below it (careful: it can be beyond the end of the array!).
///
/// Returns false when there is no fold that contains "lnum".
unsafe fn foldFind(
    mut gap: *const garray_T,
    mut lnum: linenr_T,
    mut fpp: *mut *mut fold_T,
) -> bool {
    if (*gap).ga_len == 0 {
        *fpp = ptr::null_mut();
        return false;
    }
    let mut fp: *mut fold_T = folds(&*gap);
    let mut low: linenr_T = 0;
    let mut high: linenr_T = (*gap).ga_len as linenr_T - 1;
    while low <= high {
        let mut i: linenr_T = (low + high) / 2;
        if (*fp.offset(i as isize)).fd_top > lnum {
            high = i - 1;
        } else if (*fp.offset(i as isize)).fd_top + (*fp.offset(i as isize)).fd_len <= lnum {
            low = i + 1;
        } else {
            *fpp = fp.offset(i as isize);
            return true;
        }
    }
    *fpp = fp.offset(low as isize);
    return false;
}
/// Returns fold level at line number "lnum" in window "wp".
unsafe fn foldLevelWin(mut wp: *mut win_T, mut lnum: linenr_T) -> c_int {
    let mut fp: *mut fold_T = ptr::null_mut();
    let mut lnum_rel: linenr_T = lnum;
    let mut level: c_int = 0;
    let mut gap: *mut garray_T = &raw mut (*wp).w_folds;
    while foldFind(gap, lnum_rel, &raw mut fp) {
        gap = &raw mut (*fp).fd_nested;
        lnum_rel -= (*fp).fd_top;
        level += 1;
    }
    return level;
}
/// Check if the folds in window "wp" are invalid and update them if needed.
unsafe fn checkupdate(mut wp: *mut win_T) {
    if !(*wp).w_foldinvalid {
        return;
    }
    foldUpdate(wp, 1, MAXLNUM as c_int as linenr_T);
    (*wp).w_foldinvalid = false;
}
/// Delete fold "idx" from growarray "gap".
///
/// `recursive` — when true, also delete all the folds contained in it.
///                   when false, contained folds are moved one level up.
unsafe fn deleteFoldEntry(gap: *mut garray_T, idx: c_int, recursive: bool) {
    let mut fp: *mut fold_T = fold_at(&*gap, idx);
    if recursive || (*fp).fd_nested.ga_len <= 0 {
        deleteFoldRecurse(&raw mut (*fp).fd_nested);
        (*gap).ga_len -= 1;
        if idx < (*gap).ga_len {
            memmove(
                fp as *mut c_void,
                fp.offset(1) as *const c_void,
                size_of::<fold_T>().wrapping_mul(((*gap).ga_len - idx) as size_t),
            );
        }
    } else {
        let mut moved: c_int = (*fp).fd_nested.ga_len;
        ga_grow(gap, moved - 1);
        fp = fold_at(&*gap, idx);
        let mut nfp: *mut fold_T = folds(&(*fp).fd_nested);
        let mut i: c_int = 0;
        while i < moved {
            (*nfp.offset(i as isize)).fd_top += (*fp).fd_top;
            if (*fp).fd_flags as c_int == FD_LEVEL as c_int {
                (*nfp.offset(i as isize)).fd_flags = FD_LEVEL as c_int as c_char;
            }
            if (*fp).fd_small as c_int == kNone as c_int {
                (*nfp.offset(i as isize)).fd_small = kNone;
            }
            i += 1;
        }
        if (idx + 1) < (*gap).ga_len {
            memmove(
                fp.offset(moved as isize) as *mut c_void,
                fp.offset(1) as *const c_void,
                size_of::<fold_T>().wrapping_mul(((*gap).ga_len - (idx + 1)) as size_t),
            );
        }
        memmove(
            fp as *mut c_void,
            nfp as *const c_void,
            size_of::<fold_T>().wrapping_mul(moved as size_t),
        );
        xfree(nfp as *mut c_void);
        (*gap).ga_len += moved - 1;
    };
}
/// Free `gap` and every fold nested inside it.
pub unsafe fn deleteFoldRecurse(gap: *mut garray_T) {
    if !(*gap).ga_data.is_null() {
        for i in 0..(*gap).ga_len {
            deleteFoldRecurse(&raw mut (*fold_at(&*gap, i)).fd_nested);
        }
    }
    ga_clear(gap);
}
/// Get the lowest 'foldlevel' value that makes the deepest nested fold in
/// window `wp`.
pub unsafe fn getDeepestNesting(mut wp: *mut win_T) -> c_int {
    checkupdate(wp);
    return getDeepestNestingRecurse(&raw mut (*wp).w_folds);
}
unsafe fn getDeepestNestingRecurse(mut gap: *mut garray_T) -> c_int {
    let mut maxlevel: c_int = 0;
    let mut fp: *mut fold_T = folds(&*gap);
    let mut i: c_int = 0;
    while i < (*gap).ga_len {
        let mut level: c_int =
            getDeepestNestingRecurse(&raw mut (*fp.offset(i as isize)).fd_nested) + 1;
        maxlevel = if maxlevel > level { maxlevel } else { level };
        i += 1;
    }
    return maxlevel;
}
/// Update fd_small field of fold "fp".
///
/// `lnum_off` — offset for fp->fd_top
unsafe fn checkSmall(wp: *mut win_T, fp: *mut fold_T, lnum_off: linenr_T) {
    if (*fp).fd_small as c_int != kNone as c_int {
        return;
    }
    setSmallMaybe(&raw mut (*fp).fd_nested);
    if (*fp).fd_len as OptInt > (*wp).w_onebuf_opt.wo_fml {
        (*fp).fd_small = kFalse;
    } else {
        let mut count: c_int = 0;
        let mut n: c_int = 0;
        while (n as linenr_T) < (*fp).fd_len {
            count += plines_win_nofold(wp, (*fp).fd_top + lnum_off + n as linenr_T);
            if count as OptInt > (*wp).w_onebuf_opt.wo_fml {
                (*fp).fd_small = kFalse;
                return;
            }
            n += 1;
        }
        (*fp).fd_small = kTrue;
    };
}
/// Set small flags in "gap" to kNone.
unsafe fn setSmallMaybe(mut gap: *mut garray_T) {
    let mut fp: *mut fold_T = folds(&*gap);
    let mut i: c_int = 0;
    while i < (*gap).ga_len {
        (*fp.offset(i as isize)).fd_small = kNone;
        i += 1;
    }
}
pub const true_0: c_int = 1;
pub const false_0: c_int = 0;
