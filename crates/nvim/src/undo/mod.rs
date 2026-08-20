//! Undo and redo: the tree of changes a buffer has been through.
//!
//! A change the user makes is recorded against an *undo header*, which
//! names the lines it is about to destroy; the headers form a tree, since
//! changing something after an undo forks a new branch rather than throwing
//! the old one away. [`store`] owns the headers and says how one names
//! another, [`tree`] frees them, [`apply`] walks the tree and swaps text
//! back, and [`file`]/[`read`]/[`write`] put the whole thing on disk.
//!
//! Original: `src/nvim/undo.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::autocmd::{block_autocmds, unblock_autocmds};
use crate::buffer::{bt_dontwrite, bt_prompt, buf_is_empty};
use crate::buffer_updates::{buf_updates_changedtick, buf_updates_unload};
use crate::change::{
    change_warning, changed, changed_bytes, changed_lines, file_ff_differs, unchanged,
};
use crate::cursor::{
    check_cursor, check_cursor_col, check_cursor_lnum, check_pos, coladvance, getviscol,
};
use crate::drawscreen::redraw_win_line;
use crate::edit::beginline;
use crate::eval::funcs::get_buf_arg;
use crate::eval::typval::{
    tv_dict_add_list, tv_dict_add_nr, tv_dict_alloc, tv_dict_alloc_ret, tv_get_string,
    tv_list_alloc, tv_list_append_dict,
};
use crate::event::libuv::uv_strerror;
use crate::ex_docmd::expr_map_locked;
use crate::ex_getln::{text_locked, text_locked_msg};
use crate::extmark::{extmark_apply_undo, extmark_splice_cols};
use crate::fileio::{get2c, get4c, get8ctime, read_eintr};
use crate::fold::fold_open_cursor;
use crate::getchar::beep_flush;
use crate::global_cell::GlobalCell;
use crate::main::{
    KeyTyped, VIsual, VIsual_active, curbuf, curwin, e_modifiable, e_sandbox, e_textlock,
    fdo_flags, firstbuf, global_busy, got_int, no_u_sync, p_fs, p_udir, p_ul, p_verbose, sandbox,
    textlock,
};
use crate::mark::{free_fmark, mark_adjust, setpcmark};
use crate::mbyte::utfc_ptr2len;
use crate::memline::{ml_append_flags, ml_delete, ml_get, ml_get_buf, ml_replace, resolve_symlink};
use crate::memory::{time_to_bytes, xfree, xmalloc, xmallocz, xrealloc, xstrdup};
use crate::message::{
    emsg, give_warning, iemsg, internal_error, messaging, msg, msg_end, msg_ext_set_kind,
    msg_putchar, msg_puts, msg_puts_hl, msg_start, verb_msg, verbose_enter, verbose_leave,
};
use crate::option::copy_option_part;
use crate::options::kOptFdoFlagUndo;
use crate::os::cshim::{getc, gettext, ngettext};
use crate::os::fs::{
    os_fchown, os_fileinfo, os_fopen, os_free_acl, os_fsync, os_get_acl, os_getperm, os_isdir,
    os_mkdir_recurse, os_open, os_path_exists, os_remove, os_set_acl, os_setperm,
};
use crate::os::input::fast_breakcheck;
use crate::os::time::{os_localtime_r, os_time, tm_zeroed};
use crate::path::{concat_fnames, full_name_save, path_tail, vim_ispathsep};
use crate::pos::clearpos;
use crate::sha256::{SHA256_SUM_SIZE, Sha256};
use crate::spell::spell_check_window;
use crate::state::virtual_active;
use crate::strings::vim_snprintf;
use crate::types::*;
use ::libc::{
    close, fclose, fdopen, fflush, fread, fwrite, getuid, strcmp, strftime, strlen, time,
};
use core::ffi::{c_char, c_int, c_uint, c_ulong, c_void};
use core::ptr;

/// Constants the transpiler copied in from the headers this module includes.
mod header {
    use super::{ExtmarkOp, UndoObjectType, c_int, c_ulong};

    pub const EOF: c_int = -1;
    pub const SIZE_MAX: c_ulong = 18446744073709551615;
    pub const NMARKS: c_int = 26;

    /// `open` flags.
    pub const O_RDONLY: c_int = 0;
    pub const O_WRONLY: c_int = 0o1;
    pub const O_CREAT: c_int = 0o100;
    pub const O_EXCL: c_int = 0o200;
    pub const O_NOFOLLOW: c_int = 0o400000;

    /// `ml_append`/`ml_delete` flags.
    pub const ML_EMPTY: c_int = 0x1;

    /// The `u_header_T::uh_flags` bits.
    pub const UH_CHANGED: c_int = 1;
    pub const UH_EMPTYBUF: c_int = 2;
    pub const UH_RELOAD: c_int = 4;

    pub const kExtmarkNOOP: ExtmarkOp = 0;
    pub const kExtmarkUndo: ExtmarkOp = 1;
    pub const kExtmarkSplice: UndoObjectType = 0;
    pub const kExtmarkMove: UndoObjectType = 1;
}
use header::*;

mod apply;
mod eval;
mod file;
pub mod format;
mod read;
pub mod store;
mod tree;
mod write;

use crate::winlayer::Buf;
use store::{Header, header_adopt, header_chain};
use tree::*;

pub use apply::{u_redo, u_undo, u_undo_and_forget, undo_time};
pub use eval::{ex_undolist, f_undofile, f_undotree, u_force_get_undo_header};
pub use file::{u_compute_hash, u_get_undo_file_name};
pub use read::u_read_undo;
pub use tree::{u_blockfree, u_clearall, u_clearallandblockfree, u_clearline, u_undoline};
pub use write::u_write_undo;

/// The length of an undo file's buffer hash, in bytes: a SHA-256 digest.
pub const UNDO_HASH_SIZE: c_int = 32;

/// Says something about an undo file, quietly.
///
/// A read or write the editor decided on by itself (`automatic`) reports
/// only under `'verbose'`, and inside a `verbose_enter`/`verbose_leave` pair;
/// one the user asked for by name reports outright.
///
/// # Safety
///
/// `say` emits messages, so there must be a live message state.
pub(crate) unsafe fn verbosely(automatic: bool, say: impl FnOnce()) {
    if automatic && p_verbose.get() <= 0 {
        return;
    }
    if automatic {
        // SAFETY: nothing here holds a borrow of the message state.
        unsafe { verbose_enter() };
    }
    say();
    if automatic {
        // SAFETY: as above.
        unsafe { verbose_leave() };
    }
}

#[derive(Copy, Clone)]
pub struct bufinfo_T {
    pub bi_buf: *mut buf_T,
    pub bi_fp: *mut FILE,
}
pub const NO_LOCAL_UNDOLEVEL: c_int = -123456;
static u_newcount: GlobalCell<c_int> = GlobalCell::new(0);
static u_oldcount: GlobalCell<c_int> = GlobalCell::new(0);
static undo_undoes: GlobalCell<bool> = GlobalCell::new(false);
static lastmark: GlobalCell<c_int> = GlobalCell::new(0);
/// Undo could not record the lines a change is about to touch, so the change
/// must not be made.
///
/// The `u_save*` family is `pub` with 64 call sites across the tree and still
/// answers `OK`/`FAIL`; [`saved`] is the conversion a converted caller puts
/// at the call, and the domain error of whatever is being edited absorbs this
/// one with a `From` impl (see `ops::delete::NotDeleted`). Converting the
/// family itself is a later job.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UndoFailed;

/// One of the `u_save*` answers as a `Result`.
///
/// The conversion belongs at the call and nowhere else, exactly as
/// `eval::typval::added` does for the `tv_dict_add_*` family.
pub fn saved(status: c_int) -> Result<(), UndoFailed> {
    if status == FAIL {
        Err(UndoFailed)
    } else {
        Ok(())
    }
}

/// Saves the lines a change about to be made to the cursor's line would
/// destroy.
///
/// # Safety
///
/// A live current buffer and window.
pub unsafe fn u_save_cursor() -> c_int {
    // SAFETY: a live current window, by the contract above.
    let cur: linenr_T = unsafe { (*curwin.get()).w_cursor.lnum };
    // SAFETY: a live current buffer, by the contract above.
    unsafe { u_save((cur - 1).max(0), cur + 1) }
}

/// [`u_save_buf`] for the current buffer.
///
/// # Safety
///
/// A live current buffer and window.
pub unsafe fn u_save(top: linenr_T, bot: linenr_T) -> c_int {
    // SAFETY: a live current buffer, by the contract above.
    unsafe { u_save_buf(curbuf.get(), top, bot) }
}

/// Saves the lines strictly between `top` and `bot` — the lines a change
/// about to be made would destroy.
///
/// # Safety
///
/// `buf` points at a live buffer, and there is a live current window.
pub unsafe fn u_save_buf(buf: *mut buf_T, top: linenr_T, bot: linenr_T) -> c_int {
    // SAFETY: a live buffer, by the contract above.
    let b = unsafe { Buf::new(buf) };
    if top >= bot || bot > b.line_count() + 1 {
        return FAIL;
    }
    // SAFETY: a live buffer and window, by the contract above.
    unsafe {
        if top + 2 == bot {
            // A single line: `U` can put it back.
            u_saveline(buf, top + 1);
        }
        u_savecommon(buf, top, bot, 0, false)
    }
}

/// Saves the line a `:substitute` is about to replace.
///
/// # Safety
///
/// A live current buffer and window.
pub unsafe fn u_savesub(lnum: linenr_T) -> c_int {
    // SAFETY: a live current buffer and window, by the contract above.
    unsafe { u_savecommon(curbuf.get(), lnum - 1, lnum + 1, lnum + 1, false) }
}

/// Saves the position a `:substitute` is about to insert a line at.
///
/// # Safety
///
/// A live current buffer and window.
pub unsafe fn u_inssub(lnum: linenr_T) -> c_int {
    // SAFETY: as [`u_savesub`].
    unsafe { u_savecommon(curbuf.get(), lnum - 1, lnum, lnum + 1, false) }
}

/// Saves the `nlines` lines from `lnum` that are about to be deleted.
///
/// # Safety
///
/// A live current buffer and window.
pub unsafe fn u_savedel(lnum: linenr_T, nlines: linenr_T) -> c_int {
    // SAFETY: a live current buffer and window, by the contract above.
    unsafe {
        let whole_buffer = nlines == (*curbuf.get()).b_ml.ml_line_count;
        u_savecommon(
            curbuf.get(),
            lnum - 1,
            lnum + nlines,
            if whole_buffer { 2 } else { lnum },
            false,
        )
    }
}

/// Whether `buf` may be changed at all, saying why not when it may not.
///
/// # Safety
///
/// `buf` points at a live buffer.
pub unsafe fn undo_allowed(buf: *mut buf_T) -> bool {
    // SAFETY: a live buffer, by the contract above, and three NUL-terminated
    // message literals.
    unsafe {
        if (*buf).b_p_ma == 0 {
            emsg(gettext(&raw const e_modifiable as *const c_char));
            return false;
        }
        if sandbox.get() != 0 {
            emsg(gettext(&raw const e_sandbox as *const c_char));
            return false;
        }
        if textlock.get() != 0 || expr_map_locked() {
            emsg(gettext(&raw const e_textlock as *const c_char));
            return false;
        }
        true
    }
}

/// `'undolevels'` for `buf`: its own, or the global one when it has none.
///
/// # Safety
///
/// `buf` points at a live buffer.
unsafe fn get_undolevel(buf: *mut buf_T) -> OptInt {
    // SAFETY: a live buffer, by the contract above.
    let local = unsafe { (*buf).b_p_ul };
    if local == OptInt::from(NO_LOCAL_UNDOLEVEL) {
        return p_ul.get();
    }
    local
}

/// Drops the shada payload hanging off a buffer's named marks.
///
/// The undo header takes a *copy* of the marks, and two owners of one
/// `additional_data` allocation is one too many; the header's copies are the
/// ones that keep it, so the buffer's give it up.
///
/// # Safety
///
/// `fmarks` points at [`NMARKS`] live marks.
#[inline]
unsafe fn zero_fmark_additional_data(fmarks: &mut [fmark_T; NMARKS as usize]) {
    for mark in fmarks {
        // SAFETY: this module's own allocation, dropped exactly once.
        unsafe { xfree(mark.additional_data.cast()) };
        mark.additional_data = ptr::null_mut();
    }
}

/// Records the lines between `top` and `bot` so that a change about to be
/// made to them can be undone.
///
/// `newbot`, when it is not zero, is the line the change will leave below
/// the region; zero means work it out afterwards. `reload` is a change the
/// editor is making on the user's behalf (a file reload), which skips the
/// "may this buffer be changed" question and marks the header.
///
/// # Safety
///
/// `buf` points at a live buffer, and there is a live current window.
pub unsafe fn u_savecommon(
    buf: *mut buf_T,
    top: linenr_T,
    bot: linenr_T,
    newbot: linenr_T,
    reload: bool,
) -> c_int {
    // SAFETY: a live buffer, by the contract above.
    let b = unsafe { Buf::new(buf) };
    if !reload {
        // SAFETY: a live buffer and window, by the contract above.
        unsafe {
            if !undo_allowed(buf) {
                return FAIL;
            }
            if core::ptr::eq(buf, curbuf.get()) {
                change_warning(buf, 0);
            }
            if bot > b.line_count() + 1 {
                emsg(gettext(c"E881: Line count changed unexpectedly".as_ptr()));
                return FAIL;
            }
        }
    }
    let size: linenr_T = bot - top - 1;
    if b.b_u_synced {
        // A boundary: this change starts an undo header of its own.
        // SAFETY: a live buffer and window.
        if !unsafe { start_new_header(b) } {
            return OK;
        }
    } else {
        // SAFETY: a live buffer.
        if unsafe { get_undolevel(buf) } < 0 {
            return OK;
        }
        // SAFETY: a live buffer.
        unsafe {
            if size == 1 && extend_last_entry(b, top, bot, newbot) {
                return OK;
            }
            u_getbot(buf);
        }
    }
    // SAFETY: a live buffer and window, and a newest header to record
    // against — either the one just started or the one being extended.
    unsafe { record_entry(b, top, size, bot, newbot, reload) }
}

/// Starts a new undo header for the change about to be made, trimming the
/// tree back to `'undolevels'` first.
///
/// Answers whether there is a header now: undo turned off for this buffer
/// makes none, and the branch the cursor was on goes instead.
///
/// # Safety
///
/// `b` is a live buffer, and there is a live current window.
unsafe fn start_new_header(mut b: Buf) -> bool {
    let buf = b.raw();
    b.b_new_change = true;

    // The sequence number is the header's name, so it has to be handed out
    // and the header handed to the store *before* anything can link to it.
    // The transpiled code did this at the end, when a link was a pointer and
    // the number was only for the undo file.
    // SAFETY: a live buffer, by the contract above.
    let fresh = if unsafe { get_undolevel(buf) } >= 0 {
        // `.max(0)`: a sequence number is a header's name and 0 means "no
        // link", so it has to be positive. `b_u_seq_last` comes out of the
        // undo file unvalidated and a corrupt one can make it negative.
        b.b_u_seq_last = b.b_u_seq_last.max(0) + 1;
        // SAFETY: a fresh allocation, written before anything reads it, and
        // handed straight to the store that owns it from here on.
        unsafe {
            let uhp: *mut u_header_T = xmalloc(size_of::<u_header_T>()).cast();
            uhp.write(u_header_T {
                uh_seq: b.b_u_seq_last,
                ..Default::default()
            });
            let link = header_adopt(buf, uhp);
            Header::new(uhp).map(|uh| (uh, link))
        }
    } else {
        None
    };

    let mut old_curhead = b.b_u_curhead;
    if let Some(old_cur) = b.header(old_curhead) {
        // We were somewhere up the branch; the change forks from here.
        b.b_u_newhead = old_cur.uh_next;
        b.b_u_curhead = UndoLink::NONE;
    }
    // SAFETY: a live buffer, and every header freed here is one the tree
    // still holds; `old_curhead` is cleared for us if it goes.
    unsafe {
        while OptInt::from(b.b_u_numhead) > get_undolevel(buf) {
            let Some(oldest) = b.header(b.b_u_oldhead) else {
                break;
            };
            if b.b_u_oldhead == old_curhead {
                u_freebranch(buf, oldest.raw(), &raw mut old_curhead);
            } else if oldest.uh_alt_next.is_none() {
                u_freeheader(buf, oldest.raw(), &raw mut old_curhead);
            } else {
                // The far end of the oldest header's alternate chain.
                let far = header_chain(buf, b.b_u_oldhead, |uh| uh.uh_alt_next).last();
                u_freebranch(buf, far.unwrap_or(oldest).raw(), &raw mut old_curhead);
            }
        }
    }

    let Some((mut uhp, link)) = fresh else {
        // Undo is off for this buffer, so the branch we forked from has
        // nothing left to lead back to.
        if let Some(old_cur) = b.header(old_curhead) {
            // SAFETY: a live buffer and a header the tree holds.
            unsafe { u_freebranch(buf, old_cur.raw(), ptr::null_mut()) };
        }
        b.b_u_synced = false;
        return false;
    };

    uhp.uh_next = b.b_u_newhead;
    uhp.uh_alt_next = old_curhead;
    if let Some(mut old_cur) = b.header(old_curhead) {
        // The new header joins the run of alternates in front of the one we
        // forked from.
        uhp.uh_alt_prev = old_cur.uh_alt_prev;
        if let Some(mut alt_prev) = b.header(uhp.uh_alt_prev) {
            alt_prev.uh_alt_next = link;
        }
        old_cur.uh_alt_prev = link;
        if b.b_u_oldhead == old_curhead {
            b.b_u_oldhead = link;
        }
    }
    if let Some(mut newhead) = b.header(b.b_u_newhead) {
        newhead.uh_prev = link;
    }
    b.b_u_seq_cur = uhp.uh_seq;
    // SAFETY: the C clock, and a live current window.
    unsafe {
        uhp.uh_time = time(ptr::null_mut());
        uhp.uh_cursor = (*curwin.get()).w_cursor;
        uhp.uh_cursor_vcol = if virtual_active(curwin.get()) && (*curwin.get()).w_cursor.coladd > 0
        {
            getviscol()
        } else {
            -1
        };
    }
    b.b_u_time_cur = uhp.uh_time + 1;
    uhp.uh_flags = if b.b_changed != 0 { UH_CHANGED } else { 0 }
        | if b.b_ml.ml_flags & ML_EMPTY != 0 {
            UH_EMPTYBUF
        } else {
            0
        };
    // SAFETY: the buffer's own marks; the header takes the copies over.
    unsafe { zero_fmark_additional_data(&mut b.b_namedm) };
    uhp.uh_namedm = b.b_namedm;
    uhp.uh_visual = b.b_visual;
    b.b_u_newhead = link;
    if b.b_u_oldhead.is_none() {
        b.b_u_oldhead = link;
    }
    b.b_u_numhead += 1;
    true
}

/// Folds a one-line change into an entry the newest header already holds,
/// when it covers the same line. Answers whether it did.
///
/// Only the ten newest entries are looked at: this is a fast path for typing,
/// not a search.
///
/// # Safety
///
/// `b` is a live buffer, and there is a live current window.
unsafe fn extend_last_entry(mut b: Buf, top: linenr_T, bot: linenr_T, newbot: linenr_T) -> bool {
    // SAFETY: a live buffer, by the contract above.
    let mut uep = unsafe { u_get_headentry(b.raw()) };
    let Some(mut newhead) = b.header(b.b_u_newhead) else {
        return false;
    };
    let mut prev_uep: *mut u_entry_T = ptr::null_mut();
    // SAFETY: every entry here belongs to the newest header's list, which
    // this walks one node at a time and does not free.
    unsafe {
        for i in 0..10 {
            if uep.is_null() {
                return false;
            }
            // The entry has to still describe the buffer as it stands.
            let stale = if newhead.uh_getbot_entry == uep {
                (*uep).ue_lcount != b.line_count()
            } else {
                let below = if (*uep).ue_bot == 0 {
                    b.line_count() + 1
                } else {
                    (*uep).ue_bot
                };
                (*uep).ue_top + (*uep).ue_size + 1 != below
            };
            // ... and the line must not already be inside a multi-line entry.
            let covered = (*uep).ue_size > 1
                && top >= (*uep).ue_top
                && top + 2 <= (*uep).ue_top + (*uep).ue_size + 1;
            if stale || covered {
                return false;
            }
            if (*uep).ue_size == 1 && (*uep).ue_top == top {
                if i > 0 {
                    // Move it to the front of the list, so the next change to
                    // the same line finds it first.
                    u_getbot(b.raw());
                    b.b_u_synced = false;
                    (*prev_uep).ue_next = (*uep).ue_next;
                    (*uep).ue_next = newhead.uh_entry;
                    newhead.uh_entry = uep;
                }
                set_entry_bottom(b, &mut newhead, uep, bot, newbot);
                return true;
            }
            prev_uep = uep;
            uep = (*uep).ue_next;
        }
    }
    false
}

/// Records where the change leaves the bottom of the entry's region: the
/// line the caller named, the "to the end of the buffer" sentinel, or a
/// promise to work it out in `u_getbot`.
///
/// # Safety
///
/// `uep` points at a live entry of `newhead`'s list.
unsafe fn set_entry_bottom(
    b: Buf,
    newhead: &mut Header,
    uep: *mut u_entry_T,
    bot: linenr_T,
    newbot: linenr_T,
) {
    // SAFETY: a live entry, by the contract above.
    unsafe {
        if newbot != 0 {
            (*uep).ue_bot = newbot;
        } else if bot > b.line_count() {
            // The change reaches the end of the buffer, and `ue_bot` is the
            // sentinel for that rather than a line number.
            (*uep).ue_bot = 0;
        } else {
            (*uep).ue_lcount = b.line_count();
            newhead.uh_getbot_entry = uep;
        }
    }
}

/// Builds the entry that holds the saved lines and hangs it off the newest
/// header.
///
/// # Safety
///
/// `b` is a live buffer holding lines `top + 1 ..= top + size`, and there is
/// a newest header to record against.
unsafe fn record_entry(
    mut b: Buf,
    top: linenr_T,
    size: linenr_T,
    bot: linenr_T,
    newbot: linenr_T,
    reload: bool,
) -> c_int {
    let mut newhead = b
        .header(b.b_u_newhead)
        .expect("the newest header is the one this change is recorded against");
    // SAFETY: a fresh allocation, written before anything reads it.
    let uep: *mut u_entry_T = unsafe {
        let uep: *mut u_entry_T = xmalloc(size_of::<u_entry_T>()).cast();
        uep.write(u_entry_T {
            ue_size: size,
            ue_top: top,
            ..Default::default()
        });
        uep
    };
    // SAFETY: the entry just built, and its own array.
    unsafe {
        set_entry_bottom(b, &mut newhead, uep, bot, newbot);
        if size > 0 {
            let array: *mut *mut c_char = xmalloc(size_of::<*mut c_char>() * size as size_t).cast();
            (*uep).ue_array = array;
            for i in 0..size {
                fast_breakcheck();
                if got_int.get() {
                    // Only the lines already copied are there to free.
                    u_freeentry(uep, i);
                    return FAIL;
                }
                *array.add(i as size_t) = u_save_line_buf(b.raw(), top + 1 + i);
            }
        }
        (*uep).ue_next = newhead.uh_entry;
    }
    newhead.uh_entry = uep;
    if reload {
        newhead.uh_flags |= UH_RELOAD;
    }
    b.b_u_synced = false;
    undo_undoes.set(false);
    OK
}

/// Writes when `tt` was into `buf`: a clock time for anything older than a
/// hundred seconds, and "N seconds ago" for the rest.
///
/// # Safety
///
/// `buf` points at `buflen` writable bytes.
pub unsafe fn undo_fmt_time(buf: *mut c_char, buflen: size_t, tt: time_t) {
    // SAFETY: `buflen` writable bytes and NUL-terminated format literals, by
    // the contract above.
    unsafe {
        let age = time(ptr::null_mut()) - tt;
        if age < 100 {
            let seconds = int64_t::from(age);
            vim_snprintf(
                buf,
                buflen,
                ngettext(
                    c"%ld second ago".as_ptr(),
                    c"%ld seconds ago".as_ptr(),
                    c_ulong::from(seconds as uint32_t),
                ),
                seconds,
            );
            return;
        }
        let mut when: tm = tm_zeroed();
        os_localtime_r(tt, &mut when);
        let format = if age < 60 * 60 * 12 {
            c"%H:%M:%S".as_ptr()
        } else {
            c"%Y/%m/%d %H:%M:%S".as_ptr()
        };
        if strftime(buf, buflen, format, &raw mut when) == 0 {
            *buf = NUL as c_char;
        }
    }
}

/// Closes the current undo header, so that the next change starts a new one.
///
/// `force` overrides the `no_u_sync` block that a mapping or a script sets
/// while it is running.
///
/// # Safety
///
/// A live current buffer.
pub unsafe fn u_sync(force: bool) {
    // SAFETY: a live current buffer, by the contract above.
    let mut b = unsafe { Buf::current() };
    if b.b_u_synced || (!force && no_u_sync.get() > 0) {
        return;
    }
    // SAFETY: a live current buffer.
    if unsafe { get_undolevel(b.raw()) } < 0 {
        b.b_u_synced = true;
        return;
    }
    // SAFETY: as above.
    unsafe { u_getbot(b.raw()) };
    b.b_u_curhead = UndoLink::NONE;
}

/// `:undojoin` — fold the next change into the header the last one made.
///
/// # Safety
///
/// A live current buffer.
pub unsafe fn ex_undojoin(_eap: *mut exarg_T) {
    // SAFETY: a live current buffer, by the contract above.
    let mut b = unsafe { Buf::current() };
    if b.b_u_newhead.is_none() {
        return;
    }
    if b.b_u_curhead.is_some() {
        // SAFETY: a NUL-terminated literal.
        unsafe {
            emsg(gettext(
                c"E790: undojoin is not allowed after undo".as_ptr(),
            ));
        }
        return;
    }
    // SAFETY: a live current buffer.
    if !b.b_u_synced || unsafe { get_undolevel(b.raw()) } < 0 {
        return;
    }
    b.b_u_synced = false;
}

/// Marks the whole tree as describing an unchanged buffer, which is what
/// writing the file out makes true.
///
/// # Safety
///
/// `buf` points at a live buffer.
pub unsafe fn u_unchanged(buf: *mut buf_T) {
    // SAFETY: a live buffer, by the contract above.
    unsafe {
        let mut b = Buf::new(buf);
        u_unch_branch(buf, b.b_u_oldhead);
        b.b_did_warn = false;
    }
}

/// Moves the newest header's cursor to the first line that actually differs
/// from what it saved, so that undoing lands where the change was.
///
/// # Safety
///
/// A live current buffer.
pub unsafe fn u_find_first_changed() {
    // SAFETY: a live current buffer, by the contract above.
    let b = unsafe { Buf::current() };
    let Some(mut uhp) = b.header(b.b_u_newhead).filter(|_| b.b_u_curhead.is_none()) else {
        return;
    };
    let uep: *mut u_entry_T = uhp.uh_entry;
    // SAFETY: the newest header's own entry list, and its saved lines.
    unsafe {
        if (*uep).ue_top != 0 || (*uep).ue_bot != 0 {
            // Not a whole-buffer entry: there is nothing to line up against.
            return;
        }
        let mut lnum: linenr_T = 1;
        while lnum < b.line_count() && lnum <= (*uep).ue_size {
            if strcmp(
                ml_get_buf(b.raw(), lnum),
                *(*uep).ue_array.offset((lnum - 1) as isize),
            ) != 0
            {
                clearpos(&mut uhp.uh_cursor);
                uhp.uh_cursor.lnum = lnum;
                return;
            }
            lnum += 1;
        }
        if b.line_count() != (*uep).ue_size {
            clearpos(&mut uhp.uh_cursor);
            uhp.uh_cursor.lnum = lnum;
        }
    }
}

/// Numbers the change the buffer currently sits on as the Nth file write, so
/// that `:earlier 1f` can find it again.
///
/// # Safety
///
/// `buf` points at a live buffer.
pub unsafe fn u_update_save_nr(buf: *mut buf_T) {
    // SAFETY: a live buffer, by the contract above.
    let mut b = unsafe { Buf::new(buf) };
    b.b_u_save_nr_last += 1;
    b.b_u_save_nr_cur = b.b_u_save_nr_last;
    let above = match b.header(b.b_u_curhead) {
        Some(curhead) => b.header(curhead.uh_next),
        None => b.header(b.b_u_newhead),
    };
    if let Some(mut uhp) = above {
        uhp.uh_save_nr = b.b_u_save_nr_last;
    }
}

/// Whether `buf` holds changes that writing it out would save.
///
/// # Safety
///
/// `buf` points at a live buffer.
pub unsafe fn buf_is_changed(buf: *mut buf_T) -> bool {
    // SAFETY: a live buffer, by the contract above.
    unsafe {
        if bt_prompt(buf) {
            return (*buf).b_modified_was_set;
        }
        !bt_dontwrite(buf) && ((*buf).b_changed != 0 || file_ff_differs(buf, true))
    }
}

/// Whether any buffer at all holds unsaved changes.
///
/// # Safety
///
/// A live buffer list.
pub unsafe fn any_buf_is_changed() -> bool {
    // SAFETY: the buffer list, walked to its NULL end.
    unsafe {
        let mut buf: *mut buf_T = firstbuf.get();
        while !buf.is_null() {
            if buf_is_changed(buf) {
                return true;
            }
            buf = (*buf).b_next;
        }
    }
    false
}

/// [`buf_is_changed`] for the current buffer.
///
/// # Safety
///
/// A live current buffer.
pub unsafe fn curbuf_is_changed() -> bool {
    // SAFETY: a live current buffer, by the contract above.
    unsafe { buf_is_changed(curbuf.get()) }
}
