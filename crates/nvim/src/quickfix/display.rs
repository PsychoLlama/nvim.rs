//! Printing a list to the message area.
//!
//! [`qf_list`] is `:clist`, one line per entry via [`qf_list_entry`];
//! [`qf_age`] is `:colder`/`:cnewer`, [`qf_history`] is `:chistory` and
//! [`qf_view_result`] is what `CTRL-W_<Enter>` does in the quickfix window.
//!
//! The text of one line is built by [`build_line`] in a buffer shared with
//! the quickfix window ([`qf_buf_add_line`]) and the jump message
//! (`qf_jump_print_msg`) so that listing a long list does not allocate per
//! entry.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::highlight_group::{HLF_D, HLF_N, HLF_QFL};
use crate::semsg_c;
use crate::types::{CMD_colder, CMD_lolder, IOSIZE};
use core::ffi::{CStr, c_char, c_int};
use std::ffi::CString;

use crate::cstr;
use core::{ptr, slice};

/// The shared line buffer. See [`build_line`].
static SCRATCH: GlobalCell<Vec<u8>> = GlobalCell::new(Vec::new());

/// Build one line of text in the shared buffer and answer it, NUL included,
/// as its last byte.
///
/// The answer stays good until the next line is built, which is what `fill`
/// must not cause: it may not print, run an autocommand or evaluate an
/// expression, because those build lines of their own through this same
/// buffer.
pub(crate) fn build_line(fill: impl FnOnce(&mut Vec<u8>)) -> &'static [u8] {
    // SAFETY: `fill` cannot reach the buffer (see above), so this is the
    // only live borrow. The answer outlives it because the buffer is a
    // static, and stays valid until the next line replaces it.
    let out = unsafe { &mut *SCRATCH.ptr() };
    out.clear();
    fill(out);
    out.push(0);
    unsafe { slice::from_raw_parts(out.as_ptr(), out.len()) }
}

/// Give back the memory of a buffer that grew large; a modest one is kept
/// for the next command.
pub(crate) fn release_scratch() {
    SCRATCH.with_mut(|out| {
        if out.capacity() > 1000 {
            *out = Vec::new();
        } else {
            out.clear();
        }
    });
}

/// Append a C string.
///
/// # Safety
///
/// `text` must be a live NUL-terminated string.
#[inline]
pub(crate) unsafe fn push_cstr(out: &mut Vec<u8>, text: *const c_char) {
    // SAFETY: forwarded from the caller.
    out.extend_from_slice(unsafe { CStr::from_ptr(text) }.to_bytes())
}

/// Print one entry of `:clist`, unless `:filter` rejects it.
///
/// # Safety
///
/// `qfp` must be a live entry.
unsafe fn qf_list_entry(qfp: *mut qfline_T, qf_idx: c_int, cursel: bool) {
    // SAFETY: the caller's promise -- a live `qfline_T`.
    let qfp = unsafe { Qfe::new(qfp) };
    // The heading. Upstream assembles it in `IObuff` and then calls
    // `message_filtered` and `msg_outtrans`, both of which re-enter the
    // message machinery.
    let mut heading = [0 as c_char; IOSIZE as usize];
    let mut fname = ptr::null_mut::<c_char>();
    // SAFETY: forwarded from the caller.
    let module = qfp.qf_module;
    if !module.is_null() && unsafe { *module } != 0 {
        let heading = heading.as_mut_ptr();
        let size = IOSIZE as size_t;
        let fmt = c"%2d %s".as_ptr();
        unsafe { vim_snprintf(heading, size, fmt, qf_idx, module) };
    } else {
        let buf = if qfp.qf_fnum != 0 {
            find_buf(qfp.qf_fnum).map_or(ptr::null_mut(), |mut b| b.raw())
        } else {
            ptr::null_mut()
        };
        if !buf.is_null() {
            fname = if qfp.qf_fname.is_null() {
                unsafe { (*buf).b_fname }
            } else {
                qfp.qf_fname
            };
            if qfp.qf_type as c_int == 1 {
                // :helpgrep entries name the help file only.
                fname = unsafe { path_tail(fname) };
            }
        }
        if fname.is_null() {
            let heading = heading.as_mut_ptr();
            let size = IOSIZE as size_t;
            let fmt = c"%2d".as_ptr();
            unsafe { snprintf(heading, size, fmt, qf_idx) };
        } else {
            let heading = heading.as_mut_ptr();
            let size = IOSIZE as size_t;
            let fmt = c"%2d %s".as_ptr();
            unsafe { vim_snprintf(heading, size, fmt, qf_idx, fname) };
        }
    }

    // `:filter /pat/ clist` matches the module name, the file name, the
    // search pattern and the text; the entry is dropped only when every
    // one of them is filtered out.
    let mut filtered = true;
    if !module.is_null() && unsafe { *module } != 0 {
        filtered = unsafe { message_filtered(module) };
    }
    if filtered && !fname.is_null() {
        filtered = unsafe { message_filtered(fname) };
    }
    if filtered && !qfp.qf_pattern.is_null() {
        filtered = unsafe { message_filtered(qfp.qf_pattern) };
    }
    if filtered {
        filtered = unsafe { message_filtered(qfp.qf_text) };
    }
    if filtered {
        return;
    }

    if msg_col.get() > 0 {
        unsafe { msg_putchar('\n' as c_int) };
    }
    let cursel = if cursel { HLF_QFL } else { qfFile_hl_id.get() };
    unsafe { msg_outtrans(heading.as_mut_ptr(), cursel, false) };

    // The position: "<lnum>[-<end>][ col <col>[-<end>]][ <type> <nr>]".
    if qfp.qf_lnum != 0 {
        unsafe { msg_puts_hl(c":".as_ptr(), qfSep_hl_id.get(), false) };
    }
    let position = build_line(|out| {
        if qfp.qf_lnum != 0 {
            unsafe { qf_range_text(out, qfp.raw().cast_const()) };
        }
        let types = qf_types(qfp.qf_type as c_int, qfp.qf_nr);
        unsafe { push_cstr(out, types.as_ptr()) };
    });
    if position[0] != 0 {
        unsafe { msg_puts_hl(position.as_ptr().cast(), qfLine_hl_id.get(), false) };
    }
    unsafe { msg_puts_hl(c":".as_ptr(), qfSep_hl_id.get(), false) };

    if !qfp.qf_pattern.is_null() {
        let pattern = build_line(|out| unsafe { qf_fmt_text(out, qfp.qf_pattern) });
        unsafe { msg_puts(pattern.as_ptr().cast()) };
        unsafe { msg_puts_hl(c":".as_ptr(), qfSep_hl_id.get(), false) };
    }
    unsafe { msg_puts(c" ".as_ptr()) };

    // The message itself. An unrecognized line keeps its indent, since
    // the compiler may be marking a word with "^^^^".
    let text = if !fname.is_null() || qfp.qf_lnum != 0 {
        unsafe { skipwhite(qfp.qf_text) }
    } else {
        qfp.qf_text
    };
    let line = build_line(|out| unsafe { qf_fmt_text(out, text) })
        .as_ptr()
        .cast();
    unsafe { msg_prt_line(line, false) };
}

/// `:clist`/`:llist`: print the entries of the current list.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn qf_list(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live `exarg_T`.
    let eap = unsafe { Ea::new(eap) };
    let Some(mut qi) = qf_cmd_stack(eap, true) else {
        return;
    };
    if qf_is_empty(qi) || qfl_is_empty(qf_current_list(qi)) {
        qf_emsg(e_no_errors.as_ptr());
        return;
    }

    // "+N" lists N entries from the current one; otherwise the argument
    // is a range, counted from the end when negative.
    let mut arg = eap.arg;
    let plus = unsafe { *arg } == b'+' as c_char;
    if plus {
        arg = unsafe { arg.add(1) };
    }
    let mut idx1: c_int = 1;
    let mut idx2: c_int = -1;
    if unsafe { get_list_range(&raw mut arg, &raw mut idx1, &raw mut idx2) } == 0
        || unsafe { *arg } != 0
    {
        // SAFETY: the message macros expand to a `vim_snprintf` over the
        // format literal above and the editor's message buffers.
        unsafe { semsg_c!(gettext(e_trailing_arg), arg) };
        return;
    }
    let qfl = qf_current_list(qi);
    if plus {
        idx2 = qfl.qf_index + idx1;
        idx1 = qfl.qf_index;
    } else {
        let count = qfl.qf_count;
        if idx1 < 0 {
            idx1 = if -idx1 > count { 0 } else { idx1 + count + 1 };
        }
        if idx2 < 0 {
            idx2 = if -idx2 > count { 0 } else { idx2 + count + 1 };
        }
    }

    // Shorten all the file names, so that it is easy to read.
    unsafe { shorten_fnames(false as c_int) };

    // The highlighting comes from the qf.vim syntax file.
    qfFile_hl_id.set(unsafe { syn_name2id(c"qfFileName".as_ptr()) });
    if qfFile_hl_id.get() == 0 {
        qfFile_hl_id.set(HLF_D);
    }
    qfSep_hl_id.set(unsafe { syn_name2id(c"qfSeparator".as_ptr()) });
    if qfSep_hl_id.get() == 0 {
        qfSep_hl_id.set(HLF_D);
    }
    qfLine_hl_id.set(unsafe { syn_name2id(c"qfLineNr".as_ptr()) });
    if qfLine_hl_id.get() == 0 {
        qfLine_hl_id.set(HLF_N);
    }

    // Without "!" only recognised entries are listed — unless none of
    // them is recognised, when they all are.
    let all = eap.forceit != 0 || qfl.qf_nonevalid;
    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    let mut i: c_int = 1;
    let mut qfp = qfl.qf_start;
    while !got_int.get() && i <= qfl.qf_count && !qfp.is_null() {
        if (unsafe { (*qfp).qf_valid } != 0 || all) && idx1 <= i && i <= idx2 {
            unsafe { qf_list_entry(qfp, i, i == qfl.qf_index) };
        }
        os_breakcheck();
        i += 1;
        qfp = unsafe { (*qfp).qf_next };
    }
    release_scratch();
}

/// Append an error message with its newlines, and the whitespace following
/// them, squeezed into single spaces.
///
/// # Safety
///
/// `text` must be a live NUL-terminated string.
pub(crate) unsafe fn qf_fmt_text(out: &mut Vec<u8>, text: *const c_char) {
    // SAFETY: forwarded from the caller.
    let mut p = text.cast::<u8>();
    while unsafe { *p } != 0 {
        if unsafe { *p } == b'\n' {
            out.push(b' ');
            loop {
                p = unsafe { p.add(1) };
                if unsafe { *p } == 0
                    || !ascii_iswhite(unsafe { *p } as c_int) && unsafe { *p } != b'\n'
                {
                    break;
                }
            }
        } else {
            out.push(unsafe { *p });
            p = unsafe { p.add(1) };
        }
    }
}

/// Append an entry's position: the line, the end line, and the columns when
/// the entry has them.
///
/// # Safety
///
/// `qfp` must be a live entry.
pub(crate) unsafe fn qf_range_text(out: &mut Vec<u8>, qfp: *const qfline_T) {
    // SAFETY: the caller's promise -- a live `qfline_T`.
    let qfp = unsafe { Qfe::new(qfp.cast_mut()) };
    let mut range = [0 as c_char; IOSIZE as usize];
    // SAFETY: forwarded from the caller. Each `vim_snprintf_safelen`
    // answers what it wrote, so the next one appends where it stopped.
    let buf = range.as_mut_ptr();
    let size = IOSIZE as size_t;
    let fmt = c"%d".as_ptr();
    let lnum = qfp.qf_lnum;
    let mut len = unsafe { vim_snprintf_safelen(buf, size, fmt, lnum) };
    if qfp.qf_end_lnum > 0 && qfp.qf_lnum != qfp.qf_end_lnum {
        let at = unsafe { buf.add(len) };
        let room = IOSIZE as size_t - len;
        let fmt = c"-%d".as_ptr();
        let end_lnum = qfp.qf_end_lnum;
        len += unsafe { vim_snprintf_safelen(at, room, fmt, end_lnum) };
    }
    if qfp.qf_col > 0 {
        let at = unsafe { buf.add(len) };
        let room = IOSIZE as size_t - len;
        let fmt = c" col %d".as_ptr();
        let col = qfp.qf_col;
        len += unsafe { vim_snprintf_safelen(at, room, fmt, col) };
        if qfp.qf_end_col > 0 && qfp.qf_col != qfp.qf_end_col {
            let at = unsafe { buf.add(len) };
            let room = IOSIZE as size_t - len;
            let fmt = c"-%d".as_ptr();
            let end_col = qfp.qf_end_col;
            len += unsafe { vim_snprintf_safelen(at, room, fmt, end_col) };
        }
    }
    out.extend_from_slice(unsafe { slice::from_raw_parts(buf.cast::<u8>(), len) });
}

/// Print the number, size and title of one list in the stack.
///
/// # Safety
///
/// `qi` must be a live stack holding a list at `which`, and `lead` a live
/// string.
unsafe fn qf_msg(qi: *mut qf_info_T, which: c_int, lead: *const c_char) {
    // SAFETY: the caller's promise -- a live `qf_info_T`.
    let mut qi = unsafe { Qi::new(qi) };
    // SAFETY: forwarded from the caller.
    let qfl = qf_nth_list(qi, which);
    let mut buf: [c_char; IOSIZE as usize] = [0; IOSIZE as usize];
    let size = IOSIZE as size_t;
    let fmt = gettext(c"%serror list %d of %d; %d errors ");
    let listcount = qi.qf_listcount;
    let count = qfl.qf_count;
    let len = unsafe {
        vim_snprintf_safelen(
            buf.as_mut_ptr(),
            size,
            fmt.as_ptr(),
            lead,
            which + 1,
            listcount,
            count,
        )
    };
    if !qfl.qf_title.is_null() {
        // The title starts at a fixed column, when there is room.
        if len < 34 {
            buf[len..34].fill(b' ' as c_char);
            buf[34] = 0;
        }
        unsafe { xstrlcat(buf.as_mut_ptr(), qfl.qf_title, IOSIZE as size_t) };
    }
    let title = buf.as_mut_ptr();
    let room = Columns.get() - 1;
    unsafe { trunc_string(title, buf.as_mut_ptr(), room, IOSIZE) };
    msg(cstr::in_chars(&buf), 0);
}

/// `:colder`/`:cnewer`/`:lolder`/`:lnewer`: move up or down the stack.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn qf_age(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live `exarg_T`.
    let eap = unsafe { Ea::new(eap) };
    let Some(mut qi) = qf_cmd_stack(eap, true) else {
        return;
    };
    let count = if eap.addr_count != 0 {
        eap.line2 as c_int
    } else {
        1
    };
    let older = eap.cmdidx == CMD_colder || eap.cmdidx == CMD_lolder;
    for _ in 0..count {
        if older {
            if qi.qf_curlist == 0 {
                qf_emsg(c"E380: At bottom of quickfix stack".as_ptr());
                break;
            }
            qi.qf_curlist -= 1;
        } else {
            if qi.qf_curlist >= qi.qf_listcount - 1 {
                qf_emsg(c"E381: At top of quickfix stack".as_ptr());
                break;
            }
            qi.qf_curlist += 1;
        }
    }
    unsafe { qf_msg(qi.raw(), qi.qf_curlist, c"".as_ptr()) };
    qf_redraw(qi, ptr::null_mut());
}

/// `:chistory`/`:lhistory`: print every list in the stack, or with a count,
/// go to one of them.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn qf_history(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live `exarg_T`.
    let eap = unsafe { Ea::new(eap) };
    // SAFETY: forwarded from the caller.
    let stack = qf_cmd_stack(eap, false);
    if eap.addr_count > 0 {
        match stack {
            None => qf_emsg(e_loclist.as_ptr()),
            Some(mut qi) if eap.line2 > 0 && eap.line2 <= qi.qf_listcount as linenr_T => {
                qi.qf_curlist = (eap.line2 - 1) as c_int;
                // SAFETY: `qi` is live and `qf_curlist` names one of its
                // lists, which is the whole of `qf_msg`'s precondition.
                unsafe { qf_msg(qi.raw(), qi.qf_curlist, c"".as_ptr()) };
                qf_redraw(qi, ptr::null_mut());
            }
            Some(_) => qf_emsg(e_invrange.as_ptr()),
        }
        return;
    }
    // No location list at all counts as an empty stack, which is what
    // `qf_stack_empty` answered for the null pointer this used to hold.
    match stack.filter(|qi| !qf_is_empty(*qi)) {
        None => {
            msg(gettext(c"No entries"), 0);
        }
        Some(qi) => {
            for i in 0..qi.qf_listcount {
                let lead = if i == qi.qf_curlist {
                    c"> ".as_ptr()
                } else {
                    c"  ".as_ptr()
                };
                // SAFETY: a live stack, and `i` is one of its lists.
                unsafe { qf_msg(qi.raw(), i, lead) };
            }
        }
    }
}

/// The type of an entry as it is printed: `" error"`, `" warning"`, … plus
/// the error number when there is one.
///
/// Upstream answers one of two static buffers the next call overwrites; this
/// answers a string the caller owns.
pub(crate) fn qf_types(c: c_int, nr: c_int) -> CString {
    const W: c_int = b'W' as c_int;
    const LOWER_W: c_int = b'w' as c_int;
    const I: c_int = b'I' as c_int;
    const LOWER_I: c_int = b'i' as c_int;
    const N: c_int = b'N' as c_int;
    const LOWER_N: c_int = b'n' as c_int;
    const E: c_int = b'E' as c_int;
    const LOWER_E: c_int = b'e' as c_int;
    let name: &[u8] = match c {
        W | LOWER_W => b" warning",
        I | LOWER_I => b" info",
        N | LOWER_N => b" note",
        E | LOWER_E => b" error",
        0 if nr > 0 => b" error",
        0 | 1 => b"",
        other => return numbered(&[b' ', other as u8], nr),
    };
    numbered(name, nr)
}

/// `qf_types`' tail: `name`, with ` %3d` of `nr` after it when there is one.
fn numbered(name: &[u8], nr: c_int) -> CString {
    if nr <= 0 {
        return cstr::owned(name);
    }
    let mut text = name.to_vec();
    text.extend_from_slice(format!(" {nr:3}").as_bytes());
    cstr::owned(&text)
}

/// Open the entry under the cursor in the quickfix window, in a new window
/// when `split`.
///
/// # Safety
///
/// Must be called from a quickfix or location list window.
pub unsafe fn qf_view_result(split: bool) {
    let in_ll_window = is_ll_window(cur_win());
    // SAFETY: a location list window always references a live stack, which
    // is what `is_ll_window` just established.
    let qi = if in_ll_window {
        unsafe { Qi::new(cur_win().w_llist_ref) }
    } else {
        qf_global()
    };
    if qfl_is_empty(qf_current_list(qi)) {
        qf_emsg(e_no_errors.as_ptr());
        return;
    }
    if split {
        unsafe { qf_jump_newwin(qi.raw(), 0, cur_win().w_cursor.lnum as c_int, 0, true) };
        unsafe { do_cmdline_cmd(c"clearjumps".as_ptr()) };
        return;
    }
    unsafe {
        do_cmdline_cmd(if in_ll_window {
            c".ll".as_ptr()
        } else {
            c".cc".as_ptr()
        })
    };
}
