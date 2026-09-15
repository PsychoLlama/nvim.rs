//! `nvim_open_win()`: creating a window from a config.
//!
//! The entry point for both kinds of window the config describes: a float,
//! which is created directly, and a split, which goes through `win_split_dir`
//! and `win_split_flags` to turn the `split`/`vertical` keys into the
//! `WSP_*` flags `win_split_ins` takes.  `win_can_move_tp` and
//! `win_find_altwin` are the checks a window has to pass before it can be
//! moved to another tabpage.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::buffer::BufRef;
use crate::guard::Suppress;
use crate::types::CmdIdx;
use crate::winfloat::WIN_CONFIG_INIT;
use crate::winlayer::Buf;
use crate::winlayer::{TabPage, Win, WinId};
use core::ffi::c_int;

/// A window being created, and the state the `'_cleanup` label shared with
/// the body above it.
///
/// Every window here is named twice over -- as a [`Win`] and as a [`WinId`]
/// -- because the `WinNew` autocommand and the `:enter` below can close the
/// window that was just made, and every question asked after them is about
/// exactly that.
struct Opening {
    /// The buffer the new window is to show, and a reference that says
    /// whether it is still there once autocommands have run.
    buffer: Buf,
    bufref: Option<BufRef>,
    /// The config the window is made from, whose `noautocmd`, `style` and
    /// `_cmdline_offset` are read after it has been handed over.
    config: WinConfig,
    noautocmd: bool,
    style: WinStyle,
    cmdline_offset: c_int,
    /// Whether the config asks for a split rather than a float.
    is_split: bool,
    /// The window to split, and the tab page the new window is on -- which
    /// is re-read after anything that can move or close it.
    parent: Option<Win>,
    tabpage: Option<TabPage>,
}

impl Opening {
    /// Make the window and settle it: its buffer, its tab page, and the
    /// options a `style` asks for.
    ///
    /// Answers the new window's handle, or 0 when there is none to answer
    /// for. An `Err` is the exception to report; `Ok` may still leave one in
    /// `raised`, because an autocommand that threw does not un-create the
    /// window it threw in.
    fn run(
        &mut self,
        enter: bool,
        keys: CfgKeys,
        raised: &mut Option<Error>,
    ) -> Result<WindowHandle, Error> {
        if !self.resolve_parent(keys)? {
            return Ok(0);
        }
        let Some(window) = self.make(keys, raised)? else {
            let why = Error::exception(c"Failed to create window");
            return Err(raised.take().unwrap_or(why));
        };
        // The new window's identity, taken now: every `win_find_tabpage`
        // below is asking about *this* window rather than whichever one is
        // current by then.
        let window_id = window.id();
        if self.cmdline_offset < INT_MAX {
            cmdline_win.set(Some(window_id));
        }
        self.bufref = Some(BufRef::of(self.buffer));
        if !self.noautocmd {
            self.announce(window, window_id);
        }
        if let (Some(at), true) = (self.tabpage, enter) {
            goto_tabpage_win(at, window);
            self.tabpage = win_find_tabpage(window_id);
        }
        // A buffer the window would not take is reported, but does not stop
        // the window being answered.
        if let Err(why) = self.show_buffer(window, window_id) {
            *raised = Some(why);
        }
        if self.tabpage.is_none() {
            // Whatever was raised is moot: the window the caller would have
            // been handed is gone.
            *raised = None;
            return Err(Error::exception(c"Window was closed immediately"));
        }
        if self.style == kWinStyleMinimal {
            win_set_minimal_style(window);
            didset_window_options(window, true);
            changed_window_setting(window);
        }
        Ok(window.handle)
    }

    /// Resolve the `win` key into the window to split or position against.
    ///
    /// `false` says a handle resolved to no window at all, which upstream
    /// answers by making none and reporting nothing.
    fn resolve_parent(&mut self, keys: CfgKeys) -> Result<bool, Error> {
        if keys.win.unwrap_or(0) <= 0 {
            return Ok(true);
        }
        let Some(found) = find_window_by_handle(self.config.window)? else {
            return Ok(false);
        };
        self.parent = Some(found);
        if self.is_split && found.w_floating {
            return Err(Error::exception(c"Cannot split a floating window"));
        }
        self.tabpage = win_find_tabpage(found.id());
        Ok(true)
    }

    /// The window itself: a split of its parent, or a float.
    fn make(&mut self, keys: CfgKeys, raised: &mut Option<Error>) -> Result<Option<Win>, Error> {
        if !self.is_split {
            // SAFETY: `curwin` is live for the editor's whole run, and so is
            // the buffer it shows.
            let locked = unsafe { (*Win::current().w_buffer).b_locked_split } != 0;
            if locked {
                let why = c"E1159: Cannot open a float when closing the buffer";
                return Err(Error::exception(why));
            }
            return win_new_float(None, false, self.config.clone());
        }

        let target = self.parent.unwrap_or_else(Win::current);
        check_split_disallowed_err(target)?;
        // `vertical` without `split` picks the side from 'splitright' and
        // 'splitbelow'.
        if let Some(vertical) = keys.vertical
            && keys.split.is_none()
        {
            self.config.split = if vertical {
                if p_spr.get() != 0 {
                    kWinSplitRight
                } else {
                    kWinSplitLeft
                }
            } else if p_sb.get() != 0 {
                kWinSplitBelow
            } else {
                kWinSplitAbove
            };
        }
        let flags =
            win_split_flags(self.config.split, self.parent.is_none()) | WSP_NOENTER.cast_signed();
        let vertical = flags & WSP_VERT.cast_signed() != 0;
        let size = if vertical {
            self.config.width
        } else {
            self.config.height
        };
        let mut tstate = TryState::default();
        // SAFETY: `tstate` is this frame's own, live until `try_leave`.
        unsafe { try_enter(&raw mut tstate) };
        let made = self.split_ins(size, flags);
        // SAFETY: `tstate` is what the `try_enter` above filled in.
        *raised = unsafe { try_leave(&raw mut tstate) }.err();

        if let Some(mut new) = made {
            new.w_config = self.config.clone();
            let (width, height) = (new.w_width, new.w_height);
            if size > 0 {
                if vertical && width != size {
                    win_setwidth_win(size, new);
                } else if !vertical && height != size {
                    win_setheight_win(size, new);
                }
            }
        }
        Ok(made)
    }

    /// [`win_split_ins`] inside the parent window, when that is not the
    /// current one.
    fn split_ins(&self, size: c_int, flags: c_int) -> Option<Win> {
        let Some(parent) = self.parent.filter(|p| !p.is_current()) else {
            return split_ins(size, flags);
        };
        let mut switchwin = SwitchWin::default();
        let tabpage = self.tabpage.expect("a live window is on a tab page");
        // SAFETY: `switchwin` is this frame's own, and `parent`/`tabpage`
        // are the live window and tab page to split in.
        let result = unsafe { switch_win(&raw mut switchwin, parent, Some(tabpage), true) };
        debug_assert!(result.is_ok(), "the window was switched to");
        let made = split_ins(size, flags);
        // SAFETY: the matching restore of the switch above.
        unsafe { restore_win(&raw mut switchwin, true) };
        made
    }

    /// Fire `WinNew` in the new window, and re-read its tab page if the
    /// autocommands moved it.
    fn announce(&mut self, window: Win, window_id: WinId) {
        let mut switchwin = SwitchWin::default();
        // SAFETY: `switchwin` is this frame's own, and `window`/`tabpage`
        // name the window just made and the tab page it is on.
        let result = unsafe { switch_win_noblock(&raw mut switchwin, window, self.tabpage, true) };
        debug_assert!(result.is_ok(), "the window was switched to");
        // SAFETY: an autocommand with neither a file name nor a pattern.
        let switched = unsafe {
            apply_autocmds(
                AutoEvent::WinNew,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false,
                Buf::current_or_none(),
            )
        };
        if switched {
            self.tabpage = win_find_tabpage(window_id);
        }
        // SAFETY: the matching restore of the switch above.
        unsafe { restore_win_noblock(&raw mut switchwin, true) };
    }

    /// Put the buffer in the new window, if the window is not already
    /// showing it and both are still there.
    fn show_buffer(&mut self, window: Win, window_id: WinId) -> Result<(), Error> {
        // The window is read only once its tab page still holds it, which is
        // what says the autocommands above did not close it.
        let held = self.tabpage.is_some() && self.bufref.as_ref().is_some_and(|b| b.valid());
        if !held || Some(self.buffer) == window.buffer_or_none() {
            return Ok(());
        }
        let quiet =
            (!window.is_current() && !self.noautocmd).then(Suppress::win_enter_leave_autocmds);
        let shown = win_set_buf(window, self.buffer);
        if !self.noautocmd {
            self.tabpage = win_find_tabpage(window_id);
        }
        drop(quiet);
        shown
    }
}

/// Create a window showing `buf` from the `config` dictionary: a float, a
/// split, or an external window.
///
/// # Safety
/// `config` must be the caller's decoded keyset -- NUL-terminated strings and
/// arrays that name their own items.
pub unsafe fn nvim_open_win(
    buf: BufferHandle,
    enter: Boolean,
    config: *mut KeyDict_win_config,
) -> Result<WindowHandle, Error> {
    // SAFETY: `config` is the caller's keyset, live for the whole call.
    let keys = unsafe { CfgKeys::new(config) };
    // The lookup answers a live buffer or a null.
    let Some(buffer) = find_buffer_by_handle(buf)? else {
        return Ok(0 as WindowHandle);
    };
    if cmdwin_type.get() != 0 && enter || cmdwin_buf.get() == Some(buffer.id()) {
        return Err(Error::exception(e_cmdwin));
    }
    let mut fconfig = WIN_CONFIG_INIT;
    // SAFETY: `fconfig` is this frame's own, live for the whole call.
    let cfg = unsafe { WinCfg::new(&raw mut fconfig) };
    if matches!(parse_win_config(None, keys, cfg, false)?, Parsed::Refused) {
        // The config named a window that is not there. Upstream makes no
        // window for it and reports nothing.
        return Ok(0 as WindowHandle);
    }

    debug_assert!(Win::current_or_none().is_some(), "curwin != NULL");
    let mut opening = Opening {
        buffer,
        bufref: None,
        noautocmd: fconfig.noautocmd,
        style: fconfig.style,
        cmdline_offset: fconfig._cmdline_offset,
        is_split: keys.split.is_some() || keys.vertical.is_some(),
        parent: (keys.win.unwrap_or(0) == 0).then(Win::current),
        tabpage: Some(TabPage::current()),
        config: fconfig,
    };
    if opening.noautocmd {
        block_autocmds();
    }
    // An autocommand that threw while the window was being made does not
    // un-make it: the window is still answered, and the exception with it.
    let mut raised = None;
    let made = opening.run(enter, keys, &mut raised);
    if opening.noautocmd {
        unblock_autocmds();
    }
    match (made, raised) {
        (Err(why), _) | (Ok(_), Some(why)) => Err(why),
        (Ok(handle), None) => Ok(handle),
    }
}

/// `win_split_ins` as this file calls it: a new window of `size`, with no
/// window or frame to place it against.
fn split_ins(size: ::core::ffi::c_int, flags: ::core::ffi::c_int) -> Option<Win> {
    win_split_ins(size, flags, None, 0 as ::core::ffi::c_int, None)
}

/// Which side of its neighbour `win` was split off, for the `split` key.
///
/// A window with no frame, or one whose frame is the tab page's `topframe`,
/// answers the default rather than naming a side.
pub(crate) fn win_split_dir(win: Win) -> WinSplit {
    let Some(frame) = win.frame_or_none() else {
        return kWinSplitLeft;
    };
    let Some(parent) = frame.parent() else {
        return kWinSplitLeft;
    };
    // A window in a column was split off the one below it when it still has a
    // sibling ahead of it, and off the one above it otherwise; in a row the
    // same test reads left/right.
    let column = parent.fr_layout as ::core::ffi::c_int == FR_COL;
    match (frame.next().is_none(), column) {
        (false, true) => kWinSplitAbove,
        (true, true) => kWinSplitBelow,
        (false, false) => kWinSplitLeft,
        (true, false) => kWinSplitRight,
    }
}

pub(crate) fn win_split_flags(split: WinSplit, toplevel: bool) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if split as ::core::ffi::c_uint == kWinSplitAbove as ::core::ffi::c_int as ::core::ffi::c_uint
        || split as ::core::ffi::c_uint
            == kWinSplitBelow as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        flags |= WSP_HOR as ::core::ffi::c_int;
    } else {
        flags |= WSP_VERT as ::core::ffi::c_int;
    }
    if split as ::core::ffi::c_uint == kWinSplitAbove as ::core::ffi::c_int as ::core::ffi::c_uint
        || split as ::core::ffi::c_uint
            == kWinSplitLeft as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        flags |= if toplevel as ::core::ffi::c_int != 0 {
            WSP_TOP as ::core::ffi::c_int
        } else {
            WSP_ABOVE as ::core::ffi::c_int
        };
    } else {
        flags |= if toplevel as ::core::ffi::c_int != 0 {
            WSP_BOT as ::core::ffi::c_int
        } else {
            WSP_BELOW as ::core::ffi::c_int
        };
    }
    flags
}

/// Whether `window` may be moved to tab page `tabpage`, answering why not.
pub(crate) fn win_can_move_tp(window: Win, tabpage: TabPage) -> Result<(), Error> {
    let w = window;
    let other_tab = if tabpage == TabPage::current() {
        ::core::ptr::null_mut::<Tabpage>()
    } else {
        tabpage.raw()
    };
    // SAFETY: the caller's window and tab page.
    if unsafe { one_window(w, TabPage::from_raw(other_tab)) } {
        return Err(Error::exception(c"Cannot move last non-floating window"));
    }
    if win_locked(w) != 0 {
        return Err(Error::exception(
            c"Cannot move window to another tabpage whilst in use",
        ));
    }
    window_layout_locked_err(CmdIdx::SIZE)?;
    if textlock.get() != 0 || expr_map_locked() {
        return Err(Error::exception(e_textlock));
    }
    if is_aucmd_win(window) {
        return Err(Error::exception(
            c"Cannot move autocmd window to another tabpage",
        ));
    }
    if cmdwin_win.get() == Some(window.id()) || cmdwin_old_curwin.get() == Some(window.id()) {
        return Err(Error::exception(e_cmdwin));
    }
    Ok(())
}

/// The window that takes `win`'s place in tab page `tabpage` once it leaves: its
/// neighbour in the layout, or the tab page's own choice for a float.
pub(crate) fn win_find_altwin(win: Win, tabpage: TabPage) -> Option<Win> {
    let w = win;
    let at = (tabpage != TabPage::current()).then_some(tabpage);
    if win.w_floating {
        // `at` names the tab page to look in.
        win_float_find_altwin(win, at)
    } else {
        find_altwin(w, at).map(|alt| alt.win)
    }
}
