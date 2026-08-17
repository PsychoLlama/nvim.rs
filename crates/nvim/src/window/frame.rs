//! The frame tree -- removing a window's frame and giving its room away.
//!
//! [`winframe_remove`] unlinks a window's leaf and returns the frame that
//! inherits its space, [`find_altwin`] picks that neighbour (which is what
//! decides where the cursor goes after `:close`), [`flatten`] collapses a row
//! or column left with a single child, and [`winframe_restore`] puts one back
//! when the close is undone.  [`alt_frame`], [`frame2win`], [`frame_has_win`]
//! and [`is_bottom_window`] are the small queries over the tree the rest of the
//! family asks.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;
use core::ptr;

use super::*;
use crate::main::{cmdline_win, first_tabpage, lastused_tabpage, p_sb, p_spr, tcl_flags};
use crate::options::{kOptTclFlagLeft, kOptTclFlagUselast};
use crate::types::{frame_T, tabpage_T, win_T};
use crate::winfloat::win_float_find_altwin;
use crate::winlayer::{Frame, TabPage, Win, tabs};

/// Which neighbour inherits a closing window's room, and along which axis --
/// the C's `wp`, `*altfr` and `*dirp` out-parameters as one value.
#[derive(Clone, Copy)]
pub(crate) struct AltWin {
    /// The window the cursor goes to.
    pub win: Win,
    /// The frame that grows into the closing one's room.
    pub frame: Frame,
    /// `'v'` when the room is given away vertically, `'h'` horizontally.
    pub dir: c_int,
}

pub(crate) unsafe extern "C" fn win_free_mem(
    win: *mut win_T,
    dirp: *mut c_int,
    tp: *mut tabpage_T,
) -> *mut win_T {
    // SAFETY: the caller's promise -- a live window, a live tab page or null,
    // and a writable `dirp`.
    unsafe {
        let (wp, dir) = free_mem(Win::new(win), TabPage::from_raw(tp));
        *dirp = dir;
        wp.map_or(ptr::null_mut(), Win::raw)
    }
}

/// Free `win`'s frame and the window itself, and say which neighbour took its
/// room and along which axis.
pub(crate) fn free_mem(win: Win, tp: Option<TabPage>) -> (Option<Win>, c_int) {
    let mut win_tp = tp.unwrap_or_else(cur_tab);
    let (wp, dir) = if win.w_floating {
        // SAFETY: a live window and tab page.
        let alt = unsafe { win_float_find_altwin(win.raw(), raw_tab(tp)) };
        // SAFETY: the window that takes a float's place is live, or null when
        // there is none.
        (unsafe { Win::from_raw(alt) }, 'h' as c_int)
    } else {
        let frp = win.frame();
        let (wp, dir) = remove(win, tp, None);
        free(frp.raw());
        (wp, dir)
    };
    // SAFETY: a live window and tab page.
    unsafe { win_free(win.raw(), raw_tab(tp)) };
    if win_tp.tp_curwin == win.raw() {
        win_tp.tp_curwin = wp.map_or(ptr::null_mut(), Win::raw);
    }
    if win.raw() == cmdline_win.get() {
        cmdline_win.set(ptr::null_mut::<win_T>());
    }
    (wp, dir)
}

pub unsafe extern "C" fn winframe_remove(
    win: *mut win_T,
    dirp: *mut c_int,
    tp: *mut tabpage_T,
    unflat_altfr: *mut *mut frame_T,
) -> *mut win_T {
    // SAFETY: the caller's promise -- a live window, a live tab page or null,
    // and writable out-parameters (`unflat_altfr` may be null).
    unsafe {
        // `then_some` would form the reference before testing the pointer.
        let unflat = unflat_altfr.as_mut();
        let (wp, dir) = remove(Win::new(win), TabPage::from_raw(tp), unflat);
        *dirp = dir;
        wp.map_or(ptr::null_mut(), Win::raw)
    }
}

/// Take `win`'s frame out of the layout tree and give its room to a
/// neighbour, from `winframe_remove()`.
///
/// With `unflat_altfr` the frame that grew is handed back *unflattened*, so
/// [`winframe_restore`] can undo the whole thing; without it the tree is
/// tidied here. Answers the window the caller should go to, and by which axis
/// the room moved.
fn remove(
    win: Win,
    tp: Option<TabPage>,
    unflat_altfr: Option<&mut *mut frame_T>,
) -> (Option<Win>, c_int) {
    let Some(alt) = find_altwin(win, tp) else {
        return (None, 0);
    };
    let frp_close = win.frame();
    frame_locked.set(frame_locked.get() + 1);

    // Save the position of the containing frame (which will also contain the
    // altframe) before we remove anything, to recompute window positions later.
    let parent = frp_close.parent().expect("a window that is not alone");
    let topleft = frame2window(parent);
    let mut row = topleft.w_winrow;
    let mut col = topleft.w_wincol;

    // If `win` is the last window in a row, its separator column goes with it.
    if win.w_vsep_width == 0 && parent.fr_layout as c_int == FR_ROW {
        if let Some(prev) = frp_close.prev() {
            set_vsep(prev, false);
        }
    }
    frame_remove(frp_close);
    let mut altfr = alt.frame;
    if alt.dir == 'v' as c_int {
        let taller = altfr.fr_height + frp_close.fr_height;
        new_height(altfr, taller, Some(altfr) == frp_close.next(), false, false);
    } else {
        debug_assert!(alt.dir == 'h' as c_int, "*dirp == 'h'");
        let wider = altfr.fr_width + frp_close.fr_width;
        new_width(altfr, wider, Some(altfr) == frp_close.next(), false);
    }
    if Some(altfr) != frp_close.prev() {
        comp_pos(parent, &mut row, &mut col);
    }
    match unflat_altfr {
        None => flatten(altfr),
        Some(slot) => *slot = altfr.raw(),
    }
    frame_locked.set(frame_locked.get() - 1);
    (Some(alt.win), alt.dir)
}

pub unsafe extern "C" fn winframe_find_altwin(
    win: *mut win_T,
    dirp: *mut c_int,
    tp: *mut tabpage_T,
    altfr: *mut *mut frame_T,
) -> *mut win_T {
    // SAFETY: the caller's promise -- a live window, a live tab page or null,
    // and writable out-parameters (`altfr` may be null).
    unsafe {
        let Some(alt) = find_altwin(Win::new(win), TabPage::from_raw(tp)) else {
            return ptr::null_mut();
        };
        *dirp = alt.dir;
        if !altfr.is_null() {
            *altfr = alt.frame.raw();
        }
        alt.win.raw()
    }
}

/// Which window inherits `win`'s room when it closes, from
/// `winframe_find_altwin()`. `None` when `win` is the only one, in which case
/// nothing can be taken away.
///
/// The neighbour [`alt_frame`] picks is refused when its window is pinned by
/// `'winfix{height,width}'`: the search then walks outwards from the closing
/// frame, `fr_prev` and `fr_next` in step, taking the first frame that is not.
pub(crate) fn find_altwin(win: Win, tp: Option<TabPage>) -> Option<AltWin> {
    debug_assert!(
        tp.is_none_or(|tp| !tp.is_current()),
        "tp == NULL || tp != curtab"
    );
    if is_only_window(win, tp) {
        return None;
    }
    let frp_close = win.frame();
    let mut frame = alt_frame(win, tp);
    let mut wp = frame2window(frame);
    let vertical = frp_close
        .parent()
        .is_some_and(|p| p.fr_layout as c_int == FR_COL);
    // A leaf frame whose window is pinned along the axis the room moves.
    let pinned = |fr: Frame| {
        fr.win().is_some_and(|w| {
            if vertical {
                w.w_onebuf_opt.wo_wfh != 0
            } else {
                w.w_onebuf_opt.wo_wfw != 0
            }
        })
    };
    // Whole-frame version of the same test, which a row or column answers for
    // its children.
    let fixed = if vertical {
        frame_fixed_height
    } else {
        frame_fixed_width
    };
    if pinned(frame) {
        // Find another frame in the column (or row), as close to the closed
        // frame as possible, to distribute the room to. Upstream's two walks
        // are deliberately not symmetric: backwards it accepts any frame that
        // is not fixed, forwards only a *leaf* whose window is not pinned.
        let mut back = frp_close.prev();
        let mut fwd = frp_close.next();
        while back.is_some() || fwd.is_some() {
            if let Some(before) = back {
                if !fixed(before) {
                    frame = before;
                    wp = frame2window(frame);
                    break;
                }
                back = before.prev();
            }
            let Some(after) = fwd else {
                continue;
            };
            if let Some(w) = after.win().filter(|_| !pinned(after)) {
                frame = after;
                wp = w;
                break;
            }
            fwd = after.next();
        }
    }
    let dir = if vertical { 'v' } else { 'h' } as c_int;
    debug_assert!(
        wp != win && frame != frp_close,
        "wp != win && frp2 != frp_close"
    );
    Some(AltWin {
        win: wp,
        frame,
        dir,
    })
}

pub(crate) unsafe extern "C" fn frame_flatten(frp: *mut frame_T) {
    // SAFETY: the caller's promise -- a live frame.
    flatten(unsafe { Frame::new(frp) });
}

/// Collapse `frp` into its parent when it is the only child left, and then the
/// parent into *its* parent when the two have the same layout.
pub(crate) fn flatten(frp: Frame) {
    if frp.next().is_some() || frp.prev().is_some() {
        return;
    }
    // There is no other frame in this list: move the info from `frp` into its
    // parent and free `frp`.
    let mut parent = frp.parent().expect("a lone child has a parent");
    parent.fr_layout = frp.fr_layout;
    parent.fr_child = frp.fr_child;
    for mut child in frp.children() {
        child.fr_parent = parent.raw();
    }
    parent.fr_win = frp.fr_win;
    if let Some(mut win) = frp.win() {
        win.w_frame = parent.raw();
    }
    let mut top = current_topframe();
    if top.fr_child == frp.raw() {
        top.fr_child = parent.raw();
    }
    free(frp.raw());

    // Now `parent` may have the same layout as *its* parent, in which case its
    // children move up a level and it goes too.
    let Some(mut grand) = parent.parent() else {
        return;
    };
    if grand.fr_layout != parent.fr_layout {
        return;
    }
    if grand.fr_child == parent.raw() {
        grand.fr_child = parent.fr_child;
    }
    let first = parent.child().expect("frp2->fr_child");
    let mut first = first;
    first.fr_prev = parent.fr_prev;
    if let Some(mut before) = parent.prev() {
        before.fr_next = first.raw();
    }
    let mut child = first;
    loop {
        child.fr_parent = grand.raw();
        let Some(next) = child.next() else {
            child.fr_next = parent.fr_next;
            if let Some(mut after) = parent.next() {
                after.fr_prev = child.raw();
            }
            break;
        };
        child = next;
    }
    if top.fr_child == parent.raw() {
        top.fr_child = grand.raw();
    }
    free(parent.raw());
}

pub unsafe extern "C" fn winframe_restore(wp: *mut win_T, dir: c_int, unflat_altfr: *mut frame_T) {
    // SAFETY: the caller's promise -- a live window and the live frame
    // `winframe_remove` handed back unflattened.
    unsafe { restore(Win::new(wp), dir, Frame::new(unflat_altfr)) };
}

/// Undo a [`remove`] that was told to leave the tree unflattened: link `wp`'s
/// frame back in and take its room off the frame that grew into it.
fn restore(wp: Win, dir: c_int, unflat_altfr: Frame) {
    let frp = wp.frame();
    // Restore the lists of frames the window was in.
    match frp.prev() {
        Some(prev) => frame_append(prev, frp),
        None => frame_insert(frp.next().expect("a frame list has two entries"), frp),
    }
    let parent = frp.parent().expect("a restored frame has a parent");
    // Restore the separator or status line the window gave up on the way out.
    if wp.w_vsep_width == 0 && parent.fr_layout as c_int == FR_ROW {
        if let Some(prev) = frp.prev() {
            set_vsep(prev, true);
        }
    }
    if parent.fr_layout as c_int == FR_COL {
        if let Some(prev) = frp.prev() {
            if global_stl_rows() == 0 && wp.w_status_height == 0 {
                add_statusline(prev);
            } else if global_stl_rows() > 0 && wp.w_hsep_height == 0 {
                add_hsep(prev);
            }
        }
    }
    if dir == 'v' as c_int {
        let shorter = unflat_altfr.fr_height - frp.fr_height;
        let topfirst = Some(unflat_altfr) == frp.next();
        new_height(unflat_altfr, shorter, topfirst, false, false);
    } else if dir == 'h' as c_int {
        let narrower = unflat_altfr.fr_width - frp.fr_width;
        let leftfirst = Some(unflat_altfr) == frp.next();
        new_width(unflat_altfr, narrower, leftfirst, false);
    }
    if Some(unflat_altfr) != frp.prev() {
        let topleft = frame2window(parent);
        let mut row = topleft.w_winrow;
        let mut col = topleft.w_wincol;
        comp_pos(parent, &mut row, &mut col);
    }
}

pub(crate) unsafe extern "C" fn win_altframe(win: *mut win_T, tp: *mut tabpage_T) -> *mut frame_T {
    // SAFETY: the caller's promise -- a live window and a live tab page or null.
    unsafe { alt_frame(Win::new(win), TabPage::from_raw(tp)).raw() }
}

/// The frame that would take `win`'s room, before `'winfix*'` is considered.
///
/// The neighbour after `win` unless `'splitbelow'`/`'splitright'` says the one
/// before, and then the other one anyway if the chosen frame is fixed and the
/// other is not.
pub(crate) fn alt_frame(win: Win, tp: Option<TabPage>) -> Frame {
    debug_assert!(
        tp.is_none_or(|tp| !tp.is_current()),
        "tp == NULL || tp != curtab"
    );
    if is_only_window(win, tp) {
        // Last window in this tab page, will go to next tab page.
        // SAFETY: every tab page has a current window, which is live.
        return unsafe { Win::new(alt_tab_page().tp_curwin) }.frame();
    }
    let frp = win.frame();
    let (Some(next), Some(prev)) = (frp.next(), frp.prev()) else {
        return frp.next().or_else(|| frp.prev()).expect("not alone");
    };
    let row = frp.parent().is_some_and(|p| p.fr_layout as c_int == FR_ROW);
    let col = frp.parent().is_some_and(|p| p.fr_layout as c_int == FR_COL);
    // Set a preference between the next and previous frame.
    let before = (col && p_sb.get() != 0) || (row && p_spr.get() != 0);
    let (target, other) = if before { (prev, next) } else { (next, prev) };
    // Prefer the frame that is not fixed along the axis the room moves.
    let fixed = if row {
        frame_fixed_width
    } else {
        frame_fixed_height
    };
    if fixed(target) && !fixed(other) {
        other
    } else {
        target
    }
}

pub(crate) unsafe extern "C" fn alt_tabpage() -> *mut tabpage_T {
    alt_tab_page().raw()
}

/// The tab page to go to when the current one closes: the last used one when
/// `'tabclose'` says so, otherwise the next, or the previous when the current
/// is last (or `'tabclose'` says "left" and it is not first).
pub(crate) fn alt_tab_page() -> TabPage {
    if tcl_flags.get() & kOptTclFlagUselast != 0 {
        // SAFETY: `valid_tabpage` only compares the pointer against the list.
        if unsafe { valid_tabpage(lastused_tabpage.get()) } {
            // SAFETY: just proved live.
            return unsafe { TabPage::new(lastused_tabpage.get()) };
        }
    }
    let cur = cur_tab();
    let forward = cur.next().is_some()
        && (tcl_flags.get() & kOptTclFlagLeft == 0 || cur.raw() == first_tabpage.get());
    match cur.next() {
        Some(next) if forward => next,
        _ => tabs()
            .find(|tp| tp.tp_next == cur.raw())
            .expect("a tab page before the current one"),
    }
}

pub unsafe extern "C" fn frame2win(frp: *mut frame_T) -> *mut win_T {
    // SAFETY: the caller's promise -- a live frame.
    frame2window(unsafe { Frame::new(frp) }).raw()
}

/// The first window in frame `frp`, following `fr_child` down to a leaf.
pub(crate) fn frame2window(frp: Frame) -> Win {
    let mut frp = frp;
    loop {
        match frp.win() {
            Some(win) => return win,
            None => frp = frp.child().expect("a frame that is not a leaf has a child"),
        }
    }
}

/// Whether `wp` is one of the windows in frame `frp`.
pub(crate) fn frame_has_win(frp: Frame, wp: Option<Win>) -> bool {
    if frp.fr_layout as c_int == FR_LEAF {
        return frp.win() == wp;
    }
    frp.children().any(|child| frame_has_win(child, wp))
}

/// Whether `wp` is along the bottom of the screen: no frame below it, all the
/// way up to the top frame.
pub(crate) fn is_bottom_window(wp: Win) -> bool {
    let mut frp = wp.frame();
    while let Some(parent) = frp.parent() {
        if parent.fr_layout as c_int == FR_COL && frp.next().is_some() {
            return false;
        }
        frp = parent;
    }
    true
}
