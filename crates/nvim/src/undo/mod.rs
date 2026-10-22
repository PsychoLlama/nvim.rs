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

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::autocmd::{block_autocmds, unblock_autocmds};
use crate::buffer::{buf_is_dontwrite, buf_is_empty, buf_is_prompt};
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
use crate::eval::typval::{tv_dict_alloc, tv_dict_alloc_ret, tv_list_alloc};
use crate::ex_docmd::expr_map_locked;
use crate::ex_docmd::state::global_busy;
use crate::ex_getln::{text_locked, text_locked_msg};
use crate::extmark::{extmark_apply_undo, extmark_splice_cols};
use crate::fold::fold_open_cursor;
use crate::formatted;
use crate::getchar::beep_flush;
use crate::getchar::state::{KeyTyped, got_int};
use crate::global_cell::GlobalCell;
use crate::guard::{sandbox, textlock};
use crate::mark::{mark_adjust, setpcmark};
use crate::mbyte::c_cluster_len;
use crate::memline::MlFlags;
use crate::memline::{ml_delete, resolve_symlink};
use crate::memory::XString;
use crate::message::{e_modifiable, e_sandbox, e_textlock};
use crate::message::{
    emsg, give_warning, iemsg, internal_error, messaging, msg, msg_end, msg_ext_set_kind,
    msg_putchar, msg_start, msg_str, msg_str_hl, verb_msg, verbose_enter, verbose_leave,
};
use crate::option::vars::{fdo_flags, p_fs, p_ul, p_verbose};
use crate::options::kOptFdoFlagUndo;
use crate::os::cshim::{gettext, ngettext};
use crate::os::fs::{os_free_acl, os_get_acl, os_remove, os_set_acl};
use crate::os::input::fast_breakcheck;
use crate::os::time::{os_localtime_r, os_strftime, os_time, os_time_raw, tm_zeroed};
use crate::path::{concat_fnames, full_name_save, tail_index, vim_ispathsep};
use crate::pos::clearpos;
use crate::sha256::Sha256;
use crate::spell::spell_check_window;
use crate::state::virtual_active;
use crate::types::Failed;
use crate::types::*;
use crate::winlayer::Win;
use core::ffi::{c_int, c_uint, c_ulong};
use core::ptr;

/// Constants the transpiler copied in from the headers this module includes.
mod header {
    use super::{ExtmarkOp, UndoObjectType, c_int};

    pub(super) const EOF: c_int = -1;
    pub(super) const NMARKS: c_int = 26;

    /// The `UndoHeader::flags` bits.
    pub(super) const UH_CHANGED: c_int = 1;
    pub(super) const UH_EMPTYBUF: c_int = 2;
    pub(super) const UH_RELOAD: c_int = 4;

    pub(super) const kExtmarkNOOP: ExtmarkOp = 0;
    pub(super) const kExtmarkUndo: ExtmarkOp = 1;
    pub(super) const kExtmarkSplice: UndoObjectType = 0;
    pub(super) const kExtmarkMove: UndoObjectType = 1;
}
use header::*;

mod apply;
mod bufinfo;
mod eval;
mod file;
pub mod format;
mod read;
pub mod store;
mod tree;
mod write;

use crate::winlayer::{Buf, buffers};
use store::{header_adopt, header_chain};
use tree::*;

pub use apply::{u_redo, u_undo, u_undo_and_forget, undo_time};
pub(crate) use bufinfo::BufInfo;
pub use eval::{ex_undolist, f_undofile, f_undotree, u_force_get_undo_header};
pub use file::{u_compute_hash, u_get_undo_file_name};
pub use read::u_read_undo;
pub use tree::{u_blockfree, u_clearall, u_clearallandblockfree, u_clearline, u_undoline};
pub use write::u_write_undo;

pub(crate) static no_u_sync: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static u_sync_once: GlobalCell<c_int> = GlobalCell::new(0 as c_int);

/// The length of an undo file's buffer hash, in bytes: a SHA-256 digest.
pub const UNDO_HASH_SIZE: c_int = 32;

/// Says something about an undo file, quietly.
///
/// A read or write the editor decided on by itself (`automatic`) reports
/// only under `'verbose'`, and inside a `verbose_enter`/`verbose_leave` pair;
/// one the user asked for by name reports outright.
pub(crate) fn verbosely(automatic: bool, say: impl FnOnce()) {
    if automatic && p_verbose() <= 0 {
        return;
    }
    if automatic {
        verbose_enter();
    }
    say();
    if automatic {
        verbose_leave();
    }
}

pub const NO_LOCAL_UNDOLEVEL: c_int = -123456;
static u_newcount: GlobalCell<c_int> = GlobalCell::new(0);
static u_oldcount: GlobalCell<c_int> = GlobalCell::new(0);
static undo_undoes: GlobalCell<bool> = GlobalCell::new(false);
static lastmark: GlobalCell<c_int> = GlobalCell::new(0);
/// Undo could not record the lines a change is about to touch, so the change
/// must not be made.
///
/// The `u_save*` family answers the anonymous [`Failed`]; this is the name
/// that failure has for a caller who was about to change the buffer, and the
/// domain error of whatever is being edited absorbs it in turn with a `From`
/// impl (see `ops::delete::NotDeleted`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UndoFailed;

impl From<Failed> for UndoFailed {
    fn from(_: Failed) -> Self {
        UndoFailed
    }
}

/// Saves the lines a change about to be made to the cursor's line would
/// destroy.
///
/// Safe: as [`u_save`], over the cursor's line.
pub fn u_save_cursor() -> Result<(), Failed> {
    let cur: LineNr = Win::current().w_cursor.lnum;
    u_save((cur - 1).max(0), cur + 1)
}

/// [`u_save_buf`] for the current buffer.
///
/// Safe: the only promise is that the editor exists; `u_save_buf` validates
/// the line range itself and answers `Err` when it is out of range.
pub fn u_save(top: LineNr, bot: LineNr) -> Result<(), Failed> {
    u_save_buf(Buf::current(), top, bot)
}

/// Saves the lines strictly between `top` and `bot` — the lines a change
/// about to be made would destroy.
///
/// Safe: the line range is validated here, and `Err` is the answer for one
/// that is out of range.
pub fn u_save_buf(buffer: Buf, top: LineNr, bot: LineNr) -> Result<(), Failed> {
    if top >= bot || bot > buffer.line_count() + 1 {
        return Err(Failed);
    }
    if top + 2 == bot {
        // A single line: `U` can put it back.
        u_saveline(buffer, top + 1);
    }
    u_savecommon(buffer, top, bot, 0, false)
}

/// Saves the line a `:substitute` is about to replace.
///
/// Safe: as [`u_save`].
pub fn u_savesub(lnum: LineNr) -> Result<(), Failed> {
    u_savecommon(Buf::current(), lnum - 1, lnum + 1, lnum + 1, false)
}

/// Saves the position a `:substitute` is about to insert a line at.
///
/// Safe: as [`u_savesub`].
pub fn u_inssub(lnum: LineNr) -> Result<(), Failed> {
    u_savecommon(Buf::current(), lnum - 1, lnum, lnum + 1, false)
}

/// Saves the `nlines` lines from `lnum` that are about to be deleted.
///
/// Safe: as [`u_save`].
pub fn u_savedel(lnum: LineNr, nlines: LineNr) -> Result<(), Failed> {
    let whole_buffer = nlines == Buf::current().b_ml.ml_line_count;
    u_savecommon(
        Buf::current(),
        lnum - 1,
        lnum + nlines,
        if whole_buffer { 2 } else { lnum },
        false,
    )
}

/// Whether `buffer` may be changed at all, saying why not when it may not.
///
/// Safe: a [`Buf`] carries the whole of the promise this needs.
pub fn undo_allowed(buffer: Buf) -> bool {
    if buffer.b_p_ma == 0 {
        emsg(gettext(e_modifiable));
        return false;
    }
    if sandbox.get() != 0 {
        emsg(gettext(e_sandbox));
        return false;
    }
    if textlock.get() != 0 || expr_map_locked() {
        emsg(gettext(e_textlock));
        return false;
    }
    true
}

/// `'undolevels'` for `buffer`: its own, or the global one when it has none.
///
/// Safe: a [`Buf`] carries the whole of the promise this needs.
fn get_undolevel(buffer: Buf) -> OptInt {
    let local = buffer.b_p_ul;
    if local == OptInt::from(NO_LOCAL_UNDOLEVEL) {
        return p_ul();
    }
    local
}

/// Drops the shada payload hanging off a buffer's named marks.
///
/// The undo header takes a copy of the marks. Upstream drops the payload
/// because its copy is shallow and two owners of one allocation is one too
/// many; the copy here is deep, but the payload goes all the same, so what
/// a ShaDa file is written with stays upstream's.
#[inline]
fn drop_named_mark_extras(fmarks: &mut [FileMark; NMARKS as usize]) {
    for mark in fmarks {
        mark.extra = None;
    }
}

/// Records the lines between `top` and `bot` so that a change about to be
/// made to them can be undone.
///
/// `newbot`, when it is not zero, is the line the change will leave below
/// the region; zero means work it out afterwards. `reload` is a change the
/// editor is making on the user's behalf (a file reload), which skips the
/// "may this buffer be changed" question and marks the header.
pub fn u_savecommon(
    buffer: Buf,
    top: LineNr,
    bot: LineNr,
    newbot: LineNr,
    reload: bool,
) -> Result<(), Failed> {
    let b = buffer;
    if !reload {
        if !undo_allowed(buffer) {
            return Err(Failed);
        }
        if ptr::eq(buffer.raw(), Buf::current_raw()) {
            change_warning(buffer, 0);
        }
        if bot > b.line_count() + 1 {
            emsg(gettext(c"E881: Line count changed unexpectedly"));
            return Err(Failed);
        }
    }
    let size: LineNr = bot - top - 1;
    if b.b_u_synced {
        // A boundary: this change starts an undo header of its own.
        if !start_new_header(b) {
            return Ok(());
        }
    } else {
        if get_undolevel(buffer) < 0 {
            return Ok(());
        }
        if size == 1 && extend_last_entry(b, top, bot, newbot) {
            return Ok(());
        }
        u_getbot(buffer);
    }
    record_entry(b, top, size, bot, newbot, reload)
}

/// Starts a new undo header for the change about to be made, trimming the
/// tree back to `'undolevels'` first.
///
/// Answers whether there is a header now: undo turned off for this buffer
/// makes none, and the branch the cursor was on goes instead.
fn start_new_header(mut b: Buf) -> bool {
    b.b_new_change = true;

    // The sequence number is the header's name, so it has to be handed out
    // and the header handed to the store *before* anything can link to it.
    let fresh = if get_undolevel(b) >= 0 {
        // `.max(0)`: a sequence number is a header's name and 0 means "no
        // link", so it has to be positive. `b_u_seq_last` comes out of the
        // undo file unvalidated and a corrupt one can make it negative.
        b.b_u_seq_last = b.b_u_seq_last.max(0) + 1;
        let header = Box::new(UndoHeader {
            seq: b.b_u_seq_last,
            ..Default::default()
        });
        Some(header_adopt(b, header)).filter(|link| link.is_some())
    } else {
        None
    };

    let mut old_curhead = b.b_u_curhead;
    if let Some(next) = b.header(old_curhead).map(|uh| uh.next) {
        // We were somewhere up the branch; the change forks from here.
        b.b_u_newhead = next;
        b.b_u_curhead = UndoLink::NONE;
    }
    // Every header freed here is one the tree still holds; `old_curhead` is
    // cleared for us if it goes.
    while OptInt::from(b.b_u_numhead) > get_undolevel(b) {
        let oldest = b.b_u_oldhead;
        let Some(alt_next) = b.header(oldest).map(|uh| uh.alt_next) else {
            break;
        };
        if oldest == old_curhead {
            u_freebranch(b, oldest, Some(&mut old_curhead));
        } else if alt_next.is_none() {
            u_freeheader(b, oldest, Some(&mut old_curhead));
        } else {
            // The far end of the oldest header's alternate chain.
            let far = header_chain(b, oldest, |uh| uh.alt_next)
                .last()
                .unwrap_or(oldest);
            u_freebranch(b, far, Some(&mut old_curhead));
        }
    }

    let Some(link) = fresh else {
        // Undo is off for this buffer, so the branch we forked from has
        // nothing left to lead back to.
        if b.has_header(old_curhead) {
            u_freebranch(b, old_curhead, None);
        }
        b.b_u_synced = false;
        return false;
    };

    let next = b.b_u_newhead;
    let mut alt_prev = UndoLink::NONE;
    if let Some(old_cur) = b.header(old_curhead) {
        // The new header joins the run of alternates in front of the one we
        // forked from.
        alt_prev = old_cur.alt_prev;
        if let Some(before) = b.header_mut(alt_prev) {
            before.alt_next = link;
        }
        if let Some(old_cur) = b.header_mut(old_curhead) {
            old_cur.alt_prev = link;
        }
        if b.b_u_oldhead == old_curhead {
            b.b_u_oldhead = link;
        }
    }
    if let Some(newhead) = b.header_mut(next) {
        newhead.prev = link;
    }
    let seq = b.b_u_seq_last;
    b.b_u_seq_cur = seq;
    let win = Win::current();
    let now = os_time_raw();
    let cursor_vcol = if virtual_active(win) && win.w_cursor.coladd > 0 {
        getviscol()
    } else {
        -1
    };
    b.b_u_time_cur = now + 1;
    let flags = if b.b_changed != 0 { UH_CHANGED } else { 0 }
        | if b.b_ml.ml_flags.has(MlFlags::EMPTY) {
            UH_EMPTYBUF
        } else {
            0
        };
    drop_named_mark_extras(&mut b.b_namedm);
    let named_marks = b.b_namedm.each_ref().map(SavedMark::of);
    let visual = b.b_visual;
    if let Some(uh) = b.header_mut(link) {
        uh.next = next;
        uh.alt_next = old_curhead;
        uh.alt_prev = alt_prev;
        uh.time = now;
        uh.cursor = win.w_cursor;
        uh.cursor_vcol = cursor_vcol;
        uh.flags = flags;
        uh.named_marks = named_marks;
        uh.visual = visual;
    }
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
fn extend_last_entry(mut b: Buf, top: LineNr, bot: LineNr, newbot: LineNr) -> bool {
    if !u_get_headentry(b) {
        return false;
    }
    let newhead = b.b_u_newhead;
    let line_count = b.line_count();
    let Some(uh) = b.header(newhead) else {
        return false;
    };
    let count = uh.entries.len();
    // Newest first: the end of the vector is the head of upstream's list.
    for (i, at) in (0..count).rev().take(10).enumerate() {
        let entry = &uh.entries[at];
        // The entry has to still describe the buffer as it stands.
        let stale = if uh.getbot == Some(at) {
            entry.lcount != line_count
        } else {
            let below = if entry.bot == 0 {
                line_count + 1
            } else {
                entry.bot
            };
            entry.top + entry.size() + 1 != below
        };
        // ... and the line must not already be inside a multi-line entry.
        let covered =
            entry.size() > 1 && top >= entry.top && top + 2 <= entry.top + entry.size() + 1;
        if stale || covered {
            return false;
        }
        if entry.size() == 1 && entry.top == top {
            if i > 0 {
                // Move it to the front of the list, so the next change to
                // the same line finds it first.
                u_getbot(b);
                b.b_u_synced = false;
            }
            let Some(uh) = b.header_mut(newhead) else {
                return true;
            };
            let mut at = at;
            if i > 0 {
                let moved = uh.entries.remove(at);
                uh.entries.push(moved);
                let last = uh.entries.len() - 1;
                uh.getbot = uh.getbot.map(|g| match g {
                    g if g == at => last,
                    g if g > at => g - 1,
                    g => g,
                });
                at = last;
            }
            set_entry_bottom(uh, at, line_count, bot, newbot);
            return true;
        }
    }
    false
}

/// Records where the change leaves the bottom of entry `at`'s region: the
/// line the caller named, the "to the end of the buffer" sentinel, or a
/// promise to work it out in `u_getbot`. `line_count` is the buffer's.
fn set_entry_bottom(
    uh: &mut UndoHeader,
    at: usize,
    line_count: LineNr,
    bot: LineNr,
    newbot: LineNr,
) {
    let Some(entry) = uh.entries.get_mut(at) else {
        return;
    };
    if newbot != 0 {
        entry.bot = newbot;
    } else if bot > line_count {
        // The change reaches the end of the buffer, and `bot` is the
        // sentinel for that rather than a line number.
        entry.bot = 0;
    } else {
        entry.lcount = line_count;
        uh.getbot = Some(at);
    }
}

/// Builds the entry that holds the saved lines and hangs it off the newest
/// header.
///
/// The lines are copied before the entry joins the header, so a CTRL-C
/// during the copy leaves the header as it was; upstream had already pointed
/// `uh_getbot_entry` at the entry it then freed.
fn record_entry(
    mut b: Buf,
    top: LineNr,
    size: LineNr,
    bot: LineNr,
    newbot: LineNr,
    reload: bool,
) -> Result<(), Failed> {
    let newhead = b.b_u_newhead;
    let mut lines = Vec::with_capacity(usize::try_from(size).unwrap_or(0));
    for i in 0..size {
        fast_breakcheck();
        if got_int.get() {
            return Err(Failed);
        }
        lines.push(line_copy(b, top + 1 + i));
    }
    let entry = UndoEntry {
        top,
        lines,
        ..Default::default()
    };
    let line_count = b.line_count();
    let uh = b
        .header_mut(newhead)
        .expect("the newest header is the one this change is recorded against");
    uh.entries.push(entry);
    if reload {
        uh.flags |= UH_RELOAD;
    }
    let at = uh.entries.len() - 1;
    set_entry_bottom(uh, at, line_count, bot, newbot);
    b.b_u_synced = false;
    undo_undoes.set(false);
    Ok(())
}

/// When `tt` was, as a `room`-byte buffer would hold it: a clock time for
/// anything older than a hundred seconds, and "N seconds ago" for the rest.
/// Empty when the clock time does not fit, as `strftime` answers it.
pub(crate) fn undo_time_text(tt: time_t, room: usize) -> XString {
    let age = os_time_raw() - tt;
    if age < 100 {
        let seconds = int64_t::from(age);
        // A time in the future (a clock that moved back, an undo file from
        // a skewed machine) is a negative age, which upstream's `unsigned
        // long` cast wraps into a plural.
        let count: c_ulong = seconds.cast_unsigned();
        let template = ngettext(c"%ld second ago", c"%ld seconds ago", count);
        return XString::from_vec(formatted!(room, template.to_bytes(), seconds));
    }
    let mut when: tm = tm_zeroed();
    os_localtime_r(tt, &mut when);
    let format = if age < 60 * 60 * 12 {
        c"%H:%M:%S"
    } else {
        c"%Y/%m/%d %H:%M:%S"
    };
    let text = os_strftime(format, &when);
    if text.len() >= room {
        return XString::new();
    }
    XString::from_vec(text)
}

/// Closes the current undo header, so that the next change starts a new one.
///
/// `force` overrides the `no_u_sync` block that a mapping or a script sets
/// while it is running.
///
/// Safe: `curbuf` is set from startup to exit.
pub fn u_sync(force: bool) {
    let mut b = Buf::current();
    if b.b_u_synced || (!force && no_u_sync.get() > 0) {
        return;
    }
    if get_undolevel(b) < 0 {
        b.b_u_synced = true;
        return;
    }
    u_getbot(b);
    b.b_u_curhead = UndoLink::NONE;
}

/// `:undojoin` — fold the next change into the header the last one made.
pub fn ex_undojoin(_excmd: &mut ExArg) {
    let mut b = Buf::current();
    if b.b_u_newhead.is_none() {
        return;
    }
    if b.b_u_curhead.is_some() {
        emsg(gettext(c"E790: undojoin is not allowed after undo"));
        return;
    }
    if !b.b_u_synced || get_undolevel(b) < 0 {
        return;
    }
    b.b_u_synced = false;
}

/// Marks the whole tree as describing an unchanged buffer, which is what
/// writing the file out makes true.
pub fn u_unchanged(mut buffer: Buf) {
    u_unch_branch(buffer, buffer.b_u_oldhead);
    buffer.b_did_warn = false;
}

/// Moves the newest header's cursor to the first line that actually differs
/// from what it saved, so that undoing lands where the change was.
pub fn u_find_first_changed() {
    let mut b = Buf::current();
    let newhead = b.b_u_newhead;
    if b.b_u_curhead.is_some() {
        return;
    }
    // The head of upstream's entry list: the newest entry.
    let Some(size) = b
        .header(newhead)
        .and_then(|uh| uh.entries.last())
        .filter(|entry| entry.top == 0 && entry.bot == 0)
        .map(UndoEntry::size)
    else {
        // No entry, or not a whole-buffer one: nothing to line up against.
        return;
    };
    let mut lnum: LineNr = 1;
    let mut found = false;
    while lnum < b.line_count() && lnum <= size {
        let at = usize::try_from(lnum - 1).expect("the loop counts up from one");
        let mut lines = b.lines();
        let differs = b
            .header(newhead)
            .and_then(|uh| uh.entries.last())
            .is_some_and(|entry| *lines.line(lnum) != *entry.lines[at]);
        if differs {
            found = true;
            break;
        }
        lnum += 1;
    }
    if (found || b.line_count() != size)
        && let Some(uh) = b.header_mut(newhead)
    {
        clearpos(&mut uh.cursor);
        uh.cursor.lnum = lnum;
    }
}

/// Numbers the change the buffer currently sits on as the Nth file write, so
/// that `:earlier 1f` can find it again.
pub fn u_update_save_nr(buffer: Buf) {
    let mut b = buffer;
    b.b_u_save_nr_last += 1;
    b.b_u_save_nr_cur = b.b_u_save_nr_last;
    let above = if b.has_header(b.b_u_curhead) {
        b.header_link(b.b_u_curhead, |uh| uh.next)
    } else {
        b.b_u_newhead
    };
    let save_nr = b.b_u_save_nr_last;
    if let Some(uh) = b.header_mut(above) {
        uh.save_nr = save_nr;
    }
}

/// Whether `buffer` holds changes that writing it out would save.
///
/// Safe: a [`Buf`] carries the whole of the promise this needs.
pub fn buf_is_changed(buffer: Buf) -> bool {
    if buf_is_prompt(Some(buffer)) {
        return buffer.b_modified_was_set;
    }
    !buf_is_dontwrite(Some(buffer)) && (buffer.b_changed != 0 || file_ff_differs(buffer, true))
}

/// Whether any buffer at all holds unsaved changes.
///
/// Safe: the buffer list is the editor's own and lives from startup to exit.
pub fn any_buf_is_changed() -> bool {
    buffers().any(buf_is_changed)
}

/// [`buf_is_changed`] for the current buffer.
///
/// Safe: as [`buf_is_changed`], over the current buffer.
pub fn curbuf_is_changed() -> bool {
    buf_is_changed(Buf::current())
}

#[cfg(all(test, not(miri)))]
mod tests {
    use super::*;

    /// A timestamp from the future is a negative age, said as such, not a
    /// panic.
    #[test]
    fn a_time_in_the_future_is_a_negative_age() {
        let text = undo_time_text(os_time_raw() + 50, 80);
        let text = String::from_utf8(text.to_vec()).expect("ASCII");
        assert!(
            text.ends_with(" seconds ago") && text.starts_with('-'),
            "{text}"
        );
        // And a room too small for the clock time leaves nothing.
        assert!(undo_time_text(0, 5).is_empty());
    }
}
