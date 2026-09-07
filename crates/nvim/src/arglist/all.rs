//! `:all`, `:sall` and `:tab drop`: lay the argument list out over windows.
//!
//! Two passes, and the state they share is [`ArgAllState`]. The first
//! closes every window whose buffer is not an argument (and records, in
//! `opened`, how good a candidate each surviving window is for becoming the
//! new current one); the second opens a window for every argument that has
//! none. Autocommands run throughout — `autowrite`, `win_close`, `do_ecmd` —
//! so both passes re-check that the window they are walking still exists.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::buffer::BufRef;
use crate::ex_cmds::EcmdFlags;
use crate::ex_cmds::newlnum;
use crate::guard::{Allow, Lock, Suppress};
use crate::memory::xstrdup;
use crate::types::CmdIdx;
use crate::window::valid_tab;
use crate::window::valid_win;
use crate::winlayer::{TabId, WinId};

use crate::types::Failed;
use crate::window::{WSP_BELOW, WSP_ROOM, goto_tab};
use crate::winlayer::{TabPage, Win, first_tab, first_window, last_window, windows};

/// What the two passes of `:all` share. A stack local of [`do_arg_all`],
/// never handed to C, so the passes take it by reference and its fields
/// need no unsafe to reach.
struct ArgAllState {
    /// The argument list being laid out; held by a reference of its own.
    alist: *mut ArgList,
    /// `cmdmod.cmod_tab` as `:all` started: nonzero for `:tab all`.
    had_tab: c_int,
    /// `:tab drop`: keep the existing tab pages and windows.
    keep_tabs: bool,
    forceit: bool,
    /// The first window is free to be re-used for the next argument.
    use_firstwin: bool,
    /// Per argument, how good the best window showing it is: 0 not open,
    /// 1 open in another tab page, 2 open in this one, 3 open in the
    /// current window.
    opened: *mut uint8_t,
    /// Length of `opened`, i.e. `ARGCOUNT` when `:all` started.
    opened_len: c_int,
    /// The window and tab page `:all` will end in. Handles: they are picked
    /// during the walk and entered at the very end, with every window's
    /// autocommands in between.
    new_curwin: Option<WinId>,
    new_curtab: Option<TabId>,
}

// ---------------------------------------------------------------------------
// Pass one: close the windows that hold no argument.

/// Where the window walk starts, and where it restarts when an autocommand
/// invalidated it: floating windows are walked first, backwards, then the
/// ordinary ones.
fn first_window_to_walk() -> Option<Win> {
    let last = last_window().expect("the editor always has a window");
    match last.w_floating {
        true => Some(last),
        false => first_window(),
    }
}

/// The window after `window` in that walk, or null at its end.
///
/// # Safety
///
/// `window` must be a valid window.
unsafe fn next_window_to_walk(window: Win) -> Option<Win> {
    // SAFETY: the caller's promise -- a live `Window`.
    // SAFETY: caller contract; the window list is well formed.
    if window.w_floating {
        let prev = window.prev().expect("a float is never the first window");
        match prev.w_floating {
            true => Some(prev),
            false => first_window(),
        }
    } else {
        window.next().filter(|next| !next.w_floating)
    }
}

/// Which argument the buffer in `window` is, or `opened_len` when it is none of
/// them (and the window is therefore a candidate for closing). On the way it
/// records how good a candidate `window` is for becoming the new current window,
/// and adopts the argument list into it.
///
/// # Safety
///
/// `aall` must be the live state and `window` a valid window holding `buffer`.
unsafe fn arg_index_for_window(
    aall: &mut ArgAllState,
    mut window: Win,
    buffer: Buf,
    old_curwin: Win,
    old_curtab: TabPage,
) -> c_int {
    // SAFETY: the caller's promise -- a live `Buffer`.
    // SAFETY: the caller's promise -- a live `Window`.
    // SAFETY: caller contract; the window, its buffer and the argument list
    // are all valid here.
    // SAFETY: `window` is the window being considered, live for this walk.
    // Reading it here rather than inside the test below costs nothing: the
    // call has no side effect and the chain has no bounds check in it.
    let aucmd_win = is_aucmd_win(window);
    let unwanted = buffer.b_ffname.is_null()
        || !aall.keep_tabs
            && (buffer.b_nwindows > 1
                || window.w_width != Columns.get()
                || window.w_floating && !aucmd_win);
    if unwanted {
        return aall.opened_len;
    }
    let mut i = 0;
    while i < aall.opened_len {
        if i >= alist_count(aall.alist) {
            i += 1;
            continue;
        }
        let entry = alist_arg(aall.alist, i);
        // SAFETY: `i` is an entry of the list, which is locked and so stays
        // put; the buffer is valid.
        // SAFETY: `entry` is the `i`th of a list that holds more than `i`,
        // and both file names are NUL-terminated.
        let holds_arg = unsafe { (*entry).ae_fnum } == buffer.handle
            || unsafe { same_file(alist_name(entry), buffer.b_ffname) };
        if !holds_arg {
            i += 1;
            continue;
        }
        // A window in the current tab page beats one elsewhere, and the
        // current window beats another in the same tab page.
        let mut weight = 1;
        if old_curtab == TabPage::current() {
            weight += 1;
            if old_curwin == window {
                weight += 1;
            }
        }
        // SAFETY: `opened` is `opened_len` bytes long and `i` is below it;
        // `new_curwin`, when set, is a window that is still live.
        let best = unsafe { *aall.opened.offset(i as isize) } as c_int;
        if weight > best {
            // SAFETY: as above.
            unsafe { *aall.opened.offset(i as isize) = weight as uint8_t };
            if i == 0 {
                if let Some(mut prev) = aall.new_curwin.and_then(WinId::get) {
                    prev.w_arg_idx = aall.opened_len;
                }
                aall.new_curwin = Some(window.id());
                aall.new_curtab = Some(TabPage::current().id());
            }
        } else if aall.keep_tabs {
            i = aall.opened_len;
        }
        // SAFETY: `window` holds a reference of its own to whatever list it has,
        // so dropping it here is balanced by the one taken for `aall.alist`.
        if window.w_alist != aall.alist {
            // Use the current argument list for every window holding a
            // file from it.
            unsafe { alist_unlink(window.w_alist) };
            window.w_alist = aall.alist;
            unsafe { (*aall.alist).al_refcount.retain() };
        }
        return i;
    }
    i
}

/// Close `window`, whose buffer is not in the argument list — unless it is the
/// last window, which is re-used for the first argument instead. Answers the
/// window the walk continues from, which is the top again when an
/// autocommand invalidated it.
///
/// # Safety
///
/// `aall` must be the live state, `window` a valid window holding `buffer`, and
/// `wpnext` the window the walk would continue to, if there is one.
unsafe fn close_unused_window(
    aall: &mut ArgAllState,
    window: Win,
    buffer: Buf,
    wpnext: Option<Win>,
) -> Option<Win> {
    // SAFETY: `buffer` is the window's own buffer.
    let hide = unsafe { buf_hide(buffer) };
    // SAFETY: as above.
    let changed = buf_is_changed(buffer);
    let nwindows = buffer.b_nwindows;
    if !(hide || aall.forceit || nwindows > 1 || !changed) {
        return wpnext;
    }
    if !hide && nwindows <= 1 && changed {
        // The buffer was changed and we would like to hide it, so try
        // autowriting.
        // SAFETY: `window` and `buffer` are live on entry; both are re-validated
        // afterwards, since `autowrite` runs autocommands.
        // `buffer` is live until the autowrite -- which is exactly what the
        // re-check afterwards is for.
        let bufref = BufRef::of(buffer);
        // SAFETY: as above; this may fire autocommands.
        let _ = unsafe { autowrite(buffer, false) };
        // `win_valid` and `BufRef::valid` are the questions to ask after one.
        let survived = win_valid(window.id()) && bufref.valid();
        if !survived {
            // Autocommands removed the window; start all over.
            return first_window_to_walk();
        }
    }
    // Don't close the last window.
    if firstwin.get() == lastwin.get() {
        let only_tab = first_tab().is_none_or(|tp| tp.next().is_none());
        if only_tab || aall.had_tab == 0 {
            aall.use_firstwin = true;
            return wpnext;
        }
    }
    // SAFETY: `window` is live, and `wpnext` is re-validated because closing a
    // window runs autocommands. Whether the buffer goes with the window is
    // asked again here rather than reused from above: a successful
    // `autowrite` leaves it unchanged, and then it is the close's to free.
    // SAFETY: `buffer` is `window`'s buffer; a hidden or changed one is kept.
    // SAFETY: `buffer` is `window`'s buffer; a hidden or changed one is kept.
    let free_buf = unsafe { !buf_hide(buffer) } && !buf_is_changed(buffer);
    let wpnext_id = wpnext.map(Win::id);
    // SAFETY: `window` is a live window, and not the last one (checked above).
    unsafe { win_close(window, free_buf, false) };
    if let Some(next) = wpnext_id.and_then(valid_win) {
        return Some(next);
    }
    // Autocommands removed the next window; start all over.
    first_window_to_walk()
}

/// Close every window of the current tab page whose buffer is not in the
/// argument list, recording in `w_arg_idx` which argument each surviving
/// window holds.
///
/// # Safety
///
/// `aall` must be the live state.
unsafe fn close_unused_windows_in_tab(
    aall: &mut ArgAllState,
    old_curwin: Win,
    old_curtab: TabPage,
) {
    let mut next = first_window_to_walk();
    while let Some(mut wp) = next {
        // SAFETY: caller contract; `wp` is live on entry, and both callees
        // answer a window that has been re-validated.
        // `wp` is a live window for as long as this step runs, and
        // `arg_index_for_window` may not close it.
        let wpnext = unsafe { next_window_to_walk(wp) };
        let buf = wp.buffer();
        let i = unsafe { arg_index_for_window(aall, wp, buf, old_curwin, old_curtab) };
        wp.w_arg_idx = i;
        next = if i == aall.opened_len && !aall.keep_tabs {
            // SAFETY: as above.
            unsafe { close_unused_window(aall, wp, buf, wpnext) }
        } else {
            wpnext
        };
    }
}

/// Close all the windows holding files that are not in the argument list —
/// over every tab page when `:tab` was used.
///
/// # Safety
///
/// `aall` must be the live state.
unsafe fn arg_all_close_unused_windows(aall: &mut ArgAllState) {
    let old_curwin = Win::current();
    let old_curtab = TabPage::current();
    if aall.had_tab > 0 {
        goto_tab(
            first_tab().expect("there is always a first tab page"),
            true,
            true,
        );
    }
    // Moving tab pages around in an autocommand may cause an endless loop.
    let _no_move = Lock::tabpage_move();
    loop {
        // The identity, taken while the tab page is live: the closes below
        // can free it, and that is what the check after them asks about.
        let tpnext = TabPage::current().next().map(TabPage::id);
        // SAFETY: as above.
        unsafe { close_unused_windows_in_tab(aall, old_curwin, old_curtab) };
        // Without the ":tab" modifier only do the current tab page.
        let (false, Some(tpnext)) = (aall.had_tab == 0, tpnext) else {
            break;
        };
        // A tab page that is gone falls back to the first one.
        let tpnext = valid_tab(tpnext)
            .unwrap_or_else(|| first_tab().expect("there is always a first tab page"));
        goto_tab(tpnext, true, true);
    }
}

// ---------------------------------------------------------------------------
// Pass two: open a window for every argument that has none.

/// Move the window already showing argument `i` below the current one — or,
/// with `keep_tabs`, only remember it. Answers true when the layout changed
/// unexpectedly, which stops `:all` entirely (E249).
///
/// # Safety
///
/// `aall` must be the live state.
unsafe fn move_existing_window_for_arg(aall: &mut ArgAllState, i: c_int) -> bool {
    // SAFETY: curwin is valid.
    if Win::current().w_arg_idx == i {
        return false;
    }
    let Some(wp) = windows().find(|wp| wp.w_arg_idx == i) else {
        return false;
    };
    if aall.keep_tabs {
        aall.new_curwin = Some(wp.id());
        aall.new_curtab = Some(TabPage::current().id());
        return false;
    }
    // A window that changed parents is a layout change; a float has no parent
    // to change, and answers "not moved". Two identities compared, so neither
    // frame is read.
    let moved = wp.w_floating || wp.frame().fr_parent == Win::current().frame().fr_parent;
    if !moved {
        crate::semsg!("E249: Window layout changed unexpectedly");
        return true;
    }
    // A floating window is left where it is.
    if !wp.w_floating {
        // SAFETY: `wp` and `curwin` are live windows of the same tab page.
        unsafe { win_move_after(wp, Win::current()) };
    }
    false
}

/// Split a window — or re-use the first one — and edit argument `i` in it.
/// Answers `Err` when the split failed, after which nothing more is opened.
///
/// # Safety
///
/// `aall` must be the live state and `i` an index into its argument list.
unsafe fn open_window_for_arg(
    aall: &mut ArgAllState,
    i: c_int,
    count: c_int,
    tab_drop_empty_window: bool,
) -> Result<(), Failed> {
    // Trigger the events for a tab drop.
    //
    // Both lifts are guards: the `Err` return below sits between the C's
    // decrement and its matching increment, so a failed split left the
    // enclosing `do_arg_all`'s suppression one short for the rest of the
    // session.
    let tab_drop_last = tab_drop_empty_window && i == count - 1;
    let _enter = tab_drop_last.then(Allow::win_enter_autocmds);
    let _leave = if aall.use_firstwin {
        // The first window: run the autocommands for leaving its buffer.
        Some(Allow::win_leave_autocmds())
    } else {
        None
    };
    if !aall.use_firstwin {
        // Split the current window, taking space from all of them.
        let p_ea_save = p_ea.get() != 0;
        p_ea.set(c_int::from(true));
        let split_ret = win_split(0, WSP_ROOM as c_int | WSP_BELOW as c_int);
        p_ea.set(c_int::from(p_ea_save));
        split_ret?;
    }
    // SAFETY: curwin is the window just split (or the first one), and the
    // argument name outlives `do_ecmd`'s use of it.
    Win::current().w_arg_idx = i;
    if i == 0 {
        aall.new_curwin = Some(Win::current().id());
        aall.new_curtab = Some(TabPage::current().id());
    }
    // SAFETY: as above; `i` is an entry of the locked argument list.
    let buf = Win::current().buffer();
    let hide = unsafe { buf_hide(buf) } || buf_is_changed(buf);
    let flags = EcmdFlags::HIDE.when(hide) | EcmdFlags::OLDBUF;
    let ffname = unsafe { alist_name(alist_arg(aall.alist, i)) };
    let sfname = ptr::null_mut();
    let eap2 = ptr::null_mut();
    let newlnum = newlnum::ONE as LineNr;
    let here = Some(Win::current().id());
    let _ = unsafe { do_ecmd(0, ffname, sfname, eap2, newlnum, flags, here) };
    aall.use_firstwin = false;
    Ok(())
}

/// Open up to `count` windows for the files in `aall.alist`.
///
/// # Safety
///
/// `aall` must be the live state.
unsafe fn arg_all_open_windows(aall: &mut ArgAllState, count: c_int) {
    // ":tab drop file" should re-use an empty window, so that "--remote-tab"
    // does not leave an empty tab page when it runs locally.
    // SAFETY: caller contract; curbuf is valid.
    let tab_drop_empty_window = unsafe {
        aall.keep_tabs
            && buf_is_empty(Buf::current())
            && Buf::current().b_nwindows == 1
            && Buf::current().b_ffname.is_null()
            && Buf::current().b_changed == 0
    };
    if tab_drop_empty_window {
        aall.use_firstwin = true;
    }
    let mut split_ret = Ok(());
    let mut i = 0;
    while i < count && !got_int.get() {
        // SAFETY: caller contract; `i` is below `count`, which is at most
        // `opened_len`, the length of `opened`.
        if aall.alist == global_arglist() && i == alist_count(global_arglist()) - 1 {
            arg_had_last.set(true);
        }
        // SAFETY: `opened` is `opened_len` bytes and `i` is below `count`,
        // which is at most that.
        let already_open = unsafe { *aall.opened.offset(i as isize) } as c_int > 0;
        let split_failed = if already_open {
            // SAFETY: caller contract.
            if unsafe { move_existing_window_for_arg(aall, i) } {
                // E249: stop after finishing this pass of the loop.
                i = count;
            }
            false
        } else if split_ret.is_ok() {
            // SAFETY: caller contract.
            split_ret = unsafe { open_window_for_arg(aall, i, count, tab_drop_empty_window) };
            split_ret.is_err()
        } else {
            false
        };
        if !split_failed {
            os_breakcheck();
            // With ":tab", open a new tab page for each new window.
            let room = tabpage_index(None) as OptInt <= p_tpm.get();
            if aall.had_tab > 0 && room {
                cmdmod.with_mut(|m| m.cmod_tab = 9999);
            }
        }
        i += 1;
    }
}

/// Open up to `count` windows, one per argument. `keep_tabs` is
/// `:tab drop`'s "leave the existing layout alone".
unsafe fn do_arg_all(count: c_int, forceit: bool, keep_tabs: bool) {
    debug_assert!(first_window().is_some(), "firstwin != NULL");
    if cmdwin_type.get() != 0 {
        crate::semsg!("E11: Invalid in command-line window; <CR> executes, CTRL-C quits");
        return;
    }
    if argcount() <= 0 {
        // Don't give an error message: it is not wanted when the ":all"
        // command sits in the vimrc.
        return;
    }
    // SAFETY: curwin always has an argument list, and the reference taken
    // here keeps it alive across every autocommand below.
    setpcmark();
    let alist = win_alist(Win::current());
    unsafe { (*alist).al_refcount.retain() };
    let opened = unsafe { xcalloc(argcount() as size_t, 1) }.cast::<uint8_t>();
    let mut aall = ArgAllState {
        alist,
        had_tab: cmdmod.with(|m| m.cmod_tab),
        keep_tabs,
        forceit,
        use_firstwin: false,
        opened,
        opened_len: argcount(),
        new_curwin: None,
        new_curtab: None,
    };
    let prev_arglist_locked = ARGLIST_LOCKED.get();
    ARGLIST_LOCKED.set(true);
    let new_lu_tp = TabPage::current().id();
    // Stop Visual mode: the cursor and "VIsual" may well be invalid after
    // switching to another buffer.
    // SAFETY: the state is this frame's and the list is locked.
    reset_visual_and_resel();
    unsafe { arg_all_close_unused_windows(&mut aall) };
    // ARGCOUNT may have changed while doing that, because of autocommands,
    // so the count is against the recorded length.
    let count = if count > aall.opened_len || count <= 0 {
        aall.opened_len
    } else {
        count
    };
    // Don't run the Win/Buf Enter/Leave autocommands here.
    let no_enter = Suppress::win_enter_autocmds();
    let no_leave = Suppress::win_leave_autocmds();
    let last_curwin = Win::current().id();
    let last_curtab = TabPage::current().id();
    // SAFETY: lastwin may be aucmd_win, which `lastwin_nofloating` skips.
    unsafe { win_enter(lastwin_nofloating(None), false) };
    unsafe { arg_all_open_windows(&mut aall, count) };
    // Remove the "lock" on the argument list.
    unsafe { alist_unlink(aall.alist) };
    ARGLIST_LOCKED.set(prev_arglist_locked);
    // The release order is load-bearing: the windows entered below fire
    // `WinEnter`/`BufEnter` but still no `WinLeave`/`BufLeave`.
    drop(no_enter);
    // SAFETY: every window and tab page recorded above is checked for still
    // being live before it is entered.
    // Restore the last referenced tab page's current window.
    if Some(last_curtab) != aall.new_curtab {
        if let Some(tp) = valid_tab(last_curtab) {
            unsafe { goto_tabpage_tp(tp, true, true) };
        }
        if let Some(win) = valid_win(last_curwin) {
            unsafe { win_enter(win, false) };
        }
    }
    // Go to the window holding the first argument.
    if let Some(tp) = aall.new_curtab.and_then(valid_tab) {
        unsafe { goto_tabpage_tp(tp, true, true) };
    }
    // Set the last used tab page to where we started.
    if valid_tabpage(new_lu_tp) {
        lastused_tabpage.set(Some(new_lu_tp));
    }
    if let Some(win) = aall.new_curwin.and_then(valid_win) {
        unsafe { win_enter(win, false) };
    }
    drop(no_leave);
    // SAFETY: `opened` is this frame's allocation and nothing refers to it.
    unsafe { xfree(aall.opened as *mut c_void) };
}

/// `:all` and `:sall`, and `:tab drop file ...` once it has set the
/// argument list.
///
/// # Safety
///
/// `args` must be a live command block.
pub unsafe fn ex_all(args: *mut ExArg) {
    // SAFETY: the caller's promise -- a live `ExArg`.
    let mut args = unsafe { Ea::new(args) };
    // `:all` takes an optional count as its range.
    if args.addr_count == 0 {
        args.line2 = 9999 as LineNr;
    }
    let count = args.line2 as c_int;
    let forceit = args.forceit != 0;
    let drop = args.cmdidx == CmdIdx::drop;
    // SAFETY: no reference into the command block is held across this.
    unsafe { do_arg_all(count, forceit, drop) };
}

/// Every argument, space separated, in one owned string. Spaces,
/// backslashes and backticks in a name are escaped with a backslash, since
/// the result goes back through argument splitting (`##` on a command line).
pub unsafe fn arg_all() -> *mut c_char {
    let mut out: Vec<u8> = Vec::new();
    for idx in 0..argcount() {
        let p = arg_name(idx);
        if p.is_null() {
            continue;
        }
        if !out.is_empty() {
            // Insert a space between names.
            out.push(b' ');
        }
        // SAFETY: every entry's name is NUL-terminated and outlives the copy.
        let name = unsafe { CStr::from_ptr(p) }.to_bytes();
        for &byte in name {
            if byte == b' ' || byte == b'\\' || byte == b'`' {
                out.push(b'\\');
            }
            out.push(byte);
        }
    }
    out.push(0);
    // SAFETY: `out` is NUL-terminated; the copy is the caller's to free.
    unsafe { xstrdup(out.as_ptr() as *const c_char) }
}
