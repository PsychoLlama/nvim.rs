//! Ex ranges: one address, the pair around a `,` or `;`, and the defaults a
//! command supplies when it was given none.
//!
//! An address is not always a line number. `cmd_addr_type` decides whether
//! it counts lines, windows, buffers, arguments, tab pages or quickfix
//! entries, and the same syntax means different things for each — `.` is
//! the current *window* for `:wincmd`, the current *buffer* for `:bdelete`
//! and the cursor line for `:print`. Every function here therefore has one
//! arm per address kind, and c2rust rendered those arms as bare numbers;
//! they are named now.
//!
//! Each function is a walk over the command line the caller owns, so each
//! takes one `unsafe` block for its whole body — see `scan.rs`.
#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr;
use std::ffi::CString;

use crate::buffer::{buf_is_quickfix, current_buf, get_highest_fnum};
use crate::charset::{getdigits, getdigits_int32};

use crate::cursor::check_cursor_col;

use crate::ex_docmd::lookup::find_ex_command;
use crate::ex_docmd::scan::skip_colon_white;
use crate::ex_docmd::window::{current_tab_nr, current_win_nr};
use crate::ex_docmd::{
    INT32_MAX, cmdnames, e_backslash, e_invrange, e_line_number_out_of_range, e_no_errors,
    e_norange, kMarkAll, kMarkBufLocal, searchcmdlen,
};

use crate::fold::has_folding;
use crate::main::{curbuf, curtab, curwin};
use crate::mark::{mark_check, mark_get, mark_move_to};

use crate::message::iemsg;
use crate::option::magic_isset;
use crate::os::cshim::gettext;
use crate::pos::{MAXCOL, MAXLNUM};
use crate::quickfix::qf_get_size;

use crate::regexp::{RE_SEARCH, RE_SUBST, skip_regexp};
use crate::search::{BACKWARD, FORWARD, SEARCH_HIS, SEARCH_KEEP, SEARCH_MSG, do_search, searchit};
use crate::strings::vim_strchr;
use crate::types::{
    CMD_SIZE, CMD_cc, CMD_diffget, CMD_diffput, CMD_ll, CMD_wincmd, CmdAddr, Direction, ExArgt,
    ExpandContext, FAIL, MarkGet, MarkMove, NUL, OK, buf_T, colnr_T, exarg_T, fmark_T, linenr_T,
    pos_T, size_t,
};
use crate::winlayer::{Buf, Ea, Win, first_buffer, last_buffer};
use ::libc::strlen;

/// Where a `+N`/`-N` offset lands when the addresses count buffers.
///
/// Buffer numbers are not contiguous, so an offset is a *walk* over the
/// buffer list rather than arithmetic. `CmdAddr::LoadedBuffers` skips the
/// unloaded ones on the way, and then — the tail loop below — walks back
/// the other way if it ended on one anyway.
pub(crate) fn compute_buffer_local_count(
    addr_type: CmdAddr,
    lnum: linenr_T,
    offset: c_int,
) -> c_int {
    let loaded_only = addr_type == CmdAddr::LoadedBuffers;
    let mut count = offset;
    let mut buf = head();
    while (buf.handle as linenr_T) < lnum {
        let Some(next) = buf.next() else { break };
        buf = next;
    }
    let step = |b: Buf| if offset < 0 { b.prev() } else { b.next() };
    while count != 0 {
        count += if count < 0 { 1 } else { -1 };
        let Some(next) = step(buf) else { break };
        buf = next;
        if loaded_only {
            while buf.b_ml.ml_mfp.is_null() {
                let Some(next) = step(buf) else { break };
                buf = next;
            }
        }
    }
    // Landing on an unloaded buffer is still possible — the walk above
    // gives up at the end of the list. Back out in the *opposite*
    // direction to the one the offset asked for.
    if loaded_only {
        while buf.b_ml.ml_mfp.is_null() {
            let next = if offset >= 0 { buf.prev() } else { buf.next() };
            let Some(next) = next else { break };
            buf = next;
        }
    }
    buf.handle as c_int
}

/// The head of the buffer list, which upstream dereferences here without
/// checking: the editor has a buffer from startup to exit, and everything
/// below runs from a command line.
fn head() -> Buf {
    first_buffer().expect("the editor always has a buffer")
}

/// The tail of the buffer list. [`head`].
fn tail() -> Buf {
    last_buffer().expect("the editor always has a buffer")
}

/// `:wincmd`'s address kind depends on the window command it is given: `w`
/// counts windows, `^` counts buffers, most of the tree counts something
/// the window code names itself, and the rest take no address at all.
///
/// Upstream spells the four sets as one `switch` with 68 labels. They are
/// four tables here, which is the same thing said once.
#[rustfmt::skip]
const WINCMD_OTHER: &[u8] = b"SsnjkTrRKJ+-_|]gvhlHL><}fFid\x13\x0e\x0a\x0b\x12\x1f\x1d\x07\x16\x08\x0c\x06\x09\x04";
const WINCMD_BUFFERS: &[u8] = b"^\x1e";
const WINCMD_WINDOWS: &[u8] = b"qcowWx\x11\x03\x0f\x17\x18";
const WINCMD_NONE: &[u8] = b"zPtbp=\x1a\x14\x02\x10\x0d";

pub(crate) unsafe fn get_wincmd_addr_type(arg: *const c_char, mut eap: Ea) {
    let c = unsafe { *arg } as u8;
    eap.addr_type = if WINCMD_OTHER.contains(&c) {
        CmdAddr::Other
    } else if WINCMD_BUFFERS.contains(&c) {
        CmdAddr::Buffers
    } else if WINCMD_WINDOWS.contains(&c) {
        CmdAddr::Windows
    } else if WINCMD_NONE.contains(&c) {
        CmdAddr::NoRange
    } else {
        // Anything else keeps whatever the command table said.
        return;
    };
}

/// Take the address kind from the command table, with the three exceptions
/// the table cannot express.
pub unsafe fn set_cmd_addr_type(eap: *mut exarg_T, p: *mut c_char) {
    let mut ea = unsafe { Ea::new(eap) };
    if (ea.cmdidx as c_int) < 0 {
        return;
    }
    ea.addr_type = if ea.cmdidx as c_int != CMD_SIZE as c_int {
        cmdnames[ea.cmdidx as usize].cmd_addr_type
    } else {
        CmdAddr::Lines
    };
    if ea.cmdidx as c_int == CMD_wincmd as c_int && !p.is_null() {
        unsafe { get_wincmd_addr_type(skipwhite(p), ea) };
    }
    // `:cc`/`:ll` in a quickfix window address the window's entries.
    if (ea.cmdidx as c_int == CMD_cc as c_int || ea.cmdidx as c_int == CMD_ll as c_int)
        && buf_is_quickfix(current_buf())
    {
        ea.addr_type = CmdAddr::Other;
    }
}

/// The address `.` stands for, which is also what a bare `+N`/`-N` counts
/// from.
pub unsafe fn get_cmd_default_range(eap: *mut exarg_T) -> linenr_T {
    let mut eap = unsafe { Ea::new(eap) };
    match eap.addr_type {
        CmdAddr::Lines | CmdAddr::Other => {
            // Not the cursor line but the *last* line when the cursor is
            // past it, which a buffer shrinking under a command allows.
            cur_win().w_cursor.lnum.min(cur_buf().b_ml.ml_line_count)
        }
        CmdAddr::Windows => current_win_nr(curwin.get()) as linenr_T,
        CmdAddr::Arguments => {
            let len = unsafe { arglist_len() };
            if cur_win().w_arg_idx + 1 < len {
                cur_win().w_arg_idx as linenr_T + 1
            } else {
                len as linenr_T
            }
        }
        CmdAddr::LoadedBuffers | CmdAddr::Buffers => cur_buf().handle as linenr_T,
        CmdAddr::Tabs => current_tab_nr(curtab.get()) as linenr_T,
        CmdAddr::TabsRelative | CmdAddr::Unsigned => 1,
        CmdAddr::Quickfix => qf_get_cur_idx(eap.raw()) as linenr_T,
        CmdAddr::QuickfixValid => qf_get_cur_valid_idx(eap.raw()) as linenr_T,
        _ => 0,
    }
}

/// The range an `ExArgt::DFLALL` command means by "no range": everything.
pub unsafe fn set_cmd_dflall_range(eap: *mut exarg_T) {
    let mut ea = unsafe { Ea::new(eap) };
    ea.line1 = 1;
    match ea.addr_type {
        CmdAddr::Lines | CmdAddr::Other => {
            ea.line2 = cur_buf().b_ml.ml_line_count;
        }
        CmdAddr::LoadedBuffers => {
            let (first, last) = loaded_buffer_range();
            ea.line1 = first;
            ea.line2 = last;
        }
        CmdAddr::Buffers => {
            ea.line1 = head().handle as linenr_T;
            ea.line2 = tail().handle as linenr_T;
        }
        CmdAddr::Windows => {
            ea.line2 = current_win_nr(ptr::null()) as linenr_T;
        }
        CmdAddr::Tabs => {
            ea.line2 = current_tab_nr(ptr::null_mut()) as linenr_T;
        }
        CmdAddr::TabsRelative => ea.line2 = 1,
        CmdAddr::Arguments => {
            let len = unsafe { arglist_len() };
            if len == 0 {
                ea.line2 = 0;
                ea.line1 = 0;
            } else {
                ea.line2 = len as linenr_T;
            }
        }
        CmdAddr::QuickfixValid => {
            ea.line2 = qf_get_valid_size(eap) as linenr_T;
            if ea.line2 == 0 {
                ea.line2 = 1;
            }
        }
        t if t == CmdAddr::NoRange || t == CmdAddr::Unsigned || t == CmdAddr::Quickfix => {
            unsafe {
                iemsg(gettext(
                c"INTERNAL: Cannot use ExArgt::DFLALL with CmdAddr::NoRange, CmdAddr::Unsigned or CmdAddr::Quickfix"
                    .as_ptr(),
            ))
            };
        }
        _ => {}
    }
}

/// How many files are in the current window's argument list.
unsafe fn arglist_len() -> c_int {
    unsafe { (*cur_win().w_alist).al_ga.ga_len }
}

/// The handles of the first and last *loaded* buffers.
fn loaded_buffer_range() -> (linenr_T, linenr_T) {
    let mut buf = head();
    while buf.b_ml.ml_mfp.is_null() {
        let Some(next) = buf.next() else { break };
        buf = next;
    }
    let first = buf.handle as linenr_T;
    let mut buf = tail();
    while buf.b_ml.ml_mfp.is_null() {
        let Some(prev) = buf.prev() else { break };
        buf = prev;
    }
    (first, buf.handle as linenr_T)
}

/// Where the command word starts, without consuming the range.
pub(crate) fn find_excmd_after_range(mut ea: Ea) -> *mut c_char {
    let cmd = ea.cmd;
    ea.cmd = unsafe { skip_range(ea.cmd, ptr::null_mut()) };
    // SAFETY: the command is the caller's, promised live by `Ea`.
    let p = unsafe { find_ex_command(ea.raw(), ptr::null_mut()) };
    ea.cmd = cmd;
    p
}

/// Read the whole range — one address, or a pair around `,` or `;` — into
/// `eap->line1`/`line2`/`addr_count`.
///
/// `;` differs from `,` in moving the cursor to the first address before
/// the second is resolved, which is what makes `:.;+3` mean "three lines
/// from here" however the first address was spelled.
pub unsafe fn parse_cmd_address(
    eap: *mut exarg_T,
    errormsg: &mut Option<CString>,
    silent: bool,
) -> c_int {
    // The records `:*` reads the Visual marks into: they are never adjusted
    // and have no store, so each is computed into a record of its own.
    let (mut first, mut last) = (fmark_T::UNSET, fmark_T::UNSET);
    let mut ea = unsafe { Ea::new(eap) };
    let mut address_count = 1;
    let mut lnum: linenr_T = 0;
    let mut need_check_cursor = false;
    let mut ret = FAIL;

    'theend: {
        loop {
            ea.line1 = ea.line2;
            ea.line2 = unsafe { get_cmd_default_range(eap) };
            ea.cmd = skipwhite(ea.cmd);
            lnum = unsafe {
                get_address(
                    eap,
                    ea.cmd_ptr(),
                    ea.addr_type,
                    ea.skip != 0,
                    silent,
                    (ea.addr_count == 0) as c_int,
                    address_count,
                    errormsg,
                )
            };
            address_count += 1;
            if ea.cmd.is_null() {
                break 'theend;
            }
            if lnum != MAXLNUM as linenr_T {
                ea.line2 = lnum;
            } else if unsafe { *ea.cmd } as c_int == '%' as c_int {
                // `%` is not an address, it is a whole range, so it is
                // only recognised where an address was expected and
                // none was found.
                ea.cmd = unsafe { ea.cmd.add(1) };
                if !whole_range(ea, errormsg) {
                    break 'theend;
                }
                ea.addr_count += 1;
            } else if unsafe { *ea.cmd } as c_int == '*' as c_int {
                if ea.addr_type != CmdAddr::Lines {
                    *errormsg = Some(ex_msg(e_invrange.as_ptr()));
                    break 'theend;
                }
                ea.cmd = unsafe { ea.cmd.add(1) };
                if ea.skip == 0 {
                    let fm = mark_get_visual(curbuf.get(), &raw mut first, '<' as c_int);
                    if !unsafe { mark_check(fm, errormsg) } {
                        break 'theend;
                    }
                    debug_assert!(!fm.is_null());
                    ea.line1 = unsafe { (*fm).mark.lnum };
                    let fm = mark_get_visual(curbuf.get(), &raw mut last, '>' as c_int);
                    if !unsafe { mark_check(fm, errormsg) } {
                        break 'theend;
                    }
                    debug_assert!(!fm.is_null());
                    ea.line2 = unsafe { (*fm).mark.lnum };
                    ea.addr_count += 1;
                }
            }
            ea.addr_count += 1;
            if unsafe { *ea.cmd } as c_int == ';' as c_int {
                if ea.skip == 0 {
                    cur_win().w_cursor.lnum = ea.line2;
                    // A zero line number is not a position, so only the
                    // column is worth correcting there.
                    if ea.line2 > 0 {
                        check_cursor(unsafe { Win::current() });
                    } else {
                        check_cursor_col(unsafe { Win::current() });
                    }
                    need_check_cursor = true;
                }
            } else if unsafe { *ea.cmd } as c_int != ',' as c_int {
                break;
            }
            ea.cmd = unsafe { ea.cmd.add(1) };
        }
        if ea.addr_count == 1 {
            ea.line1 = ea.line2;
            // One address that resolved to nothing is no address.
            if lnum == MAXLNUM as linenr_T {
                ea.addr_count = 0;
            }
        }
        ret = OK;
    }
    // The `;` above may have left the cursor on a line the command is
    // about to delete; putting it back is the caller's problem, so this
    // only re-clamps it.
    if need_check_cursor {
        check_cursor(unsafe { Win::current() });
    }
    ret
}

/// Fill in the range `%` means for this address kind. Answers false when
/// the kind has no "all", having reported why.
fn whole_range(mut eap: Ea, errormsg: &mut Option<CString>) -> bool {
    match eap.addr_type {
        CmdAddr::Lines | CmdAddr::Other => {
            eap.line1 = 1;
            eap.line2 = cur_buf().b_ml.ml_line_count;
        }
        CmdAddr::LoadedBuffers => {
            let (first, last) = loaded_buffer_range();
            eap.line1 = first;
            eap.line2 = last;
        }
        CmdAddr::Buffers => {
            eap.line1 = head().handle as linenr_T;
            eap.line2 = tail().handle as linenr_T;
        }
        CmdAddr::Windows | CmdAddr::Tabs => {
            // Only a *user* command may say `%` over windows or tab
            // pages; a builtin one would not know what to do with it.
            if (eap.cmdidx as c_int) >= 0 {
                *errormsg = Some(ex_msg(e_invrange.as_ptr()));
                return false;
            }
            eap.line1 = 1;
            eap.line2 = if eap.addr_type == CmdAddr::Windows {
                current_win_nr(ptr::null()) as linenr_T
            } else {
                current_tab_nr(ptr::null_mut()) as linenr_T
            };
        }
        CmdAddr::TabsRelative | CmdAddr::Unsigned | CmdAddr::Quickfix => {
            *errormsg = Some(ex_msg(e_invrange.as_ptr()));
            return false;
        }
        CmdAddr::Arguments => {
            let len = unsafe { arglist_len() };
            if len == 0 {
                eap.line2 = 0;
                eap.line1 = 0;
            } else {
                eap.line1 = 1;
                eap.line2 = len as linenr_T;
            }
        }
        CmdAddr::QuickfixValid => {
            // SAFETY: the caller's promise -- a live command.
            let valid = qf_get_valid_size(eap.raw()) as linenr_T;
            eap.line1 = 1;
            eap.line2 = valid;
            if eap.line2 == 0 {
                eap.line2 = 1;
            }
        }
        // `NoRange` reaches here for a user command and is accepted
        // without setting anything, as upstream does.
        CmdAddr::NoRange => {}
    }
    true
}

/// Step over a range without resolving it. Used wherever the command word
/// has to be found before the range can mean anything — the modifier scan,
/// completion, and `find_excmd_after_range`.
pub unsafe fn skip_range(cmd: *const c_char, ctx: *mut ExpandContext) -> *mut c_char {
    let mut cmd = cmd;
    while !unsafe { vim_strchr(c" \t0123456789.$%'/?-+,;\\".as_ptr(), *cmd as u8 as c_int) }
        .is_null()
    {
        if unsafe { *cmd } as c_int == '\\' as c_int {
            // Only `\/`, `\?` and `\&` are addresses; any other
            // backslash ends the range.
            let next = unsafe { *cmd.add(1) } as c_int;
            if next != '?' as c_int && next != '/' as c_int && next != '&' as c_int {
                break;
            }
            cmd = unsafe { cmd.add(1) };
        } else if unsafe { *cmd } as c_int == '\'' as c_int {
            cmd = unsafe { cmd.add(1) };
            if unsafe { *cmd } as c_int == NUL && !ctx.is_null() {
                unsafe { *ctx = ExpandContext::Nothing };
            }
        } else if unsafe { *cmd } as c_int == '/' as c_int
            || unsafe { *cmd } as c_int == '?' as c_int
        {
            let delim = unsafe { *cmd };
            cmd = unsafe { cmd.add(1) };
            while unsafe { *cmd } as c_int != NUL && unsafe { *cmd } != delim {
                let at = cmd;
                cmd = unsafe { cmd.add(1) };
                if unsafe { *at } as c_int == '\\' as c_int && unsafe { *cmd } as c_int != NUL {
                    cmd = unsafe { cmd.add(1) };
                }
            }
            if unsafe { *cmd } as c_int == NUL && !ctx.is_null() {
                unsafe { *ctx = ExpandContext::Nothing };
            }
        }
        if unsafe { *cmd } as c_int != NUL {
            cmd = unsafe { cmd.add(1) };
        }
    }
    cmd = unsafe { skip_colon_white(cmd, false) };
    // `:*` is the "last Visual area" range, spelled after the colons.
    if unsafe { *cmd } as c_int == '*' as c_int {
        cmd = unsafe { skipwhite(cmd.add(1)) };
    }
    cmd as *mut c_char
}

/// E493 or E481, depending on whether the command takes a range at all.
pub(crate) unsafe fn addr_error(addr_type: CmdAddr) -> CString {
    if addr_type == CmdAddr::NoRange {
        ex_msg(e_norange.as_ptr())
    } else {
        ex_msg(e_invrange.as_ptr())
    }
}

/// Read one address, including any `+N`/`-N` offsets after it.
///
/// Answers `MAXLNUM` for "there was no address here", which is not the same
/// as an address that resolved to nothing, and writes null through `ptr` to
/// report an error (the message goes to `errormsg`).
#[allow(clippy::too_many_arguments)]
pub unsafe fn get_address(
    eap: *mut exarg_T,
    ptr: *mut *mut c_char,
    addr_type: CmdAddr,
    skip: bool,
    silent: bool,
    to_other_file: c_int,
    address_count: c_int,
    errormsg: &mut Option<CString>,
) -> linenr_T {
    let ea = unsafe { Ea::new(eap) };
    // The record a `'m` address answers into; see `mark_get`.
    let mut slot = fmark_T::UNSET;
    let mut cmd: *mut c_char = unsafe { skipwhite(*ptr) };
    let mut lnum: linenr_T = MAXLNUM as linenr_T;
    let mut pos = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    'error: loop {
        match unsafe { *cmd } as u8 {
            b'.' | b'$' => {
                let want_last = unsafe { *cmd } as u8 == b'$';
                cmd = unsafe { cmd.add(1) };
                let at = if want_last {
                    last_lnum(ea, addr_type)
                } else {
                    dot_lnum(ea, addr_type)
                };
                match at {
                    Addr::At(n) => lnum = n,
                    Addr::Unchanged => {}
                    Addr::Refused => {
                        *errormsg = Some(unsafe { addr_error(addr_type) });
                        cmd = ptr::null_mut();
                        break;
                    }
                }
            }
            b'\'' => {
                cmd = unsafe { cmd.add(1) };
                if unsafe { *cmd } as c_int == NUL {
                    cmd = ptr::null_mut();
                    break;
                }
                if addr_type != CmdAddr::Lines {
                    *errormsg = Some(unsafe { addr_error(addr_type) });
                    cmd = ptr::null_mut();
                    break;
                }
                if skip {
                    cmd = unsafe { cmd.add(1) };
                } else {
                    // A mark in another file is only followed when it
                    // is the whole address and the command can change
                    // file; otherwise only this buffer's marks count.
                    let flag = if to_other_file != 0 && unsafe { *cmd.add(1) } as c_int == NUL {
                        kMarkAll as c_int
                    } else {
                        kMarkBufLocal as c_int
                    } as MarkGet;
                    let fm = unsafe {
                        mark_get(
                            curbuf.get(),
                            curwin.get(),
                            &raw mut slot,
                            flag,
                            *cmd as c_int,
                        )
                    };
                    cmd = unsafe { cmd.add(1) };
                    if !fm.is_null() && unsafe { (*fm).fnum } != cur_buf().handle {
                        unsafe { mark_move_to(fm, 0 as MarkMove) };
                        lnum = cur_win().w_cursor.lnum;
                    } else if !unsafe { mark_check(fm, errormsg) } {
                        cmd = ptr::null_mut();
                        break;
                    } else {
                        debug_assert!(!fm.is_null());
                        lnum = unsafe { (*fm).mark.lnum };
                    }
                }
            }
            c @ (b'/' | b'?') => {
                cmd = unsafe { cmd.add(1) };
                let c = c as c_int;
                if addr_type != CmdAddr::Lines {
                    *errormsg = Some(unsafe { addr_error(addr_type) });
                    cmd = ptr::null_mut();
                    break;
                }
                if skip {
                    cmd = unsafe { skip_regexp(cmd, c, magic_isset() as c_int) };
                    if unsafe { *cmd } as c_int == c {
                        cmd = unsafe { cmd.add(1) };
                    }
                } else {
                    // The search starts from the address read so far,
                    // so `:3/pat/` searches from line 3.
                    pos = cur_win().w_cursor;
                    if lnum > 0 && lnum != MAXLNUM as linenr_T {
                        cur_win().w_cursor.lnum = lnum.min(cur_buf().b_ml.ml_line_count);
                    }
                    cur_win().w_cursor.col = if c == '/' as c_int && cur_win().w_cursor.lnum > 0 {
                        MAXCOL as colnr_T
                    } else {
                        0
                    };
                    searchcmdlen.set(0);
                    let flags = if silent {
                        SEARCH_KEEP as c_int
                    } else {
                        SEARCH_HIS as c_int | SEARCH_MSG as c_int
                    };
                    if unsafe {
                        do_search(
                            ptr::null_mut(),
                            c,
                            c,
                            cmd,
                            strlen(cmd),
                            1,
                            flags,
                            ptr::null_mut(),
                        )
                    } == 0
                    {
                        cur_win().w_cursor = pos;
                        cmd = ptr::null_mut();
                        break;
                    }
                    lnum = cur_win().w_cursor.lnum;
                    cur_win().w_cursor = pos;
                    cmd = unsafe { cmd.add(searchcmdlen.get() as usize) };
                }
            }
            b'\\' => {
                // `\/` and `\?` repeat the last search pattern, `\&`
                // the last substitute pattern.
                cmd = unsafe { cmd.add(1) };
                if addr_type != CmdAddr::Lines {
                    *errormsg = Some(unsafe { addr_error(addr_type) });
                    cmd = ptr::null_mut();
                    break;
                }
                let i = if unsafe { *cmd } as c_int == '&' as c_int {
                    RE_SUBST as c_int
                } else if unsafe { *cmd } as c_int == '?' as c_int
                    || unsafe { *cmd } as c_int == '/' as c_int
                {
                    RE_SEARCH as c_int
                } else {
                    *errormsg = Some(ex_msg(e_backslash.as_ptr()));
                    cmd = ptr::null_mut();
                    break;
                };
                if !skip {
                    pos.lnum = if lnum != MAXLNUM as linenr_T {
                        lnum
                    } else {
                        cur_win().w_cursor.lnum
                    };
                    pos.col = if unsafe { *cmd } as c_int != '?' as c_int {
                        MAXCOL as colnr_T
                    } else {
                        0
                    };
                    pos.coladd = 0;
                    let dir = if unsafe { *cmd } as c_int == '?' as c_int {
                        BACKWARD as c_int
                    } else {
                        FORWARD as c_int
                    } as Direction;
                    if unsafe {
                        searchit(
                            Some(Win::current()),
                            Buf::current(),
                            &raw mut pos,
                            ptr::null_mut(),
                            dir,
                            c"".as_ptr() as *mut c_char,
                            0,
                            1,
                            SEARCH_MSG as c_int,
                            i,
                            ptr::null_mut(),
                        )
                    } == FAIL
                    {
                        cmd = ptr::null_mut();
                        break;
                    }
                    lnum = pos.lnum;
                }
                cmd = unsafe { cmd.add(1) };
            }
            _ => {
                if ascii_isdigit(unsafe { *cmd } as c_int) {
                    lnum = unsafe { getdigits(&raw mut cmd, false, 0) } as linenr_T;
                }
            }
        }

        // Offsets. A `+`/`-` with no address before it counts from the
        // address kind's "here".
        loop {
            cmd = skipwhite(cmd);
            if unsafe { *cmd } as c_int != '-' as c_int
                && unsafe { *cmd } as c_int != '+' as c_int
                && !ascii_isdigit(unsafe { *cmd } as c_int)
            {
                break;
            }
            if lnum == MAXLNUM as linenr_T
                && let Addr::At(n) = offset_base(ea, addr_type)
            {
                lnum = n;
            }
            let i = if ascii_isdigit(unsafe { *cmd } as c_int) {
                '+' as c_int
            } else {
                let at = cmd;
                cmd = unsafe { cmd.add(1) };
                unsafe { *at as u8 as c_int }
            };
            let n: linenr_T = if !ascii_isdigit(unsafe { *cmd } as c_int) {
                1
            } else {
                let n = unsafe { getdigits_int32(&raw mut cmd, false, MAXLNUM as i32) } as linenr_T;
                if n == MAXLNUM as linenr_T {
                    *errormsg = Some(ex_msg(e_line_number_out_of_range.as_ptr()));
                    cmd = ptr::null_mut();
                    break 'error;
                }
                n
            };
            if addr_type == CmdAddr::TabsRelative {
                *errormsg = Some(ex_msg(e_invrange.as_ptr()));
                cmd = ptr::null_mut();
                break 'error;
            } else if addr_type == CmdAddr::LoadedBuffers || addr_type == CmdAddr::Buffers {
                let offset = if i == '-' as c_int { -n } else { n };
                lnum = compute_buffer_local_count(addr_type, lnum, offset) as linenr_T;
            } else {
                // An offset in the *second* address of a range counts
                // from the end of a closed fold, so `:.,+1d` deletes
                // the whole fold and the line after it.
                if addr_type == CmdAddr::Lines
                    && (i == '-' as c_int || i == '+' as c_int)
                    && address_count >= 2
                {
                    has_folding(unsafe { Win::current() }, lnum, None, Some(&mut lnum));
                }
                if i == '-' as c_int {
                    lnum -= n;
                } else if lnum >= 0 && n >= INT32_MAX as linenr_T - lnum {
                    *errormsg = Some(ex_msg(e_line_number_out_of_range.as_ptr()));
                    cmd = ptr::null_mut();
                    break 'error;
                } else {
                    lnum += n;
                }
            }
        }

        // A search address may be followed by another one, which
        // searches on from where the first landed.
        if unsafe { *cmd } as c_int != '/' as c_int && unsafe { *cmd } as c_int != '?' as c_int {
            break;
        }
    }
    unsafe { *ptr = cmd };
    lnum
}

/// What an address kind answers for `.`, `$` or a bare offset.
///
/// The three are *not* the same table, and the difference is exactly the
/// three kinds that have no cursor: `.` and `$` report E481/E493 for
/// relative tabs, `CmdAddr::NoRange` and `CmdAddr::Unsigned`, while a bare `+N`
/// counts from 1 for relative tabs and from 0 for the other two. The
/// transpile wrote the three tables out separately and it is worth keeping
/// them apart.
enum Addr {
    /// Use this line number.
    At(linenr_T),
    /// Report `addr_error` and give up on the address.
    Refused,
    /// Leave whatever the caller had. Only reachable for an address kind
    /// outside the enumeration, which nothing in the command table holds.
    Unchanged,
}

/// What `.` means for this address kind.
fn dot_lnum(eap: Ea, addr_type: CmdAddr) -> Addr {
    Addr::At(match addr_type {
        CmdAddr::Lines | CmdAddr::Other => cur_win().w_cursor.lnum,
        CmdAddr::Windows => current_win_nr(curwin.get()) as linenr_T,
        CmdAddr::Arguments => (cur_win().w_arg_idx + 1) as linenr_T,
        CmdAddr::LoadedBuffers | CmdAddr::Buffers => cur_buf().handle as linenr_T,
        CmdAddr::Tabs => current_tab_nr(curtab.get()) as linenr_T,
        CmdAddr::Quickfix => qf_get_cur_idx(eap.raw()) as linenr_T,
        CmdAddr::QuickfixValid => qf_get_cur_valid_idx(eap.raw()) as linenr_T,
        t if t == CmdAddr::NoRange || t == CmdAddr::TabsRelative || t == CmdAddr::Unsigned => {
            return Addr::Refused;
        }
        _ => return Addr::Unchanged,
    })
}

/// What `$` means for this address kind.
fn last_lnum(eap: Ea, addr_type: CmdAddr) -> Addr {
    Addr::At(match addr_type {
        CmdAddr::Lines | CmdAddr::Other => cur_buf().b_ml.ml_line_count,
        CmdAddr::Windows => current_win_nr(ptr::null()) as linenr_T,
        CmdAddr::Arguments => unsafe { arglist_len() as linenr_T },
        CmdAddr::LoadedBuffers => loaded_buffer_range().1,
        CmdAddr::Buffers => tail().handle as linenr_T,
        CmdAddr::Tabs => current_tab_nr(ptr::null_mut()) as linenr_T,
        // An empty quickfix list still has a last entry, numbered 1.
        CmdAddr::Quickfix => (unsafe { qf_get_size(eap.raw()) } as linenr_T).max(1),
        CmdAddr::QuickfixValid => (qf_get_valid_size(eap.raw()) as linenr_T).max(1),
        t if t == CmdAddr::NoRange || t == CmdAddr::TabsRelative || t == CmdAddr::Unsigned => {
            return Addr::Refused;
        }
        _ => return Addr::Unchanged,
    })
}

/// What a bare `+N`/`-N` counts from. Unlike `.`, the three cursor-less
/// kinds answer a number here rather than an error.
fn offset_base(eap: Ea, addr_type: CmdAddr) -> Addr {
    match addr_type {
        CmdAddr::TabsRelative => Addr::At(1),
        CmdAddr::NoRange | CmdAddr::Unsigned => Addr::At(0),
        _ => dot_lnum(eap, addr_type),
    }
}

/// Is the range this command was given out of bounds? Answers the message
/// to report, or null.
pub(crate) unsafe fn invalid_range(eap: *mut exarg_T) -> Option<CString> {
    let ea = unsafe { Ea::new(eap) };
    let invrange = || Some(ex_msg(e_invrange.as_ptr()));
    if ea.line1 < 0 || ea.line2 < 0 || ea.line1 > ea.line2 {
        return invrange();
    }
    if !ea.argt.has(ExArgt::RANGE) {
        return None;
    }
    match ea.addr_type {
        CmdAddr::Lines => {
            // `:diffget`/`:diffput` accept one line past the end: they
            // may add a line there.
            let extra = (ea.cmdidx as c_int == CMD_diffget as c_int
                || ea.cmdidx as c_int == CMD_diffput as c_int) as c_int;
            if ea.line2 > cur_buf().b_ml.ml_line_count + extra {
                return invrange();
            }
        }
        CmdAddr::Arguments => {
            // An empty argument list still accepts line 1, which is
            // what makes `:argdelete` on it report a better message.
            let len = unsafe { arglist_len() };
            if ea.line2 > len as linenr_T + (len == 0) as c_int {
                return invrange();
            }
        }
        CmdAddr::Buffers => {
            if ea.line1 < 1 || ea.line2 > get_highest_fnum() as linenr_T {
                return invrange();
            }
        }
        CmdAddr::LoadedBuffers => {
            let mut buf = head();
            while buf.b_ml.ml_mfp.is_null() {
                let Some(next) = buf.next() else {
                    return invrange();
                };
                buf = next;
            }
            if ea.line1 < buf.handle as linenr_T {
                return invrange();
            }
            let mut buf = tail();
            while buf.b_ml.ml_mfp.is_null() {
                let Some(prev) = buf.prev() else {
                    return invrange();
                };
                buf = prev;
            }
            if ea.line2 > buf.handle as linenr_T {
                return invrange();
            }
        }
        CmdAddr::Windows => {
            if ea.line2 > current_win_nr(ptr::null()) as linenr_T {
                return invrange();
            }
        }
        CmdAddr::Tabs => {
            if ea.line2 > current_tab_nr(ptr::null_mut()) as linenr_T {
                return invrange();
            }
        }
        CmdAddr::Quickfix => {
            debug_assert!(ea.line2 >= 0);
            if ea.line2 <= 0 {
                // "no errors" reads better than "invalid range" when
                // the user did not ask for a particular entry.
                if ea.addr_count == 0 {
                    return Some(ex_msg(e_no_errors.as_ptr()));
                }
                return invrange();
            }
        }
        CmdAddr::QuickfixValid
            if ((ea.line2 != 1 && ea.line2 as size_t > qf_get_valid_size(eap)) || ea.line2 < 0) =>
        {
            return invrange();
        }
        _ => {}
    }
    None
}

/// Turn line 0 into line 1 for a command that does not accept a zero
/// address. `:0read` does, `:0print` does not.
pub(crate) fn correct_range(mut ea: Ea) {
    if ea.argt.has(ExArgt::ZEROR) {
        return;
    }
    if ea.line1 == 0 {
        ea.line1 = 1;
    }
    if ea.line2 == 0 {
        ea.line2 = 1;
    }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// `ascii_isdigit()` as checked code.
fn ascii_isdigit(c: c_int) -> bool {
    // SAFETY: reads the editor's own state, which exists from startup to exit.
    crate::ascii::ascii_isdigit(c)
}

/// `check_cursor()` as checked code.
fn check_cursor(wp: Win) {
    // SAFETY: reads the editor's own state, which exists from startup to exit.
    crate::cursor::check_cursor(wp)
}

/// `ex_msg()` as checked code.
fn ex_msg(msg: *const c_char) -> CString {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::ex_docmd::ex_msg(msg) }
}

/// `mark_get_visual()` as checked code.
fn mark_get_visual(buf: *mut buf_T, fmp: *mut fmark_T, name: c_int) -> *mut fmark_T {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::mark::mark_get_visual(buf, fmp, name) }
}

/// `qf_get_cur_idx()` as checked code.
fn qf_get_cur_idx(eap: *mut exarg_T) -> size_t {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::quickfix::qf_get_cur_idx(eap) }
}

/// `qf_get_cur_valid_idx()` as checked code.
fn qf_get_cur_valid_idx(eap: *mut exarg_T) -> c_int {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::quickfix::qf_get_cur_valid_idx(eap) }
}

/// `qf_get_valid_size()` as checked code.
fn qf_get_valid_size(eap: *mut exarg_T) -> size_t {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::quickfix::qf_get_valid_size(eap) }
}

/// `skipwhite()` as checked code.
fn skipwhite(p: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated string.
    unsafe { crate::charset::skipwhite(p) }
}
