//! `:argdo`, `:windo`, `:bufdo`, `:tabdo`, `:cdo`, `:ldo`, `:cfdo` and
//! `:lfdo`: run one ex command once per item of some list.
//!
//! The eight are one function with a dozen `eap->cmdidx ==` tests through
//! it. [`ListDo`] takes that decision once, at the top, so the walk below
//! reads as five cases rather than as a chain of command names; the cursor
//! it carries -- an argument index, a window, a tab page, a buffer, or a
//! quickfix index -- is whichever of `i`/`wp`/`tp`/`buf` that case uses.
//!
//! The whole point is that the command being run is arbitrary: it can wipe
//! the buffer, close the window, or leave the tab page the walk was standing
//! on. Every step therefore re-validates what it is about to touch and
//! stops rather than guess, which is why so much of [`listdo_walk`] is
//! `break`.
//!
//! Syntax autocommands are suppressed for the whole walk (skipping the
//! syntax file is a large speed improvement) and fired afterwards, once,
//! for the buffers that were loaded meanwhile.
//!
//! Original: `src/nvim/ex_cmds2.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::flag::{CCGD_AW, CCGD_EXCMD, CCGD_FORCEIT, DOBUF_FIRST};
use super::{buffers, check_changed};
use crate::arglist::{do_argfile, editing_arg_idx};
use crate::autocmd::{
    apply_autocmds, au_event_disable, au_event_restore, aucmd_prepbuf, aucmd_restbuf,
};
use crate::buffer::{BufFlags, buf_hide, goto_buffer};
use crate::ex_docmd::state::listcmd_busy;
use crate::ex_docmd::{DoCmdOpts, do_cmdline};
use crate::getchar::state::got_int;
use crate::guard::Suppress;
use crate::mark::setpcmark;
use crate::message::emsg;
use crate::r#move::validate_cursor;
use crate::normal::do_check_scrollbind;
use crate::pos::MAXLNUM;
use crate::quickfix::{ex_cc, ex_cnext, qf_get_cur_idx, qf_get_valid_size};
use crate::search::FORWARD;
use crate::types::AutoEvent;
use crate::types::CmdIdx;
use crate::types::{AcoSave, ExArg, LineNr, size_t};
use crate::window::{goto_tab, valid_tab, valid_win, win_goto, win_split, win_valid};
use crate::winlayer::prev_window;
use crate::winlayer::{Buf, TabId, TabPage, Win, WinId, first_buffer, first_tab, first_window};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// Which list [`ex_listdo`] walks.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ListDo {
    /// `:argdo` -- the argument list of the current window.
    Args,
    /// `:windo` -- the windows of the current tab page.
    Windows,
    /// `:tabdo` -- the tab pages.
    Tabs,
    /// `:bufdo` -- the listed buffers.
    Buffers,
    /// `:cdo`/`:cfdo`, or `:ldo`/`:lfdo` when `location` is set -- the
    /// entries of the quickfix or location list.
    Quickfix { location: bool },
}

impl ListDo {
    /// The eight commands the ex command table routes to [`ex_listdo`].
    /// Nothing else reaches it, so anything else is `None`.
    fn from_cmdidx(cmdidx: CmdIdx) -> Option<Self> {
        Some(match cmdidx {
            CmdIdx::argdo => Self::Args,
            CmdIdx::windo => Self::Windows,
            CmdIdx::tabdo => Self::Tabs,
            CmdIdx::bufdo => Self::Buffers,
            CmdIdx::cdo | CmdIdx::cfdo => Self::Quickfix { location: false },
            CmdIdx::ldo | CmdIdx::lfdo => Self::Quickfix { location: true },
            _ => return None,
        })
    }

    /// Whether the walk changes which buffer a window shows. `:windo` and
    /// `:tabdo` only move between existing windows.
    fn changes_buffer(self) -> bool {
        !matches!(self, Self::Windows | Self::Tabs)
    }
}

/// `:argdo`, `:windo`, `:bufdo`, `:tabdo`, `:cdo`, `:ldo`, `:cfdo` and
/// `:lfdo`.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_listdo(args: *mut ExArg) {
    // SAFETY: module contract.
    let (cmdidx, forceit) = unsafe { ((*args).cmdidx, (*args).forceit != 0) };
    let Some(list) = ListDo::from_cmdidx(cmdidx) else {
        return;
    };
    if !leave_winfixbuf(list, forceit) {
        return;
    }

    // Temporarily override ShmFlag::OVER and ShmFlag::OVERALL so that a file message
    // does not overwrite output from the command.
    let keep_messages = Suppress::message_overwrite();

    // Don't run Syntax autocommands: skipping the syntax file is a large
    // speed improvement.
    let mut save_ei = ptr::null_mut();
    if list.changes_buffer() {
        // SAFETY: module contract.
        save_ei = unsafe { au_event_disable(c",Syntax".as_ptr().cast_mut()) };
        for mut buf in buffers() {
            buf.b_flags.clear(BufFlags::SYN_SET);
        }
    }

    // SAFETY: module contract.
    let may_run = unsafe {
        !list.changes_buffer()
            || buf_hide(Buf::current())
            || !check_changed(
                Buf::current(),
                CCGD_AW | if forceit { CCGD_FORCEIT } else { 0 } | CCGD_EXCMD,
            )
    };
    if may_run {
        // SAFETY: module contract.
        unsafe { listdo_walk(args, list) };
    }

    drop(keep_messages);
    if !save_ei.is_null() {
        // SAFETY: `save_ei` is what `au_event_disable` returned.
        unsafe { restore_syntax_events(save_ei) };
    }
}

/// A walk that changes buffers cannot start in a 'winfixbuf' window: move to
/// one without it, splitting if there is none. Answers false when the
/// command must not run at all.
fn leave_winfixbuf(list: ListDo, forceit: bool) -> bool {
    const E_WINFIXBUF: &CStr = c"E1513: Cannot switch buffer. 'winfixbuf' is enabled";
    if Win::current().w_onebuf_opt.wo_wfb == 0 {
        return true;
    }
    if list == (ListDo::Quickfix { location: true }) && !forceit {
        // ":ldo" would have to leave the location list's own window.
        emsg(E_WINFIXBUF);
        return false;
    }
    if let Some(prev) = prev_window().filter(|p| win_valid(p.id()) && p.w_onebuf_opt.wo_wfb == 0) {
        win_goto(prev);
    }
    if Win::current().w_onebuf_opt.wo_wfb != 0 {
        // The new window is 'nowinfixbuf' and becomes the current one.
        let _ = win_split(0, 0);
        if Win::current().w_onebuf_opt.wo_wfb != 0 {
            // Autocommands set 'winfixbuf', or sent us to another window
            // that has it set, or the split failed. Give up.
            emsg(E_WINFIXBUF);
            return false;
        }
    }
    true
}

/// Position the walk at `args.line1`, then run `args.arg` once per item
/// until the list runs out, the range does, or something goes wrong.
///
/// # Safety
/// Module contract.
unsafe fn listdo_walk(args: *mut ExArg, list: ListDo) {
    // SAFETY: module contract. The command being run can do anything at all,
    // which is why every step re-validates what it is about to touch.
    let mut i: c_int = 0;
    // Start at the eap->line1'th argument/window/tab page.
    // Identities, not addresses: the command run for each entry can close the
    // window or tab page the walk is standing on, and the next round asks
    // whether it is still there.
    let mut wp = first_window().map(Win::id);
    let mut tp = first_tab().map(TabPage::id);
    match list {
        ListDo::Windows => {
            while let Some(cur) = wp
                .and_then(WinId::get)
                .filter(|_| (i as LineNr + 1) < unsafe { (*args).line1 })
            {
                i += 1;
                wp = cur.next().map(Win::id);
            }
        }
        ListDo::Tabs => {
            while let Some(cur) = tp
                .and_then(TabId::get)
                .filter(|_| (i as LineNr + 1) < unsafe { (*args).line1 })
            {
                i += 1;
                tp = cur.next().map(TabPage::id);
            }
        }
        ListDo::Args => i = unsafe { (*args).line1 } as c_int - 1,
        _ => {}
    }

    let mut buf = Buf::current_raw();
    let mut qf_size: size_t = 0;
    match list {
        ListDo::Buffers => {
            // Advance to the first listed buffer after "eap->line1".
            let mut cur = first_buffer();
            let unlisted =
                |b: &Buf| (b.handle as LineNr) < unsafe { (*args).line1 } || b.b_p_bl == 0;
            while let Some(b) = cur.filter(unlisted) {
                if b.handle as LineNr > unsafe { (*args).line2 } {
                    cur = None;
                    break;
                }
                cur = b.next();
            }
            buf = cur.map_or(ptr::null_mut(), Buf::raw);
            if !buf.is_null() {
                unsafe {
                    goto_buffer(
                        args,
                        DOBUF_FIRST as c_int,
                        FORWARD as c_int,
                        (*buf).handle as c_int,
                    )
                };
            }
        }
        ListDo::Quickfix { .. } => {
            qf_size = unsafe { qf_get_valid_size(args) };
            debug_assert!(unsafe { (*args).line1 } >= 0 as LineNr, "eap->line1 >= 0");
            if qf_size == 0 || unsafe { (*args).line1 } as size_t > qf_size {
                buf = ptr::null_mut();
            } else {
                unsafe { ex_cc(args) };
                buf = Buf::current_raw();
                i = unsafe { (*args).line1 } as c_int - 1;
                if unsafe { (*args).addr_count } <= 0 {
                    // Default to every quickfix/location list entry.
                    debug_assert!(qf_size < MAXLNUM as c_int as size_t, "qf_size < MAXLNUM");
                    unsafe { (*args).line2 = qf_size as LineNr };
                }
            }
        }
        // `:argdo`, `:windo` and `:tabdo` set the previous-context mark
        // instead: they are not going anywhere on their own.
        _ => setpcmark(),
    }

    // Avoids setting the previous-context mark for every step below.
    listcmd_busy.set(true);
    let mut next_fnum: c_int = 0;
    while !got_int.get() && !buf.is_null() {
        let mut execute = true;
        match list {
            ListDo::Args => {
                // Go to argument "i".
                if i == unsafe { (*Win::current().w_alist).al_ga.len() as c_int } {
                    break;
                }
                // Don't call `do_argfile` when already there, it would
                // try reloading the file.
                if Win::current().w_arg_idx != i || !editing_arg_idx(Win::current()) {
                    unsafe { do_argfile(args, i) };
                }
                if Win::current().w_arg_idx != i {
                    break;
                }
            }
            ListDo::Windows => {
                // Go to window "wp".
                let Some(cur) = wp.and_then(valid_win) else {
                    break;
                };
                execute = !cur.w_floating || (!cur.w_config.hide && cur.w_config.focusable);
                if execute {
                    win_goto(cur);
                    if Win::current_raw() != cur.raw() {
                        // Something must be wrong.
                        break;
                    }
                }
                wp = cur.next().map(Win::id);
            }
            ListDo::Tabs => {
                // Go to tab page "tp".
                let Some(cur) = tp.and_then(valid_tab) else {
                    break;
                };
                goto_tab(cur, true, true);
                tp = cur.next().map(TabPage::id);
            }
            ListDo::Buffers => {
                // Remember the number of the next listed buffer, in case
                // ":bwipe" is used or autocommands do something strange.
                next_fnum = -1;
                let mut bp = Buf::current().next();
                while let Some(b) = bp {
                    if b.b_p_bl != 0 {
                        next_fnum = b.handle as c_int;
                        break;
                    }
                    bp = b.next();
                }
            }
            ListDo::Quickfix { .. } => {}
        }

        i += 1;
        if execute {
            let _ = unsafe {
                do_cmdline(
                    (*args).arg,
                    (*args).ea_getline,
                    (*args).cookie,
                    DoCmdOpts::VERBOSE | DoCmdOpts::NOWAIT,
                )
            };
        }

        match list {
            ListDo::Buffers => {
                // Done?
                if next_fnum < 0 || next_fnum as LineNr > unsafe { (*args).line2 } {
                    break;
                }
                // Does the buffer still exist?
                if !buffers().any(|bp| bp.handle == next_fnum) {
                    break;
                }
                unsafe { goto_buffer(args, DOBUF_FIRST as c_int, FORWARD as c_int, next_fnum) };
                // If autocommands took us elsewhere, quit here.
                if Buf::current().handle != next_fnum {
                    break;
                }
            }
            ListDo::Quickfix { .. } => {
                debug_assert!(i >= 0, "i >= 0");
                if i as size_t >= qf_size || i as LineNr >= unsafe { (*args).line2 } {
                    break;
                }
                let qf_idx = unsafe { qf_get_cur_idx(args) };
                unsafe { ex_cnext(args) };
                // If jumping to the next quickfix entry fails, quit here.
                if unsafe { qf_get_cur_idx(args) } == qf_idx {
                    break;
                }
            }
            ListDo::Windows => {
                if execute {
                    // The cursor may have moved.
                    validate_cursor(Win::current());
                    // Required when 'scrollbind' has been set.
                    if Win::current().w_onebuf_opt.wo_scb != 0 {
                        unsafe { do_check_scrollbind(true) };
                    }
                }
                if i as LineNr + 1 > unsafe { (*args).line2 } {
                    break;
                }
            }
            ListDo::Tabs => {
                if i as LineNr + 1 > unsafe { (*args).line2 } {
                    break;
                }
            }
            ListDo::Args => {
                if i as LineNr >= unsafe { (*args).line2 } {
                    break;
                }
            }
        }
    }
    listcmd_busy.set(false);
}

/// Put the Syntax event back and fire it for the buffers that were opened
/// while it was suppressed.
///
/// # Safety
/// `save_ei` is what [`au_event_disable`] returned, and module contract.
unsafe fn restore_syntax_events(save_ei: *mut c_char) {
    // SAFETY: caller contract. `apply_autocmds` can do anything to the
    // buffer list, so the walk starts over whenever it has run.
    let mut aco = AcoSave::default();
    unsafe { au_event_restore(save_ei) };

    let mut cur = first_buffer();
    while let Some(mut buf) = cur {
        let mut bnext = buf.next();
        if buf.b_nwindows > 0 && buf.b_flags.has(BufFlags::SYN_SET) {
            buf.b_flags.clear(BufFlags::SYN_SET);
            if buf.raw() == Buf::current_raw() {
                unsafe {
                    apply_autocmds(
                        AutoEvent::Syntax,
                        Buf::current().b_p_syn,
                        Buf::current().b_fname,
                        true,
                        Buf::current_or_none(),
                    )
                };
            } else {
                let (syn, name, raw) = (buf.b_p_syn, buf.b_fname, buf.raw());
                unsafe { aucmd_prepbuf(&raw mut aco, Buf::new(raw)) };
                unsafe { apply_autocmds(AutoEvent::Syntax, syn, name, true, Buf::from_raw(raw)) };
                unsafe { aucmd_restbuf(&raw mut aco) };
            }
            // Start over, in case autocommands messed things up.
            bnext = first_buffer();
        }
        cur = bnext;
    }
}
