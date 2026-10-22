//! Where the cursor was -- the per-window remembered position.
//!
//! Every window remembers, for every buffer it has shown, the cursor position
//! and the topline it was at, in a [`WinInfo`].  [`buflist_setfpos`] records
//! one, [`find_wininfo`] picks the entry to restore (preferring this window,
//! then this tab page, then any), [`get_winopts`] restores the window-local
//! options and folds along with it, and [`buflist_findfmark`] answers the
//! same question for a mark rather than a window.
//!
//! The entries live in `Buffer`'s `b_wininfo`, a `Vec<WinInfo>`, most
//! recently used first.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::winlayer::WinId;

use super::*;
use crate::fold::clear_folding;
use crate::mark::mark_view_make;
use crate::option::vars::p_fdls;
use crate::option::{clear_winopt, copy_winopt, didset_window_options};
use crate::pos::MAXLNUM;
use crate::types::{
    AdditionalData, ColNr, FileMark, FileMarkView, LineNr, OptInt, Pos, Timestamp, WinInfo,
};
use crate::winfloat::win_set_minimal_style;
use crate::winlayer::{Buf, Win, windows};

impl WinInfo {
    /// The window this entry belongs to, `None` for the entry `:badd`
    /// leaves and once that window is gone.
    pub(crate) fn window(&self) -> Option<Win> {
        self.wi_win.and_then(WinId::get)
    }
}

fn current_win() -> Win {
    Win::current()
}

/// Remember `lnum`/`col` (and, with `copy_options`, the window-local options
/// and folds) as where `win` was in `buffer`.
///
/// `win` is `None` for `:badd`, which records a position for no window at
/// all.
pub fn buflist_setfpos(
    mut buffer: Buf,
    win: Option<Win>,
    mut lnum: LineNr,
    col: ColNr,
    copy_options: bool,
) {
    let found = buffer.b_wininfo.iter().position(|e| e.window() == win);
    let mut entry = match found {
        None => {
            if lnum == 0 as LineNr {
                // Set lnum even when it is 0.
                lnum = 1 as LineNr;
            }
            WinInfo::new(win.map(Win::id))
        }
        Some(i) => {
            let mut entry = buffer.b_wininfo.remove(i);
            if copy_options && entry.wi_optset {
                clear_winopt(&mut entry.wi_opt);
                entry.wi_folds = Vec::new();
            }
            entry
        }
    };

    if lnum != 0 as LineNr {
        entry.wi_mark.mark.lnum = lnum;
        entry.wi_mark.mark.col = col;
        if let Some(win) = win {
            entry.wi_mark.view = mark_view_make(win, entry.wi_mark.mark);
        }
    }
    if let Some(win) = win {
        entry.wi_changelistidx = win.w_changelistidx;
    }
    if copy_options && let Some(win) = win {
        // Save the window-specific option values.
        copy_winopt(&win.w_onebuf_opt, &mut entry.wi_opt);
        entry.wi_fold_manual = win.w_fold_manual;
        entry.wi_folds = win.w_folds.clone();
        entry.wi_optset = true;
    }

    buffer.b_wininfo.insert(0, entry);
}

/// Whether `entry` has `'diff'` set and the diff belongs to another tab page
/// -- a diff is local to a tab page.
fn wininfo_other_tab_diff(entry: &WinInfo) -> bool {
    if entry.wi_opt.wo_diff == 0 {
        return false;
    }
    // A window of the current tab page means the buffer was in diff mode
    // here.
    !windows().any(|wp| entry.window() == Some(wp))
}

/// The index of the entry for the current window in `buffer`, or failing
/// that of the most recently used one.
///
/// `need_options` skips entries whose options were never saved;
/// `skip_diff_buffer` skips windows whose `'diff'` is another tab page's.
fn find_wininfo(buffer: Buf, need_options: bool, skip_diff_buffer: bool) -> Option<usize> {
    let cur = Some(current_win());
    let list = &buffer.b_wininfo;
    let found = list.iter().position(|e| {
        e.window() == cur
            && (!skip_diff_buffer || !wininfo_other_tab_diff(e))
            && (!need_options || e.wi_optset)
    });
    if found.is_some() {
        return found;
    }

    // No entry for curwin: use the first in the list that does not have
    // 'diff' set in another tab page. With "need_options", skip entries
    // whose options were never set -- unless the window is editing "buf",
    // so that the options can be copied from the window itself.
    if skip_diff_buffer {
        return list.iter().position(|e| {
            !wininfo_other_tab_diff(e)
                && (!need_options
                    || e.wi_optset
                    || e.window().is_some_and(|w| w.w_buffer == buffer))
        });
    }
    (!list.is_empty()).then_some(0)
}

/// Reset the current window's buffer-local options to the values last used
/// in this window; failing that, to the most recently used window's; failing
/// that, to the window's own global values.
pub fn get_winopts(buffer: Buf) {
    let mut cur = current_win();
    clear_winopt(&mut cur.w_onebuf_opt);
    clear_folding(cur);

    let found = find_wininfo(buffer, true, true);
    let entry = found.map(|i| &buffer.b_wininfo[i]);
    let entry_win = entry.and_then(WinInfo::window);

    match (entry, entry_win) {
        // The entry names another window still showing this buffer: copy
        // from the window itself, so that its current values are used.
        (Some(_), Some(wp))
            if wp != cur && wp.w_buffer == buffer && wp.w_config.style != kWinStyleMinimal =>
        {
            copy_winopt(&wp.w_onebuf_opt, &mut cur.w_onebuf_opt);
            cur.w_fold_manual = wp.w_fold_manual;
            cur.w_foldinvalid = true;
            cur.w_folds = wp.w_folds.clone();
        }
        (Some(entry), win)
            if entry.wi_optset
                && win.is_none_or(|wp| wp == cur || wp.w_config.style != kWinStyleMinimal) =>
        {
            copy_winopt(&entry.wi_opt, &mut cur.w_onebuf_opt);
            cur.w_fold_manual = entry.wi_fold_manual;
            cur.w_foldinvalid = true;
            cur.w_folds = entry.wi_folds.clone();
        }
        _ => {
            let window: &mut crate::types::Window = &mut cur;
            copy_winopt(&window.w_allbuf_opt, &mut window.w_onebuf_opt);
        }
    }
    if let Some(entry) = entry {
        cur.w_changelistidx = entry.wi_changelistidx;
    }

    if cur.w_config.style == kWinStyleMinimal {
        didset_window_options(cur, false);
        win_set_minimal_style(cur);
    }

    // Set 'foldlevel' to 'foldlevelstart' if it's not negative.
    if p_fdls() >= 0 as OptInt {
        cur.w_onebuf_opt.wo_fdl = p_fdls();
    }
    didset_window_options(cur, false);
}

/// The line of `buffer`'s most recently remembered position -- the front of
/// its window list, whichever window that was -- or `None` when no window
/// remembers one. What `:mksession`'s `badd +N` writes.
pub(crate) fn first_wininfo_lnum(buffer: Buf) -> Option<LineNr> {
    buffer
        .b_wininfo
        .first()
        .map(|entry| entry.wi_mark.mark.lnum)
}

/// Where `buffer` was last left in the current window, copied out: the mark
/// of the entry [`find_wininfo`] picks, or line 1 when there is none.
pub fn buflist_findfmark(buffer: Buf) -> FileMark {
    match find_wininfo(buffer, false, false) {
        Some(i) => buffer.b_wininfo[i].wi_mark.clone(),
        None => FileMark {
            mark: Pos {
                lnum: 1 as LineNr,
                col: 0 as ColNr,
                coladd: 0 as ColNr,
            },
            fnum: 0,
            timestamp: 0 as Timestamp,
            view: FileMarkView {
                topline_offset: MAXLNUM,
                skipcol: 0 as ColNr,
            },
            additional_data: ptr::null_mut::<AdditionalData>(),
        },
    }
}

/// [`buflist_findfmark`]'s position.
pub(crate) fn buflist_findfmark_pos(buffer: Buf) -> Pos {
    buflist_findfmark(buffer).mark
}

pub fn buflist_findlnum(buffer: Buf) -> LineNr {
    buflist_findfmark(buffer).mark.lnum
}
