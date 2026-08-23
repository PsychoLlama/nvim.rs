//! `:marks` and `:delmarks`.
//!
//! The listing is a surface of its own rather than a rendering of
//! `getmarklist()`: it prints the internal 0-based column, it shows a mark
//! whose line no longer exists as `-invalid-`, and it re-orders `'<`/`'>` so
//! that the earlier end is always `'<`. A change that moves one of the two
//! surfaces and not the other is what `1787242636-jmarksweep`'s `K` and `X`
//! lines are laid out side by side to catch.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::ascii::{ascii_isdigit, ascii_islower, ascii_isupper};
use crate::buffer::{bt_prompt, buflist_findnr, buflist_nr2name};
use crate::charset::{ptr2cells, skipwhite};
use crate::global_cell::GlobalCell;
use crate::main::{Columns, IObuff, e_argreq, e_invarg, e_invarg2, got_int};
use crate::mbyte::utfc_ptr2len;
use crate::memline::ml_get;
use crate::memory::{xfree, xstrdup};
use crate::message::{
    emsg, message_filtered, msg, msg_ext_set_kind, msg_outtrans, msg_putchar, msg_puts_title,
};
use crate::os::cshim::{gettext, snprintf};
use crate::os::time::os_time;
use crate::pos::lt;
use crate::semsg_c;
use crate::strings::{vim_strchr, xstrnsave};
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use super::store::{GlobalMarks, NUL_BYTE, UNSET_POS, mark_name};
use super::*;
use crate::highlight_group::HLF_D;

/// print the marks
///
/// # Safety
/// `eap` must be a live `exarg_T` and the editor's globals must be live.
pub unsafe fn ex_marks(eap: *mut exarg_T) {
    // SAFETY: the caller promised a live command.
    let mut arg = unsafe { (*eap).arg };
    // An empty argument is the same as none: `:marks` with a trailing space
    // must not filter everything out.
    // SAFETY: `arg` is a NUL-terminated string or null.
    if !arg.is_null() && c_int::from(unsafe { *arg }) == NUL {
        arg = ptr::null_mut();
    }
    // SAFETY: `curwin`/`curbuf` are live from startup to exit.
    let (win, buf) = unsafe { (Win::current(), Buf::current()) };
    // SAFETY: a `'static` C string.
    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };

    // `'<` is whichever end of the Visual range comes FIRST, so a selection
    // made backwards still lists in order.
    let (start, end) = (buf.b_visual.vi_start, buf.b_visual.vi_end);
    let starts_first = (lt(start, end) || end.lnum == 0) && start.lnum != 0;
    let (first, second) = if starts_first {
        (start, end)
    } else {
        (end, start)
    };

    // Every row that carries no file name, in listing order. The global table
    // goes between the locals and the tick family, which is why it is not one
    // list.
    let locals = (0..NMARKS).map(|i| (i + 'a' as c_int, buf.named_mark(i).pos()));
    let ticks = [
        ('"' as c_int, buf.last_cursor().pos()),
        ('[' as c_int, buf.b_op_start),
        (']' as c_int, buf.b_op_end),
        ('^' as c_int, buf.last_insert().pos()),
        ('.' as c_int, buf.last_change().pos()),
    ];
    // SAFETY: `buf` is live, which is all `bt_prompt` reads.
    let prompt = unsafe { bt_prompt(buf.raw()) }.then(|| (':' as c_int, buf.prompt_start().pos()));
    let visual = [('<' as c_int, first), ('>' as c_int, second)];

    // SAFETY: `arg` is a NUL-terminated string or null, every name below is
    // one too, and `show_one_mark` allocates and frees its own line text.
    unsafe {
        show_one_mark('\'' as c_int, arg, win.w_pcmark, ptr::null_mut(), 1);
        for (c, pos) in locals {
            show_one_mark(c, arg, pos, ptr::null_mut(), 1);
        }
        // The global table, whose rows carry a FILE NAME rather than the
        // line's text when the mark is in another buffer. A slot whose buffer
        // is loaded allocates its name here; one that came out of the shada
        // file lends the name it still carries, which must not be freed.
        for (i, mark) in GlobalMarks::indexed() {
            let fnum = mark.fmark().fnum();
            let name = if fnum != 0 {
                fm_getname(mark.fmark().raw(), 15)
            } else {
                mark.fname()
            };
            if name.is_null() {
                continue;
            }
            let c = if i >= NMARKS {
                i - NMARKS + '0' as c_int
            } else {
                i + 'A' as c_int
            };
            let current = c_int::from(fnum == buf.handle);
            show_one_mark(c, arg, mark.fmark().pos(), name, current);
            if fnum != 0 {
                xfree(name.cast());
            }
        }
        for (c, pos) in ticks.into_iter().chain(prompt).chain(visual) {
            show_one_mark(c, arg, pos, ptr::null_mut(), 1);
        }
        // The footer: "No marks set" (or E283) if nothing above printed a
        // title.
        finish_marks(arg);
    }
}

/// The `-1` row of upstream's `show_one_mark`: whatever `:marks` prints when
/// it printed nothing else.
///
/// # Safety
/// `arg` must be null or a NUL-terminated string, and the editor's globals
/// must be live.
unsafe fn finish_marks(arg: *mut c_char) {
    if DID_TITLE.replace(false) {
        return;
    }
    // SAFETY: `'static` C strings, and `arg` is the caller's.
    unsafe {
        if arg.is_null() {
            msg(gettext(c"No marks set".as_ptr()), 0);
        } else {
            semsg_c!(gettext(c"E283: No marks matching \"%s\"".as_ptr()), arg);
        }
    }
}

/// Whether `:marks` has printed its column header. Reset by [`finish_marks`],
/// so the next `:marks` prints it again.
static DID_TITLE: GlobalCell<bool> = GlobalCell::new(false);

/// One `:marks` row.
///
/// `current` — in current file
///
/// # Safety
/// `arg` must be null or a NUL-terminated string, and `name_arg` must be null
/// or a NUL-terminated string this function does not own.
pub(super) unsafe fn show_one_mark(
    c: c_int,
    arg: *mut c_char,
    pos: pos_T,
    name_arg: *mut c_char,
    current: c_int,
) {
    // SAFETY: the caller promised a NUL-terminated `arg` or null.
    let wanted = unsafe { arg.is_null() || !vim_strchr(arg, c).is_null() };
    if got_int.get() || !wanted || pos.lnum == 0 {
        return;
    }

    // A row in the current file shows the LINE'S TEXT; one in another file
    // shows the file name, which the caller allocated and still owns.
    let mut name = name_arg;
    let mustfree = name.is_null() && current != 0;
    if mustfree {
        // SAFETY: `mark_line` allocates its answer.
        name = unsafe { mark_line(pos, 15) };
    }
    // SAFETY: `name` is null or a NUL-terminated string, and `IObuff` is
    // `IOSIZE` bytes of live storage.
    unsafe {
        if !message_filtered(name) {
            if !DID_TITLE.replace(true) {
                msg_puts_title(gettext(c"\nmark line  col file/text".as_ptr()));
            }
            msg_putchar('\n' as c_int);
            if !got_int.get() {
                snprintf(
                    IObuff.ptr().cast::<c_char>(),
                    IOSIZE as size_t,
                    c" %c %6d %4d ".as_ptr(),
                    c,
                    pos.lnum,
                    pos.col,
                );
                msg_outtrans(IObuff.ptr().cast::<c_char>(), 0, false);
                if !name.is_null() {
                    msg_outtrans(name, if current != 0 { HLF_D } else { 0 }, false);
                }
            }
        }
        if mustfree {
            xfree(name.cast());
        }
    }
}

/// ":delmarks[!] [marks]"
///
/// # Safety
/// `eap` must be a live `exarg_T` and the editor's globals must be live.
pub unsafe fn ex_delmarks(eap: *mut exarg_T) {
    // SAFETY: the caller promised a live command whose `arg` is a
    // NUL-terminated string.
    let (arg, forceit) = unsafe { ((*eap).arg, (*eap).forceit != 0) };
    // SAFETY: `curbuf` is live from startup to exit.
    let mut buf = unsafe { Buf::current() };
    // SAFETY: `arg` is a NUL-terminated string.
    let empty = c_int::from(unsafe { *arg }) == NUL;

    if empty && forceit {
        // SAFETY: the editor's globals are live.
        unsafe { delmarks_all(buf) };
        return;
    }
    if forceit {
        // `:delmarks!` takes no argument at all; naming one is E474 rather
        // than "clear these, forcefully".
        // SAFETY: a `'static` message.
        unsafe { emsg(gettext((&raw const e_invarg).cast::<c_char>())) };
        return;
    }
    if empty {
        // SAFETY: as above.
        unsafe { emsg(gettext((&raw const e_argreq).cast::<c_char>())) };
        return;
    }

    let timestamp = os_time();
    // The MarkSet payload's position, shared by every announcement here: a
    // deletion reports the mark at line 0, not where it used to be.
    let mut gone = UNSET_POS;
    // The argument as bytes, so the walk below is ordinary slice indexing
    // rather than pointer arithmetic. The `-` form looks two bytes ahead,
    // which on a raw pointer is only in bounds because a NUL is not `-` and a
    // NUL fails every class test; over a slice it is a bounds check.
    // SAFETY: the caller promised a NUL-terminated string.
    let names = unsafe { CStr::from_ptr(arg) }.to_bytes();

    let mut i = 0;
    while i < names.len() {
        let here = c_int::from(names[i]);
        // Where the message points: the rest of the argument from here, which
        // is what upstream's `%s` prints.
        let rest = arg.wrapping_add(i);
        let lower = ascii_islower(here);
        let digit = ascii_isdigit(here);
        let upper = ascii_isupper(here);
        if !(lower || digit || upper) {
            // SAFETY: `buf` and `gone` are live, and `rest` points inside the
            // NUL-terminated argument.
            if !unsafe { delmarks_one(&mut buf, mark_name(here), &mut gone, timestamp) } {
                // SAFETY: as above.
                unsafe { semsg_c!(gettext((&raw const e_invarg2).cast::<c_char>()), rest) };
                return;
            }
            i += 1;
            continue;
        }
        // A range `x-y`: both ends must be in the same class, and the second
        // must not come before the first.
        let from = here;
        let to = if names.get(i + 1) == Some(&b'-') {
            let end = names.get(i + 2).map_or(-1, |&b| c_int::from(b));
            let same_class = if lower {
                ascii_islower(end)
            } else if digit {
                ascii_isdigit(end)
            } else {
                ascii_isupper(end)
            };
            if !same_class || end < from {
                // SAFETY: `rest` points inside the NUL-terminated argument.
                unsafe { semsg_c!(gettext((&raw const e_invarg2).cast::<c_char>()), rest) };
                return;
            }
            i += 2;
            end
        } else {
            from
        };
        for c in from..=to {
            if lower {
                let mark = buf.named_mark(c - 'a' as c_int);
                if mark.is_set() {
                    // SAFETY: `gone` is on this stack and `buf` is live.
                    unsafe { do_markset_autocmd(mark_name(c), &raw mut gone, buf.raw()) };
                }
                // Only the line and the timestamp are cleared, not the whole
                // record: `:delmarks a` is not `clear_fmark`, and the shada
                // writer wants the stamp to say when.
                mark.set_lnum(0);
                mark.set_timestamp(timestamp);
            } else {
                // The fourth and fifth writings of the `namedfm` index
                // formula; see `lookup::mark_global_index`.
                let n = if digit {
                    c - '0' as c_int + NMARKS
                } else {
                    c - 'A' as c_int
                };
                let slot = GlobalMarks::at(n);
                if slot.fmark().is_set() {
                    // The event is announced against the mark's OWN buffer
                    // where it still exists, so an autocommand sees the file
                    // the mark was in.
                    let owner = buflist_findnr(slot.fmark().fnum());
                    let owner = if owner.is_null() { buf.raw() } else { owner };
                    // SAFETY: `gone` is on this stack and `owner` is a live
                    // buffer.
                    unsafe { do_markset_autocmd(mark_name(c), &raw mut gone, owner) };
                }
                slot.fmark().set_lnum(0);
                slot.fmark().set_fnum(0);
                slot.fmark().set_timestamp(timestamp);
                slot.clear_fname();
            }
        }
        i += 1;
    }
}

/// `:delmarks!` — every buffer-local mark, plus the tick family.
///
/// # Safety
/// The editor's globals must be live.
unsafe fn delmarks_all(buf: Buf) {
    let mut gone = UNSET_POS;
    // Announced before the clearing, so an autocommand can still read the
    // buffer the mark was in. `'<`/`'>` are NOT announced and not cleared:
    // `clrallmarks` leaves the Visual range alone.
    for i in 0..NMARKS {
        if buf.named_mark(i).is_set() {
            // SAFETY: `gone` is on this stack and `buf` is live.
            unsafe { do_markset_autocmd(mark_name('a' as c_int + i), &raw mut gone, buf.raw()) };
        }
    }
    for (name, set) in [
        ('"', buf.last_cursor().is_set()),
        ('^', buf.last_insert().is_set()),
        ('.', buf.last_change().is_set()),
        ('[', buf.b_op_start.lnum != 0),
        (']', buf.b_op_end.lnum != 0),
    ] {
        if set {
            // SAFETY: as above.
            unsafe { do_markset_autocmd(name as c_char, &raw mut gone, buf.raw()) };
        }
    }
    // SAFETY: `buf` is live.
    unsafe { clrallmarks(buf.raw(), os_time()) };
}

/// One non-alphanumeric `:delmarks` name. `false` means the name is not a
/// mark at all, which is E474 in the caller.
///
/// # Safety
/// `buf` must be live, `gone` must point at a live position, and the editor's
/// globals must be live.
unsafe fn delmarks_one(
    buf: &mut Buf,
    name: c_char,
    gone: &mut pos_T,
    timestamp: Timestamp,
) -> bool {
    // `:` and a space are accepted and do nothing: the prompt mark is not the
    // user's to delete, and a space is how `:delmarks a b` separates names.
    let lnum = match c_int::from(name) {
        34 => buf.last_cursor().lnum(),
        94 => buf.last_insert().lnum(),
        46 => buf.last_change().lnum(),
        91 => buf.b_op_start.lnum,
        93 => buf.b_op_end.lnum,
        60 => buf.b_visual.vi_start.lnum,
        62 => buf.b_visual.vi_end.lnum,
        58 | 32 => return true,
        _ => return false,
    };
    if lnum != 0 {
        // SAFETY: `gone` and `buf` are the caller's, both live.
        unsafe { do_markset_autocmd(name, &raw mut *gone, buf.raw()) };
    }
    // The three fmark stores are released; the four positions are only
    // invalidated, because they own nothing.
    match c_int::from(name) {
        // SAFETY: the store is live and its allocation is the buffer's.
        34 => unsafe { clear_fmark(buf.last_cursor().raw(), timestamp) },
        // SAFETY: as above.
        94 => unsafe { clear_fmark(buf.last_insert().raw(), timestamp) },
        // SAFETY: as above.
        46 => unsafe { clear_fmark(buf.last_change().raw(), timestamp) },
        91 => buf.b_op_start.lnum = 0,
        93 => buf.b_op_end.lnum = 0,
        60 => buf.b_visual.vi_start.lnum = 0,
        _ => buf.b_visual.vi_end.lnum = 0,
    }
    true
}

/// Return the line at mark "mp".  Truncate to fit in window.
/// The returned string has been allocated.
///
/// # Safety
/// `curbuf` must be live, which it is from startup to exit.
pub(super) unsafe fn mark_line(pos: pos_T, lead_len: c_int) -> *mut c_char {
    // SAFETY: `curbuf` is live from startup to exit.
    let buf = unsafe { Buf::current() };
    if pos.lnum == 0 || pos.lnum > buf.b_ml.ml_line_count {
        // SAFETY: a `'static` C string.
        return unsafe { xstrdup(c"-invalid-".as_ptr()) };
    }
    debug_assert!(Columns.get() >= 0, "Columns >= 0");
    // Five bytes per column is the widest a displayed cell can be, so the
    // copy can never be shorter than what the truncation below keeps.
    // SAFETY: `pos.lnum` is a line of the current buffer.
    let s = unsafe {
        xstrnsave(
            skipwhite(ml_get(pos.lnum)),
            size_t::try_from(Columns.get()).unwrap_or(0) * 5,
        )
    };
    // Truncate to the screen width, measured in CELLS rather than bytes so a
    // multi-byte or double-width character is not cut in half.
    let mut p = s;
    let mut len: c_int = 0;
    // SAFETY: `s` is a NUL-terminated allocation, so the walk stops at its
    // end; `utfc_ptr2len` never steps past the NUL.
    unsafe {
        while c_int::from(*p) != NUL {
            len += ptr2cells(p);
            if len >= Columns.get() - lead_len {
                break;
            }
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        *p = NUL_BYTE;
    }
    s
}

/// Get name of file from a filemark.
/// When it's in the current buffer, return the text at the mark.
/// Returns an allocated string.
///
/// # Safety
/// `fmark` must point at a live `fmark_T` and the editor's globals must be
/// live.
pub unsafe fn fm_getname(fmark: *mut fmark_T, lead_len: c_int) -> *mut c_char {
    // SAFETY: the caller promised a live record; `curbuf` is live.
    let (fm, buf) = unsafe { (Fmark::new(fmark), Buf::current()) };
    if fm.fnum() == buf.handle {
        // SAFETY: `curbuf` is live.
        return unsafe { mark_line(fm.pos(), lead_len) };
    }
    buflist_nr2name(fm.fnum(), 0, 1)
}
