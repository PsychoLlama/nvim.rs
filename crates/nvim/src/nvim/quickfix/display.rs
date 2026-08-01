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

#[allow(unused_imports)]
use super::*;
use core::ffi::{CStr, c_char, c_int};
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
    unsafe {
        let out = &mut *SCRATCH.ptr();
        out.clear();
        fill(out);
        out.push(0);
        slice::from_raw_parts(out.as_ptr(), out.len())
    }
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
    unsafe { out.extend_from_slice(CStr::from_ptr(text).to_bytes()) }
}

/// Print one entry of `:clist`, unless `:filter` rejects it.
///
/// # Safety
///
/// `qfp` must be a live entry.
unsafe fn qf_list_entry(qfp: *mut qfline_T, qf_idx: c_int, cursel: bool) {
    // SAFETY: forwarded from the caller.
    unsafe {
        // The heading: the entry number and where it points.
        let mut fname = ptr::null_mut::<c_char>();
        let module = (*qfp).qf_module;
        if !module.is_null() && *module != 0 {
            vim_snprintf(
                IObuff.ptr().cast(),
                IOSIZE as size_t,
                c"%2d %s".as_ptr(),
                qf_idx,
                module,
            );
        } else {
            let buf = if (*qfp).qf_fnum != 0 {
                buflist_findnr((*qfp).qf_fnum)
            } else {
                ptr::null_mut()
            };
            if !buf.is_null() {
                fname = if (*qfp).qf_fname.is_null() {
                    (*buf).b_fname
                } else {
                    (*qfp).qf_fname
                };
                if (*qfp).qf_type as c_int == 1 {
                    // :helpgrep entries name the help file only.
                    fname = path_tail(fname);
                }
            }
            if fname.is_null() {
                snprintf(
                    IObuff.ptr().cast(),
                    IOSIZE as size_t,
                    c"%2d".as_ptr(),
                    qf_idx,
                );
            } else {
                vim_snprintf(
                    IObuff.ptr().cast(),
                    IOSIZE as size_t,
                    c"%2d %s".as_ptr(),
                    qf_idx,
                    fname,
                );
            }
        }

        // `:filter /pat/ clist` matches the module name, the file name, the
        // search pattern and the text; the entry is dropped only when every
        // one of them is filtered out.
        let mut filtered = true;
        if !module.is_null() && *module != 0 {
            filtered = message_filtered(module);
        }
        if filtered && !fname.is_null() {
            filtered = message_filtered(fname);
        }
        if filtered && !(*qfp).qf_pattern.is_null() {
            filtered = message_filtered((*qfp).qf_pattern);
        }
        if filtered {
            filtered = message_filtered((*qfp).qf_text);
        }
        if filtered {
            return;
        }

        if msg_col.get() > 0 {
            msg_putchar('\n' as c_int);
        }
        msg_outtrans(
            IObuff.ptr().cast(),
            if cursel {
                HLF_QFL as c_int
            } else {
                qfFile_hl_id.get()
            },
            false,
        );

        // The position: "<lnum>[-<end>][ col <col>[-<end>]][ <type> <nr>]".
        if (*qfp).qf_lnum != 0 {
            msg_puts_hl(c":".as_ptr(), qfSep_hl_id.get(), false);
        }
        let position = build_line(|out| {
            if (*qfp).qf_lnum != 0 {
                qf_range_text(out, qfp);
            }
            push_cstr(out, qf_types((*qfp).qf_type as c_int, (*qfp).qf_nr));
        });
        if position[0] != 0 {
            msg_puts_hl(position.as_ptr().cast(), qfLine_hl_id.get(), false);
        }
        msg_puts_hl(c":".as_ptr(), qfSep_hl_id.get(), false);

        if !(*qfp).qf_pattern.is_null() {
            let pattern = build_line(|out| qf_fmt_text(out, (*qfp).qf_pattern));
            msg_puts(pattern.as_ptr().cast());
            msg_puts_hl(c":".as_ptr(), qfSep_hl_id.get(), false);
        }
        msg_puts(c" ".as_ptr());

        // The message itself. An unrecognized line keeps its indent, since
        // the compiler may be marking a word with "^^^^".
        let text = if !fname.is_null() || (*qfp).qf_lnum != 0 {
            skipwhite((*qfp).qf_text)
        } else {
            (*qfp).qf_text
        };
        msg_prt_line(
            build_line(|out| qf_fmt_text(out, text)).as_ptr().cast(),
            false,
        );
    }
}

/// `:clist`/`:llist`: print the entries of the current list.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn qf_list(eap: *mut exarg_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qi = qf_cmd_get_stack(eap, true);
        if qi.is_null() {
            return;
        }
        if qf_stack_empty(qi) || qf_list_empty(qf_get_curlist(qi)) {
            emsg(gettext(&raw const e_no_errors as *const c_char));
            return;
        }

        // "+N" lists N entries from the current one; otherwise the argument
        // is a range, counted from the end when negative.
        let mut arg = (*eap).arg;
        let plus = *arg == b'+' as c_char;
        if plus {
            arg = arg.add(1);
        }
        let mut idx1: c_int = 1;
        let mut idx2: c_int = -1;
        if get_list_range(&raw mut arg, &raw mut idx1, &raw mut idx2) == 0 || *arg != 0 {
            semsg(gettext(&raw const e_trailing_arg as *const c_char), arg);
            return;
        }
        let qfl = qf_get_curlist(qi);
        if plus {
            idx2 = (*qfl).qf_index + idx1;
            idx1 = (*qfl).qf_index;
        } else {
            let count = (*qfl).qf_count;
            if idx1 < 0 {
                idx1 = if -idx1 > count { 0 } else { idx1 + count + 1 };
            }
            if idx2 < 0 {
                idx2 = if -idx2 > count { 0 } else { idx2 + count + 1 };
            }
        }

        // Shorten all the file names, so that it is easy to read.
        shorten_fnames(false as c_int);

        // The highlighting comes from the qf.vim syntax file.
        qfFile_hl_id.set(syn_name2id(c"qfFileName".as_ptr()));
        if qfFile_hl_id.get() == 0 {
            qfFile_hl_id.set(HLF_D as c_int);
        }
        qfSep_hl_id.set(syn_name2id(c"qfSeparator".as_ptr()));
        if qfSep_hl_id.get() == 0 {
            qfSep_hl_id.set(HLF_D as c_int);
        }
        qfLine_hl_id.set(syn_name2id(c"qfLineNr".as_ptr()));
        if qfLine_hl_id.get() == 0 {
            qfLine_hl_id.set(HLF_N as c_int);
        }

        // Without "!" only recognised entries are listed — unless none of
        // them is recognised, when they all are.
        let all = (*eap).forceit != 0 || (*qfl).qf_nonevalid;
        msg_ext_set_kind(c"list_cmd".as_ptr());
        let mut i: c_int = 1;
        let mut qfp = (*qfl).qf_start;
        while !got_int.get() && i <= (*qfl).qf_count && !qfp.is_null() {
            if ((*qfp).qf_valid != 0 || all) && idx1 <= i && i <= idx2 {
                qf_list_entry(qfp, i, i == (*qfl).qf_index);
            }
            os_breakcheck();
            i += 1;
            qfp = (*qfp).qf_next;
        }
        release_scratch();
    }
}

/// Append an error message with its newlines, and the whitespace following
/// them, squeezed into single spaces.
///
/// # Safety
///
/// `text` must be a live NUL-terminated string.
pub(crate) unsafe fn qf_fmt_text(out: &mut Vec<u8>, text: *const c_char) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut p = text.cast::<u8>();
        while *p != 0 {
            if *p == b'\n' {
                out.push(b' ');
                loop {
                    p = p.add(1);
                    if *p == 0 || !ascii_iswhite(*p as c_int) && *p != b'\n' {
                        break;
                    }
                }
            } else {
                out.push(*p);
                p = p.add(1);
            }
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
    // SAFETY: forwarded from the caller. Each `vim_snprintf_safelen`
    // answers what it wrote, so the next one appends where it stopped.
    unsafe {
        let buf = IObuff.ptr().cast::<c_char>();
        let mut len = vim_snprintf_safelen(buf, IOSIZE as size_t, c"%d".as_ptr(), (*qfp).qf_lnum);
        if (*qfp).qf_end_lnum > 0 && (*qfp).qf_lnum != (*qfp).qf_end_lnum {
            len += vim_snprintf_safelen(
                buf.add(len),
                IOSIZE as size_t - len,
                c"-%d".as_ptr(),
                (*qfp).qf_end_lnum,
            );
        }
        if (*qfp).qf_col > 0 {
            len += vim_snprintf_safelen(
                buf.add(len),
                IOSIZE as size_t - len,
                c" col %d".as_ptr(),
                (*qfp).qf_col,
            );
            if (*qfp).qf_end_col > 0 && (*qfp).qf_col != (*qfp).qf_end_col {
                len += vim_snprintf_safelen(
                    buf.add(len),
                    IOSIZE as size_t - len,
                    c"-%d".as_ptr(),
                    (*qfp).qf_end_col,
                );
            }
        }
        out.extend_from_slice(slice::from_raw_parts(buf.cast::<u8>(), len));
    }
}

/// Print the number, size and title of one list in the stack.
///
/// # Safety
///
/// `qi` must be a live stack holding a list at `which`, and `lead` a live
/// string.
unsafe fn qf_msg(qi: *mut qf_info_T, which: c_int, lead: *const c_char) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qfl = qf_get_list(qi, which);
        let mut buf: [c_char; IOSIZE as usize] = [0; IOSIZE as usize];
        let len = vim_snprintf_safelen(
            buf.as_mut_ptr(),
            IOSIZE as size_t,
            gettext(c"%serror list %d of %d; %d errors ".as_ptr()),
            lead,
            which + 1,
            (*qi).qf_listcount,
            (*qfl).qf_count,
        );
        if !(*qfl).qf_title.is_null() {
            // The title starts at a fixed column, when there is room.
            if len < 34 {
                buf[len..34].fill(b' ' as c_char);
                buf[34] = 0;
            }
            xstrlcat(buf.as_mut_ptr(), (*qfl).qf_title, IOSIZE as size_t);
        }
        trunc_string(
            buf.as_mut_ptr(),
            buf.as_mut_ptr(),
            Columns.get() - 1,
            IOSIZE,
        );
        msg(buf.as_ptr(), 0);
    }
}

/// `:colder`/`:cnewer`/`:lolder`/`:lnewer`: move up or down the stack.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn qf_age(eap: *mut exarg_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qi = qf_cmd_get_stack(eap, true);
        if qi.is_null() {
            return;
        }
        let count = if (*eap).addr_count != 0 {
            (*eap).line2 as c_int
        } else {
            1
        };
        let older = (*eap).cmdidx == CMD_colder || (*eap).cmdidx == CMD_lolder;
        for _ in 0..count {
            if older {
                if (*qi).qf_curlist == 0 {
                    emsg(gettext(c"E380: At bottom of quickfix stack".as_ptr()));
                    break;
                }
                (*qi).qf_curlist -= 1;
            } else {
                if (*qi).qf_curlist >= (*qi).qf_listcount - 1 {
                    emsg(gettext(c"E381: At top of quickfix stack".as_ptr()));
                    break;
                }
                (*qi).qf_curlist += 1;
            }
        }
        qf_msg(qi, (*qi).qf_curlist, c"".as_ptr());
        qf_update_buffer(qi, ptr::null_mut());
    }
}

/// `:chistory`/`:lhistory`: print every list in the stack, or with a count,
/// go to one of them.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn qf_history(eap: *mut exarg_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qi = qf_cmd_get_stack(eap, false);
        if (*eap).addr_count > 0 {
            if qi.is_null() {
                emsg(gettext(&raw const e_loclist as *const c_char));
            } else if (*eap).line2 > 0 && (*eap).line2 <= (*qi).qf_listcount as linenr_T {
                (*qi).qf_curlist = ((*eap).line2 - 1) as c_int;
                qf_msg(qi, (*qi).qf_curlist, c"".as_ptr());
                qf_update_buffer(qi, ptr::null_mut());
            } else {
                emsg(gettext(&raw const e_invrange as *const c_char));
            }
            return;
        }

        if qf_stack_empty(qi) {
            msg(gettext(c"No entries".as_ptr()), 0);
        } else {
            for i in 0..(*qi).qf_listcount {
                let lead = if i == (*qi).qf_curlist {
                    c"> ".as_ptr()
                } else {
                    c"  ".as_ptr()
                };
                qf_msg(qi, i, lead);
            }
        }
    }
}

/// The type of an entry as it is printed: `" error"`, `" warning"`, … plus
/// the error number when there is one.
///
/// The answer points at one of two static buffers, which the next call
/// overwrites.
pub(crate) fn qf_types(c: c_int, nr: c_int) -> *const c_char {
    /// Room for an unrecognized type: a space, the letter and a NUL.
    static OTHER: GlobalCell<[c_char; 3]> = GlobalCell::new([0; 3]);
    /// Room for the longest type plus " %3d".
    static NUMBERED: GlobalCell<[c_char; 20]> = GlobalCell::new([0; 20]);

    // SAFETY: both buffers are only ever read back through the answer, and
    // `snprintf` truncates to the size it is given.
    unsafe {
        let name = if c == 'W' as c_int || c == 'w' as c_int {
            c" warning".as_ptr()
        } else if c == 'I' as c_int || c == 'i' as c_int {
            c" info".as_ptr()
        } else if c == 'N' as c_int || c == 'n' as c_int {
            c" note".as_ptr()
        } else if c == 'E' as c_int || c == 'e' as c_int || c == 0 && nr > 0 {
            c" error".as_ptr()
        } else if c == 0 || c == 1 {
            c"".as_ptr()
        } else {
            (*OTHER.ptr()) = [' ' as c_char, c as c_char, 0];
            OTHER.ptr().cast::<c_char>()
        };
        if nr <= 0 {
            return name;
        }
        snprintf(
            NUMBERED.ptr().cast(),
            size_of::<[c_char; 20]>(),
            c"%s %3d".as_ptr(),
            name,
            nr,
        );
        NUMBERED.ptr().cast::<c_char>()
    }
}

/// Open the entry under the cursor in the quickfix window, in a new window
/// when `split`.
///
/// # Safety
///
/// Must be called from a quickfix or location list window.
pub unsafe fn qf_view_result(split: bool) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut qi = ql_info.get();
        debug_assert!(!qi.is_null());
        let in_ll_window = is_ll_window(curwin.get());
        if in_ll_window {
            qi = (*curwin.get()).w_llist_ref;
        }
        if qf_list_empty(qf_get_curlist(qi)) {
            emsg(gettext(&raw const e_no_errors as *const c_char));
            return;
        }
        if split {
            qf_jump_newwin(qi, 0, (*curwin.get()).w_cursor.lnum as c_int, 0, true);
            do_cmdline_cmd(c"clearjumps".as_ptr());
            return;
        }
        do_cmdline_cmd(if in_ll_window {
            c".ll".as_ptr()
        } else {
            c".cc".as_ptr()
        });
    }
}
