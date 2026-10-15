//! The buffer list -- creating an entry and finding one.
//!
//! [`buflist_new`] is the only way a buffer joins the list: reuse an existing
//! entry for the same file if there is one, otherwise allocate, assign the
//! next buffer number, copy the option defaults and fire `BufNew`/`BufAdd`.
//! [`buflist_findpat`] is the search the command line uses -- the four-attempt
//! match over full names, then tails, then patterns -- and the
//! `buflist_findname*` group the exact-name lookups.  [`buflist_getfile`]
//! switches to an entry and puts the cursor where it was.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use crate::cstr;
use crate::types::AutoEvent;
use core::ffi::{CStr, c_int};
use core::ptr;

use super::*;
use crate::allocator::Owned;
use crate::cursor::{check_cursor_col, check_cursor_lnum};
use crate::diff::diff_mode_buf;
use crate::eval::typval::callback_free;
use crate::eval::vars::new_var_scope;
use crate::ex_cmds::getfile;
use crate::ex_docmd::tabpage_new;
use crate::ex_eval::aborting;
use crate::ex_getln::text_or_buf_locked;
use crate::fileio::file_pat_to_reg_pat;
use crate::guard::Suppress;
use crate::hashtab::hash_reset;
use crate::indent::replace_tabstop_array;
use crate::insexpand::clear_buffer_cpt_callbacks;
use crate::mark::{clrallmarks, fmarks_check_names, mark_view_restore};
use crate::message::e_noalt;
use crate::message::state::{emsg_silent, in_assert_fails};
use crate::message::{emsg, msg_delay};
use crate::message_fmt::msg_cstr;
use crate::option::vars::{jop_flags, p_sol, swb_flags};
use crate::option::{buf_copy_options, magic_isset};
use crate::options::{kOptJopFlagView, kOptSwbFlagNewtab, kOptSwbFlagSplit, kOptSwbFlagVsplit};
use crate::os::cshim::gettext;
use crate::os::fs::file_id;
use crate::path::full_name_save;
use crate::pos::MAXLNUM;
use crate::regexp::{OwnedMatch, RE_MAGIC, free_prog_in};
use crate::semsg;
use crate::types::{
    AdditionalData, Buffer, Callback, ColNr, Failed, FileID, FileMark, FileMarkView, Handle,
    LineNr, OptInt, Pos, Timestamp, VAR_SCOPE, WinInfo, int16_t, uint64_t,
};
use crate::undo::curbuf_is_changed;
use crate::window::{WSP_VERT, swbuf_goto_win_with_buf, win_split};
pub(crate) use crate::winlayer::graph::alloc_unregistered_buffer;
use crate::winlayer::graph::{firstbuf, lastbuf};
use crate::winlayer::{Buf, Win, buffers_back, register_buffer, windows};

use super::expand::buflist_match;

/// `INIT_FMARK`: a mark that has never been set.
pub(crate) const INIT_FMARK: FileMark = FileMark {
    mark: Pos {
        lnum: 0 as LineNr,
        col: 0 as ColNr,
        coladd: 0 as ColNr,
    },
    fnum: 0,
    timestamp: 0 as Timestamp,
    view: FileMarkView {
        topline_offset: MAXLNUM,
        skipcol: 0 as ColNr,
    },
    additional_data: ptr::null_mut::<AdditionalData>(),
};

// ---------------------------------------------------------------------------
// The neighbours, wrapped

fn is_diff_mode(buffer: Buf) -> bool {
    diff_mode_buf(buffer)
}

/// The current buffer, which is null only before the first one is created.
fn current_buf() -> Option<Buf> {
    Buf::current_or_none()
}

fn current_win() -> Win {
    Win::current()
}

fn current_last() -> Option<Buf> {
    last_buffer()
}

fn fire_buf_event(event: AutoEvent, buffer: Buf) -> bool {
    fire_autocmds(event, false, Some(buffer))
}

fn copy_options_into(buffer: Buf, flags: c_int) {
    buf_copy_options(buffer, flags);
}

// ---------------------------------------------------------------------------
// Creating an entry

/// [`buflist_new`] for a name the caller holds, or for none: list the file
/// as `name` with no separate short name.
pub(crate) fn buflist_add_name(name: Option<&CStr>, lnum: LineNr, flags: c_int) -> Option<Buf> {
    buflist_new(name, None, lnum, flags)
}

/// Add a file to the buffer list, or answer the entry it already has.
///
/// `ffname_arg` is the file's name and `sfname_arg` its short form (the name
/// itself when `None`); `lnum` is the line to remember for it and `flags` the
/// `BLN_*` set. Both names are copied before anything runs. The answer is
/// `None` when an autocommand deleted the buffer under us.
pub fn buflist_new(
    ffname_arg: Option<&CStr>,
    sfname_arg: Option<&CStr>,
    lnum: LineNr,
    flags: c_int,
) -> Option<Buf> {
    let names = fname_expand(ffname_arg, sfname_arg);

    // The file id works better than the name for hard links, when the file
    // exists.
    let short = names
        .as_ref()
        .map_or(sfname_arg, |(_, short)| Some(short.as_cstr()));
    let file_id = short.and_then(file_id);

    // If the file name is already in the list, update that entry.
    if let Some((ffname, _)) = &names
        && flags & (BLN_DUMMY as c_int | BLN_NEW as c_int) == 0
        && let Some(buf) = buflist_findname_file_id(ffname.as_cstr(), file_id.as_ref())
    {
        return reuse_entry(buf, lnum, flags);
    }

    // The current buffer, when it has no name and no contents, otherwise a
    // fresh one. This is the ONLY place a buffer structure is allocated.
    let mut reusable = None;
    if flags & BLN_CURBUF as c_int != 0 && curbuf_reusable() {
        let cur = current_buf().expect("curbuf != NULL");
        let bufref = BufRef::of(cur);
        trigger_undo_ftplugin(cur, current_win());
        // It is as if this buffer were deleted. Watch out for autocommands
        // that change curbuf: if that happens, allocate a new buffer anyway.
        buf_freeall(cur, BFA_WIPE as c_int | BFA_DEL as c_int);
        if aborting() {
            // Autocommands may abort script processing.
            return None;
        }
        // When the buffer was deleted, allocate a new one instead.
        reusable = bufref.get();
    }
    // Upstream re-reads `curbuf` here: `buf_freeall`'s autocommands may have
    // made another buffer current, and then this one is not reusable after
    // all.
    let reused_curbuf = reusable.is_some() && reusable == current_buf();
    // The allocation of a fresh buffer, held here until `append_to_list`
    // gives it to the registry; `None` when the current buffer is reused,
    // which the registry has owned since it was made.
    let mut fresh = None;
    let mut buf = match reusable.filter(|_| reused_curbuf) {
        Some(buf) => buf,
        None => {
            // `fresh` keeps it alive until the registry takes it over a few
            // lines below.
            let owned = new_buffer();
            let buf = Buf::owned_by(&owned);
            fresh = Some(owned);
            buf
        }
    };

    if let Some((full, short)) = names {
        buf.name.set(full, Some(short));
    }

    clear_wininfo(buf);
    buf.b_wininfo.push(WinInfo::new(None));

    if reused_curbuf {
        // Delete the local variables and the rest.
        free_buffer_stuff(buf, kBffInitChangedtick as c_int);
        // Init the options.
        buf.b_p_initialized = false;
        copy_options_into(buf, BCO_ENTER as c_int);
        // The keymaps have to be reloaded and b:keymap_name set.
        buf.b_kmap_state = (buf.b_kmap_state as c_int | KEYMAP_INIT) as int16_t;
    } else {
        // `fresh` is `Some` exactly when the buffer is not the reused
        // current one, which is the branch this is.
        buf = append_to_list(buf, fresh.take().expect("a fresh buffer was allocated"));
        // Always copy the options from the current buffer.
        copy_options_into(buf, BCO_ALWAYS as c_int);
    }

    // The entry pushed above, which nothing since has replaced: the first
    // `clear_wininfo` emptied the list, and the paths in between only clear
    // the buffer's variables and options.
    let mut mark = INIT_FMARK;
    mark.mark.lnum = lnum;
    if let Some(entry) = buf.b_wininfo.last_mut() {
        entry.wi_mark = mark;
        entry.wi_win = Some(current_win().id());
    }

    hash_reset(&mut buf.b_s.b_keywtab);
    hash_reset(&mut buf.b_s.b_keywtab_ic);

    buf.name.show_short();
    buf.file_id_valid = file_id.is_some();
    if let Some(file_id) = file_id {
        buf.file_id = file_id;
    }
    buf.b_u_synced = true;
    buf.b_flags = BufFlags::CHECK_RO | BufFlags::NEVERLOADED;
    if flags & BLN_DUMMY as c_int != 0 {
        buf.b_flags |= BufFlags::DUMMY;
    }
    buf_clear_file(buf);
    clrallmarks(buf, 0 as Timestamp);
    fmarks_check_names(buf);
    // Init 'buflisted'.
    buf.b_p_bl = if flags & BLN_LISTED as c_int != 0 {
        1
    } else {
        0
    };
    reset_update_subscribers(buf);

    if flags & BLN_DUMMY as c_int == 0 && !announce_new_buffer(buf, flags) {
        return None;
    }

    buf.b_prompt_callback = Callback::None;
    buf.b_prompt_interrupt = Callback::None;
    buf.b_prompt_start = INIT_FMARK;
    // The default prompt is "% ".
    buf.b_prompt_start.mark.col = 2 as ColNr;
    buf.b_prompt_append_new_line = true;

    Some(buf)
}

/// The entry a buffer with this name already has: refresh its position and
/// options, and list it if `BLN_LISTED` asked and it was not listed.
fn reuse_entry(mut buffer: Buf, lnum: LineNr, flags: c_int) -> Option<Buf> {
    if lnum != 0 as LineNr {
        let win = (flags & BLN_NOCURWIN as c_int == 0).then(current_win);
        buflist_setfpos(buffer, win, lnum, 0 as ColNr, false);
    }
    if flags & BLN_NOOPT as c_int == 0 {
        // Copy the options now, if 'cpo' doesn't have 's' and not done
        // already.
        copy_options_into(buffer, 0);
    }
    if flags & BLN_LISTED as c_int != 0 && buffer.b_p_bl == 0 {
        buffer.b_p_bl = 1;
        let bufref = BufRef::of(buffer);
        if flags & BLN_DUMMY as c_int == 0
            && fire_buf_event(AutoEvent::BufAdd, buffer)
            && !bufref.valid()
        {
            return None;
        }
    }
    Some(buffer)
}

/// A zeroed `Buffer` with its `b:` dictionary and `b:changedtick` in place.
///
/// The allocation is the caller's until [`append_to_list`] hands it to the
/// registry, which owns every buffer that has a number.
fn new_buffer() -> Owned<Buffer> {
    // A zeroed `Buffer` is what upstream starts one from; `append_to_list`
    // gives it its number and puts it in the registry.
    let owned = alloc_unregistered_buffer();
    let mut buf = Buf::owned_by(&owned);
    // Init the b: variables. The entry holds the dictionary until
    // `release_var_scope`; `b_vars` mirrors it for the raw readers.
    buf.b_bufvar = new_var_scope(VAR_SCOPE);
    buf.b_vars = buf.b_bufvar.dict_ptr();
    buf_init_changedtick(buf);
    owned
}

/// A registered buffer holding nothing — no memline, no `b:` dictionary —
/// for a lib test that needs a current buffer. Its number is far above any
/// the editor hands out. [`free_bare_buffer`] gives it back.
#[cfg(test)]
pub(crate) fn bare_buffer() -> Buf {
    // One at a time: a test holds the editor lock while it has one.
    const HANDLE: Handle = 1 << 24;
    let owned = alloc_unregistered_buffer();
    Buf::owned_by(&owned).handle = HANDLE;
    register_buffer(HANDLE, owned)
}

/// Give back what [`bare_buffer`] made, with the option values a case gave
/// it: dropping the allocation runs `Buffer`'s destructor, which releases
/// every owning field.
#[cfg(test)]
pub(crate) fn free_bare_buffer(buffer: Buf) {
    free_buf_options(buffer, true);
    drop(crate::winlayer::forget_buffer(buffer.handle));
}

/// Put a new buffer at the end of the buffer list, give it its number and
/// hand its allocation to the registry, which owns it from here on.
///
/// Answers the buffer **with its number in it**: a [`Buf`] carries a copy of
/// the number it was built with, and the caller's was built before there was
/// one, so it must take this one back or its `id()` names nothing.
#[must_use = "the caller's Buf predates the number; use the one this answers"]
fn append_to_list(mut buffer: Buf, owned: Owned<Buffer>) -> Buf {
    // The number and the registry entry come first, ahead of upstream's
    // order: from here on `buffer.id()` names the buffer, and the list links
    // are made of exactly that. Nothing between the two reads either.
    buffer.set_handle(top_file_num.get() as Handle);
    top_file_num.set(top_file_num.get() + 1);
    register_buffer(buffer.handle(), owned);

    buffer.b_next = None;
    match current_last() {
        // The buffer list is empty.
        None => {
            buffer.b_prev = None;
            firstbuf.set(Some(buffer.id()));
        }
        // Append the new buffer at the end of the list.
        Some(mut last) => {
            last.b_next = Some(buffer.id());
            buffer.b_prev = Some(last.id());
        }
    }
    lastbuf.set(Some(buffer.id()));
    if top_file_num.get() < 0 {
        // Wrap around; this may cause duplicates.
        emsg(gettext(c"W14: Warning: List of file names overflow"));
        if emsg_silent.get() == 0 && !in_assert_fails.get() {
            // Make sure it is noticed.
            msg_delay(3001 as uint64_t, true);
        }
        top_file_num.set(1);
    }
    buffer
}

/// `kv_destroy` + `kv_init` of the two buffer-update subscriber arrays: a
/// reused buffer must not keep the old one's subscribers.
fn reset_update_subscribers(buffer: Buf) {
    crate::buffer_updates::reset_subscribers(buffer);
}

/// Fire `BufNew` and, when the buffer is listed, `BufAdd`. Answers false
/// when the buffer did not survive them, or script processing was aborted.
///
/// Tricky: these autocommands may change the buffer list. They could also
/// split the window and re-use the one empty buffer, which may result in
/// unexpectedly losing that buffer.
fn announce_new_buffer(buffer: Buf, flags: c_int) -> bool {
    let bufref = BufRef::of(buffer);
    if fire_buf_event(AutoEvent::BufNew, buffer) && !bufref.valid() {
        return false;
    }
    if flags & BLN_LISTED as c_int != 0
        && fire_buf_event(AutoEvent::BufAdd, buffer)
        && !bufref.valid()
    {
        return false;
    }
    // Autocommands may abort script processing.
    !aborting()
}

/// Whether the current buffer is empty, unnamed, unmodified and shown in
/// only one window -- which means it can be reused.
pub fn curbuf_reusable() -> bool {
    let Some(buf) = current_buf() else {
        return false;
    };
    let empty = buf.b_ml.ml_mfp.is_none() || buf_is_empty(buf);
    buf.name.full().is_none()
        && buf.b_nwindows <= 1
        && buf.terminal.is_null()
        && empty
        && !buf_is_quickfix(Some(buf))
        && !curbuf_is_changed()
}

// ---------------------------------------------------------------------------
// Freeing one's options

/// Free the memory for a buffer's options. `free_p_ff` frees `'fileformat'`,
/// `'buftype'` and `'fileencoding'` too.
pub fn free_buf_options(mut buffer: Buf, free_p_ff: bool) {
    if free_p_ff {
        buffer.b_p_fenc = None;
        buffer.b_p_ff = None;
        buffer.b_p_bh = None;
        buffer.b_p_bt = None;
    }
    buffer.b_p_def = None;
    buffer.b_p_inc = None;
    buffer.b_p_inex = None;
    buffer.b_p_inde = None;
    buffer.b_p_indk = None;
    buffer.b_p_fp = None;
    buffer.b_p_fex = None;
    buffer.b_p_kp = None;
    buffer.b_p_mps = None;
    buffer.b_p_fo = None;
    buffer.b_p_flp = None;
    buffer.b_p_isk = None;
    buffer.b_p_vsts = None;
    buffer.b_p_vsts_nopaste = None;
    replace_tabstop_array(&mut buffer.b_p_vsts_array, None);
    buffer.b_p_vts = None;
    replace_tabstop_array(&mut buffer.b_p_vts_array, None);
    buffer.b_p_keymap = None;
    buffer.b_kmap_ga = Vec::new();
    buffer.b_p_com = None;
    buffer.b_p_cms = None;
    buffer.b_p_nf = None;
    buffer.b_p_syn = None;
    buffer.b_s.b_syn_isk = None;
    buffer.b_s.b_p_spc = None;
    buffer.b_s.b_p_spf = None;
    free_prog_in(&mut buffer.b_s.b_cap_prog);
    buffer.b_s.b_p_spl = None;
    buffer.b_s.b_p_spo = None;
    buffer.b_p_sua = None;
    buffer.b_p_ft = None;
    buffer.b_p_cink = None;
    buffer.b_p_cino = None;
    buffer.b_p_lop = None;
    buffer.b_p_cinsd = None;
    buffer.b_p_cinw = None;
    buffer.b_p_cot = None;
    buffer.b_p_cpt = None;
    buffer.b_p_cfu = None;
    callback_free(&mut buffer.b_cfu_cb);
    buffer.b_p_ofu = None;
    callback_free(&mut buffer.b_ofu_cb);
    buffer.b_p_tsrfu = None;
    callback_free(&mut buffer.b_tsrfu_cb);
    clear_buffer_cpt_callbacks(buffer);
    buffer.b_p_gefm = None;
    buffer.b_p_gp = None;
    buffer.b_p_mp = None;
    buffer.b_p_efm = None;
    buffer.b_p_ep = None;
    buffer.b_p_path = None;
    buffer.b_p_tags = None;
    buffer.b_p_tc = None;
    buffer.b_p_tfu = None;
    callback_free(&mut buffer.b_tfu_cb);
    buffer.b_p_ffu = None;
    callback_free(&mut buffer.b_ffu_cb);
    buffer.b_p_dict = None;
    buffer.b_p_dia = None;
    buffer.b_p_tsr = None;
    buffer.b_p_qe = None;
    buffer.b_p_ac = -1;
    buffer.b_p_ar = -1;
    buffer.b_p_fs = -1;
    buffer.b_p_ul = NO_LOCAL_UNDOLEVEL as OptInt;
    buffer.b_p_lw = None;
    buffer.b_p_bkc = None;
    buffer.b_p_menc = None;
}

// ---------------------------------------------------------------------------
// Switching to one

/// Go to buffer `n`, putting the cursor where it was left.
pub fn buflist_getfile(
    n: c_int,
    mut lnum: LineNr,
    options: c_int,
    forceit: c_int,
) -> Result<(), Failed> {
    let Some(buf) = find_buf(n) else {
        if options & GETF_ALT as c_int != 0 && n == 0 {
            emsg(gettext(e_noalt));
        } else {
            semsg!("E92: Buffer {n} not found");
        }
        return Err(Failed);
    };

    // There is nothing to do when it is the current buffer.
    if buf.raw() == Buf::current_raw() {
        return Ok(());
    }

    if text_or_buf_locked() {
        return Err(Failed);
    }

    let mut col: ColNr = 0;
    // Default line number: where the cursor was left last time -- a copy,
    // taken before the switch runs anything.
    let fm = (lnum == 0 as LineNr).then(|| buflist_findfmark(buf));
    if let Some(fm) = &fm {
        (lnum, col) = (fm.mark.lnum, fm.mark.col);
    }

    if options & GETF_SWITCH as c_int != 0 && !goto_existing_window(buf) {
        return Err(Failed);
    }

    let redraw_off = Suppress::redraw();
    let (handle, setmark) = (buf.handle as c_int, options & GETF_SETMARK as c_int != 0);
    // A live buffer's handle, and no file name to load under it.
    let failed = getfile(handle, None, None, setmark, lnum, forceit != 0) > 0;
    drop(redraw_off);
    if failed {
        return Err(Failed);
    }

    if !p_sol() && col != 0 {
        let mut win = current_win();
        win.w_cursor.col = col;
        check_cursor_col(win);
        win.w_cursor.coladd = 0 as ColNr;
        win.w_set_curswant = true;
    }
    if jop_flags.get() & kOptJopFlagView as c_int as u32 != 0 && fm.is_some() {
        mark_view_restore(fm.as_ref());
    }
    Ok(())
}

/// The `'switchbuf'` half of [`buflist_getfile`]: go to a window already
/// showing `buffer`, or make one. Answers false when the split failed.
fn goto_existing_window(buffer: Buf) -> bool {
    let wp = swbuf_goto_win_with_buf(Some(buffer));
    let splits = (kOptSwbFlagVsplit as c_int
        | kOptSwbFlagSplit as c_int
        | kOptSwbFlagNewtab as c_int) as u32;
    if wp.is_some() || swb_flags.get() & splits == 0 || buf_is_empty(Buf::current()) {
        return true;
    }
    if swb_flags.get() & kOptSwbFlagNewtab as c_int as u32 != 0 {
        tabpage_new();
    } else {
        let vertical = swb_flags.get() & kOptSwbFlagVsplit as c_int as u32 != 0;
        let flags = if vertical { WSP_VERT as c_int } else { 0 };
        if win_split(0, flags).is_err() {
            return false;
        }
    }
    let mut win = current_win();
    win.w_onebuf_opt.wo_scb = 0;
    win.w_onebuf_opt.wo_crb = 0;
    true
}

/// Put the cursor where it was left in the current buffer.
pub(crate) fn buflist_getfpos() {
    let fm = buflist_findfmark(Buf::current());
    let (lnum, col) = (fm.mark.lnum, fm.mark.col);

    let mut win = current_win();
    win.w_cursor.lnum = lnum;
    check_cursor_lnum(win);
    if p_sol() {
        win.w_cursor.col = 0 as ColNr;
    } else {
        win.w_cursor.col = col;
        check_cursor_col(win);
        win.w_cursor.coladd = 0 as ColNr;
        win.w_set_curswant = true;
    }
    if jop_flags.get() & kOptJopFlagView as c_int as u32 != 0 {
        mark_view_restore(Some(&fm));
    }
}

// ---------------------------------------------------------------------------
// Finding one by name

/// The buffer for `fname`, resolved to a full path first.
pub(crate) fn find_buffer_by_name(fname: &CStr) -> Option<Buf> {
    let ffname = full_name_save(fname, true);
    buflist_findname(ffname.as_cstr())
}

/// The buffer whose full name is `ffname`, or whose file id matches it.
pub fn buflist_findname(ffname: &CStr) -> Option<Buf> {
    buflist_findname_file_id(ffname, file_id(ffname).as_ref())
}

/// [`buflist_findname`] with the file id already looked up (`None` when the
/// file does not exist). Dummy buffers do not count.
pub(crate) fn buflist_findname_file_id(ffname: &CStr, file_id: Option<&FileID>) -> Option<Buf> {
    buffers_back().find(|buf| {
        !buf.b_flags.has(BufFlags::DUMMY) && !otherfile_buf(*buf, ffname, Some(file_id))
    })
}

// ---------------------------------------------------------------------------
// Finding one by pattern

/// The number of the buffer matching `pattern`: `-1` when there is none and
/// `-2` when more than one does, both reported to the user.
///
/// `unlisted` searches the unlisted buffers too, when no listed one matched;
/// `curtab_only` ignores buffers not open in the current tab page.
///
/// The pattern is the first `len` bytes of `text` (panics past its end); a
/// failure names the whole of `text`, as upstream's message does when the
/// pattern is a word inside a command line.
pub(crate) fn buflist_findpat_prefix(
    text: &CStr,
    len: usize,
    unlisted: bool,
    diffmode: bool,
    curtab_only: bool,
) -> c_int {
    buflist_findpat(text, Some(len), unlisted, diffmode, curtab_only)
}

/// The number of the buffer whose name matches the first `len` bytes of
/// `text` (all of it for `None`); see [`buflist_findpat_prefix`].
pub fn buflist_findpat(
    text: &CStr,
    len: Option<usize>,
    unlisted: bool,
    diffmode: bool,
    curtab_only: bool,
) -> c_int {
    let pattern = text.to_bytes();
    let len = len.unwrap_or(pattern.len());
    let head = if len == 1 { pattern[0] } else { 0 };
    let matched = if head == b'%' || head == b'#' {
        match_shorthand(head, diffmode)
    } else {
        match match_pattern(pattern, len, unlisted, diffmode, curtab_only) {
            // An unusable pattern is reported by whoever built it.
            None => return -1,
            Some(matched) => matched,
        }
    };

    if matched == -2 {
        let pattern = msg_cstr(text);
        semsg!("E93: More than one match for {pattern}");
    } else if matched < 0 {
        let pattern = msg_cstr(text);
        semsg!("E94: No matching buffer for {pattern}");
    }
    matched
}

/// `%` (the current buffer) and `#` (the alternate file), which never reach
/// the regexp.
fn match_shorthand(head: u8, diffmode: bool) -> c_int {
    let matched = if head == b'%' {
        Buf::current().handle as c_int
    } else {
        current_win().w_alt_fnum
    };
    if diffmode && !find_buf(matched).is_some_and(is_diff_mode) {
        return -1;
    }
    matched
}

/// The four-attempt regexp search. `None` when the pattern could not be
/// turned into a regexp at all, which upstream reports without a message.
fn match_pattern(
    text: &[u8],
    end: usize,
    unlisted: bool,
    diffmode: bool,
    curtab_only: bool,
) -> Option<c_int> {
    let (pat, _) = file_pat_to_reg_pat(text, end)?;
    // The pattern and its terminator, which the attempts below write.
    let mut buf = pat.into_vec();
    let patlen = buf.len() - 1;
    // Whether the pattern ends in '$', which attempts 0 and 1 take off.
    let toggledollar = patlen > 1 && buf[patlen - 1] == b'$';

    let mut matched = -1;
    // First try finding a listed buffer. When there is none and "unlisted"
    // is set, try again over the unlisted ones.
    let mut find_listed = true;
    loop {
        // Try four ways of matching a buffer:
        //   0: without '^' or '$' (at any position)
        //   1: with '^' at start (only at position 0)
        //   2: with '$' at end (only match at end)
        //   3: with '^' at start and '$' at end (only a full match)
        for attempt in 0..=3 {
            if toggledollar {
                // Add or remove the '$'.
                buf[patlen - 1] = if attempt < 2 { 0 } else { b'$' };
            }
            // Add or remove the '^'.
            let from = usize::from(buf[0] == b'^' && attempt & 1 == 0);
            let flags = if magic_isset() { RE_MAGIC } else { 0 };
            let mut regmatch = OwnedMatch::compile(cstr::in_bytes(&buf[from..]), flags, false)
                .unwrap_or_else(OwnedMatch::none);

            for b in buffers_back() {
                if regmatch.regprog.is_null() {
                    // An invalid pattern, possibly after switching engine.
                    return None;
                }
                if (b.b_p_bl != 0) != find_listed
                    || diffmode && !is_diff_mode(b)
                    || buflist_match(&mut regmatch, b, false).is_none()
                {
                    continue;
                }
                // Ignore the match when the buffer is not open in the
                // current tab page.
                if curtab_only && !windows().any(|win| win.w_buffer == b) {
                    continue;
                }
                if matched >= 0 {
                    // A match was already found.
                    matched = -2;
                    break;
                }
                // Remember the first match.
                matched = b.handle as c_int;
            }

            drop(regmatch);
            if matched >= 0 {
                // Found one match.
                break;
            }
        }

        // Only search the unlisted buffers when no listed one matched.
        if !unlisted || !find_listed || matched != -1 {
            break;
        }
        find_listed = false;
    }

    Some(matched)
}

// ---------------------------------------------------------------------------
// Sorting by last-used time

/// An element of an array sorted by its buffer's last use.
pub(crate) trait LastUsed {
    fn last_used_buffer(&self) -> Buf;
}

impl LastUsed for Buf {
    fn last_used_buffer(&self) -> Buf {
        *self
    }
}

/// Sort `items` most recently used buffer first. Two buffers entered in the
/// same second tie, and the order of a tie is whatever `qsort` lands on --
/// which is why this is `qsort`.
pub(crate) fn sort_by_last_used<T: LastUsed + Copy>(items: &mut [T]) {
    crate::memory::qsort_by(items, |a, b| {
        let (a, b) = (a.last_used_buffer(), b.last_used_buffer());
        b.b_last_used.cmp(&a.b_last_used)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_cell::editor_state_lock;
    use crate::optionstr::BUF_STRING_OPTIONS;
    use crate::window::{bare_window, free_bare_window};

    /// The zeroed allocators write every field whose all-zero bytes are not
    /// a valid value. Each is read back here through its own type, which is
    /// what Miri checks the validity of: a field left zeroed would read as
    /// `Some` over a null vector, or as a vector with a null pointer.
    #[test]
    fn a_fresh_buffer_holds_valid_owners() {
        let _held = editor_state_lock();
        let mut buffer = bare_buffer();
        {
            let b: &mut Buffer = &mut buffer;
            for field in BUF_STRING_OPTIONS {
                assert!(
                    field.of(b).is_none(),
                    "string option at {}",
                    field.field_offset()
                );
            }
            assert!(b.name.full().is_none() && b.name.short().is_none());
            assert!(b.b_ucmds.is_empty());
            assert!(b.b_kmap_ga.is_empty());
            assert!(b.b_wininfo.is_empty());
            assert!(b.b_s.b_syn_patterns.is_empty());
            assert!(b.b_s.b_syn_clusters.is_empty());
            assert!(b.b_s.b_syn_linecont_pat.is_none());
            assert!(!b.b_s.b_sst.is_allocated());
            assert_eq!(b.b_s.b_keywtab.items().count(), 0);
            assert_eq!(b.b_s.b_keywtab_ic.items().count(), 0);
            assert!(b.b_ml.ml_stack.is_empty());
            assert_eq!(b.b_ml.ml_chunks.len(), 0);
            assert!(b.b_ml.ml_locked.is_none());
            assert!(b.b_marktree.id2node.is_empty());
            assert!(b.b_extmark_ns.is_empty());
            assert!(b.b_start_fenc.is_none());
            assert!(b.b_prompt_text.is_none());
            assert!(b.additional_data.is_none());
            assert!(b.b_maps.is_none());
            assert!(b.b_next.is_none() && b.b_prev.is_none());
        }
        free_bare_buffer(buffer);
    }

    /// [`a_fresh_buffer_holds_valid_owners`] for a window.
    #[test]
    fn a_fresh_window_holds_valid_owners() {
        let _held = editor_state_lock();
        let window = bare_window();
        {
            let w: &crate::types::Window = &window;
            assert!(w.w_ns_set.is_empty());
            assert!(!w.w_grid_alloc.is_allocated());
            assert!(!w.w_grid_alloc.tracks_dirty_cols());
            for opt in [&w.w_onebuf_opt, &w.w_allbuf_opt] {
                for value in [
                    &opt.wo_briopt,
                    &opt.wo_fdc,
                    &opt.wo_eiw,
                    &opt.wo_fdc_save,
                    &opt.wo_fdi,
                    &opt.wo_fdm,
                    &opt.wo_fdm_save,
                    &opt.wo_fde,
                    &opt.wo_fdt,
                    &opt.wo_fmr,
                    &opt.wo_ve,
                    &opt.wo_rlc,
                    &opt.wo_culopt,
                    &opt.wo_cc,
                    &opt.wo_sbr,
                    &opt.wo_stc,
                    &opt.wo_stl,
                    &opt.wo_wbr,
                    &opt.wo_cocu,
                    &opt.wo_scl,
                    &opt.wo_winhl,
                    &opt.wo_lcs,
                    &opt.wo_fcs,
                ] {
                    assert!(value.is_none());
                }
            }
            assert!(w.w_tagstack.iter().all(|tag| tag.tagname.is_none()));
            assert!(w.w_llist.is_none() && w.w_llist_ref.is_none());
            assert!(w.w_prev.is_none() && w.w_next.is_none() && w.w_frame.is_none());
        }
        free_bare_window(window);
    }
}
