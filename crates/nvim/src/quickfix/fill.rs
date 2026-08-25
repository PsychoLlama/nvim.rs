//! Writing the list into the quickfix buffer.
//!
//! [`qf_update_buffer`] is what every command that changes a list calls: it
//! finds the buffer the quickfix window shows, has [`qf_fill_buffer`] write
//! one line per entry into it, and tells the editor what changed.
//!
//! The text of a line is [`qf_buf_add_line`]'s, unless `'quickfixtextfunc'`
//! is set — then [`call_qftf_func`] asks the user's function for the lines
//! first, and any entry it answers a string for uses that instead.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::eval::typval::NumBuf;
use crate::guard::Lock;
use crate::memline::MlFlags;
use crate::types::{
    FAIL, MAXPATHL, OptionSetFlags, VAR_DICT, VAR_FIXED, VAR_LIST, VAR_UNKNOWN, VAR_UNLOCKED,
};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// The directory file names are shortened against, resolved the first time
/// an entry needs it.
struct CurrentDir([c_char; MAXPATHL as usize]);

impl CurrentDir {
    fn new() -> CurrentDir {
        CurrentDir([0; MAXPATHL as usize])
    }

    /// The current directory. Empty if the system would not say, in which
    /// case the next entry asks again — as upstream does.
    unsafe fn get(&mut self) -> *mut c_char {
        if self.0[0] == 0 {
            // SAFETY: the buffer is exactly the length passed.
            unsafe { os_dirname(self.0.as_mut_ptr(), MAXPATHL as size_t) };
        }
        self.0.as_mut_ptr()
    }
}

/// Update the quickfix buffer, if one exists, after the list changed.
///
/// With `old_last` the entries after it are appended; otherwise the whole
/// buffer is rewritten.
///
/// # Safety
///
/// `qi` must be a live stack, `old_last` null or one of its entries.
pub(crate) unsafe fn qf_update_buffer(qi: *mut qf_info_T, old_last: *mut qfline_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let buf = qf_find_buf(qi);
        if buf.is_null() {
            return;
        }

        let old_line_count = (*buf).b_ml.ml_line_count;
        let old_endcol = ml_get_buf_len(buf, old_line_count);
        let old_bytecount = get_region_bytecount(buf, 1, old_line_count, 0, old_endcol);

        // A location list's window id goes to 'quickfixtextfunc'; it is the
        // window the list belongs to, not the one showing it.
        let mut qf_winid = 0;
        if (*qi).qfl_type == QFLT_LOCATION {
            let win = if (*curwin.get()).w_llist == qi {
                curwin.get()
            } else {
                // The file window, or failing that the location list window.
                let mut win = qf_find_win_with_loclist(qi);
                if win.is_null() {
                    win = qf_find_win(qi);
                }
                if win.is_null() {
                    return;
                }
                win
            };
            qf_winid = (*win).handle;
        }

        // Autocommands may cause trouble.
        incr_quickfix_busy();

        let mut aco = aco_save_T::default();
        if old_last.is_null() {
            // Set curwin/curbuf to buf and save a few things.
            aucmd_prepbuf(&raw mut aco, buf);
        }
        qf_update_win_titlevar(qi);
        qf_fill_buffer(qf_get_curlist(qi), buf, old_last, qf_winid);

        let new_line_count = (*buf).b_ml.ml_line_count;
        let new_endcol = ml_get_buf_len(buf, new_line_count);
        let delta = new_line_count - old_line_count;
        if old_last.is_null() {
            let new_byte_count = get_region_bytecount(buf, 1, new_line_count, 0, new_endcol);
            extmark_splice(
                buf,
                0,
                0,
                old_line_count - 1,
                0,
                old_bytecount,
                new_line_count - 1,
                new_endcol,
                new_byte_count,
                kExtmarkNoUndo,
            );
            changed_lines(
                buf,
                1,
                0,
                if old_line_count > 0 {
                    old_line_count + 1
                } else {
                    1
                },
                delta,
                true,
            );
        } else if delta > 0 {
            let start_lnum = old_line_count + 1;
            let new_byte_count =
                get_region_bytecount(buf, start_lnum, new_line_count, 0, new_endcol);
            extmark_splice(
                buf,
                old_line_count - 1,
                old_endcol,
                0,
                0,
                0,
                delta,
                new_endcol,
                new_byte_count,
                kExtmarkNoUndo,
            );
            changed_lines(buf, start_lnum, 0, start_lnum, delta, true);
        }
        (*buf).b_changed = false as c_int;

        if old_last.is_null() {
            qf_win_pos_update(qi, 0);
            // Restore curwin/curbuf and a few other things.
            aucmd_restbuf(&raw mut aco);
        }

        // Only redraw when the added lines are visible, to avoid flicker.
        let win = qf_find_win(qi);
        if !win.is_null() && old_line_count < (*win).w_botline {
            redraw_buf_later(buf, UPD_NOT_VALID);
        }

        // Always called after incr_quickfix_busy().
        decr_quickfix_busy();
    }
}

/// Append one entry to the quickfix buffer as a line of text, answering
/// whether it fit.
///
/// # Safety
///
/// `buf` must be the quickfix buffer and `qfp` a live entry.
unsafe fn qf_buf_add_line(
    buf: *mut buf_T,
    lnum: linenr_T,
    qfp: *const qfline_T,
    dir: &mut CurrentDir,
    qftf_str: *const c_char,
    first_bufline: bool,
) -> bool {
    // SAFETY: forwarded from the caller. Nothing in the line building
    // prints or runs an autocommand, which is `build_line`'s contract.
    unsafe {
        let line = build_line(|out| {
            // A non-empty string from 'quickfixtextfunc' is the whole line.
            if !qftf_str.is_null() && *qftf_str != 0 {
                push_cstr(out, qftf_str);
                return;
            }

            // "<where>|<position>| <message>".
            if !(*qfp).qf_module.is_null() {
                push_cstr(out, (*qfp).qf_module);
            } else {
                let errbuf = if (*qfp).qf_fnum != 0 {
                    buflist_findnr((*qfp).qf_fnum)
                } else {
                    ptr::null_mut()
                };
                if !errbuf.is_null() && !(*errbuf).b_fname.is_null() {
                    if (*qfp).qf_type as c_int == 1 {
                        // :helpgrep entries name the help file only.
                        push_cstr(out, path_tail((*errbuf).b_fname));
                    } else {
                        // Shorten the file name if not done already. For
                        // speed, only for the first entry of each buffer.
                        if first_bufline
                            && ((*errbuf).b_sfname.is_null()
                                || path_is_absolute((*errbuf).b_sfname))
                        {
                            shorten_buf_fname(errbuf, dir.get(), false as c_int);
                        }
                        push_cstr(
                            out,
                            if (*qfp).qf_fname.is_null() {
                                (*errbuf).b_fname
                            } else {
                                (*qfp).qf_fname
                            },
                        );
                    }
                }
            }

            out.push(b'|');
            if (*qfp).qf_lnum > 0 {
                qf_range_text(out, qfp);
                out.extend(qf_types((*qfp).qf_type as c_int, (*qfp).qf_nr).to_bytes());
            } else if !(*qfp).qf_pattern.is_null() {
                qf_fmt_text(out, (*qfp).qf_pattern);
            }
            out.push(b'|');
            out.push(b' ');

            // Remove newlines and leading whitespace from the text. An
            // unrecognized line — one with nothing but the two bars before
            // it — keeps its indent: the compiler may be marking a word
            // with "^^^^".
            let recognized = out.len() > 3;
            let text = if recognized {
                skipwhite((*qfp).qf_text)
            } else {
                (*qfp).qf_text
            };
            qf_fmt_text(out, text);
        });

        ml_append_buf(
            buf,
            lnum,
            line.as_ptr().cast_mut().cast(),
            line.len() as colnr_T,
            false,
        ) != FAIL
    }
}

/// Ask `'quickfixtextfunc'` for the text of the entries `start_idx` to
/// `end_idx`, or answer null when there is no such function.
///
/// The list-local function wins over the global one.
///
/// # Safety
///
/// `qfl` must be a live list.
unsafe fn call_qftf_func(
    qfl: *mut qf_list_T,
    qf_winid: c_int,
    start_idx: c_int,
    end_idx: c_int,
) -> *mut list_T {
    /// This does not work properly recursively.
    static RECURSIVE: GlobalCell<bool> = GlobalCell::new(false);

    // SAFETY: forwarded from the caller.
    unsafe {
        if RECURSIVE.get() {
            return ptr::null_mut();
        }
        let cb = if (*qfl).qf_qftf_cb.type_0 != kCallbackNone {
            &raw mut (*qfl).qf_qftf_cb
        } else {
            qftf_cb.ptr()
        };
        if (*cb).type_0 == kCallbackNone {
            return ptr::null_mut();
        }
        RECURSIVE.set(true);

        let dict = tv_dict_alloc_lock(VAR_FIXED);
        let add = |key: &CStr, value: varnumber_T| {
            tv_dict_add_nr(dict, key.as_ptr(), key.count_bytes(), value);
        };
        add(
            c"quickfix",
            varnumber_T::from((*qfl).qfl_type == QFLT_QUICKFIX),
        );
        add(c"winid", qf_winid as varnumber_T);
        add(c"id", (*qfl).qf_id as varnumber_T);
        add(c"start_idx", start_idx as varnumber_T);
        add(c"end_idx", end_idx as varnumber_T);
        (*dict).dv_refcount += 1;

        let mut args = [typval_T {
            v_type: VAR_DICT,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_dict: dict },
        }];
        let mut rettv = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut answer = ptr::null_mut::<list_T>();
        let locked = Lock::text();
        if callback_call(cb, 1, args.as_mut_ptr(), &raw mut rettv) {
            if rettv.v_type == VAR_LIST {
                answer = rettv.vval.v_list;
                tv_list_ref(answer);
            }
            tv_clear(&raw mut rettv);
        }
        drop(locked);
        tv_dict_unref(dict);

        RECURSIVE.set(false);
        answer
    }
}

/// Empty the quickfix buffer, which must be the current one.
///
/// Answers false if a line would not delete, which would otherwise loop
/// forever.
///
/// # Safety
///
/// `curbuf` must be the quickfix buffer.
unsafe fn clear_qf_buffer() -> bool {
    // SAFETY: forwarded from the caller.
    unsafe {
        // No undo information is stored — the quickfix buffer is usually
        // not modifiable — so the undo stack is cleaned up instead, or an
        // autocommand could invalidate it.
        while !(*curbuf.get()).b_ml.ml_flags.has(MlFlags::EMPTY) {
            if ml_delete(1) == FAIL {
                internal_error(c"qf_fill_buffer()".as_ptr());
                return false;
            }
        }
        find_tab_win(|wp| {
            if (*wp).w_buffer == curbuf.get() {
                (*wp).w_skipcol = 0;
            }
            false
        });
        u_clearallandblockfree(curbuf.get());
        true
    }
}

/// Set the options a freshly filled quickfix buffer wants, and tell the
/// autocommands about it.
///
/// # Safety
///
/// `curbuf` must be the quickfix buffer.
unsafe fn finish_qf_buffer() {
    // SAFETY: forwarded from the caller.
    unsafe {
        // Set 'filetype' to "qf" each time after filling the buffer. This
        // resembles reading a file into a buffer, which is more logical
        // when using autocommands.
        (*curbuf.get()).b_ro_locked += 1;
        set_option_value_give_err(kOptFiletype, string_optval(c"qf"), OptionSetFlags::LOCAL);
        (*curbuf.get()).b_p_ma = false as c_int;

        (*curbuf.get()).b_keep_filetype = true; // don't detect 'filetype'
        apply_autocmds(
            EVENT_BUFREADPOST,
            c"quickfix".as_ptr().cast_mut(),
            ptr::null_mut(),
            false,
            curbuf.get(),
        );
        apply_autocmds(
            EVENT_BUFWINENTER,
            c"quickfix".as_ptr().cast_mut(),
            ptr::null_mut(),
            false,
            curbuf.get(),
        );
        (*curbuf.get()).b_keep_filetype = false;
        (*curbuf.get()).b_ro_locked -= 1;

        // Make sure it will be redrawn.
        redraw_curbuf_later(UPD_NOT_VALID);
    }
}

/// Fill the quickfix buffer with the list, replacing what it held.
///
/// With `old_last` the entries after that one are appended instead, and
/// `buf` need not be the current buffer; without it `buf` must be `curbuf`,
/// because lines are deleted and autocommands are triggered.
///
/// # Safety
///
/// `buf` must be a live buffer and `qfl` null or a live list.
pub(crate) unsafe fn qf_fill_buffer(
    qfl: *mut qf_list_T,
    buf: *mut buf_T,
    old_last: *mut qfline_T,
    qf_winid: c_int,
) {
    let mut numbuf = NumBuf::new();
    // SAFETY: forwarded from the caller.
    unsafe {
        let old_key_typed = KeyTyped.get();
        let rewriting = old_last.is_null();
        if rewriting {
            if buf != curbuf.get() {
                internal_error(c"qf_fill_buffer()".as_ptr());
                return;
            }
            if !clear_qf_buffer() {
                return;
            }
        }

        if !qfl.is_null() && !(*qfl).qf_start.is_null() {
            let mut dir = CurrentDir::new();
            // One line per entry, from the start or after the last entry
            // that is already in the buffer.
            let (mut qfp, mut lnum) = if rewriting {
                ((*qfl).qf_start, 0)
            } else if (*old_last).qf_next.is_null() {
                (old_last, (*buf).b_ml.ml_line_count)
            } else {
                ((*old_last).qf_next, (*buf).b_ml.ml_line_count)
            };

            let qftf_list = call_qftf_func(qfl, qf_winid, lnum as c_int + 1, (*qfl).qf_count);
            let mut qftf_li = tv_list_first(qftf_list);
            let mut prev_bufnr = -1;
            let mut invalid_val = false;

            while lnum < (*qfl).qf_count as linenr_T {
                // Use the text the user's function supplied, if any. Once
                // it answers something that is not a string, the rest of
                // its answer is ignored too.
                let mut qftf_str = ptr::null::<c_char>();
                if !qftf_li.is_null() && !invalid_val {
                    qftf_str = numbuf.string_chk(&raw mut (*qftf_li).li_tv);
                    if qftf_str.is_null() {
                        invalid_val = true;
                    }
                }

                if !qf_buf_add_line(
                    buf,
                    lnum,
                    qfp,
                    &mut dir,
                    qftf_str,
                    prev_bufnr != (*qfp).qf_fnum,
                ) {
                    break;
                }
                prev_bufnr = (*qfp).qf_fnum;
                lnum += 1;
                qfp = (*qfp).qf_next;
                if qfp.is_null() {
                    break;
                }
                if !qftf_li.is_null() {
                    qftf_li = (*qftf_li).li_next;
                }
            }
            if rewriting {
                // Delete the empty line which is now at the end.
                ml_delete(lnum + 1);
            }
            release_scratch();
        }

        // Correct cursor position.
        check_lnums(true);

        if rewriting {
            finish_qf_buffer();
        }

        // Restore KeyTyped, setting 'filetype' may reset it.
        KeyTyped.set(old_key_typed);
    }
}
