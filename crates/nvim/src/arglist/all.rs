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
use crate::memory::xstrdup;
use crate::types::CMD_drop;
use crate::window::{WSP_BELOW, WSP_ROOM};

/// What the two passes of `:all` share. A stack local of [`do_arg_all`],
/// never handed to C, so the passes take it by reference and its fields
/// need no unsafe to reach.
struct ArgAllState {
    /// The argument list being laid out; held by a reference of its own.
    alist: *mut alist_T,
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
    new_curwin: *mut win_T,
    new_curtab: *mut tabpage_T,
}

// ---------------------------------------------------------------------------
// Pass one: close the windows that hold no argument.

/// Where the window walk starts, and where it restarts when an autocommand
/// invalidated it: floating windows are walked first, backwards, then the
/// ordinary ones.
fn first_window_to_walk() -> *mut win_T {
    // SAFETY: firstwin and lastwin are always valid.
    unsafe {
        if (*lastwin.get()).w_floating {
            lastwin.get()
        } else {
            firstwin.get()
        }
    }
}

/// The window after `wp` in that walk, or null at its end.
///
/// # Safety
///
/// `wp` must be a valid window.
unsafe fn next_window_to_walk(wp: *mut win_T) -> *mut win_T {
    // SAFETY: caller contract; the window list is well formed.
    unsafe {
        if (*wp).w_floating {
            if (*(*wp).w_prev).w_floating {
                (*wp).w_prev
            } else {
                firstwin.get()
            }
        } else if (*wp).w_next.is_null() || (*(*wp).w_next).w_floating {
            ptr::null_mut()
        } else {
            (*wp).w_next
        }
    }
}

/// Which argument the buffer in `wp` is, or `opened_len` when it is none of
/// them (and the window is therefore a candidate for closing). On the way it
/// records how good a candidate `wp` is for becoming the new current window,
/// and adopts the argument list into it.
///
/// # Safety
///
/// `aall` must be the live state and `wp` a valid window holding `buf`.
unsafe fn arg_index_for_window(
    aall: &mut ArgAllState,
    wp: *mut win_T,
    buf: *mut buf_T,
    old_curwin: *mut win_T,
    old_curtab: *mut tabpage_T,
) -> c_int {
    // SAFETY: caller contract; the window, its buffer and the argument list
    // are all valid here.
    let unwanted = unsafe {
        (*buf).b_ffname.is_null()
            || !aall.keep_tabs
                && ((*buf).b_nwindows > 1
                    || (*wp).w_width != Columns.get()
                    || (*wp).w_floating && !is_aucmd_win(wp))
    };
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
        let holds_arg = unsafe {
            (*entry).ae_fnum == (*buf).handle || same_file(alist_name(entry), (*buf).b_ffname)
        };
        if !holds_arg {
            i += 1;
            continue;
        }
        // A window in the current tab page beats one elsewhere, and the
        // current window beats another in the same tab page.
        let mut weight = 1;
        if old_curtab == curtab.get() {
            weight += 1;
            if old_curwin == wp {
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
                if !aall.new_curwin.is_null() {
                    // SAFETY: as above.
                    unsafe { (*aall.new_curwin).w_arg_idx = aall.opened_len };
                }
                aall.new_curwin = wp;
                aall.new_curtab = curtab.get();
            }
        } else if aall.keep_tabs {
            i = aall.opened_len;
        }
        // SAFETY: `wp` holds a reference of its own to whatever list it has,
        // so dropping it here is balanced by the one taken for `aall.alist`.
        unsafe {
            if (*wp).w_alist != aall.alist {
                // Use the current argument list for every window holding a
                // file from it.
                alist_unlink((*wp).w_alist);
                (*wp).w_alist = aall.alist;
                (*aall.alist).al_refcount += 1;
            }
        }
        return i;
    }
    i
}

/// Close `wp`, whose buffer is not in the argument list — unless it is the
/// last window, which is re-used for the first argument instead. Answers the
/// window the walk continues from, which is the top again when an
/// autocommand invalidated it.
///
/// # Safety
///
/// `aall` must be the live state, `wp` a valid window holding `buf`, and
/// `wpnext` the window the walk would continue to.
unsafe fn close_unused_window(
    aall: &mut ArgAllState,
    wp: *mut win_T,
    buf: *mut buf_T,
    wpnext: *mut win_T,
) -> *mut win_T {
    // SAFETY: caller contract; `buf` is the window's own buffer.
    let (hide, changed, nwindows) =
        unsafe { (buf_hide(buf), bufIsChanged(buf), (*buf).b_nwindows) };
    if !(hide || aall.forceit || nwindows > 1 || !changed) {
        return wpnext;
    }
    if !hide && nwindows <= 1 && changed {
        // The buffer was changed and we would like to hide it, so try
        // autowriting.
        // SAFETY: `wp` and `buf` are live on entry; both are re-validated
        // afterwards, since `autowrite` runs autocommands.
        let survived = unsafe {
            let mut bufref = bufref_T::default();
            set_bufref(&raw mut bufref, buf);
            autowrite(buf, false);
            win_valid(wp) && bufref_valid(&raw mut bufref)
        };
        if !survived {
            // Autocommands removed the window; start all over.
            return first_window_to_walk();
        }
    }
    // Don't close the last window.
    if firstwin.get() == lastwin.get() {
        // SAFETY: there is always a first tab page.
        let only_tab = unsafe { (*first_tabpage.get()).tp_next.is_null() };
        if only_tab || aall.had_tab == 0 {
            aall.use_firstwin = true;
            return wpnext;
        }
    }
    // SAFETY: `wp` is live, and `wpnext` is re-validated because closing a
    // window runs autocommands. Whether the buffer goes with the window is
    // asked again here rather than reused from above: a successful
    // `autowrite` leaves it unchanged, and then it is the close's to free.
    unsafe {
        win_close(wp, !buf_hide(buf) && !bufIsChanged(buf), false);
        if win_valid(wpnext) {
            return wpnext;
        }
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
    old_curwin: *mut win_T,
    old_curtab: *mut tabpage_T,
) {
    let mut wp = first_window_to_walk();
    while !wp.is_null() {
        // SAFETY: caller contract; `wp` is live on entry, and both callees
        // answer a window that has been re-validated.
        let (buf, wpnext, i) = unsafe {
            let wpnext = next_window_to_walk(wp);
            let buf = (*wp).w_buffer;
            let i = arg_index_for_window(aall, wp, buf, old_curwin, old_curtab);
            (*wp).w_arg_idx = i;
            (buf, wpnext, i)
        };
        wp = if i == aall.opened_len && !aall.keep_tabs {
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
    let old_curwin = curwin.get();
    let old_curtab = curtab.get();
    if aall.had_tab > 0 {
        // SAFETY: there is always a first tab page.
        unsafe { goto_tabpage_tp(first_tabpage.get(), true, true) };
    }
    // Moving tab pages around in an autocommand may cause an endless loop.
    tabpage_move_disallowed.set(tabpage_move_disallowed.get() + 1);
    loop {
        // SAFETY: caller contract; curtab is valid, and `tpnext` is
        // re-validated below because closing windows runs autocommands.
        let tpnext = unsafe {
            let tpnext = (*curtab.get()).tp_next;
            close_unused_windows_in_tab(aall, old_curwin, old_curtab);
            tpnext
        };
        // Without the ":tab" modifier only do the current tab page.
        if aall.had_tab == 0 || tpnext.is_null() {
            break;
        }
        // SAFETY: a tab page that is gone falls back to the first one.
        unsafe {
            let tpnext = if valid_tabpage(tpnext) {
                tpnext
            } else {
                first_tabpage.get()
            };
            goto_tabpage_tp(tpnext, true, true);
        }
    }
    tabpage_move_disallowed.set(tabpage_move_disallowed.get() - 1);
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
    if unsafe { (*curwin.get()).w_arg_idx } == i {
        return false;
    }
    let mut wp = firstwin.get();
    while !wp.is_null() {
        // SAFETY: the window list of the current tab page is well formed.
        unsafe {
            if (*wp).w_arg_idx != i {
                wp = (*wp).w_next;
                continue;
            }
        }
        if aall.keep_tabs {
            aall.new_curwin = wp;
            aall.new_curtab = curtab.get();
            return false;
        }
        // SAFETY: `wp` is live and every window sits in a frame.
        let moved = unsafe {
            (*wp).w_floating || (*(*wp).w_frame).fr_parent == (*(*curwin.get()).w_frame).fr_parent
        };
        if !moved {
            crate::semsg!("E249: Window layout changed unexpectedly");
            return true;
        }
        // SAFETY: as above; a floating window is left where it is.
        unsafe {
            if !(*wp).w_floating {
                win_move_after(wp, curwin.get());
            }
        }
        return false;
    }
    false
}

/// Split a window — or re-use the first one — and edit argument `i` in it.
/// Answers `FAIL` when the split failed, after which nothing more is opened.
///
/// # Safety
///
/// `aall` must be the live state and `i` an index into its argument list.
unsafe fn open_window_for_arg(
    aall: &mut ArgAllState,
    i: c_int,
    count: c_int,
    tab_drop_empty_window: bool,
) -> c_int {
    // Trigger the events for a tab drop.
    let tab_drop_last = tab_drop_empty_window && i == count - 1;
    if tab_drop_last {
        autocmd_no_enter.set(autocmd_no_enter.get() - 1);
    }
    if aall.use_firstwin {
        // The first window: run the autocommands for leaving its buffer.
        autocmd_no_leave.set(autocmd_no_leave.get() - 1);
    } else {
        // Split the current window, taking space from all of them.
        let p_ea_save = p_ea.get() != 0;
        p_ea.set(c_int::from(true));
        let split_ret = win_split(0, WSP_ROOM as c_int | WSP_BELOW as c_int);
        p_ea.set(c_int::from(p_ea_save));
        if split_ret == FAIL {
            return FAIL;
        }
    }
    // SAFETY: curwin is the window just split (or the first one), and the
    // argument name outlives `do_ecmd`'s use of it.
    unsafe {
        (*curwin.get()).w_arg_idx = i;
    }
    if i == 0 {
        aall.new_curwin = curwin.get();
        aall.new_curtab = curtab.get();
    }
    // SAFETY: as above; `i` is an entry of the locked argument list.
    unsafe {
        let buf = (*curwin.get()).w_buffer;
        let flags = flag_if(buf_hide(buf) || bufIsChanged(buf), ECMD_HIDE) | ECMD_OLDBUF as c_int;
        do_ecmd(
            0,
            alist_name(alist_arg(aall.alist, i)),
            ptr::null_mut(),
            ptr::null_mut(),
            ECMD_ONE as linenr_T,
            flags,
            curwin.get(),
        );
    }
    if tab_drop_last {
        autocmd_no_enter.set(autocmd_no_enter.get() + 1);
    }
    if aall.use_firstwin {
        autocmd_no_leave.set(autocmd_no_leave.get() + 1);
    }
    aall.use_firstwin = false;
    OK
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
            && buf_is_empty(curbuf.get())
            && (*curbuf.get()).b_nwindows == 1
            && (*curbuf.get()).b_ffname.is_null()
            && (*curbuf.get()).b_changed == 0
    };
    if tab_drop_empty_window {
        aall.use_firstwin = true;
    }
    let mut split_ret = OK;
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
        } else if split_ret == OK {
            // SAFETY: caller contract.
            split_ret = unsafe { open_window_for_arg(aall, i, count, tab_drop_empty_window) };
            split_ret == FAIL
        } else {
            false
        };
        if !split_failed {
            os_breakcheck();
            // With ":tab", open a new tab page for each new window.
            let room = tabpage_index(ptr::null_mut()) as OptInt <= p_tpm.get();
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
    let prev_arglist_locked = ARGLIST_LOCKED.get();
    debug_assert!(!firstwin.get().is_null(), "firstwin != NULL");
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
    let mut aall = unsafe {
        setpcmark();
        let alist = win_alist(curwin.get());
        (*alist).al_refcount += 1;
        ArgAllState {
            alist,
            had_tab: cmdmod.with(|m| m.cmod_tab),
            keep_tabs,
            forceit,
            use_firstwin: false,
            opened: xcalloc(argcount() as size_t, 1 as size_t) as *mut uint8_t,
            opened_len: argcount(),
            new_curwin: ptr::null_mut(),
            new_curtab: ptr::null_mut(),
        }
    };
    ARGLIST_LOCKED.set(true);
    let new_lu_tp = curtab.get();
    // Stop Visual mode: the cursor and "VIsual" may well be invalid after
    // switching to another buffer.
    // SAFETY: the state is this frame's and the list is locked.
    unsafe {
        reset_VIsual_and_resel();
        arg_all_close_unused_windows(&mut aall);
    }
    // ARGCOUNT may have changed while doing that, because of autocommands,
    // so the count is against the recorded length.
    let count = if count > aall.opened_len || count <= 0 {
        aall.opened_len
    } else {
        count
    };
    // Don't run the Win/Buf Enter/Leave autocommands here.
    autocmd_no_enter.set(autocmd_no_enter.get() + 1);
    autocmd_no_leave.set(autocmd_no_leave.get() + 1);
    let last_curwin = curwin.get();
    let last_curtab = curtab.get();
    // SAFETY: lastwin may be aucmd_win, which `lastwin_nofloating` skips.
    unsafe {
        win_enter(lastwin_nofloating(ptr::null_mut()), false);
        arg_all_open_windows(&mut aall, count);
        // Remove the "lock" on the argument list.
        alist_unlink(aall.alist);
    }
    ARGLIST_LOCKED.set(prev_arglist_locked);
    autocmd_no_enter.set(autocmd_no_enter.get() - 1);
    // SAFETY: every window and tab page recorded above is checked for still
    // being live before it is entered.
    unsafe {
        // Restore the last referenced tab page's current window.
        if last_curtab != aall.new_curtab {
            if valid_tabpage(last_curtab) {
                goto_tabpage_tp(last_curtab, true, true);
            }
            if win_valid(last_curwin) {
                win_enter(last_curwin, false);
            }
        }
        // Go to the window holding the first argument.
        if valid_tabpage(aall.new_curtab) {
            goto_tabpage_tp(aall.new_curtab, true, true);
        }
        // Set the last used tab page to where we started.
        if valid_tabpage(new_lu_tp) {
            lastused_tabpage.set(new_lu_tp);
        }
        if win_valid(aall.new_curwin) {
            win_enter(aall.new_curwin, false);
        }
    }
    autocmd_no_leave.set(autocmd_no_leave.get() - 1);
    // SAFETY: `opened` is this frame's allocation and nothing refers to it.
    unsafe { xfree(aall.opened as *mut c_void) };
}

/// `:all` and `:sall`, and `:tab drop file ...` once it has set the
/// argument list.
///
/// # Safety
///
/// `eap` must be a live command block.
pub unsafe fn ex_all(eap: *mut exarg_T) {
    // SAFETY: caller contract; `:all` takes an optional count as its range.
    let (count, forceit, drop) = unsafe {
        if (*eap).addr_count == 0 {
            (*eap).line2 = 9999 as linenr_T;
        }
        (
            (*eap).line2 as c_int,
            (*eap).forceit != 0,
            (*eap).cmdidx as c_int == CMD_drop as c_int,
        )
    };
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
        let name = unsafe { CStr::from_ptr(p).to_bytes() };
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
