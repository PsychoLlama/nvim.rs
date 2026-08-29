//! Splitting a window -- `win_split()` and `win_split_ins()`.
//!
//! [`split_ins`] is the whole operation, in stages: [`split_room`] decides the
//! new window's size from `'winheight'`/`'winwidth'` and refuses when there is
//! no room, [`insert_window`] links a `win_T` into the list, [`split_frame`]
//! puts a frame beside or around the existing one, [`size_vertical`] and
//! [`size_horizontal`] hand out the rows and columns, and the tail
//! redistributes the space and enters the new window unless `WSP_NOENTER` said
//! not to.  [`win_split`] is the thin `:split` entry point over it, and
//! [`win_init`] copies one window's state onto another.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr;

use super::arith::NextCurwin;
use super::*;
use crate::drawscreen::{UPD_NOT_VALID, comp_col, status_redraw_all};
use crate::fold::copy_folding_state;
use crate::main::{
    Columns, Rows, cmdmod, e_noroom, msg_col, msg_row, p_ch, p_ea, p_ead, p_ls, p_sb, p_spk, p_spr,
    p_wh, p_wiw, p_wmh, p_wmw, sc_col,
};
use crate::mark::copy_jumplist;
use crate::memory::{xcalloc, xstrdup};
use crate::message::msg_clr_eos_force;
use crate::r#move::WinValid;
use crate::option::win_copy_options;
use crate::quickfix::copy_loclist_stack;
use crate::types::ui::kUIMultigrid;
use crate::types::{FAIL, Integer, OK, OptInt, frame_T, qf_info_T, win_T};
use crate::ui::{ui_call_win_hide, ui_has};
use crate::ui_compositor::ui_comp_remove_grid;
use crate::winfloat::win_float_anchor_laststatus;
use crate::winlayer::{Frame, Win, WinId, frames, tabs};
use ::libc::memset;

pub fn win_split(size: c_int, flags: c_int) -> c_int {
    split(size, flags)
}

/// Split the current window, `flags` being the `WSP_*` set: which half the new
/// window takes, whether it is vertical, and whether it is entered.
pub(crate) fn split(size: c_int, flags: c_int) -> c_int {
    let cur = cur_win();
    // SAFETY: a live window.
    if unsafe { check_split_disallowed(cur.raw()) } == FAIL {
        return FAIL;
    }
    // When the ":tab" modifier was used, open a new tab page instead.
    if may_open_tabpage() == OK {
        return OK;
    }
    // Add flags from ":vertical", ":topleft" and ":botright".
    let flags = flags | cmdmod.with(|m| m.cmod_split);
    if flags & WSP_TOP as c_int != 0 && flags & WSP_BOT as c_int != 0 {
        err(c"E442: Can't split topleft and botright at the same time".as_ptr());
        return FAIL;
    }
    // When creating the help window make a snapshot of the window layout;
    // otherwise clear the snapshot, it is now invalid.
    for (flag, idx) in [
        (WSP_HELP as c_int, SNAP_HELP_IDX),
        (WSP_QUICKFIX as c_int, SNAP_QUICKFIX_IDX),
    ] {
        if flags & flag != 0 {
            take_snapshot(idx);
        } else {
            drop_snapshot(cur_tab(), idx);
        }
    }
    if split_ins(size, flags, None, 0, None).is_some() {
        OK
    } else {
        FAIL
    }
}

pub unsafe fn win_split_ins(
    size: c_int,
    flags: c_int,
    new_wp: *mut win_T,
    dir: c_int,
    to_flatten: *mut frame_T,
) -> *mut win_T {
    // SAFETY: the caller's promise -- a live window or null, and a live frame
    // or null.
    let (new_wp, to_flatten) = unsafe { (Win::from_raw(new_wp), Frame::from_raw(to_flatten)) };
    raw_win(split_ins(size, flags, new_wp, dir, to_flatten))
}

/// The room a split needs and the size it will take, from the first half of
/// `win_split_ins()`.
struct Room {
    /// The size the new window gets.
    new_size: c_int,
    /// Whether the split leaves the windows uneven enough to equalize.
    do_equal: bool,
    /// `oldwin`'s text height, the status line it may just have gained
    /// excluded. Zero for a vertical split, which does not use it.
    oldwin_height: c_int,
    /// Whether `set_fraction` has already been called on `oldwin`.
    did_set_fraction: bool,
}

/// Split `oldwin`, or insert `new_wp` at the far top/left/right/bottom when it
/// is given, and answer the new window.
///
/// On failure nothing has moved, and when `new_wp` was given the layout and
/// the sizes are exactly as they were. `to_flatten` is flattened just before
/// the frames are reorganised, so a failure leaves it unflattened.
fn split_ins(
    size: c_int,
    flags: c_int,
    new_wp: Option<Win>,
    dir: c_int,
    to_flatten: Option<Frame>,
) -> Option<Win> {
    // `aucmd_win[]` should always remain floating.
    if new_wp.is_some() && is_autocmd_window(new_wp) {
        return None;
    }
    if new_wp.is_none() {
        // SAFETY: fires `WinNewPre`, which reads no argument of ours.
        trigger_winnewpre();
    }

    let mut oldwin = if flags & WSP_TOP as c_int != 0 {
        first_win()
    } else if flags & WSP_BOT as c_int != 0 || cur_win().w_floating {
        // Can't split a float: use the last non-floating window instead.
        last_nonfloating(None)
    } else {
        cur_win()
    };

    let vertical = flags & WSP_VERT as c_int != 0;
    let toplevel = flags & (WSP_TOP as c_int | WSP_BOT as c_int) != 0;
    let layout = if vertical { FR_ROW } else { FR_COL };

    // Add a status line when 'laststatus' is 1 and the first window is split.
    let mut need_status = 0;
    let first = first_win();
    if is_only_window(first, None) && p_ls.get() == 1 as OptInt && oldwin.w_status_height == 0 {
        if oldwin.w_height as OptInt <= p_wmh.get() {
            err(e_noroom.as_ptr());
            return None;
        }
        need_status = STATUS_HEIGHT as c_int;
        win_float_anchor_laststatus();
    }

    let room = if vertical {
        split_room_vertical(size, flags, oldwin, toplevel)?
    } else {
        split_room_horizontal(size, flags, &mut oldwin, toplevel, need_status)?
    };

    let wp = insert_window(flags, new_wp, oldwin, vertical)?;
    if let Some(to_flatten) = to_flatten {
        flatten(to_flatten);
    }
    let (curfrp, frp, before) = split_frame(flags, wp, oldwin, toplevel, vertical, layout);

    if !room.did_set_fraction {
        save_fraction(oldwin);
    }
    let mut wp = wp;
    wp.w_fraction = oldwin.w_fraction;
    if vertical {
        size_vertical(
            flags,
            wp,
            oldwin,
            curfrp,
            frp,
            &room,
            before,
            toplevel,
            need_status,
        );
    } else {
        size_horizontal(flags, wp, oldwin, curfrp, frp, &room, before, toplevel);
    }
    if toplevel {
        comp_positions();
    }
    wp.redraw_later(UPD_NOT_VALID);
    oldwin.redraw_later(UPD_NOT_VALID);
    // SAFETY: marks every status line for redrawing.
    unsafe { status_redraw_all() };
    if need_status != 0 {
        // The message area is one line shorter now.
        msg_row.set(Rows.get() - 1);
        msg_col.set(sc_col.get());
        // SAFETY: clears the message area.
        unsafe { msg_clr_eos_force() };
        // SAFETY: recomputes the column it starts in.
        unsafe { comp_col() };
        msg_row.set(Rows.get() - 1);
        msg_col.set(0);
    }

    if room.do_equal || dir != 0 {
        let axis = match (vertical, dir) {
            (true, d) if d == 'v' as c_int => 'b',
            (true, _) => 'h',
            (false, d) if d == 'h' as c_int => 'b',
            (false, _) => 'v',
        };
        equal(Some(wp), true, axis as c_int);
    } else if !is_autocmd_window(Some(wp)) {
        fix_scroll(false);
    }

    // Don't change the window height/width to 'winheight'/'winwidth' when a
    // size was given: hold the option at `size` across `win_enter_ext`, which
    // is where it would otherwise be applied.
    let opt = if vertical { &p_wiw } else { &p_wh };
    let saved = opt.get() as c_int;
    if size != 0 {
        opt.set(size as OptInt);
    }
    if flags & WSP_NOENTER as c_int == 0 {
        let new_flags = if new_wp.is_none() {
            WEE_TRIGGER_NEW_AUTOCMDS as c_int
        } else {
            0
        };
        let enter = WEE_TRIGGER_ENTER_AUTOCMDS as c_int | WEE_TRIGGER_LEAVE_AUTOCMDS as c_int;
        // This fires WinNew/WinEnter/WinLeave, after which nothing derived
        // from `wp` is read.
        enter_ext(wp, new_flags | enter);
    }
    opt.set(saved as OptInt);
    // An autocommand may have closed `oldwin`.
    // SAFETY: only compares the pointer against the window list.
    if win_valid(oldwin.raw()) {
        oldwin.w_pos_changed = true;
    }
    Some(wp)
}

/// The width the new window gets from a `:vsplit`, or `None` when the layout
/// has no room for one.
fn split_room_vertical(size: c_int, flags: c_int, oldwin: Win, toplevel: bool) -> Option<Room> {
    // The current window requires at least one column.
    let wmw1 = if p_wmw.get() == 0 as OptInt {
        1
    } else {
        p_wmw.get() as c_int
    };
    let mut needed = wmw1 + 1;
    if flags & WSP_ROOM as c_int != 0 {
        needed += p_wiw.get() as c_int - wmw1;
    }
    let top = current_topframe();
    let (minwidth, available) = if toplevel {
        (minwidth(top, NextCurwin::NoWin), top.fr_width)
    } else if p_ea.get() != 0 {
        // With 'equalalways' the room is the whole screen's, so every frame in
        // every row above `oldwin` counts towards the minimum.
        let mut min = minwidth(oldwin.frame(), NextCurwin::NoWin);
        min += siblings_minwidth(oldwin.frame());
        (min, top.fr_width)
    } else {
        let frame = oldwin.frame();
        (minwidth(frame, NextCurwin::NoWin), frame.fr_width)
    };
    needed += minwidth;
    if available < needed {
        err(e_noroom.as_ptr());
        return None;
    }

    let mut new_size = size;
    if new_size == 0 {
        new_size = oldwin.w_width / 2;
    }
    new_size = new_size.min(available - minwidth - 1).max(wmw1);

    // If it doesn't fit in the current window, need `win_equal()`.
    let mut do_equal = ((oldwin.w_width - new_size - 1) as OptInt) < p_wmw.get();

    // Don't take columns for the new window from a 'winfixwidth' window: take
    // them from a window to the left or right instead, plus one separator.
    if oldwin.w_onebuf_opt.wo_wfw != 0 {
        setwidth_win(oldwin.w_width + new_size + 1, oldwin);
    }

    // Only make all windows the same width if one of them (except oldwin) is
    // wider than one of the split windows.
    if !do_equal && p_ea.get() != 0 && size == 0 && ead() != 'v' as c_int {
        do_equal = wider_sibling(oldwin, |w| {
            w.w_width > new_size || w.w_width > oldwin.w_width - new_size - 1
        });
    }
    Some(Room {
        new_size,
        do_equal,
        oldwin_height: 0,
        did_set_fraction: false,
    })
}

/// [`split_room_vertical`] for a horizontal split, which also gives `oldwin`
/// the status line the new one needs above it.
fn split_room_horizontal(
    size: c_int,
    flags: c_int,
    oldwin: &mut Win,
    toplevel: bool,
    need_status: c_int,
) -> Option<Room> {
    // The current window requires at least one line plus its window bar.
    let wmh1 = (p_wmh.get() as c_int).max(1) + oldwin.w_winbar_height;
    let mut needed = wmh1 + STATUS_HEIGHT as c_int;
    if flags & WSP_ROOM as c_int != 0 {
        needed += p_wh.get() as c_int - wmh1 + oldwin.w_winbar_height;
    }
    if p_ch.get() < 1 as OptInt {
        needed += 1; // adjust for 'cmdheight' = 0
    }
    let top = current_topframe();
    let (minheight, available) = if toplevel {
        (
            minheight(top, NextCurwin::NoWin) + need_status,
            top.fr_height,
        )
    } else if p_ea.get() != 0 {
        let mut min = minheight(oldwin.frame(), NextCurwin::NoWin) + need_status;
        min += siblings_minheight(oldwin.frame());
        (min, top.fr_height)
    } else {
        let frame = oldwin.frame();
        (
            minheight(frame, NextCurwin::NoWin) + need_status,
            frame.fr_height,
        )
    };
    needed += minheight;
    if available < needed {
        err(e_noroom.as_ptr());
        return None;
    }

    let mut oldwin_height = oldwin.w_height;
    if need_status != 0 {
        oldwin.w_status_height = STATUS_HEIGHT as c_int;
        oldwin_height -= STATUS_HEIGHT as c_int;
    }
    let mut new_size = size;
    if new_size == 0 {
        new_size = oldwin_height / 2;
    }
    new_size = new_size
        .min(available - minheight - STATUS_HEIGHT as c_int)
        .max(wmh1);

    let mut do_equal =
        ((oldwin_height - new_size - STATUS_HEIGHT as c_int) as OptInt) < p_wmh.get();

    // Don't take lines for the new window from a 'winfixheight' window.
    let mut did_set_fraction = false;
    if oldwin.w_onebuf_opt.wo_wfh != 0 {
        // Set `w_fraction` now so the cursor keeps the same relative vertical
        // position, using the old height.
        save_fraction(*oldwin);
        did_set_fraction = true;
        setheight_win(oldwin.w_height + new_size + STATUS_HEIGHT as c_int, *oldwin);
        oldwin_height = oldwin.w_height;
        if need_status != 0 {
            oldwin_height -= STATUS_HEIGHT as c_int;
        }
    }

    if !do_equal && p_ea.get() != 0 && size == 0 && ead() != 'h' as c_int {
        let oldwin = *oldwin;
        do_equal = wider_sibling(oldwin, |w| {
            w.w_height > new_size || w.w_height > oldwin_height - new_size - STATUS_HEIGHT as c_int
        });
    }
    Some(Room {
        new_size,
        do_equal,
        oldwin_height,
        did_set_fraction,
    })
}

/// The first character of `'eadirection'`.
fn ead() -> c_int {
    // SAFETY: `'eadirection'` is a NUL-terminated option string.
    unsafe { *p_ead.get() as c_int }
}

/// Whether any window beside `oldwin` in its own row or column answers `taller`
/// -- the test that decides whether a split without a size equalizes.
fn wider_sibling(oldwin: Win, taller: impl Fn(Win) -> bool) -> bool {
    let Some(parent) = oldwin.frame().parent() else {
        return false;
    };
    parent
        .children()
        .filter_map(Frame::win)
        .any(|w| w != oldwin && taller(w))
}

/// The minimum width of every frame in every *row* above `frame`, its own
/// ancestors excluded -- what a split with 'equalalways' has to leave alone.
fn siblings_minwidth(frame: Frame) -> c_int {
    let mut total = 0;
    let mut prev = frame;
    let mut up = frame.parent();
    while let Some(frp) = up {
        if frp.fr_layout as c_int == FR_ROW {
            for frp2 in frp.children().filter(|frp2| *frp2 != prev) {
                total += minwidth(frp2, NextCurwin::NoWin);
            }
        }
        prev = frp;
        up = frp.parent();
    }
    total
}

/// [`siblings_minwidth`] for the columns above `frame`.
fn siblings_minheight(frame: Frame) -> c_int {
    let mut total = 0;
    let mut prev = frame;
    let mut up = frame.parent();
    while let Some(frp) = up {
        if frp.fr_layout as c_int == FR_COL {
            for frp2 in frp.children().filter(|frp2| *frp2 != prev) {
                total += minheight(frp2, NextCurwin::NoWin);
            }
        }
        prev = frp;
        up = frp.parent();
    }
    total
}

/// Put the new window on the window list beside `oldwin`, allocating one when
/// the caller did not bring its own, and give it a frame.
fn insert_window(flags: c_int, new_wp: Option<Win>, oldwin: Win, _vertical: bool) -> Option<Win> {
    let below = flags & WSP_TOP as c_int == 0
        && (flags & WSP_BOT as c_int != 0
            || flags & WSP_BELOW as c_int != 0
            || (flags & WSP_ABOVE as c_int == 0 && split_after(_vertical)));
    let after = if below { Some(oldwin) } else { oldwin.prev() };
    let Some(mut wp) = new_wp else {
        // SAFETY: `win_alloc` answers a live window.
        let wp = unsafe { Win::new(win_alloc(raw_win(after), false)) };
        attach_frame(wp);
        // Make the contents of the new window the same as the current one.
        // SAFETY: two live windows.
        unsafe { win_init(wp.raw(), cur_win().raw(), flags) };
        return Some(wp);
    };
    append(after, wp, None);
    if !wp.w_floating {
        return Some(wp);
    }
    // A float becoming an ordinary window gives up its own grid.
    // SAFETY: the window's own grid.
    unsafe { ui_comp_remove_grid(&raw mut wp.w_grid_alloc) };
    if ui_has(kUIMultigrid) {
        wp.w_pos_changed = true;
    } else {
        // No longer a float: a non-multigrid UI shouldn't draw it as such.
        ui_call_win_hide(wp.w_grid_alloc.handle as Integer);
        free_grid(wp, true);
    }
    // External windows are independent of tab pages, and may have been the
    // `curwin` of others.
    if wp.w_config.external {
        for mut tp in tabs().filter(|tp| !tp.is_current()) {
            if tp.tp_curwin == wp.raw() {
                tp.tp_curwin = tp
                    .tp_firstwin
                    .and_then(WinId::get)
                    .map_or(ptr::null_mut(), Win::raw);
            }
        }
    }
    wp.w_floating = false;
    attach_frame(wp);
    // A non-floating window doesn't store a float config or have a border.
    let adj = (&raw mut wp.w_border_adj).cast::<c_void>();
    // SAFETY: the window's own config.
    unsafe { clear_float_config(&raw mut wp.w_config, true) };
    // SAFETY: the window's own border array.
    unsafe { memset(adj, 0, size_of::<[c_int; 4]>()) };
    Some(wp)
}

/// Whether the new window goes below (or right of) the old one by default.
fn split_after(vertical: bool) -> bool {
    if vertical {
        p_spr.get() != 0
    } else {
        p_sb.get() != 0
    }
}

/// Put the new window's frame into the tree: answer the frame the split is
/// happening in, the new window's frame, and whether it goes before.
fn split_frame(
    flags: c_int,
    wp: Win,
    oldwin: Win,
    toplevel: bool,
    vertical: bool,
    layout: c_int,
) -> (Frame, Frame, bool) {
    let top = current_topframe();
    let (mut curfrp, before) = if toplevel {
        let same_axis = (top.fr_layout as c_int == FR_COL && !vertical)
            || (top.fr_layout as c_int == FR_ROW && vertical);
        let frp = if same_axis {
            let first = top.child().expect("a row or column has a child");
            if flags & WSP_BOT as c_int != 0 {
                frames(Some(first)).last().expect("at least one")
            } else {
                first
            }
        } else {
            top
        };
        (frp, flags & WSP_TOP as c_int != 0)
    } else {
        let before = if flags & WSP_BELOW as c_int != 0 {
            false
        } else if flags & WSP_ABOVE as c_int != 0 {
            true
        } else {
            !split_after(vertical)
        };
        (oldwin.frame(), before)
    };

    // If the frame is not of the layout the split needs, make a new one around
    // it and move the old contents down a level.
    if curfrp
        .parent()
        .is_none_or(|p| p.fr_layout as c_int != layout)
    {
        let mut inner = new_frame_like(curfrp);
        let mut outer = curfrp;
        outer.fr_layout = layout as c_char;
        inner.fr_parent = outer.raw();
        inner.fr_next = ptr::null_mut::<frame_T>();
        inner.fr_prev = ptr::null_mut::<frame_T>();
        outer.fr_child = inner.raw();
        outer.fr_win = ptr::null_mut::<win_T>();
        curfrp = inner;
        match inner.win() {
            // `oldwin`'s frame moved: it now lives one level down.
            Some(_) => {
                let mut oldwin = oldwin;
                oldwin.w_frame = inner.raw();
            }
            None => {
                for mut child in inner.children() {
                    child.fr_parent = curfrp.raw();
                }
            }
        }
    }

    let mut frp = wp.frame();
    frp.fr_parent = curfrp.fr_parent;
    if before {
        frame_insert(curfrp, frp);
    } else {
        frame_append(curfrp, frp);
    }
    (curfrp, frp, before)
}

/// A copy of `frame`, allocated: the C's `*frp = *curfrp` over a fresh
/// `xcalloc`.
///
/// Written out field by field because `frame_T` is deliberately neither
/// `Copy` nor `Clone` — a frame is a node of the layout tree, and this is the
/// one place in the editor that duplicates one. The links come across with
/// the rest, exactly as the struct assignment copied them, and the caller
/// rewires them straight afterwards.
fn new_frame_like(frame: Frame) -> Frame {
    let mut copy = attach_frame_raw();
    copy.fr_layout = frame.fr_layout;
    copy.fr_width = frame.fr_width;
    copy.fr_newwidth = frame.fr_newwidth;
    copy.fr_height = frame.fr_height;
    copy.fr_newheight = frame.fr_newheight;
    copy.fr_parent = frame.fr_parent;
    copy.fr_next = frame.fr_next;
    copy.fr_prev = frame.fr_prev;
    copy.fr_child = frame.fr_child;
    copy.fr_win = frame.fr_win;
    copy
}

/// A fresh zeroed frame with no window attached.
fn attach_frame_raw() -> Frame {
    // SAFETY: `xcalloc` aborts rather than answering null; the frame is live
    // from here on.
    unsafe { Frame::new(xcalloc(1, size_of::<frame_T>()).cast::<frame_T>()) }
}

/// Hand out the columns for a `:vsplit`.
#[expect(clippy::too_many_arguments, reason = "upstream's one long arm")]
fn size_vertical(
    flags: c_int,
    wp: Win,
    oldwin: Win,
    curfrp: Frame,
    frp: Frame,
    room: &Room,
    before: bool,
    toplevel: bool,
    need_status: c_int,
) {
    let (mut wp, mut oldwin, mut frp) = (wp, oldwin, frp);
    // 'scroll' is not inherited by a horizontal split, but is by a vertical
    // one.
    wp.w_onebuf_opt.wo_scr = cur_win().w_onebuf_opt.wo_scr;
    if need_status != 0 {
        new_win_height(oldwin, oldwin.w_height - 1);
        oldwin.w_status_height = need_status;
    }
    if toplevel {
        wp.w_winrow = tabline_rows();
        let stl = (p_ls.get() == 1 as OptInt || p_ls.get() == 2 as OptInt) as c_int;
        new_win_height(wp, curfrp.fr_height - stl);
        wp.w_status_height = stl;
        wp.w_hsep_height = 0;
    } else {
        wp.w_winrow = oldwin.w_winrow;
        new_win_height(wp, oldwin.w_height);
        wp.w_status_height = oldwin.w_status_height;
        wp.w_hsep_height = oldwin.w_hsep_height;
    }
    frp.fr_height = curfrp.fr_height;

    // "new_size" of the current window goes to the new window, use one column
    // for the vertical separator.
    new_win_width(wp, room.new_size);
    if before {
        wp.w_vsep_width = 1;
    } else {
        wp.w_vsep_width = oldwin.w_vsep_width;
        oldwin.w_vsep_width = 1;
    }
    if toplevel {
        if flags & WSP_BOT as c_int != 0 {
            set_vsep(curfrp, true);
        }
        let sep = (flags & WSP_TOP as c_int != 0) as c_int;
        new_width(
            curfrp,
            curfrp.fr_width - (room.new_size + sep),
            flags & WSP_TOP as c_int != 0,
            false,
        );
    } else {
        new_win_width(oldwin, oldwin.w_width - (room.new_size + 1));
    }
    if before {
        wp.w_wincol = oldwin.w_wincol;
        oldwin.w_wincol += room.new_size + 1;
    } else {
        wp.w_wincol = oldwin.w_wincol + oldwin.w_width + 1;
    }
    frame_fix_width(oldwin);
    frame_fix_width(wp);
}

/// Hand out the rows for a `:split`.
#[expect(clippy::too_many_arguments, reason = "upstream's one long arm")]
fn size_horizontal(
    flags: c_int,
    wp: Win,
    oldwin: Win,
    curfrp: Frame,
    frp: Frame,
    room: &Room,
    before: bool,
    toplevel: bool,
) {
    let (mut wp, mut oldwin, mut frp) = (wp, oldwin, frp);
    let is_stl_global = global_stl_rows() > 0;
    if toplevel {
        wp.w_wincol = 0;
        new_win_width(wp, Columns.get());
        wp.w_vsep_width = 0;
    } else {
        wp.w_wincol = oldwin.w_wincol;
        new_win_width(wp, oldwin.w_width);
        wp.w_vsep_width = oldwin.w_vsep_width;
    }
    frp.fr_width = curfrp.fr_width;

    // "new_size" of the current window goes to the new window, use one line
    // for the status line.
    new_win_height(wp, room.new_size);
    let old_status_height = oldwin.w_status_height;
    if before {
        wp.w_hsep_height = is_stl_global as c_int;
    } else {
        wp.w_hsep_height = oldwin.w_hsep_height;
        oldwin.w_hsep_height = is_stl_global as c_int;
    }
    if toplevel {
        let mut new_fr_height = curfrp.fr_height - room.new_size;
        if is_stl_global {
            if flags & WSP_BOT as c_int != 0 {
                add_hsep(curfrp);
            } else {
                new_fr_height -= 1;
            }
        } else {
            if !(flags & WSP_BOT as c_int != 0 && p_ls.get() == 0 as OptInt) {
                new_fr_height -= STATUS_HEIGHT as c_int;
            }
            if flags & WSP_BOT as c_int != 0 {
                add_statusline(curfrp);
            }
        }
        new_height(
            curfrp,
            new_fr_height,
            flags & WSP_TOP as c_int != 0,
            false,
            false,
        );
    } else {
        new_win_height(
            oldwin,
            room.oldwin_height - (room.new_size + STATUS_HEIGHT as c_int),
        );
    }
    if before {
        wp.w_winrow = oldwin.w_winrow;
        if is_stl_global {
            wp.w_status_height = 0;
            oldwin.w_winrow += wp.w_height + 1;
        } else {
            wp.w_status_height = STATUS_HEIGHT as c_int;
            oldwin.w_winrow += wp.w_height + STATUS_HEIGHT as c_int;
        }
    } else if is_stl_global {
        wp.w_winrow = oldwin.w_winrow + oldwin.w_height + 1;
        wp.w_status_height = 0;
    } else {
        wp.w_winrow = oldwin.w_winrow + oldwin.w_height + STATUS_HEIGHT as c_int;
        wp.w_status_height = old_status_height;
        if flags & WSP_BOT as c_int == 0 {
            oldwin.w_status_height = STATUS_HEIGHT as c_int;
        }
    }
    frame_fix_height(wp);
    frame_fix_height(oldwin);
}

// ---------------------------------------------------------------------------
// Copying a window

pub unsafe fn win_init(newp: *mut win_T, oldp: *mut win_T, flags: c_int) {
    // SAFETY: the caller's promise -- two live windows.
    unsafe { init(Win::new(newp), Win::new(oldp), flags) };
}

/// Make window `newp` a copy of window `oldp`, from `win_init()`.
fn init(newp: Win, oldp: Win, flags: c_int) {
    let (mut newp, oldp) = (newp, oldp);
    let mut buf = oldp.buffer();
    newp.w_buffer = buf.raw();
    newp.w_s = &raw mut buf.b_s;
    buf.b_nwindows += 1;
    newp.w_cursor = oldp.w_cursor;
    newp.w_valid = WinValid::NONE;
    newp.w_curswant = oldp.w_curswant;
    newp.w_set_curswant = oldp.w_set_curswant;
    newp.w_topline = oldp.w_topline;
    newp.w_topfill = oldp.w_topfill;
    newp.w_leftcol = oldp.w_leftcol;
    newp.w_pcmark = oldp.w_pcmark;
    newp.w_prev_pcmark = oldp.w_prev_pcmark;
    newp.w_alt_fnum = oldp.w_alt_fnum;
    newp.w_wrow = oldp.w_wrow;
    newp.w_fraction = oldp.w_fraction;
    newp.w_prev_fraction_row = oldp.w_prev_fraction_row;
    // SAFETY: two live windows.
    unsafe { copy_jumplist(oldp.raw(), newp.raw()) };
    if flags & WSP_NEWLOC as c_int != 0 {
        // Don't copy the location list.
        newp.w_llist = ptr::null_mut::<qf_info_T>();
        newp.w_llist_ref = ptr::null_mut::<qf_info_T>();
    } else {
        copy_loclist_stack(oldp, newp);
    }
    newp.w_localdir = dup(oldp.w_localdir);
    newp.w_prevdir = dup(oldp.w_prevdir);

    // SAFETY: `'splitkeep'` is a NUL-terminated option string.
    let spk = unsafe { *p_spk.get() } as c_int;
    if spk != 'c' as c_int {
        if spk == 't' as c_int {
            newp.w_skipcol = oldp.w_skipcol;
        }
        newp.w_botline = oldp.w_botline;
        newp.w_prev_height = oldp.w_height;
        newp.w_prev_winrow = oldp.w_winrow;
    }

    // Copy the tag stack and its strings.
    for i in 0..oldp.w_tagstacklen as usize {
        let tag = &mut newp.w_tagstack[i];
        *tag = oldp.w_tagstack[i].clone();
        tag.tagname = dup(tag.tagname);
        tag.user_data = dup(tag.user_data);
    }
    newp.w_tagstackidx = oldp.w_tagstackidx;
    newp.w_tagstacklen = oldp.w_tagstacklen;
    newp.w_changelistidx = oldp.w_changelistidx;
    // SAFETY: `newp` is freshly allocated, so its fold list is still empty.
    unsafe { copy_folding_state(oldp, newp) };
    // The options and the argument list, which `win_new_tabpage` also copies
    // on its own (upstream's `win_init_some`).
    newp.w_alist = oldp.w_alist;
    // SAFETY: the argument list the two windows now share.
    unsafe { (*newp.w_alist).al_refcount.retain() };
    newp.w_arg_idx = oldp.w_arg_idx;
    // SAFETY: two live windows.
    unsafe { win_copy_options(oldp.raw(), newp.raw()) };
    newp.w_winbar_height = oldp.w_winbar_height;
}

/// `xstrdup`, keeping a null a null.
fn dup(s: *mut c_char) -> *mut c_char {
    if s.is_null() {
        return ptr::null_mut::<c_char>();
    }
    // SAFETY: a NUL-terminated string the window owns.
    unsafe { xstrdup(s) }
}
