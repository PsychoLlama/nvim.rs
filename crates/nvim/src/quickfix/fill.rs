//! Writing the list into the quickfix buffer.
//!
//! [`qf_update_buffer`] is what every command that changes a list calls: it
//! finds the buffer the quickfix window shows, has [`qf_fill_buffer`] write
//! one line per entry into it, and tells the editor what changed.
//!
//! The text of a line is [`qf_buf_line`]'s, unless `'quickfixtextfunc'` is
//! set — then [`call_qftf_func`] asks the user's function for the lines
//! first, and any entry it answers a string for uses that instead.
//!
//! The user's function runs user code, and so does appending a line to a
//! buffer, so the fill holds the list as a view and re-finds each entry by
//! position: a function that empties the list ends the fill rather than
//! leaving it reading entries that are gone.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::autocmd::AucmdBuf;
use crate::eval::list::cstr_of_chk;
use crate::eval::typval::NumBuf;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::eval::typval::{CallFrame, list_items};
use crate::fileio::shorten_buf_fname_in;
use crate::guard::Lock;
use crate::memline::{MlFlags, ml_append_bytes};
use crate::memory::XString;
use crate::os::fs::current_dir;
use crate::path::tail_index;
use crate::types::{BCount, ListRef, OptionSetFlags, VarLock};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};

/// The directory file names are shortened against, resolved the first time
/// an entry needs it.
struct CurrentDir(Option<XString>);

impl CurrentDir {
    fn new() -> CurrentDir {
        CurrentDir(None)
    }

    /// The current directory. Empty if the system would not say, in which
    /// case the next entry asks again — as upstream does.
    fn get(&mut self) -> &CStr {
        if self.0.as_ref().is_none_or(|dir| dir.is_empty()) {
            self.0 = current_dir();
        }
        self.0.as_ref().map_or(c"", XString::as_cstr)
    }
}

/// Update the quickfix buffer, if one exists, after the list changed.
///
/// With `old_last` — the position of what was the last entry — the entries
/// after it are appended; otherwise the whole buffer is rewritten.
pub(crate) fn qf_update_buffer(qi: Qi, old_last: Option<usize>) {
    let Some(mut buf) = qf_find_buf(qi) else {
        return;
    };

    let old_line_count = buf.b_ml.ml_line_count;
    let old_endcol = ml_get_buf_len(buf, old_line_count);
    let old_bytecount = get_region_bytecount(buf, 1, old_line_count, 0, old_endcol);

    // A location list's window id goes to 'quickfixtextfunc'; it is the
    // window the list belongs to, not the one showing it.
    let mut qf_winid = 0;
    if qi.kind == QFLT_LOCATION {
        let win = if Win::current().w_llist == Some(qi.id()) {
            Win::current()
        } else {
            // The file window, or failing that the location list window.
            let found = qf_find_win_with_loclist(qi.id());
            let Some(win) = found.or_else(|| qf_find_win(qi)) else {
                return;
            };
            win
        };
        qf_winid = win.handle;
    }

    // Autocommands may cause trouble.
    let busy = QuickfixBusy::hold();

    // Set curwin/curbuf to buf and save a few things.
    let aco = old_last.is_none().then(|| AucmdBuf::enter(buf));
    qf_update_win_titlevar(qi);
    qf_fill_buffer(qi.current_slot(), buf, old_last, qf_winid);

    let new_line_count = buf.b_ml.ml_line_count;
    let new_endcol = ml_get_buf_len(buf, new_line_count);
    let delta = new_line_count - old_line_count;
    if old_last.is_none() {
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
    buf.b_changed = c_int::from(false);

    if let Some(aco) = aco {
        qf_win_pos_update(qi, 0);
        // Restore curwin/curbuf and a few other things.
        drop(aco);
    }

    // Only redraw when the added lines are visible, to avoid flicker.
    if qf_find_win(qi).is_some_and(|win| old_line_count < win.w_botline) {
        redraw_buf_later(buf, UPD_NOT_VALID);
    }

    drop(busy);
}

/// One entry as a line of the quickfix buffer, appended to `out`.
///
/// `text_func` is what `'quickfixtextfunc'` answered for the entry; a
/// non-empty answer is the whole line.
fn qf_buf_line(
    out: &mut Vec<u8>,
    entry: &QfEntry,
    dir: &mut CurrentDir,
    text_func: Option<&[u8]>,
    first_bufline: bool,
) {
    if let Some(text) = text_func.filter(|text| !text.is_empty()) {
        out.extend_from_slice(text);
        return;
    }

    // "<where>|<position>| <message>".
    if let Some(module) = &entry.module {
        out.extend_from_slice(module);
    } else {
        let errbuf = if entry.fnum != 0 {
            find_buf(entry.fnum)
        } else {
            None
        };
        if let Some(errbuf) = errbuf.filter(|b| !b.name.is_unnamed()) {
            if entry.kind == 1 {
                // :helpgrep entries name the help file only.
                let shown = errbuf.name.shown().map_or(&b""[..], CStr::to_bytes);
                out.extend_from_slice(&shown[tail_index(shown)..]);
            } else {
                // Shorten the file name if not done already. For speed,
                // only for the first entry of each buffer.
                if first_bufline && errbuf.name.short().is_none_or(path_is_absolute) {
                    shorten_buf_fname_in(errbuf, dir.get(), false);
                }
                match &entry.fname {
                    Some(own) => out.extend_from_slice(own),
                    None => {
                        let shown = errbuf.name.shown().map_or(&b""[..], CStr::to_bytes);
                        out.extend_from_slice(shown);
                    }
                }
            }
        }
    }

    out.push(b'|');
    if entry.lnum > 0 {
        qf_range_text(out, entry);
        out.extend_from_slice(qf_types(c_int::from(entry.kind), entry.nr).to_bytes());
    } else if let Some(pattern) = &entry.pattern {
        qf_fmt_text(out, pattern);
    }
    out.push(b'|');
    out.push(b' ');

    // Remove newlines and leading whitespace from the text. An unrecognized
    // line — one with nothing but the two bars before it — keeps its
    // indent: the compiler may be marking a word with "^^^^".
    let recognized = out.len() > 3;
    let text: &[u8] = if recognized {
        skip_white(&entry.text)
    } else {
        &entry.text
    };
    qf_fmt_text(out, text);
}

/// Ask `'quickfixtextfunc'` for the text of the entries `start_idx` to
/// `end_idx`, or answer `None` when there is no such function.
///
/// The list-local function wins over the global one. A copy of it is
/// called, so that the function can set either without freeing what is
/// running.
fn call_qftf_func(qfl: Qfl, qf_winid: c_int, start_idx: c_int, end_idx: c_int) -> Option<ListRef> {
    /// This does not work properly recursively.
    static RECURSIVE: GlobalCell<bool> = GlobalCell::new(false);

    if RECURSIVE.get() {
        return None;
    }
    let mut cb = if qfl.text_func.is_set() {
        qfl.text_func.duplicate()
    } else {
        qftf_cb.with(Callback::duplicate)
    };
    if !cb.is_set() {
        return None;
    }
    RECURSIVE.set(true);

    let mut dict = tv_dict_alloc_lock(VarLock::Fixed);
    let numbers = [
        (&b"quickfix"[..], VarNumber::from(qfl.kind == QFLT_QUICKFIX)),
        (b"winid", VarNumber::from(qf_winid)),
        (b"id", VarNumber::from(qfl.id)),
        (b"start_idx", VarNumber::from(start_idx)),
        (b"end_idx", VarNumber::from(end_idx)),
    ];
    for (key, value) in numbers {
        let _ = dict.add_number(key, value);
    }
    let mut args = CallFrame::<1>::new();
    args.push_owned(TypVal::dict(Some(dict)));

    let mut rettv = TV_INITIAL_VALUE;
    let locked = Lock::text();
    let answer = if cb.call(args.args(), &mut rettv) {
        rettv.take_list()
    } else {
        None
    };
    drop(locked);
    drop(rettv);
    drop(args);
    cb.clear();

    RECURSIVE.set(false);
    answer
}

/// Empty the quickfix buffer, which must be the current one.
///
/// Answers false if a line would not delete, which would otherwise loop
/// forever.
fn clear_qf_buffer() -> bool {
    // No undo information is stored — the quickfix buffer is usually
    // not modifiable — so the undo stack is cleaned up instead, or an
    // autocommand could invalidate it.
    while !Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY) {
        if ml_delete(1).is_err() {
            internal_error(c"qf_fill_buffer()");
            return false;
        }
    }
    find_tab_win(|mut wp| {
        if wp.w_buffer.is_current() {
            wp.w_skipcol = 0;
        }
        false
    });
    u_clearallandblockfree(Buf::current());
    true
}

/// Set the options a freshly filled quickfix buffer wants, and tell the
/// autocommands about it.
fn finish_qf_buffer() {
    // Set 'filetype' to "qf" each time after filling the buffer. This
    // resembles reading a file into a buffer, which is more logical
    // when using autocommands.
    Buf::current().b_ro_locked += 1;
    set_option_value_give_err(kOptFiletype, string_optval(c"qf"), OptionSetFlags::LOCAL);
    Buf::current().b_p_ma = c_int::from(false);

    Buf::current().b_keep_filetype = true; // don't detect 'filetype'
    let buffer = Buf::current_or_none();
    fire_autocmds_for(
        AutoEvent::BufReadPost,
        Some(c"quickfix"),
        None,
        false,
        buffer,
    );
    let buffer = Buf::current_or_none();
    fire_autocmds_for(
        AutoEvent::BufWinEnter,
        Some(c"quickfix"),
        None,
        false,
        buffer,
    );
    Buf::current().b_keep_filetype = false;
    Buf::current().b_ro_locked -= 1;

    // Make sure it will be redrawn.
    redraw_curbuf_later(UPD_NOT_VALID);
}

/// What one rewrite of the quickfix buffer moved: where it started, and the
/// rows, columns and bytes the old and the new text took from there.
struct Splice {
    start: (LineNr, ColNr),
    old: (LineNr, ColNr, BCount),
    new: (LineNr, ColNr, BCount),
}

/// [`extmark_splice`] for a quickfix-buffer rewrite, which is never undoable.
fn splice(buffer: Buf, at: &Splice) {
    let (srow, scol) = at.start;
    let (orow, ocol, obytes) = at.old;
    let (nrow, ncol, nbytes) = at.new;
    let undo = kExtmarkNoUndo;
    extmark_splice(
        buffer, srow, scol, orow, ocol, obytes, nrow, ncol, nbytes, undo,
    );
}

/// Fill the quickfix buffer with the list, replacing what it held.
///
/// With `old_last` the entries after that one are appended instead, and
/// `buffer` need not be the current buffer; without it `buffer` must be
/// `curbuf`, because lines are deleted and autocommands are triggered.
pub(crate) fn qf_fill_buffer(qfl: Qfl, buffer: Buf, old_last: Option<usize>, qf_winid: c_int) {
    let mut numbuf = NumBuf::new();
    let old_key_typed = KeyTyped.get();
    let rewriting = old_last.is_none();
    if rewriting {
        if Buf::current_or_none() != Some(buffer) {
            internal_error(c"qf_fill_buffer()");
            return;
        }
        if !clear_qf_buffer() {
            return;
        }
    }

    if !qfl.is_empty() {
        let mut dir = CurrentDir::new();
        // One line per entry, from the start or after the last entry
        // that is already in the buffer.
        let (mut at, mut lnum) = match old_last {
            None => (0, 0),
            Some(last) if last + 1 < qfl.entries.len() => (last + 1, buffer.b_ml.ml_line_count),
            Some(last) => (last, buffer.b_ml.ml_line_count),
        };

        let qftf_list = call_qftf_func(qfl, qf_winid, lnum as c_int + 1, qfl.count());
        // An index: appending a line below runs autocommands.
        let mut qftf_at = 0;
        let mut prev_bufnr = -1;
        let mut invalid_val = false;
        let mut line = Vec::new();

        while lnum < LineNr::from(qfl.count()) {
            // Use the text the user's function supplied, if any. Once it
            // answers something that is not a string, the rest of its
            // answer is ignored too.
            let mut text_func = None;
            let have_item = qftf_list
                .as_deref()
                .is_some_and(|list| list_items(Some(list)).get(qftf_at).is_some());
            if have_item && !invalid_val {
                let list = qftf_list.as_deref();
                let item = &list_items(list)[qftf_at];
                match cstr_of_chk(&item.li_tv, &mut numbuf) {
                    Some(text) => text_func = Some(text.to_bytes().to_vec()),
                    None => invalid_val = true,
                }
            }

            let Some(entry) = qfl.entries.get(at) else {
                break;
            };
            let fnum = entry.fnum;
            line.clear();
            qf_buf_line(
                &mut line,
                entry,
                &mut dir,
                text_func.as_deref(),
                prev_bufnr != fnum,
            );
            if ml_append_bytes(buffer, lnum, &line).is_err() {
                break;
            }
            prev_bufnr = fnum;
            lnum += 1;
            at += 1;
            if at >= qfl.entries.len() {
                break;
            }
            if have_item {
                qftf_at += 1;
            }
        }
        if rewriting {
            // Delete the empty line which is now at the end.
            let _ = ml_delete(lnum + 1);
        }
    }

    // Correct cursor position.
    check_lnums(true);

    if rewriting {
        finish_qf_buffer();
    }

    // Restore KeyTyped, setting 'filetype' may reset it.
    KeyTyped.set(old_key_typed);
}
