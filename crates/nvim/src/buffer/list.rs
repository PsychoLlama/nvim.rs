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

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::{iter, ptr, slice};

use super::*;
use crate::autocmd::{EVENT_BUFADD, EVENT_BUFNEW, apply_autocmds};
use crate::cursor::{check_cursor_col, check_cursor_lnum};
use crate::diff::diff_mode_buf;
use crate::digraph::keymap_ga_clear;
use crate::eval::typval::{callback_free, kCallbackNone, tv_dict_alloc};
use crate::eval::vars::init_var_dict;
use crate::ex_cmds::getfile;
use crate::ex_docmd::tabpage_new;
use crate::ex_eval::aborting;
use crate::ex_getln::text_or_buf_locked;
use crate::fileio::file_pat_to_reg_pat;
use crate::garray::ga_clear;
use crate::guard::Suppress;
use crate::hashtab::hash_init;
use crate::insexpand::clear_cpt_callbacks;
use crate::main::{
    buffer_handles, curbuf, e_buffer_nr_not_found, e_noalt, emsg_silent, firstbuf, in_assert_fails,
    jop_flags, lastbuf, p_sol, swb_flags,
};
use crate::mark::{clrallmarks, fmarks_check_names, mark_view_restore};
use crate::memory::{xcalloc, xfree, xstrdup};
use crate::message::{emsg, msg_delay};
use crate::option::{buf_copy_options, magic_isset};
use crate::options::{kOptJopFlagView, kOptSwbFlagNewtab, kOptSwbFlagSplit, kOptSwbFlagVsplit};
use crate::optionstr::clear_string_option;
use crate::os::cshim::gettext;
use crate::os::fs::os_fileid;
use crate::path::full_name_save;
use crate::pos::MAXLNUM;
use crate::regexp::{RE_MAGIC, vim_regcomp, vim_regfree};
use crate::semsg_c;
use crate::types::{
    AdditionalData, Callback, FAIL, FileID, OK, OptInt, Timestamp, VAR_SCOPE, buf_T, colnr_T,
    event_T, fmark_T, fmarkv_T, garray_T, handle_T, int16_t, linenr_T, pos_T, ptr_t, regprog_T,
    size_t, uint64_t,
};
use crate::undo::curbuf_is_changed;
use crate::window::{WSP_VERT, swbuf_goto_win_with_buf, win_split};
use crate::winlayer::{Buf, Win, windows};
use ::libc::strlen;

use super::expand::{NO_REGMATCH, buflist_match, find_buf};
use super::pos::{Entry, WinInfos};

/// `INIT_FMARK`: a mark that has never been set.
pub(crate) const INIT_FMARK: fmark_T = fmark_T {
    mark: pos_T {
        lnum: 0 as linenr_T,
        col: 0 as colnr_T,
        coladd: 0 as colnr_T,
    },
    fnum: 0,
    timestamp: 0 as Timestamp,
    view: fmarkv_T {
        topline_offset: MAXLNUM as linenr_T,
        skipcol: 0 as colnr_T,
    },
    additional_data: ptr::null_mut::<AdditionalData>(),
};

const NO_FILE_ID: FileID = FileID {
    inode: 0,
    device_id: 0,
};

// ---------------------------------------------------------------------------
// The neighbours, wrapped

/// `_()`.
fn tr(msg: &CStr) -> *mut c_char {
    tr_raw(msg.as_ptr())
}

/// `_()` over a pointer, for the message statics `main.rs` holds as byte
/// arrays.
fn tr_raw(msg: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated literal or message static.
    unsafe { gettext(msg) }
}

fn err(msg: *mut c_char) {
    // SAFETY: a NUL-terminated message.
    unsafe { emsg(msg) };
}

/// `semsg(fmt, n)`, for the error that names a buffer number.
fn err_num(fmt: *mut c_char, n: c_int) {
    // SAFETY: a translated format taking one number, and the number.
    let _: bool = unsafe { semsg_c!(fmt, n) };
}

/// `semsg(fmt, pattern)`, for the two errors that quote the pattern.
fn err_pat(fmt: &CStr, pattern: *const c_char) {
    let fmt = tr(fmt);
    // SAFETY: a translated format taking one string, and the pattern the
    // caller promised is NUL-terminated.
    let _: bool = unsafe { semsg_c!(fmt, pattern) };
}

fn free(p: *mut c_char) {
    // SAFETY: an owned allocation or null.
    unsafe { xfree(p.cast::<c_void>()) };
}

fn dup(p: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated name; the one caller has tested it.
    unsafe { xstrdup(p) }
}

/// `XFREE_CLEAR` over a slot holding an owned array.
fn xfree_clear<T>(slot: &mut *mut T) {
    // SAFETY: an owned allocation or null.
    unsafe { xfree((*slot).cast::<c_void>()) };
    *slot = ptr::null_mut();
}

/// `clear_string_option`: free an option's value and leave the slot holding
/// the shared empty string.
fn clear_opt(slot: &mut *mut c_char) {
    // SAFETY: an option variable, holding null, the shared empty string or
    // an owned allocation.
    unsafe { clear_string_option(slot) };
}

fn clear_callback(cb: &mut Callback) {
    // SAFETY: a callback slot inside a live buffer.
    unsafe { callback_free(cb) };
}

fn clear_garray(ga: &mut garray_T) {
    // SAFETY: a growable array inside a live buffer.
    unsafe { ga_clear(ga) };
}

fn keymap_clear(ga: &mut garray_T) {
    // SAFETY: the buffer's own keymap array.
    unsafe { keymap_ga_clear(ga) };
}

fn clear_cpt(callbacks: &mut *mut Callback, count: c_int) {
    // SAFETY: the buffer's own 'complete' callback array and its length.
    unsafe { clear_cpt_callbacks(callbacks, count) };
}

fn free_regprog(prog: &mut *mut regprog_T) {
    // SAFETY: a compiled program or null.
    unsafe { vim_regfree(*prog) };
    *prog = ptr::null_mut();
}

fn regcomp(pat: &[u8], flags: c_int) -> *mut regprog_T {
    // SAFETY: a NUL-terminated pattern; the answer is null on a bad one.
    unsafe { vim_regcomp(pat.as_ptr().cast::<c_char>(), flags) }
}

fn is_diff_mode(mut buf: Buf) -> bool {
    // SAFETY: a live buffer.
    unsafe { diff_mode_buf(buf.raw()) }
}

/// The current buffer, which is null only before the first one is created.
fn current_buf() -> Option<Buf> {
    let buf = curbuf.get();
    // SAFETY: non-null, hence live.
    (!buf.is_null()).then(|| unsafe { Buf::new(buf) })
}

fn current_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

fn current_last() -> Option<Buf> {
    let last = lastbuf.get();
    // SAFETY: `lastbuf` is null only before the first buffer is created.
    (!last.is_null()).then(|| unsafe { Buf::new(last) })
}

/// The buffer list from the end -- `FOR_ALL_BUFFERS_BACKWARDS`. Lazy, as the
/// macro is.
fn buffers_backwards() -> impl Iterator<Item = Buf> {
    let mut next = lastbuf.get();
    iter::from_fn(move || {
        // SAFETY: `lastbuf`, and every `b_prev` reached from it, is a live
        // buffer or null.
        let buf = (!next.is_null()).then(|| unsafe { Buf::new(next) })?;
        next = buf.b_prev;
        Some(buf)
    })
}

fn fire_buf_event(event: event_T, mut buf: Buf) -> bool {
    let raw = buf.raw();
    // SAFETY: a live buffer; both name arguments are optional.
    unsafe { apply_autocmds(event, ptr::null_mut(), ptr::null_mut(), false, raw) }
}

fn copy_options_into(mut buf: Buf, flags: c_int) {
    // SAFETY: a live buffer.
    unsafe { buf_copy_options(buf.raw(), flags) };
}

fn check_cursor_column(mut win: Win) {
    // SAFETY: a live window.
    unsafe { check_cursor_col(win.raw()) };
}

fn check_cursor_line(mut win: Win) {
    // SAFETY: a live window.
    unsafe { check_cursor_lnum(win.raw()) };
}

// ---------------------------------------------------------------------------
// Creating an entry

/// Add a file to the buffer list, or answer the entry it already has.
///
/// `lnum` is the line to remember for it and `flags` the `BLN_*` set. The
/// answer is null when an autocommand deleted the buffer under us.
pub unsafe fn buflist_new(
    ffname_arg: *mut c_char,
    sfname_arg: *mut c_char,
    lnum: linenr_T,
    flags: c_int,
) -> *mut buf_T {
    let mut ffname = ffname_arg;
    let mut sfname = sfname_arg;

    // Will allocate ffname.
    // SAFETY: two locals holding a name each, and the current buffer.
    unsafe { fname_expand(curbuf.get(), &raw mut ffname, &raw mut sfname) };

    // The file id works better than the name for hard links, when the file
    // exists.
    let mut file_id = NO_FILE_ID;
    // SAFETY: a NUL-terminated name, and a local to fill in.
    let file_id_valid = !sfname.is_null() && unsafe { os_fileid(sfname, &raw mut file_id) };

    // If the file name is already in the list, update that entry.
    if !ffname.is_null()
        && flags & (BLN_DUMMY as c_int | BLN_NEW as c_int) == 0
        && let Some(buf) = buflist_findname_file_id(ffname, &file_id, file_id_valid)
    {
        free(ffname);
        return reuse_entry(buf, lnum, flags);
    }

    // The current buffer, when it has no name and no contents, otherwise a
    // fresh one. This is the ONLY place a buffer structure is allocated.
    let mut reusable = None;
    // SAFETY: reads the current buffer's own state.
    if flags & BLN_CURBUF as c_int != 0 && unsafe { curbuf_reusable() } {
        let cur = current_buf().expect("curbuf != NULL");
        let bufref = BufRef::of(cur);
        trigger_undo_ftplugin(cur, current_win());
        // It is as if this buffer were deleted. Watch out for autocommands
        // that change curbuf: if that happens, allocate a new buffer anyway.
        // SAFETY: a live buffer.
        unsafe { buf_freeall(cur.raw(), BFA_WIPE as c_int | BFA_DEL as c_int) };
        if aborting() {
            // Autocommands may abort script processing.
            free(ffname);
            return ptr::null_mut();
        }
        // When the buffer was deleted, allocate a new one instead.
        reusable = bufref.get();
    }
    // Upstream re-reads `curbuf` here: `buf_freeall`'s autocommands may have
    // made another buffer current, and then this one is not reusable after
    // all.
    let reused_curbuf = reusable.is_some() && reusable == current_buf();
    let mut buf = match reusable.filter(|_| reused_curbuf) {
        Some(buf) => buf,
        None => new_buffer(),
    };

    if !ffname.is_null() {
        buf.b_ffname = ffname;
        buf.b_sfname = dup(sfname);
    }

    clear_wininfo(buf);
    let mut entry = Entry::new();
    WinInfos::of(&mut buf).push(entry);

    if reused_curbuf {
        // Delete the local variables and the rest.
        free_buffer_stuff(buf, kBffInitChangedtick as c_int);
        // Init the options.
        buf.b_p_initialized = false;
        copy_options_into(buf, BCO_ENTER as c_int);
        // The keymaps have to be reloaded and b:keymap_name set.
        buf.b_kmap_state = (buf.b_kmap_state as c_int | KEYMAP_INIT) as int16_t;
    } else {
        append_to_list(buf);
        // Always copy the options from the current buffer.
        copy_options_into(buf, BCO_ALWAYS as c_int);
    }

    entry.wi_mark = INIT_FMARK;
    entry.wi_mark.mark.lnum = lnum;
    entry.wi_win = current_win().raw();

    init_hashtabs(buf);

    buf.b_fname = buf.b_sfname;
    buf.file_id_valid = file_id_valid;
    if file_id_valid {
        buf.file_id = file_id;
    }
    buf.b_u_synced = true;
    buf.b_flags = BufFlags::CHECK_RO | BufFlags::NEVERLOADED;
    if flags & BLN_DUMMY as c_int != 0 {
        buf.b_flags |= BufFlags::DUMMY;
    }
    // SAFETY: a live buffer.
    unsafe { buf_clear_file(buf.raw()) };
    // SAFETY: a live buffer; clear its marks.
    unsafe { clrallmarks(buf.raw(), 0 as Timestamp) };
    // SAFETY: a live buffer; check the file marks for this file.
    unsafe { fmarks_check_names(buf.raw()) };
    // Init 'buflisted'.
    buf.b_p_bl = if flags & BLN_LISTED as c_int != 0 {
        1
    } else {
        0
    };
    reset_update_subscribers(&mut buf);

    if flags & BLN_DUMMY as c_int == 0 && !announce_new_buffer(buf, flags) {
        return ptr::null_mut();
    }

    buf.b_prompt_callback.type_0 = kCallbackNone;
    buf.b_prompt_interrupt.type_0 = kCallbackNone;
    buf.b_prompt_text = ptr::null_mut();
    buf.b_prompt_start = INIT_FMARK;
    // The default prompt is "% ".
    buf.b_prompt_start.mark.col = 2 as colnr_T;
    buf.b_prompt_append_new_line = true;

    buf.raw()
}

/// The entry a buffer with this name already has: refresh its position and
/// options, and list it if `BLN_LISTED` asked and it was not listed.
fn reuse_entry(mut buf: Buf, lnum: linenr_T, flags: c_int) -> *mut buf_T {
    if lnum != 0 as linenr_T {
        let win = if flags & BLN_NOCURWIN as c_int != 0 {
            ptr::null_mut()
        } else {
            current_win().raw()
        };
        // SAFETY: a live buffer, and a live window or null.
        unsafe { buflist_setfpos(buf.raw(), win, lnum, 0 as colnr_T, false) };
    }
    if flags & BLN_NOOPT as c_int == 0 {
        // Copy the options now, if 'cpo' doesn't have 's' and not done
        // already.
        copy_options_into(buf, 0);
    }
    if flags & BLN_LISTED as c_int != 0 && buf.b_p_bl == 0 {
        buf.b_p_bl = 1;
        let bufref = BufRef::of(buf);
        if flags & BLN_DUMMY as c_int == 0 && fire_buf_event(EVENT_BUFADD, buf) && !bufref.valid() {
            return ptr::null_mut();
        }
    }
    buf.raw()
}

/// A zeroed `buf_T` with its `b:` dictionary and `b:changedtick` in place.
fn new_buffer() -> Buf {
    // SAFETY: `xcalloc` aborts rather than answering null, and a zeroed
    // `buf_T` is what upstream starts one from.
    let mut buf = unsafe { Buf::new(xcalloc(1, size_of::<buf_T>()).cast::<buf_T>()) };
    // Init the b: variables.
    // SAFETY: a fresh dictionary for the buffer's own `b:` scope.
    buf.b_vars = unsafe { tv_dict_alloc() };
    let (vars, scope_var) = (buf.b_vars, &raw mut buf.b_bufvar);
    // SAFETY: the dictionary just allocated and the buffer's scope variable.
    unsafe { init_var_dict(vars, scope_var, VAR_SCOPE) };
    buf_init_changedtick(buf);
    buf
}

/// Put a new buffer at the end of the buffer list and give it its number.
fn append_to_list(mut buf: Buf) {
    buf.b_next = ptr::null_mut();
    match current_last() {
        // The buffer list is empty.
        None => {
            buf.b_prev = ptr::null_mut();
            firstbuf.set(buf.raw());
        }
        // Append the new buffer at the end of the list.
        Some(mut last) => {
            last.b_next = buf.raw();
            buf.b_prev = last.raw();
        }
    }
    lastbuf.set(buf.raw());

    buf.handle = top_file_num.get() as handle_T;
    top_file_num.set(top_file_num.get() + 1);
    let (handle, raw) = (buf.handle as c_int, buf.raw().cast::<c_void>() as ptr_t);
    // The borrow of the handle map lasts only for the insertion, which does
    // not re-enter.
    buffer_handles.with_mut(|map| map_put_int_ptr_t(map, handle, raw));
    if top_file_num.get() < 0 {
        // Wrap around; this may cause duplicates.
        err(tr(c"W14: Warning: List of file names overflow"));
        if emsg_silent.get() == 0 && !in_assert_fails.get() {
            // Make sure it is noticed.
            // SAFETY: a plain delay over the message machinery.
            unsafe { msg_delay(3001 as uint64_t, true) };
        }
        top_file_num.set(1);
    }
}

fn init_hashtabs(mut buf: Buf) {
    let (keywords, keywords_ic) = (&raw mut buf.b_s.b_keywtab, &raw mut buf.b_s.b_keywtab_ic);
    // SAFETY: a hash table inside a live buffer.
    unsafe { hash_init(keywords) };
    // SAFETY: as above.
    unsafe { hash_init(keywords_ic) };
}

/// `kv_destroy` + `kv_init` of the two buffer-update subscriber arrays: a
/// reused buffer must not keep the old one's subscribers.
fn reset_update_subscribers(buf: &mut Buf) {
    xfree_clear(&mut buf.update_channels.items);
    buf.update_channels.capacity = 0 as size_t;
    buf.update_channels.size = 0 as size_t;
    xfree_clear(&mut buf.update_callbacks.items);
    buf.update_callbacks.capacity = 0 as size_t;
    buf.update_callbacks.size = 0 as size_t;
}

/// Fire `BufNew` and, when the buffer is listed, `BufAdd`. Answers false
/// when the buffer did not survive them, or script processing was aborted.
///
/// Tricky: these autocommands may change the buffer list. They could also
/// split the window and re-use the one empty buffer, which may result in
/// unexpectedly losing that buffer.
fn announce_new_buffer(buf: Buf, flags: c_int) -> bool {
    let bufref = BufRef::of(buf);
    if fire_buf_event(EVENT_BUFNEW, buf) && !bufref.valid() {
        return false;
    }
    if flags & BLN_LISTED as c_int != 0 && fire_buf_event(EVENT_BUFADD, buf) && !bufref.valid() {
        return false;
    }
    // Autocommands may abort script processing.
    !aborting()
}

/// Whether the current buffer is empty, unnamed, unmodified and shown in
/// only one window -- which means it can be reused.
pub unsafe fn curbuf_reusable() -> bool {
    let Some(mut buf) = current_buf() else {
        return false;
    };
    // SAFETY: a live buffer, in each of the three.
    let empty = buf.b_ml.ml_mfp.is_null() || unsafe { buf_is_empty(buf.raw()) };
    buf.b_ffname.is_null()
        && buf.b_nwindows <= 1
        && buf.terminal.is_null()
        && empty
        && !unsafe { bt_quickfix(buf.raw()) }
        && !unsafe { curbuf_is_changed() }
}

// ---------------------------------------------------------------------------
// Freeing one's options

/// Free the memory for a buffer's options. `free_p_ff` frees `'fileformat'`,
/// `'buftype'` and `'fileencoding'` too.
pub unsafe fn free_buf_options(buf: *mut buf_T, free_p_ff: bool) {
    // SAFETY: the caller's promise -- a live buffer.
    let mut buf = unsafe { Buf::new(buf) };
    if free_p_ff {
        clear_opt(&mut buf.b_p_fenc);
        clear_opt(&mut buf.b_p_ff);
        clear_opt(&mut buf.b_p_bh);
        clear_opt(&mut buf.b_p_bt);
    }
    clear_opt(&mut buf.b_p_def);
    clear_opt(&mut buf.b_p_inc);
    clear_opt(&mut buf.b_p_inex);
    clear_opt(&mut buf.b_p_inde);
    clear_opt(&mut buf.b_p_indk);
    clear_opt(&mut buf.b_p_fp);
    clear_opt(&mut buf.b_p_fex);
    clear_opt(&mut buf.b_p_kp);
    clear_opt(&mut buf.b_p_mps);
    clear_opt(&mut buf.b_p_fo);
    clear_opt(&mut buf.b_p_flp);
    clear_opt(&mut buf.b_p_isk);
    clear_opt(&mut buf.b_p_vsts);
    xfree_clear(&mut buf.b_p_vsts_nopaste);
    xfree_clear(&mut buf.b_p_vsts_array);
    clear_opt(&mut buf.b_p_vts);
    xfree_clear(&mut buf.b_p_vts_array);
    clear_opt(&mut buf.b_p_keymap);
    keymap_clear(&mut buf.b_kmap_ga);
    clear_garray(&mut buf.b_kmap_ga);
    clear_opt(&mut buf.b_p_com);
    clear_opt(&mut buf.b_p_cms);
    clear_opt(&mut buf.b_p_nf);
    clear_opt(&mut buf.b_p_syn);
    clear_opt(&mut buf.b_s.b_syn_isk);
    clear_opt(&mut buf.b_s.b_p_spc);
    clear_opt(&mut buf.b_s.b_p_spf);
    free_regprog(&mut buf.b_s.b_cap_prog);
    clear_opt(&mut buf.b_s.b_p_spl);
    clear_opt(&mut buf.b_s.b_p_spo);
    clear_opt(&mut buf.b_p_sua);
    clear_opt(&mut buf.b_p_ft);
    clear_opt(&mut buf.b_p_cink);
    clear_opt(&mut buf.b_p_cino);
    clear_opt(&mut buf.b_p_lop);
    clear_opt(&mut buf.b_p_cinsd);
    clear_opt(&mut buf.b_p_cinw);
    clear_opt(&mut buf.b_p_cot);
    clear_opt(&mut buf.b_p_cpt);
    clear_opt(&mut buf.b_p_cfu);
    clear_callback(&mut buf.b_cfu_cb);
    clear_opt(&mut buf.b_p_ofu);
    clear_callback(&mut buf.b_ofu_cb);
    clear_opt(&mut buf.b_p_tsrfu);
    clear_callback(&mut buf.b_tsrfu_cb);
    let cpt_count = buf.b_p_cpt_count;
    clear_cpt(&mut buf.b_p_cpt_cb, cpt_count);
    buf.b_p_cpt_count = 0;
    clear_opt(&mut buf.b_p_gefm);
    clear_opt(&mut buf.b_p_gp);
    clear_opt(&mut buf.b_p_mp);
    clear_opt(&mut buf.b_p_efm);
    clear_opt(&mut buf.b_p_ep);
    clear_opt(&mut buf.b_p_path);
    clear_opt(&mut buf.b_p_tags);
    clear_opt(&mut buf.b_p_tc);
    clear_opt(&mut buf.b_p_tfu);
    clear_callback(&mut buf.b_tfu_cb);
    clear_opt(&mut buf.b_p_ffu);
    clear_callback(&mut buf.b_ffu_cb);
    clear_opt(&mut buf.b_p_dict);
    clear_opt(&mut buf.b_p_dia);
    clear_opt(&mut buf.b_p_tsr);
    clear_opt(&mut buf.b_p_qe);
    buf.b_p_ac = -1;
    buf.b_p_ar = -1;
    buf.b_p_fs = -1;
    buf.b_p_ul = NO_LOCAL_UNDOLEVEL as OptInt;
    clear_opt(&mut buf.b_p_lw);
    clear_opt(&mut buf.b_p_bkc);
    clear_opt(&mut buf.b_p_menc);
}

// ---------------------------------------------------------------------------
// Switching to one

/// Go to buffer `n`, putting the cursor where it was left.
pub unsafe fn buflist_getfile(
    n: c_int,
    mut lnum: linenr_T,
    options: c_int,
    forceit: c_int,
) -> c_int {
    let Some(mut buf) = find_buf(n) else {
        if options & GETF_ALT as c_int != 0 && n == 0 {
            err(tr_raw((&raw const e_noalt).cast::<c_char>()));
        } else {
            let fmt = tr_raw((&raw const e_buffer_nr_not_found).cast::<c_char>());
            err_num(fmt, n);
        }
        return FAIL;
    };

    // There is nothing to do when it is the current buffer.
    if buf.raw() == curbuf.get() {
        return OK;
    }

    // SAFETY: reads the command-line and textlock state.
    if unsafe { text_or_buf_locked() } {
        return FAIL;
    }

    let mut col: colnr_T = 0;
    let mut fm: *mut fmark_T = ptr::null_mut();
    let mut restore_view = false;
    if lnum == 0 as linenr_T {
        // Default line number: where the cursor was left last time.
        // SAFETY: a live buffer; the answer is a live mark.
        fm = unsafe { buflist_findfmark(buf.raw()) };
        // SAFETY: as above.
        (lnum, col) = unsafe { ((*fm).mark.lnum, (*fm).mark.col) };
        restore_view = true;
    }

    if options & GETF_SWITCH as c_int != 0 && !goto_existing_window(buf) {
        return FAIL;
    }

    let redraw_off = Suppress::redraw();
    let (handle, setmark, no_name) = (
        buf.handle as c_int,
        options & GETF_SETMARK as c_int != 0,
        ptr::null_mut(),
    );
    // SAFETY: a live buffer's handle, and no file name to load under it.
    let failed = unsafe { getfile(handle, no_name, no_name, setmark, lnum, forceit != 0) } > 0;
    drop(redraw_off);
    if failed {
        return FAIL;
    }

    if p_sol.get() == 0 && col != 0 {
        let mut win = current_win();
        win.w_cursor.col = col;
        check_cursor_column(win);
        win.w_cursor.coladd = 0 as colnr_T;
        win.w_set_curswant = true;
    }
    if jop_flags.get() & kOptJopFlagView as c_int as u32 != 0 && restore_view {
        // SAFETY: the mark read above, which is still live.
        unsafe { mark_view_restore(fm) };
    }
    OK
}

/// The `'switchbuf'` half of [`buflist_getfile`]: go to a window already
/// showing `buf`, or make one. Answers false when the split failed.
fn goto_existing_window(mut buf: Buf) -> bool {
    // SAFETY: a live buffer; the answer is a live window or null.
    let wp = unsafe { swbuf_goto_win_with_buf(buf.raw()) };
    let splits = (kOptSwbFlagVsplit as c_int
        | kOptSwbFlagSplit as c_int
        | kOptSwbFlagNewtab as c_int) as u32;
    // SAFETY: the current buffer.
    if !wp.is_null() || swb_flags.get() & splits == 0 || unsafe { buf_is_empty(curbuf.get()) } {
        return true;
    }
    if swb_flags.get() & kOptSwbFlagNewtab as c_int as u32 != 0 {
        tabpage_new();
    } else {
        let vertical = swb_flags.get() & kOptSwbFlagVsplit as c_int as u32 != 0;
        let flags = if vertical { WSP_VERT as c_int } else { 0 };
        if win_split(0, flags) == FAIL {
            return false;
        }
    }
    let mut win = current_win();
    win.w_onebuf_opt.wo_scb = 0;
    win.w_onebuf_opt.wo_crb = 0;
    true
}

/// Put the cursor where it was left in the current buffer.
pub(crate) unsafe fn buflist_getfpos() {
    // SAFETY: the current buffer; the answer is a live mark.
    let fm = unsafe { buflist_findfmark(curbuf.get()) };
    // SAFETY: a live mark.
    let (lnum, col) = unsafe { ((*fm).mark.lnum, (*fm).mark.col) };

    let mut win = current_win();
    win.w_cursor.lnum = lnum;
    check_cursor_line(win);
    if p_sol.get() != 0 {
        win.w_cursor.col = 0 as colnr_T;
    } else {
        win.w_cursor.col = col;
        check_cursor_column(win);
        win.w_cursor.coladd = 0 as colnr_T;
        win.w_set_curswant = true;
    }
    if jop_flags.get() & kOptJopFlagView as c_int as u32 != 0 {
        // SAFETY: the mark read above, which is still live.
        unsafe { mark_view_restore(fm) };
    }
}

// ---------------------------------------------------------------------------
// Finding one by name

/// The buffer for `fname`, resolved to a full path first.
pub unsafe fn buflist_findname_exp(fname: *mut c_char) -> *mut buf_T {
    // SAFETY: a NUL-terminated name; the answer is an allocation or null.
    let ffname = unsafe { full_name_save(fname, true) };
    if ffname.is_null() {
        return ptr::null_mut();
    }
    // SAFETY: the full name just built.
    let buf = unsafe { buflist_findname(ffname) };
    free(ffname);
    buf
}

/// The buffer whose full name is `ffname`, or whose file id matches it.
pub unsafe fn buflist_findname(ffname: *mut c_char) -> *mut buf_T {
    let mut file_id = NO_FILE_ID;
    // SAFETY: the caller's promise -- a NUL-terminated name -- and a local.
    let file_id_valid = unsafe { os_fileid(ffname, &raw mut file_id) };
    buflist_findname_file_id(ffname, &file_id, file_id_valid)
        .map_or(ptr::null_mut(), |mut buf| buf.raw())
}

/// [`buflist_findname`] with the file id already looked up. Dummy buffers do
/// not count.
pub(crate) fn buflist_findname_file_id(
    ffname: *mut c_char,
    file_id: &FileID,
    file_id_valid: bool,
) -> Option<Buf> {
    let file_id = (&raw const *file_id).cast_mut();
    buffers_backwards().find(|buf| {
        // SAFETY: a live buffer, a NUL-terminated name and a live file id.
        !buf.b_flags.has(BufFlags::DUMMY)
            && !unsafe { otherfile_buf(buf.raw(), ffname, file_id, file_id_valid) }
    })
}

// ---------------------------------------------------------------------------
// Finding one by pattern

/// The number of the buffer matching `pattern`: `-1` when there is none and
/// `-2` when more than one does, both reported to the user.
///
/// `unlisted` searches the unlisted buffers too, when no listed one matched;
/// `curtab_only` ignores buffers not open in the current tab page.
pub unsafe fn buflist_findpat(
    pattern: *const c_char,
    pattern_end: *const c_char,
    unlisted: bool,
    diffmode: bool,
    curtab_only: bool,
) -> c_int {
    let one_byte = pattern_end == pattern.wrapping_add(1);
    // SAFETY: a one-byte pattern, which the caller promised is readable.
    // Upstream reads it behind the same test, in one `&&` chain.
    let head = if one_byte { unsafe { *pattern } } else { 0 };
    let shorthand = one_byte && (head == b'%' as c_char || head == b'#' as c_char);
    let matched = if shorthand {
        match_shorthand(head, diffmode)
    } else {
        match match_pattern(pattern, pattern_end, unlisted, diffmode, curtab_only) {
            // An unusable pattern is reported by whoever built it.
            None => return -1,
            Some(matched) => matched,
        }
    };

    if matched == -2 {
        err_pat(c"E93: More than one match for %s", pattern);
    } else if matched < 0 {
        err_pat(c"E94: No matching buffer for %s", pattern);
    }
    matched
}

/// `%` (the current buffer) and `#` (the alternate file), which never reach
/// the regexp.
fn match_shorthand(head: c_char, diffmode: bool) -> c_int {
    let matched = if head == b'%' as c_char {
        // SAFETY: `curbuf` is set from startup to exit.
        unsafe { Buf::current() }.handle as c_int
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
    pattern: *const c_char,
    pattern_end: *const c_char,
    unlisted: bool,
    diffmode: bool,
    curtab_only: bool,
) -> Option<c_int> {
    // SAFETY: the caller's promise -- a pattern between the two pointers.
    let pat = unsafe { file_pat_to_reg_pat(pattern, pattern_end, ptr::null_mut(), 0) };
    if pat.is_null() {
        return None;
    }
    // SAFETY: a NUL-terminated allocation.
    let patlen = unsafe { strlen(pat) };
    // SAFETY: `patlen` bytes plus the terminator, all of them writable.
    let buf = unsafe { slice::from_raw_parts_mut(pat.cast::<u8>(), patlen + 1) };
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
            let mut regmatch = NO_REGMATCH;
            regmatch.regprog = regcomp(&buf[from..], flags);

            for b in buffers_backwards() {
                if regmatch.regprog.is_null() {
                    // An invalid pattern, possibly after switching engine.
                    free(pat);
                    return None;
                }
                if (b.b_p_bl != 0) != find_listed
                    || diffmode && !is_diff_mode(b)
                    || buflist_match(&mut regmatch, b, false).is_null()
                {
                    continue;
                }
                // Ignore the match when the buffer is not open in the
                // current tab page.
                if curtab_only && !windows().any(|win| win.w_buffer == b.raw()) {
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

            free_regprog(&mut regmatch.regprog);
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

    free(pat);
    Some(matched)
}

// ---------------------------------------------------------------------------
// Sorting by last-used time

/// `qsort`'s comparison over two `buf_T *`, most recently used first. Two
/// buffers entered in the same second tie, and the order of a tie is
/// whatever `qsort` lands on -- which is why the sort stays `qsort`.
pub(crate) unsafe extern "C" fn buf_time_compare(s1: *const c_void, s2: *const c_void) -> c_int {
    // SAFETY: `qsort` hands back two elements of the array it was given,
    // each holding a live buffer pointer.
    let (buf1, buf2) = unsafe {
        (
            Buf::new(*s1.cast::<*mut buf_T>()),
            Buf::new(*s2.cast::<*mut buf_T>()),
        )
    };
    if buf1.b_last_used == buf2.b_last_used {
        return 0;
    }
    if buf1.b_last_used > buf2.b_last_used {
        -1
    } else {
        1
    }
}
