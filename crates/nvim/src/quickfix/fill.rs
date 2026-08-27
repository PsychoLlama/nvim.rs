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
    FAIL, MAXPATHL, OptionSetFlags, VAR_DICT, VAR_LIST, VAR_UNKNOWN, VarLock, bcount_t,
};
use crate::winlayer::Buf;
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
    // SAFETY: the caller's promise -- a live `qf_info_T`.
    let qi = unsafe { Qi::new(qi) };
    let Some(mut buf) = qf_find_buf(qi) else {
        return;
    };

    let old_line_count = buf.b_ml.ml_line_count;
    // SAFETY: a live buffer and a line number inside it.
    let old_endcol = unsafe { ml_get_buf_len(buf.raw(), old_line_count) };
    let old_bytecount = get_region_bytecount(buf, 1, old_line_count, 0, old_endcol);

    // A location list's window id goes to 'quickfixtextfunc'; it is the
    // window the list belongs to, not the one showing it.
    let mut qf_winid = 0;
    if qi.qfl_type == QFLT_LOCATION {
        let win = if cur_win().w_llist == qi.raw() {
            cur_win()
        } else {
            // The file window, or failing that the location list window.
            let found = qf_find_win_with_loclist(qi.raw().cast_const());
            let Some(win) = found.or_else(|| qf_find_win(qi)) else {
                return;
            };
            win
        };
        qf_winid = win.handle;
    }

    // Autocommands may cause trouble.
    incr_quickfix_busy();

    let mut aco = aco_save_T::default();
    if old_last.is_null() {
        // Set curwin/curbuf to buf and save a few things.
        // SAFETY: a live buffer, and `aco` outlives the restore below.
        unsafe { aucmd_prepbuf(&raw mut aco, buf.raw()) };
    }
    qf_update_win_titlevar(qi);
    // SAFETY: a live list, buffer and entry.
    unsafe { qf_fill_buffer(qf_get_curlist(qi.raw()), buf, old_last, qf_winid) };

    let new_line_count = buf.b_ml.ml_line_count;
    // SAFETY: a live buffer and a line number inside it.
    let new_endcol = unsafe { ml_get_buf_len(buf.raw(), new_line_count) };
    let delta = new_line_count - old_line_count;
    if old_last.is_null() {
        let bytes = get_region_bytecount(buf, 1, new_line_count, 0, new_endcol);
        splice(
            buf,
            &Splice {
                start: (0, 0),
                old: (old_line_count - 1, 0, old_bytecount),
                new: (new_line_count - 1, new_endcol, bytes),
            },
        );
        let lnume = if old_line_count > 0 {
            old_line_count + 1
        } else {
            1
        };
        changed_lines(buf, 1, 0, lnume, delta, true);
    } else if delta > 0 {
        let start_lnum = old_line_count + 1;
        let bytes = get_region_bytecount(buf, start_lnum, new_line_count, 0, new_endcol);
        splice(
            buf,
            &Splice {
                start: (old_line_count - 1, old_endcol),
                old: (0, 0, 0),
                new: (delta, new_endcol, bytes),
            },
        );
        changed_lines(buf, start_lnum, 0, start_lnum, delta, true);
    }
    buf.b_changed = false as c_int;

    if old_last.is_null() {
        qf_win_pos_update(qi, 0);
        // Restore curwin/curbuf and a few other things.
        // SAFETY: the `aco` `aucmd_prepbuf` above filled in.
        unsafe { aucmd_restbuf(&raw mut aco) };
    }

    // Only redraw when the added lines are visible, to avoid flicker.
    if qf_find_win(qi).is_some_and(|win| old_line_count < win.w_botline) {
        // SAFETY: a live buffer.
        unsafe { redraw_buf_later(buf.raw(), UPD_NOT_VALID) };
    }

    // Always called after incr_quickfix_busy().
    qf_busy_end();
}

/// Append one entry to the quickfix buffer as a line of text, answering
/// whether it fit.
///
/// # Safety
///
/// `qfp` must be a live entry, and `buf` the quickfix buffer.
unsafe fn qf_buf_add_line(
    buf: Buf,
    lnum: linenr_T,
    qfp: *const qfline_T,
    dir: &mut CurrentDir,
    qftf_str: *const c_char,
    first_bufline: bool,
) -> bool {
    // SAFETY: the caller's promise -- a live `qfline_T`.
    let qfp = unsafe { Qfe::new(qfp.cast_mut()) };
    // SAFETY: forwarded from the caller. Nothing in the line building
    // prints or runs an autocommand, which is `build_line`'s contract.
    let line = build_line(|out| {
        // A non-empty string from 'quickfixtextfunc' is the whole line.
        if !qftf_str.is_null() && unsafe { *qftf_str } != 0 {
            unsafe { push_cstr(out, qftf_str) };
            return;
        }

        // "<where>|<position>| <message>".
        if !qfp.qf_module.is_null() {
            unsafe { push_cstr(out, qfp.qf_module) };
        } else {
            let errbuf = if qfp.qf_fnum != 0 {
                find_buf(qfp.qf_fnum).map_or(ptr::null_mut(), |mut b| b.raw())
            } else {
                ptr::null_mut()
            };
            if !errbuf.is_null() && !unsafe { (*errbuf).b_fname.is_null() } {
                if qfp.qf_type as c_int == 1 {
                    // :helpgrep entries name the help file only.
                    unsafe { push_cstr(out, path_tail((*errbuf).b_fname)) };
                } else {
                    // Shorten the file name if not done already. For
                    // speed, only for the first entry of each buffer.
                    if first_bufline
                        && (unsafe { (*errbuf).b_sfname.is_null() }
                            || unsafe { path_is_absolute((*errbuf).b_sfname) })
                    {
                        // SAFETY: a live buffer and the current directory name.
                        unsafe { shorten_buf_fname(Buf::new(errbuf), dir.get(), false as c_int) };
                    }
                    let start_row = if qfp.qf_fname.is_null() {
                        unsafe { (*errbuf).b_fname }
                    } else {
                        qfp.qf_fname
                    };
                    unsafe { push_cstr(out, start_row) };
                }
            }
        }

        out.push(b'|');
        if qfp.qf_lnum > 0 {
            unsafe { qf_range_text(out, qfp.raw().cast_const()) };
            out.extend(qf_types(qfp.qf_type as c_int, qfp.qf_nr).to_bytes());
        } else if !qfp.qf_pattern.is_null() {
            unsafe { qf_fmt_text(out, qfp.qf_pattern) };
        }
        out.push(b'|');
        out.push(b' ');

        // Remove newlines and leading whitespace from the text. An
        // unrecognized line — one with nothing but the two bars before
        // it — keeps its indent: the compiler may be marking a word
        // with "^^^^".
        let recognized = out.len() > 3;
        let text = if recognized {
            unsafe { skipwhite(qfp.qf_text) }
        } else {
            qfp.qf_text
        };
        unsafe { qf_fmt_text(out, text) };
    });

    unsafe {
        ml_append_buf(
            buf.raw(),
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
    // SAFETY: the caller's promise -- a live `qf_list_T`.
    let mut qfl = unsafe { Qfl::new(qfl) };
    /// This does not work properly recursively.
    static RECURSIVE: GlobalCell<bool> = GlobalCell::new(false);

    // SAFETY: forwarded from the caller.
    if RECURSIVE.get() {
        return ptr::null_mut();
    }
    let cb = if qfl.qf_qftf_cb.type_0 != kCallbackNone {
        // SAFETY: the caller's list, whose callback outlives this call.
        &raw mut qfl.qf_qftf_cb
    } else {
        global_qftf()
    };
    if unsafe { (*cb).type_0 } == kCallbackNone {
        return ptr::null_mut();
    }
    RECURSIVE.set(true);

    let dict = unsafe { tv_dict_alloc_lock(VarLock::Fixed) };
    let add = |key: &CStr, value: varnumber_T| {
        unsafe { tv_dict_add_nr(dict, key.as_ptr(), key.count_bytes(), value) };
    };
    add(
        c"quickfix",
        varnumber_T::from(qfl.qfl_type == QFLT_QUICKFIX),
    );
    add(c"winid", qf_winid as varnumber_T);
    add(c"id", qfl.qf_id as varnumber_T);
    add(c"start_idx", start_idx as varnumber_T);
    add(c"end_idx", end_idx as varnumber_T);
    unsafe { (*dict).dv_refcount.retain() };

    let mut args = [typval_T {
        v_type: VAR_DICT,
        v_lock: VarLock::Unlocked,
        vval: typval_vval_union { v_dict: dict },
    }];
    let mut rettv = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VarLock::Unlocked,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut answer = ptr::null_mut::<list_T>();
    let locked = Lock::text();
    if unsafe { callback_call(cb, 1, args.as_mut_ptr(), &raw mut rettv) } {
        if rettv.v_type == VAR_LIST {
            answer = unsafe { rettv.vval.v_list };
            unsafe { tv_list_ref(answer) };
        }
        unsafe { tv_clear(&raw mut rettv) };
    }
    drop(locked);
    unsafe { tv_dict_unref(dict) };

    RECURSIVE.set(false);
    answer
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
    // No undo information is stored — the quickfix buffer is usually
    // not modifiable — so the undo stack is cleaned up instead, or an
    // autocommand could invalidate it.
    while !cur_buf().b_ml.ml_flags.has(MlFlags::EMPTY) {
        if unsafe { ml_delete(1) } == FAIL {
            unsafe { internal_error(c"qf_fill_buffer()".as_ptr()) };
            return false;
        }
    }
    // SAFETY: the closure only writes a field of each window it is handed.
    unsafe {
        find_tab_win(|mut wp| {
            if wp.w_buffer == curbuf.get() {
                wp.w_skipcol = 0;
            }
            false
        })
    };
    u_clearallandblockfree(unsafe { Buf::current() });
    true
}

/// Set the options a freshly filled quickfix buffer wants, and tell the
/// autocommands about it.
///
/// # Safety
///
/// `curbuf` must be the quickfix buffer.
unsafe fn finish_qf_buffer() {
    // SAFETY: forwarded from the caller.
    // Set 'filetype' to "qf" each time after filling the buffer. This
    // resembles reading a file into a buffer, which is more logical
    // when using autocommands.
    cur_buf().b_ro_locked += 1;
    set_option_value_give_err(kOptFiletype, string_optval(c"qf"), OptionSetFlags::LOCAL);
    cur_buf().b_p_ma = false as c_int;

    cur_buf().b_keep_filetype = true; // don't detect 'filetype'
    let start_row = c"quickfix".as_ptr().cast_mut();
    let start_col = ptr::null_mut();
    let old_col = curbuf.get();
    unsafe { apply_autocmds(EVENT_BUFREADPOST, start_row, start_col, false, old_col) };
    let lnum2 = c"quickfix".as_ptr().cast_mut();
    let col = ptr::null_mut();
    let old_col2 = curbuf.get();
    unsafe { apply_autocmds(EVENT_BUFWINENTER, lnum2, col, false, old_col2) };
    cur_buf().b_keep_filetype = false;
    cur_buf().b_ro_locked -= 1;

    // Make sure it will be redrawn.
    redraw_curbuf_later(UPD_NOT_VALID);
}

/// What one rewrite of the quickfix buffer moved: where it started, and the
/// rows, columns and bytes the old and the new text took from there.
struct Splice {
    start: (linenr_T, colnr_T),
    old: (linenr_T, colnr_T, bcount_t),
    new: (linenr_T, colnr_T, bcount_t),
}

/// [`extmark_splice`] for a quickfix-buffer rewrite, which is never undoable.
///
/// The nine numbers are bound out here rather than written into the call:
/// rustfmt gives an argument list this wide one line per argument, and all
/// of them would be inside the region.
fn splice(buf: Buf, at: &Splice) {
    let (srow, scol) = at.start;
    let (orow, ocol, obytes) = at.old;
    let (nrow, ncol, nbytes) = at.new;
    let undo = kExtmarkNoUndo;
    let raw = buf.raw();
    // SAFETY: `buf` is the quickfix window's buffer, live for the call.
    unsafe {
        extmark_splice(
            raw, srow, scol, orow, ocol, obytes, nrow, ncol, nbytes, undo,
        )
    };
}

/// Fill the quickfix buffer with the list, replacing what it held.
///
/// With `old_last` the entries after that one are appended instead, and
/// `buf` need not be the current buffer; without it `buf` must be `curbuf`,
/// because lines are deleted and autocommands are triggered.
///
/// # Safety
///
/// `qfl` must be null or a live list.
pub(crate) unsafe fn qf_fill_buffer(
    qfl: *mut qf_list_T,
    buf: Buf,
    old_last: *mut qfline_T,
    qf_winid: c_int,
) {
    let mut numbuf = NumBuf::new();
    // SAFETY: forwarded from the caller.
    let old_key_typed = KeyTyped.get();
    let rewriting = old_last.is_null();
    if rewriting {
        if buf.raw() != curbuf.get() {
            unsafe { internal_error(c"qf_fill_buffer()".as_ptr()) };
            return;
        }
        if !unsafe { clear_qf_buffer() } {
            return;
        }
    }

    if !qfl.is_null() && !unsafe { (*qfl).qf_start.is_null() } {
        let mut dir = CurrentDir::new();
        // One line per entry, from the start or after the last entry
        // that is already in the buffer.
        let (mut qfp, mut lnum) = if rewriting {
            (unsafe { (*qfl).qf_start }, 0)
        } else if unsafe { (*old_last).qf_next.is_null() } {
            (old_last, buf.b_ml.ml_line_count)
        } else {
            (unsafe { (*old_last).qf_next }, buf.b_ml.ml_line_count)
        };

        let qftf_list =
            unsafe { call_qftf_func(qfl, qf_winid, lnum as c_int + 1, (*qfl).qf_count) };
        let mut qftf_li = unsafe { tv_list_first(qftf_list) };
        let mut prev_bufnr = -1;
        let mut invalid_val = false;

        while lnum < unsafe { (*qfl).qf_count } as linenr_T {
            // Use the text the user's function supplied, if any. Once
            // it answers something that is not a string, the rest of
            // its answer is ignored too.
            let mut qftf_str = ptr::null::<c_char>();
            if !qftf_li.is_null() && !invalid_val {
                qftf_str = unsafe { numbuf.string_chk(&raw mut (*qftf_li).li_tv) };
                if qftf_str.is_null() {
                    invalid_val = true;
                }
            }

            if !unsafe {
                qf_buf_add_line(
                    buf,
                    lnum,
                    qfp,
                    &mut dir,
                    qftf_str,
                    prev_bufnr != (*qfp).qf_fnum,
                )
            } {
                break;
            }
            prev_bufnr = unsafe { (*qfp).qf_fnum };
            lnum += 1;
            qfp = unsafe { (*qfp).qf_next };
            if qfp.is_null() {
                break;
            }
            if !qftf_li.is_null() {
                qftf_li = unsafe { (*qftf_li).li_next };
            }
        }
        if rewriting {
            // Delete the empty line which is now at the end.
            unsafe { ml_delete(lnum + 1) };
        }
        release_scratch();
    }

    // Correct cursor position.
    check_lnums(true);

    if rewriting {
        unsafe { finish_qf_buffer() };
    }

    // Restore KeyTyped, setting 'filetype' may reset it.
    KeyTyped.set(old_key_typed);
}
