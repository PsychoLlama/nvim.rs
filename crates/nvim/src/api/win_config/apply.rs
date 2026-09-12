//! `nvim_win_set_config()`: reconfiguring an existing window.
//!
//! The two directions a reconfiguration can take: [`Relayout`] turns a
//! window into (or moves) a split, which may mean splitting a different
//! parent, changing the direction, or leaving the float layout entirely; and
//! `apply_float` applies a float config, including the tabpage move a
//! `relative` window may need.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api_error;
use crate::winlayer::{FrameId, FrameRef, TabPage, Win, WinId};
use core::ffi::c_int;

/// `None` for "the current tab page", which is how the window family spells
/// it throughout.
fn other_tab(tabpage: TabPage) -> Option<TabPage> {
    (tabpage != TabPage::current()).then_some(tabpage)
}

/// The tab page a window was just found on. [`win_find_tabpage`] answers an
/// `Option` because it is asked about windows that may be gone; here it is
/// asked about a window the caller promised is live.
fn expect_tab(tabpage: Option<TabPage>) -> TabPage {
    tabpage.expect("a live window is on a tab page")
}

/// How many frames sit in `frame`'s row or column, `frame` included.
fn sibling_count(frame: FrameRef) -> c_int {
    frame
        .children()
        .count()
        .try_into()
        .expect("a row holds fewer frames than an int can count")
}

/// What applying a config to a window came to.
///
/// Not a two-valued answer, because the split case is not: `win_split_ins`
/// runs autocommands, and one of them throwing does not undo the split the
/// editor has already made. The caller finishes applying the rest of the
/// config either way and reports the exception last -- which is what the
/// `bool` answer and a borrowed error slot used to say between them, and
/// what nothing in either signature admitted.
enum Applied {
    /// The window was reconfigured.
    Done,
    /// It was reconfigured, and an autocommand raised while it was.
    Raised(Error),
    /// Nothing was reconfigured and there is nothing to report: a window
    /// handle that resolved to no window at all.
    Refused,
}

/// A refusal, and whether the editor is still standing on the window the
/// relayout moved away from.
enum Refusal {
    /// Nothing moved, or the move itself is what failed: report and leave.
    AsIs(Error),
    /// The current window was moved off the one being split, and has to go
    /// back before the refusal is reported.
    Restore(Error),
}

/// One window on its way into a split.
///
/// Every window here is named twice over -- as a [`Win`] and as a [`WinId`]
/// -- because [`win_goto`] fires autocommands that can close either of them,
/// and every question asked after it is about exactly that.
struct Relayout {
    /// The window being moved, and its identity.
    win: Win,
    win_id: WinId,
    /// Whether it was a split before this call.
    was_split: bool,
    /// Which side of its neighbour it is to end up on.
    split: WinSplit,
    /// The window to split, its identity, and its tab page.
    parent: Option<Win>,
    parent_id: Option<WinId>,
    parent_tp: Option<TabPage>,
    /// The tab page the moved window is on, re-read after every `win_goto`.
    win_tp: Option<TabPage>,
    /// Whether the current window is leaving its tab page for this split.
    curwin_moving_tp: bool,
    /// The direction [`winframe_remove`] took the window out in, and the
    /// frame it left unflattened -- held as an identity, because
    /// `win_split_ins` can free frames. Both are what puts the layout back
    /// if the move fails.
    dir: c_int,
    unflat_altfr: Option<FrameId>,
    /// The window that takes the moved one's place where it was.
    altwin: Option<Win>,
}

impl Relayout {
    /// Resolve the window to split against and check that the move is
    /// allowed at all. `None` for a handle that named no window.
    fn open(
        win: Win,
        config: CfgKeys,
        fconfig: WinCfg,
        was_split: bool,
    ) -> Result<Option<Self>, Error> {
        let parent_handle = config.win.unwrap_or(0);
        let (parent, parent_tp) = if parent_handle == 0 {
            (Some(Win::current()), Some(TabPage::current()))
        } else if parent_handle > 0 {
            let Some(found) = find_window_by_handle(fconfig.window)? else {
                return Ok(None);
            };
            (Some(found), win_find_tabpage(found.id()))
        } else {
            (None, None)
        };
        let win_id = win.id();
        let win_tp = win_find_tabpage(win_id);
        if let Some(parent) = parent {
            if parent.w_floating {
                return Err(Error::exception(c"Cannot split a floating window"));
            }
            if win_tp != parent_tp {
                win_can_move_tp(win, expect_tab(win_tp))?;
            }
        }
        check_split_disallowed_err(win)?;
        Ok(Some(Self {
            win,
            win_id,
            was_split,
            split: fconfig.split,
            parent,
            parent_id: parent.map(Win::id),
            parent_tp,
            win_tp,
            curwin_moving_tp: win == Win::current() && parent.is_some() && win_tp != parent_tp,
            dir: 0,
            unflat_altfr: None,
            altwin: None,
        }))
    }

    /// Move the window into the split, going back to it if the move failed
    /// after the editor had already left it.
    fn run(&mut self) -> Result<Applied, Error> {
        match self.move_window() {
            Ok(applied) => Ok(applied),
            Err(Refusal::AsIs(why)) => Err(why),
            Err(Refusal::Restore(why)) => {
                if self.curwin_moving_tp && win_valid(self.win_id) {
                    win_goto(self.win);
                }
                Err(why)
            }
        }
    }

    /// Out of the layout it is in and into the split, in that order.
    fn move_window(&mut self) -> Result<Applied, Refusal> {
        self.switch_away()?;
        self.detach()?;
        self.insert()
    }

    /// Leave the window being moved, when it is the current one and the
    /// split is on another tab page, and check that both windows survived
    /// the autocommands that leaving fired.
    fn switch_away(&mut self) -> Result<(), Refusal> {
        if !self.curwin_moving_tp {
            return Ok(());
        }
        let altwin = win_find_altwin(self.win, expect_tab(self.win_tp)).expect("altwin");
        win_goto(altwin);
        if Win::current_raw() == self.win.raw() {
            let handle = self.win_id.handle();
            let why = api_error!(
                kErrorTypeException,
                "Failed to switch away from window {handle}"
            );
            // The editor is still on the window, so there is nothing to
            // put back.
            return Err(Refusal::AsIs(why));
        }
        self.win_tp = win_find_tabpage(self.win_id);
        // `win_valid_any_tab` is the check for whether the parent is still
        // there at all.
        let live_parent = self
            .parent
            .filter(|_| self.parent_id.is_some_and(win_valid_any_tab));
        let (Some(_), Some(parent)) = (self.win_tp, live_parent) else {
            let why = Error::exception(c"Windows to split were closed");
            return Err(Refusal::Restore(why));
        };
        if self.was_split == self.win.w_floating || parent.w_floating {
            let why = Error::exception(c"Floating state of windows to split changed");
            return Err(Refusal::Restore(why));
        }
        Ok(())
    }

    /// Take the window out of the layout it is in, remembering what has to
    /// be put back if the insertion below fails.
    fn detach(&mut self) -> Result<(), Refusal> {
        if self.was_split {
            self.unframe()?;
        } else {
            self.altwin = win_float_find_altwin(self.win, other_tab(expect_tab(self.win_tp)));
        }
        win_remove(self.win, other_tab(expect_tab(self.win_tp)));
        if self.win_tp == Some(TabPage::current()) {
            last_status(false);
            win_comp_pos();
        }
        Ok(())
    }

    /// The non-floating case of [`detach`](Self::detach): the window sits in
    /// a frame of the layout tree, and comes out of it.
    fn unframe(&mut self) -> Result<(), Refusal> {
        let Some(parent_frame) = self.win.frame().parent() else {
            let why = Error::exception(c"Cannot move last non-floating window");
            return Err(Refusal::Restore(why));
        };
        if !self.parent.is_some_and(|p| p.handle == self.win.handle) {
            self.remove_frame();
            return Ok(());
        }

        // Splitting the window against itself: it is taken out first, and
        // the neighbour it leaves behind becomes the window to split.
        let n_frames = sibling_count(parent_frame);
        if n_frames < 2 {
            let why = Error::exception(c"Cannot split window into itself");
            return Err(Refusal::Restore(why));
        }
        let neighbour = if n_frames > 2 {
            // Only a nested row or column has a neighbour on the side the
            // split is going.
            let ahead = self.split == kWinSplitAbove || self.split == kWinSplitLeft;
            let nested = parent_frame.parent().is_some();
            let found = nested.then(|| {
                if ahead {
                    self.win.next()
                } else {
                    self.win.prev()
                }
            });
            self.remove_frame();
            found.flatten()
        } else {
            self.remove_frame();
            self.altwin
        };
        // The parent is replaced here, so the identity it stands for is too
        // -- the neighbour is live, just found.
        self.parent = neighbour;
        self.parent_id = neighbour.map(Win::id);
        Ok(())
    }

    /// Take the window's frame out of the layout, keeping what
    /// [`winframe_restore`] would need to put it back.
    fn remove_frame(&mut self) {
        let removed = winframe_remove(self.win, other_tab(expect_tab(self.win_tp)), true);
        (self.altwin, self.dir, self.unflat_altfr) = (removed.win, removed.dir, removed.unflat);
    }

    /// Put the window back into the layout as a split of its parent, and
    /// undo the detachment if that does not work.
    fn insert(&mut self) -> Result<Applied, Refusal> {
        let flags = win_split_flags(self.split, self.parent.is_none()) | WSP_NOENTER.cast_signed();
        self.parent_tp = match self.parent_id {
            None => Some(TabPage::current()),
            Some(parent) => win_find_tabpage(parent),
        };
        let mut tstate = TryState::default();
        // SAFETY: `tstate` is this frame's own, live until `try_leave`.
        unsafe { try_enter(&raw mut tstate) };
        let split_ok = self.split_ins(flags);
        // SAFETY: `tstate` is what the `try_enter` above filled in.
        let raised = unsafe { try_leave(&raw mut tstate) }.err();

        if split_ok {
            let mut tp = expect_tab(self.win_tp);
            if self.win_tp != self.parent_tp && tp.tp_curwin == Some(self.win_id) {
                tp.tp_curwin = self.altwin.map(Win::id);
            }
            return Ok(match raised {
                Some(raised) => Applied::Raised(raised),
                None => Applied::Done,
            });
        }
        if self.was_split
            && let Some(unflat) = self.unflat_altfr.and_then(FrameId::get)
        {
            winframe_restore(self.win, self.dir, unflat);
        }
        let why = raised.unwrap_or_else(|| {
            let handle = self.win_id.handle();
            api_error!(
                kErrorTypeException,
                "Failed to move window {handle} into split"
            )
        });
        Err(Refusal::Restore(why))
    }

    /// [`win_split_ins`] inside whichever window the split goes into,
    /// putting the moved window back in its old place if it refuses.
    fn split_ins(&mut self, flags: c_int) -> bool {
        let need_switch = self.parent.is_some_and(|parent| !parent.is_current());
        let mut switchwin = SwitchWin::default();
        if need_switch {
            let parent_tp = expect_tab(self.parent_tp);
            let parent = self.parent.expect("`need_switch` says there is a parent");
            // SAFETY: `switchwin` is this frame's own, and `parent`/
            // `parent_tp` are the live window and tab page to split in.
            let result = unsafe { switch_win(&raw mut switchwin, parent, Some(parent_tp), true) };
            debug_assert!(result.is_ok(), "the window was switched to");
        }
        let split_ok = win_split_ins(
            0,
            flags,
            Some(self.win),
            0,
            self.unflat_altfr.and_then(FrameId::get),
        )
        .is_some();
        if !split_ok {
            win_append(
                self.win.prev(),
                self.win,
                other_tab(expect_tab(self.win_tp)),
            );
        }
        if need_switch {
            // SAFETY: the matching restore of the switch above.
            unsafe { restore_win(&raw mut switchwin, true) };
        }
        split_ok
    }
}

/// Apply the split half of `fconfig` to `win`: make it a split, move it to
/// another parent, or change which side of one it is on.
///
/// Safe: every argument is a handle whose construction already carries the
/// promise that what it names outlives the call.
fn apply_split(mut win: Win, config: CfgKeys, mut fconfig: WinCfg) -> Result<Applied, Error> {
    let was_split = !win.w_floating;
    let has_split = config.split.is_some();
    let has_vertical = config.vertical.is_some();
    let old_split = win_split_dir(win);
    if has_vertical && !has_split {
        fconfig.split = if config.vertical.unwrap_or(false) {
            if old_split == kWinSplitRight || p_spr.get() != 0 {
                kWinSplitRight
            } else {
                kWinSplitLeft
            }
        } else if old_split == kWinSplitBelow || p_sb.get() != 0 {
            kWinSplitBelow
        } else {
            kWinSplitAbove
        };
    }
    // Nothing about the layout is changing when neither key was given, or
    // when the window is already a split on the same side of the same
    // parent; then only the size below is applied.
    let stays_put = !has_vertical && !has_split
        || was_split && config.win.is_none() && old_split == fconfig.split;
    let mut applied = Applied::Done;
    if !stays_put {
        match Relayout::open(win, config, fconfig, was_split)? {
            None => return Ok(Applied::Refused),
            Some(mut relayout) => applied = relayout.run()?,
        }
    }

    if config.width.is_some() {
        win_setwidth_win(fconfig.width, win);
    }
    if config.height.is_some() {
        win_setheight_win(fconfig.height, win);
    }
    if !was_split {
        // SAFETY: the caller's config.
        unsafe { clear_float_config(fconfig.raw(), false) };
    }
    let merged = (*fconfig).clone();
    // SAFETY: the caller's window, whose config field is live with it.
    unsafe { merge_win_config(&raw mut win.w_config, merged) };
    Ok(applied)
}

/// Apply the float half of `fconfig` to `win`, including the move to another
/// tab page a `win` key may ask for.
///
/// Safe: as [`apply_split`].
fn apply_float(win: Win, config: CfgKeys, fconfig: WinCfg) -> Result<Applied, Error> {
    let mut float = Refloat {
        win,
        win_tp: win_find_tabpage(win.id()),
        parent_id: win.id(),
        parent_tp: win_find_tabpage(win.id()),
        curwin_moving_tp: false,
        altwin: None,
    };
    if config.win.is_some() {
        let Some(found) = find_window_by_handle(fconfig.window)? else {
            return Ok(Applied::Refused);
        };
        float.parent_id = found.id();
        float.parent_tp = win_find_tabpage(float.parent_id);
    }
    let moved = float.move_to_parent_tab(fconfig);
    // The `win_goto` in `leave_tab` left the editor on another window. A
    // move that got there goes back to this one; a move that finished does
    // not, because the window it left is on the other tab page now.
    let restores = matches!(moved, Err(Refusal::Restore(_)) | Ok(false));
    if restores && float.curwin_moving_tp && win_valid(win.id()) {
        win_goto(win);
    }
    match moved {
        Err(Refusal::AsIs(why) | Refusal::Restore(why)) => return Err(why),
        // A window handle that resolved to no window: upstream reports
        // nothing and leaves the window as it was.
        Ok(false) => return Ok(Applied::Refused),
        Ok(true) => {}
    }
    let merged = (*fconfig).clone();
    win_config_float(win, merged);
    Ok(Applied::Done)
}

/// A float on its way to another tab page, and the state that move shares
/// with the refusal that puts the current window back.
struct Refloat {
    win: Win,
    /// The tab page the float is on, re-read after every `win_goto`.
    win_tp: Option<TabPage>,
    /// The window the float is relative to, and its tab page.
    parent_id: WinId,
    parent_tp: Option<TabPage>,
    /// Whether the current window left its tab page for this move.
    curwin_moving_tp: bool,
    /// The window that takes the float's place where it was.
    altwin: Option<Win>,
}

impl Refloat {
    /// Move the window to the tab page its parent is on, making it a float
    /// first if it was not one. `false` refuses without a reason.
    fn move_to_parent_tab(&mut self, fconfig: WinCfg) -> Result<bool, Refusal> {
        if self.win_tp != self.parent_tp {
            self.leave_tab()?;
        }
        if !self.win.w_floating && !self.unfloat_to_float(fconfig)? {
            return Ok(false);
        }
        if self.win_tp != self.parent_tp {
            self.relist();
        }
        Ok(true)
    }

    /// Leave the tab page the window is on, when it is the current window,
    /// and check that both tab pages survived the autocommands.
    fn leave_tab(&mut self) -> Result<(), Refusal> {
        let allowed = win_can_move_tp(self.win, expect_tab(self.win_tp));
        allowed.map_err(Refusal::AsIs)?;
        self.altwin = win_find_altwin(self.win, expect_tab(self.win_tp));
        debug_assert!(self.altwin.is_some(), "altwin");
        if !self.win.is_current() {
            return Ok(());
        }
        self.curwin_moving_tp = true;
        win_goto(self.altwin.expect("altwin"));
        if self.win.is_current() {
            let handle = self.win.id().handle();
            let why = api_error!(
                kErrorTypeException,
                "Failed to switch away from window {handle}"
            );
            // The editor is still on the window, so there is nothing to
            // put back.
            return Err(Refusal::AsIs(why));
        }
        self.win_tp = win_find_tabpage(self.win.id());
        self.parent_tp = win_find_tabpage(self.parent_id);
        if self.win_tp.is_none() || self.parent_tp.is_none() {
            let why = Error::exception(c"Target windows were closed");
            return Err(Refusal::Restore(why));
        }
        if self.win_tp != self.parent_tp {
            let allowed = win_can_move_tp(self.win, expect_tab(self.win_tp));
            allowed.map_err(Refusal::Restore)?;
        }
        self.altwin = win_find_altwin(self.win, expect_tab(self.win_tp));
        debug_assert!(self.altwin.is_some(), "altwin");
        Ok(())
    }

    /// Turn the window into a float, which is what a `relative` key asks of
    /// a window that is still a split.
    ///
    /// `false` is the answer a handle that resolved to no window gives, and
    /// is not a failure: upstream leaves the window as it was and reports
    /// nothing.
    fn unfloat_to_float(&mut self, fconfig: WinCfg) -> Result<bool, Refusal> {
        let config = (*fconfig).clone();
        let made = win_new_float(Some(self.win), false, config).map_err(Refusal::Restore)?;
        if made.is_none() {
            return Ok(false);
        }
        redraw_later(self.win, UPD_NOT_VALID);
        Ok(true)
    }

    /// Move the window from one tab page's window list to the other's.
    fn relist(&mut self) {
        let append_tp = other_tab(expect_tab(self.parent_tp));
        win_remove(self.win, other_tab(expect_tab(self.win_tp)));
        win_append(Some(lastwin_nofloating(append_tp)), self.win, append_tp);
        let mut tp = expect_tab(self.win_tp);
        if !tp.is_current() && tp.tp_curwin == Some(self.win.id()) {
            tp.tp_curwin = self.altwin.map(Win::id);
        }
        // SAFETY: the window's own grid, which is live with it.
        unsafe { ui_comp_remove_grid(&raw mut self.win.w_grid_alloc) };
        redraw_later(self.win, UPD_NOT_VALID);
        set_must_redraw(UPD_NOT_VALID);
    }
}

/// Reconfigure `win` from the `config` dictionary.
///
/// # Safety
/// `config` must be the caller's decoded keyset -- NUL-terminated strings and
/// arrays that name their own items.
pub unsafe fn nvim_win_set_config(
    win: WindowHandle,
    config: *mut KeyDict_win_config,
) -> Result<(), Error> {
    let mut error = Error::none();
    // SAFETY: `error` is this frame's own slot, live for the whole call, and
    // `config` is the caller's keyset.
    let (report, keys) = unsafe { (ErrSlot::new(&mut error), CfgKeys::new(config)) };
    // SAFETY: `error` is this frame's slot; the lookup answers a live window or
    // a null.
    let Some(w) = find_window_by_handle(win)? else {
        return Ok(());
    };
    let was_split = !w.w_floating;
    let has_split = keys.split.is_some();
    let has_vertical = keys.vertical.is_some();
    let old_style = w.w_config.style;
    let mut fconfig = w.w_config.clone();
    let external = keys.external.unwrap_or(false);
    let relative_named = keys.relative.as_ref().is_some_and(|r| !r.is_empty());
    let to_split = !relative_named && !external && (has_split || has_vertical || was_split);
    // SAFETY: `fconfig` is this frame's own, and `keys` the caller's keyset.
    let parsed = unsafe {
        parse_win_config(
            Some(w),
            keys,
            WinCfg::new(&raw mut fconfig),
            !was_split || to_split,
            report,
        )
    };
    if !parsed {
        return ().reported(error);
    }
    // SAFETY: `w` is live, `fconfig` this frame's own, and `keys` the
    // caller's keyset.
    // SAFETY: `fconfig` is this frame's own, live for the whole call.
    let fc = unsafe { WinCfg::new(&raw mut fconfig) };
    let applied = if to_split {
        apply_split(w, keys, fc)
    } else {
        apply_float(w, keys, fc)
    }?;
    // An autocommand that threw while the window was reconfigured does not
    // undo it: the rest of the config is applied, and the exception is the
    // answer.
    let raised = match applied {
        Applied::Refused => return Ok(()),
        Applied::Raised(raised) => Some(raised),
        Applied::Done => None,
    };

    if fconfig.style == kWinStyleMinimal && old_style != fconfig.style {
        win_set_minimal_style(w);
        // SAFETY: `w` is live.
        unsafe { didset_window_options(w, true) };
        changed_window_setting(w);
    }
    if fconfig._cmdline_offset < INT_MAX {
        cmdline_win.set(Some(w.id()));
    } else if cmdline_win.get() == Some(w.id()) && fconfig._cmdline_offset == INT_MAX {
        cmdline_win.set(None);
    }
    raised.map_or(Ok(()), Err)
}
