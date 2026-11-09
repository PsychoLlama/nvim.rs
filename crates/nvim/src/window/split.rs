//! Splitting a window -- `win_split()` and `win_split_ins()`.
//!
//! [`split_ins`] is the whole operation, in stages: [`split_room`] decides the
//! new window's size from `'winheight'`/`'winwidth'` and refuses when there is
//! no room, [`insert_window`] links a `Window` into the list, [`split_frame`]
//! puts a frame beside or around the existing one, [`size_vertical`] and
//! [`size_horizontal`] hand out the rows and columns, and the tail
//! redistributes the space and enters the new window unless `WSP_NOENTER` said
//! not to.  [`win_split`] is the thin `:split` entry point over it, and
//! [`win_init`] copies one window's state onto another.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::narrow::as_index;
use crate::narrow::number_as_int;
use core::ffi::{c_char, c_int};

use super::arith::NextCurwin;
use super::*;
use crate::drawscreen::state::sc_col;
use crate::drawscreen::{UPD_NOT_VALID, comp_col, status_redraw_all};
use crate::ex_docmd::state::cmdmod;
use crate::fold::copy_folding_state;
use crate::mark::copy_jumplist;
use crate::message::msg_clr_eos_force;
use crate::message::state::{msg_col, msg_row};
use crate::r#move::WinValid;
use crate::option::vars::{
    P_WH, P_WIW, p_ch, p_ea, p_ead, p_ls, p_sb, p_spk, p_spr, p_wh, p_wiw, p_wmh, p_wmw,
};
use crate::option::win_copy_options;
use crate::types::{Failed, Integer, OptInt};
use crate::ui::state::kUIMultigrid;
use crate::ui::state::{Columns, Rows};
use crate::ui::{ui_call_win_hide, ui_has};
use crate::ui_compositor::ui_comp_remove_grid;
use crate::winfloat::win_float_anchor_laststatus;
use crate::winlayer::{FrameRef, Win, frames, tabs};

pub fn win_split(size: c_int, flags: c_int) -> Result<(), Failed> {
    split(size, flags)
}

/// Split the current window, `flags` being the `WSP_*` set: which half the new
/// window takes, whether it is vertical, and whether it is entered.
pub(crate) fn split(size: c_int, flags: c_int) -> Result<(), Failed> {
    let cur = Win::current();
    check_split_disallowed(cur)?;
    // When the ":tab" modifier was used, open a new tab page instead.
    if let Some(Ok(())) = may_open_tabpage() {
        return Ok(());
    }
    // Add flags from ":vertical", ":topleft" and ":botright".
    let flags = flags | cmdmod.with(|m| m.cmod_split);
    if flags & WSP_TOP != 0 && flags & WSP_BOT != 0 {
        return Err(err(WinError::TopAndBottom).into());
    }
    // When creating the help window make a snapshot of the window layout;
    // otherwise clear the snapshot, it is now invalid.
    for (flag, idx) in [(WSP_HELP, SNAP_HELP_IDX), (WSP_QUICKFIX, SNAP_QUICKFIX_IDX)] {
        if flags & flag != 0 {
            take_snapshot(idx);
        } else {
            drop_snapshot(TabPage::current(), idx);
        }
    }
    match split_ins(size, flags, None, 0, None) {
        Ok(_) => Ok(()),
        Err(NotSplit::Refused(refusal)) => Err(refusal.into()),
        Err(NotSplit::AutocmdWindow) => unreachable!("only a window given to insert is refused"),
    }
}

pub(crate) fn win_split_ins(
    size: c_int,
    flags: c_int,
    new_wp: Option<Win>,
    dir: u8,
    to_flatten: Option<FrameRef>,
) -> Result<Win, NotSplit> {
    split_ins(size, flags, new_wp, dir, to_flatten)
}

/// Why [`win_split_ins`] made or moved no window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NotSplit {
    /// The window to insert is an autocommand window, which stays floating.
    /// Nothing is said.
    AutocmdWindow,
    /// The layout refused, and said so.
    Refused(WinError),
}

impl From<WinError> for NotSplit {
    fn from(refusal: WinError) -> Self {
        Self::Refused(refusal)
    }
}

/// The room a split needs and the size it will take, from the first half of
/// `win_split_ins()`.
#[derive(Debug)]
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
/// On a refusal nothing has moved, and when `new_wp` was given the layout and
/// the sizes are exactly as they were. `to_flatten` is flattened just before
/// the frames are reorganised, so a failure leaves it unflattened.
fn split_ins(
    size: c_int,
    flags: c_int,
    new_wp: Option<Win>,
    dir: u8,
    to_flatten: Option<FrameRef>,
) -> Result<Win, NotSplit> {
    // `aucmd_win[]` should always remain floating.
    if new_wp.is_some() && is_autocmd_window(new_wp) {
        return Err(NotSplit::AutocmdWindow);
    }
    if new_wp.is_none() {
        trigger_winnewpre();
    }

    let mut oldwin = if flags & WSP_TOP != 0 {
        first_win()
    } else if flags & WSP_BOT != 0 || Win::current().w_floating {
        // Can't split a float: use the last non-floating window instead.
        lastwin_nofloating(None)
    } else {
        Win::current()
    };
    // Taken now because `oldwin` is reassigned below: the checks want the
    // window the split started from, not the one it ends up naming.
    let oldwin_id = oldwin.id();

    let vertical = flags & WSP_VERT != 0;
    let toplevel = flags & (WSP_TOP | WSP_BOT) != 0;
    let layout = if vertical { FR_ROW } else { FR_COL };

    // Add a status line when 'laststatus' is 1 and the first window is split.
    let mut need_status = 0;
    let first = first_win();
    if is_only_window(first, None) && p_ls() == 1 as OptInt && oldwin.w_status_height == 0 {
        if OptInt::from(oldwin.w_height) <= p_wmh() {
            return Err(err(WinError::NoRoom).into());
        }
        need_status = STATUS_HEIGHT;
        win_float_anchor_laststatus();
    }

    let room = if vertical {
        split_room_vertical(size, flags, oldwin, toplevel)?
    } else {
        split_room_horizontal(size, flags, &mut oldwin, toplevel, need_status)?
    };

    let wp = insert_window(flags, new_wp, oldwin, vertical);
    if let Some(to_flatten) = to_flatten {
        to_flatten.flatten();
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
    status_redraw_all();
    if need_status != 0 {
        // The message area is one line shorter now.
        msg_row.set(Rows.get() - 1);
        msg_col.set(sc_col.get());
        msg_clr_eos_force();
        comp_col();
        msg_row.set(Rows.get() - 1);
        msg_col.set(0);
    }

    if room.do_equal || dir != 0 {
        let axis = match (vertical, dir) {
            (true, b'v') => b'b',
            (true, _) => b'h',
            (false, b'h') => b'b',
            (false, _) => b'v',
        };
        equal(Some(wp), true, axis);
    } else if !is_autocmd_window(Some(wp)) {
        fix_scroll(false);
    }

    // Don't change the window height/width to 'winheight'/'winwidth' when a
    // size was given: hold the option at `size` across `win_enter_ext`, which
    // is where it would otherwise be applied.
    let saved = number_as_int(if vertical { p_wiw() } else { p_wh() });
    if size != 0 {
        if vertical {
            P_WIW.set(OptInt::from(size));
        } else {
            P_WH.set(OptInt::from(size));
        }
    }
    if flags & WSP_NOENTER == 0 {
        let new_flags = if new_wp.is_none() {
            WEE_TRIGGER_NEW_AUTOCMDS
        } else {
            0
        };
        let enter = WEE_TRIGGER_ENTER_AUTOCMDS | WEE_TRIGGER_LEAVE_AUTOCMDS;
        // This fires WinNew/WinEnter/WinLeave, after which nothing derived
        // from `wp` is read.
        enter_ext(wp, new_flags | enter);
    }
    if vertical {
        P_WIW.set(OptInt::from(saved));
    } else {
        P_WH.set(OptInt::from(saved));
    }
    // An autocommand may have closed `oldwin`, so the identity was taken
    // before `enter_ext` fired them.
    if win_valid(oldwin_id) {
        oldwin.w_pos_changed = true;
    }
    Ok(wp)
}

/// The width the new window gets from a `:vsplit`, or E36 when the layout
/// has no room for one.
fn split_room_vertical(
    size: c_int,
    flags: c_int,
    oldwin: Win,
    toplevel: bool,
) -> Result<Room, WinError> {
    // The current window requires at least one column.
    let wmw1 = if p_wmw() == 0 as OptInt {
        1
    } else {
        number_as_int(p_wmw())
    };
    let mut needed = wmw1 + 1;
    if flags & WSP_ROOM != 0 {
        needed += number_as_int(p_wiw()) - wmw1;
    }
    let top = current_topframe();
    let (minwidth, available) = if toplevel {
        (top.min_width(NextCurwin::NoWin), top.fr_width)
    } else if p_ea() {
        // With 'equalalways' the room is the whole screen's, so every frame in
        // every row above `oldwin` counts towards the minimum.
        let mut min = oldwin.frame().min_width(NextCurwin::NoWin);
        min += oldwin.frame().siblings_min_width();
        (min, top.fr_width)
    } else {
        let frame = oldwin.frame();
        (frame.min_width(NextCurwin::NoWin), frame.fr_width)
    };
    needed += minwidth;
    if available < needed {
        return Err(err(WinError::NoRoom));
    }

    let mut new_size = size;
    if new_size == 0 {
        new_size = oldwin.w_width / 2;
    }
    new_size = new_size.min(available - minwidth - 1).max(wmw1);

    // If it doesn't fit in the current window, need `win_equal()`.
    let mut do_equal = OptInt::from(oldwin.w_width - new_size - 1) < p_wmw();

    // Don't take columns for the new window from a 'winfixwidth' window: take
    // them from a window to the left or right instead, plus one separator.
    if oldwin.w_onebuf_opt.wo_wfw != 0 {
        setwidth_win(oldwin.w_width + new_size + 1, oldwin);
    }

    // Only make all windows the same width if one of them (except oldwin) is
    // wider than one of the split windows.
    if !do_equal && p_ea() && size == 0 && ead() != b'v' {
        do_equal = wider_sibling(oldwin, |w| {
            w.w_width > new_size || w.w_width > oldwin.w_width - new_size - 1
        });
    }
    Ok(Room {
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
) -> Result<Room, WinError> {
    // The current window requires at least one line plus its window bar.
    let wmh1 = number_as_int(p_wmh()).max(1) + oldwin.w_winbar_height;
    let mut needed = wmh1 + STATUS_HEIGHT;
    if flags & WSP_ROOM != 0 {
        needed += number_as_int(p_wh()) - wmh1 + oldwin.w_winbar_height;
    }
    if p_ch() < 1 as OptInt {
        needed += 1; // adjust for 'cmdheight' = 0
    }
    let top = current_topframe();
    let (minheight, available) = if toplevel {
        (
            top.min_height(NextCurwin::NoWin) + need_status,
            top.fr_height,
        )
    } else if p_ea() {
        let mut min = oldwin.frame().min_height(NextCurwin::NoWin) + need_status;
        min += oldwin.frame().siblings_min_height();
        (min, top.fr_height)
    } else {
        let frame = oldwin.frame();
        (
            frame.min_height(NextCurwin::NoWin) + need_status,
            frame.fr_height,
        )
    };
    needed += minheight;
    if available < needed {
        return Err(err(WinError::NoRoom));
    }

    let mut oldwin_height = oldwin.w_height;
    if need_status != 0 {
        oldwin.w_status_height = STATUS_HEIGHT;
        oldwin_height -= STATUS_HEIGHT;
    }
    let mut new_size = size;
    if new_size == 0 {
        new_size = oldwin_height / 2;
    }
    new_size = new_size
        .min(available - minheight - STATUS_HEIGHT)
        .max(wmh1);

    let mut do_equal = OptInt::from(oldwin_height - new_size - STATUS_HEIGHT) < p_wmh();

    // Don't take lines for the new window from a 'winfixheight' window.
    let mut did_set_fraction = false;
    if oldwin.w_onebuf_opt.wo_wfh != 0 {
        // Set `w_fraction` now so the cursor keeps the same relative vertical
        // position, using the old height.
        save_fraction(*oldwin);
        did_set_fraction = true;
        setheight_win(oldwin.w_height + new_size + STATUS_HEIGHT, *oldwin);
        oldwin_height = oldwin.w_height;
        if need_status != 0 {
            oldwin_height -= STATUS_HEIGHT;
        }
    }

    if !do_equal && p_ea() && size == 0 && ead() != b'h' {
        let oldwin = *oldwin;
        do_equal = wider_sibling(oldwin, |w| {
            w.w_height > new_size || w.w_height > oldwin_height - new_size - STATUS_HEIGHT
        });
    }
    Ok(Room {
        new_size,
        do_equal,
        oldwin_height,
        did_set_fraction,
    })
}

/// The first character of `'eadirection'`.
fn ead() -> u8 {
    p_ead(cstr::first)
}

/// Whether any window beside `oldwin` in its own row or column answers `taller`
/// -- the test that decides whether a split without a size equalizes.
fn wider_sibling(oldwin: Win, taller: impl Fn(Win) -> bool) -> bool {
    let Some(parent) = oldwin.frame().parent() else {
        return false;
    };
    parent
        .children()
        .filter_map(FrameRef::win)
        .any(|w| w != oldwin && taller(w))
}

impl FrameRef {
    /// The minimum width of every frame in every *row* above `frame`, its own
    /// ancestors excluded -- what a split with 'equalalways' has to leave alone.
    fn siblings_min_width(self) -> c_int {
        let mut total = 0;
        let mut prev = self;
        for frp in self.ancestors() {
            if c_int::from(frp.fr_layout) == FR_ROW {
                for frp2 in frp.children().filter(|frp2| *frp2 != prev) {
                    total += frp2.min_width(NextCurwin::NoWin);
                }
            }
            prev = frp;
        }
        total
    }

    /// [`FrameRef::siblings_min_width`] for the columns above `frame`.
    fn siblings_min_height(self) -> c_int {
        let mut total = 0;
        let mut prev = self;
        for frp in self.ancestors() {
            if c_int::from(frp.fr_layout) == FR_COL {
                for frp2 in frp.children().filter(|frp2| *frp2 != prev) {
                    total += frp2.min_height(NextCurwin::NoWin);
                }
            }
            prev = frp;
        }
        total
    }

    /// A copy of `frame`, allocated: the C's `*frp = *curfrp` over a fresh
    /// `xcalloc`.
    ///
    /// Written out field by field because `Frame` is deliberately neither
    /// `Copy` nor `Clone` — a frame is a node of the layout tree, and this is the
    /// one place in the editor that duplicates one. The links come across with
    /// the rest, exactly as the struct assignment copied them, and the caller
    /// rewires them straight afterwards.
    fn new_like(self) -> FrameRef {
        let mut copy = new_frame();
        copy.fr_layout = self.fr_layout;
        copy.fr_width = self.fr_width;
        copy.fr_newwidth = self.fr_newwidth;
        copy.fr_height = self.fr_height;
        copy.fr_newheight = self.fr_newheight;
        copy.fr_parent = self.fr_parent;
        copy.fr_next = self.fr_next;
        copy.fr_prev = self.fr_prev;
        copy.fr_child = self.fr_child;
        copy.fr_win = self.fr_win;
        copy
    }
}

/// Put the new window on the window list beside `oldwin`, allocating one when
/// the caller did not bring its own, and give it a frame.
fn insert_window(flags: c_int, new_wp: Option<Win>, oldwin: Win, _vertical: bool) -> Win {
    let below = flags & WSP_TOP == 0
        && (flags & WSP_BOT != 0
            || flags & WSP_BELOW != 0
            || (flags & WSP_ABOVE == 0 && split_after(_vertical)));
    let after = if below { Some(oldwin) } else { oldwin.prev() };
    let Some(mut wp) = new_wp else {
        let wp = win_alloc(after, false);
        attach_frame(wp);
        // Make the contents of the new window the same as the current one.
        win_init(wp, Win::current(), flags);
        return wp;
    };
    win_append(after, wp, None);
    if !wp.w_floating {
        return wp;
    }
    // A float becoming an ordinary window gives up its own grid.
    ui_comp_remove_grid(wp.grid_alloc());
    if ui_has(kUIMultigrid) {
        wp.w_pos_changed = true;
    } else {
        // No longer a float: a non-multigrid UI shouldn't draw it as such.
        ui_call_win_hide(Integer::from(wp.w_grid_alloc.handle));
        free_grid(wp, true);
    }
    // External windows are independent of tab pages, and may have been the
    // `curwin` of others.
    if wp.w_config.external {
        for mut tp in tabs().filter(|tp| !tp.is_current()) {
            if tp.tp_curwin == Some(wp.id()) {
                tp.tp_curwin = tp.tp_firstwin;
            }
        }
    }
    wp.w_floating = false;
    attach_frame(wp);
    // A non-floating window doesn't store a float config or have a border.
    clear_float_config(&mut wp.w_config, true);
    wp.w_border_adj = [0; 4];
    wp
}

/// Whether the new window goes below (or right of) the old one by default.
fn split_after(vertical: bool) -> bool {
    if vertical { p_spr() } else { p_sb() }
}

/// Put the new window's frame into the tree: answer the frame the split is
/// happening in, the new window's frame, and whether it goes before.
fn split_frame(
    flags: c_int,
    window: Win,
    oldwin: Win,
    toplevel: bool,
    vertical: bool,
    layout: c_int,
) -> (FrameRef, FrameRef, bool) {
    let top = current_topframe();
    let (mut curfrp, before) = if toplevel {
        let same_axis = (c_int::from(top.fr_layout) == FR_COL && !vertical)
            || (c_int::from(top.fr_layout) == FR_ROW && vertical);
        let frp = if same_axis {
            let first = top.child().expect("a row or column has a child");
            if flags & WSP_BOT != 0 {
                frames(Some(first)).last().expect("at least one")
            } else {
                first
            }
        } else {
            top
        };
        (frp, flags & WSP_TOP != 0)
    } else {
        let before = if flags & WSP_BELOW != 0 {
            false
        } else if flags & WSP_ABOVE != 0 {
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
        .is_none_or(|p| c_int::from(p.fr_layout) != layout)
    {
        let mut inner = curfrp.new_like();
        let mut outer = curfrp;
        outer.fr_layout = c_char::try_from(layout).expect("FR_ROW or FR_COL");
        inner.fr_parent = Some(outer.id());
        inner.fr_next = None;
        inner.fr_prev = None;
        outer.fr_child = Some(inner.id());
        outer.fr_win = None;
        curfrp = inner;
        match inner.win() {
            // `oldwin`'s frame moved: it now lives one level down.
            Some(_) => {
                let mut oldwin = oldwin;
                oldwin.w_frame = Some(inner.id());
            }
            None => {
                for mut child in inner.children() {
                    child.fr_parent = Some(curfrp.id());
                }
            }
        }
    }

    let mut frp = window.frame();
    frp.fr_parent = curfrp.fr_parent;
    if before {
        curfrp.prepend_sibling(frp);
    } else {
        curfrp.append_sibling(frp);
    }
    (curfrp, frp, before)
}

/// Hand out the columns for a `:vsplit`.
#[expect(clippy::too_many_arguments, reason = "upstream's one long arm")]
fn size_vertical(
    flags: c_int,
    window: Win,
    oldwin: Win,
    curfrp: FrameRef,
    frp: FrameRef,
    room: &Room,
    before: bool,
    toplevel: bool,
    need_status: c_int,
) {
    let (mut wp, mut oldwin, mut frp) = (window, oldwin, frp);
    // 'scroll' is not inherited by a horizontal split, but is by a vertical
    // one.
    wp.w_onebuf_opt.wo_scr = Win::current().w_onebuf_opt.wo_scr;
    if need_status != 0 {
        new_win_height(oldwin, oldwin.w_height - 1);
        oldwin.w_status_height = need_status;
    }
    if toplevel {
        wp.w_winrow = tabline_rows();
        let stl = c_int::from(p_ls() == 1 as OptInt || p_ls() == 2 as OptInt);
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
        if flags & WSP_BOT != 0 {
            curfrp.set_vsep(true);
        }
        let sep = c_int::from(flags & WSP_TOP != 0);
        new_width(
            curfrp,
            curfrp.fr_width - (room.new_size + sep),
            flags & WSP_TOP != 0,
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
    window: Win,
    oldwin: Win,
    curfrp: FrameRef,
    frp: FrameRef,
    room: &Room,
    before: bool,
    toplevel: bool,
) {
    let (mut wp, mut oldwin, mut frp) = (window, oldwin, frp);
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
        wp.w_hsep_height = c_int::from(is_stl_global);
    } else {
        wp.w_hsep_height = oldwin.w_hsep_height;
        oldwin.w_hsep_height = c_int::from(is_stl_global);
    }
    if toplevel {
        let mut new_fr_height = curfrp.fr_height - room.new_size;
        if is_stl_global {
            if flags & WSP_BOT != 0 {
                curfrp.add_hsep();
            } else {
                new_fr_height -= 1;
            }
        } else {
            if !(flags & WSP_BOT != 0 && p_ls() == 0 as OptInt) {
                new_fr_height -= STATUS_HEIGHT;
            }
            if flags & WSP_BOT != 0 {
                curfrp.add_statusline();
            }
        }
        new_height(curfrp, new_fr_height, flags & WSP_TOP != 0, false, false);
    } else {
        new_win_height(oldwin, room.oldwin_height - (room.new_size + STATUS_HEIGHT));
    }
    if before {
        wp.w_winrow = oldwin.w_winrow;
        if is_stl_global {
            wp.w_status_height = 0;
            oldwin.w_winrow += wp.w_height + 1;
        } else {
            wp.w_status_height = STATUS_HEIGHT;
            oldwin.w_winrow += wp.w_height + STATUS_HEIGHT;
        }
    } else if is_stl_global {
        wp.w_winrow = oldwin.w_winrow + oldwin.w_height + 1;
        wp.w_status_height = 0;
    } else {
        wp.w_winrow = oldwin.w_winrow + oldwin.w_height + STATUS_HEIGHT;
        wp.w_status_height = old_status_height;
        if flags & WSP_BOT == 0 {
            oldwin.w_status_height = STATUS_HEIGHT;
        }
    }
    frame_fix_height(wp);
    frame_fix_height(oldwin);
}

// ---------------------------------------------------------------------------
// Copying a window

pub fn win_init(newp: Win, oldp: Win, flags: c_int) {
    init(newp, oldp, flags);
}

/// Make window `newp` a copy of window `oldp`, from `win_init()`.
fn init(newp: Win, oldp: Win, flags: c_int) {
    let (mut newp, oldp) = (newp, oldp);
    let mut buf = oldp.buffer();
    newp.w_buffer = buf;
    newp.w_s = buf.syntax_block();
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
    copy_jumplist(oldp, newp);
    if flags & WSP_NEWLOC != 0 {
        // Don't copy the location list.
        newp.w_llist = None;
        newp.w_llist_ref = None;
    } else {
        oldp.copy_location_lists_to(newp);
    }
    newp.w_localdir = oldp.w_localdir.clone();
    newp.w_prevdir = oldp.w_prevdir.clone();

    let spk = p_spk(cstr::first);
    if spk != b'c' {
        if spk == b't' {
            newp.w_skipcol = oldp.w_skipcol;
        }
        newp.w_botline = oldp.w_botline;
        newp.w_prev_height = oldp.w_height;
        newp.w_prev_winrow = oldp.w_winrow;
    }

    // Copy the tag stack and its strings.
    for i in 0..as_index(oldp.w_tagstacklen) {
        newp.w_tagstack[i] = oldp.w_tagstack[i].clone();
    }
    newp.w_tagstackidx = oldp.w_tagstackidx;
    newp.w_tagstacklen = oldp.w_tagstacklen;
    newp.w_changelistidx = oldp.w_changelistidx;
    copy_folding_state(oldp, newp);
    // The options and the argument list, which `win_new_tabpage` also copies
    // on its own (upstream's `win_init_some`).
    newp.w_alist = oldp.w_alist;
    crate::arglist::alist_retain(newp.w_alist);
    newp.w_arg_idx = oldp.w_arg_idx;
    win_copy_options(oldp, newp);
    newp.w_winbar_height = oldp.w_winbar_height;
}
