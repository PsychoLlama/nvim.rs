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
#![allow(unsafe_code)]

use crate::cstr;
use crate::types::AutoEvent;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::{ptr, slice};

use super::*;
use crate::allocator::Owned;
use crate::autocmd::apply_autocmds;
use crate::cursor::{check_cursor_col, check_cursor_lnum};
use crate::diff::diff_mode_buf;
use crate::eval::typval::{callback_free, tv_dict_alloc};
use crate::eval::vars::init_var_dict;
use crate::ex_cmds::getfile;
use crate::ex_docmd::tabpage_new;
use crate::ex_eval::aborting;
use crate::ex_getln::text_or_buf_locked;
use crate::fileio::file_pat_to_reg_pat;
use crate::guard::Suppress;
use crate::hashtab::hash_init;
use crate::insexpand::clear_cpt_callbacks;
use crate::mark::{clrallmarks, fmarks_check_names, mark_view_restore};
use crate::memory::{xfree, xstrdup};
use crate::message::e_noalt;
use crate::message::state::{emsg_silent, in_assert_fails};
use crate::message::{emsg_ptr, msg_delay};
use crate::message_fmt::c_str;
use crate::option::vars::{jop_flags, p_sol, swb_flags};
use crate::option::{buf_copy_options, magic_isset};
use crate::options::{kOptJopFlagView, kOptSwbFlagNewtab, kOptSwbFlagSplit, kOptSwbFlagVsplit};
use crate::optionstr::clear_string_option;
use crate::os::cshim::gettext_ptr;
use crate::os::fs::os_fileid;
use crate::path::full_name_save;
use crate::pos::MAXLNUM;
use crate::regexp::{RE_MAGIC, vim_regcomp, vim_regfree};
use crate::registry::id_map;
use crate::semsg;
use crate::syntax::init_synblock;
use crate::types::{
    AdditionalData, Buffer, Callback, ColNr, Failed, FileID, FileMark, FileMarkView, Handle,
    LineNr, MemLine, OptInt, Pos, RegProg, Timestamp, VAR_SCOPE, int16_t, size_t, uint64_t,
};
use crate::undo::curbuf_is_changed;
use crate::window::{WSP_VERT, swbuf_goto_win_with_buf, win_split};
use crate::winlayer::graph::{firstbuf, lastbuf};
use crate::winlayer::{Buf, Win, buffers_back, register_buffer, windows};

use super::expand::{NO_REGMATCH, buflist_match, find_buf};
use super::pos::{Entry, WinInfos};

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
    unsafe { gettext_ptr(msg).as_ptr().cast_mut() }
}

fn err(msg: *mut c_char) {
    // SAFETY: a NUL-terminated message.
    unsafe { emsg_ptr(msg) };
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

fn clear_cpt(callbacks: &mut *mut Callback, count: c_int) {
    // SAFETY: the buffer's own 'complete' callback array and its length.
    unsafe { clear_cpt_callbacks(callbacks, count) };
}

fn free_regprog(prog: &mut *mut RegProg) {
    // SAFETY: a compiled program or null.
    unsafe { vim_regfree(*prog) };
    *prog = ptr::null_mut();
}

fn regcomp(pat: &[u8], flags: c_int) -> *mut RegProg {
    // SAFETY: a NUL-terminated pattern; the answer is null on a bad one.
    unsafe { vim_regcomp(pat.as_ptr().cast::<c_char>(), flags) }
}

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
    let (none, some) = (ptr::null_mut(), Some(buffer));
    // SAFETY: a live buffer, and no pattern to match against.
    unsafe { apply_autocmds(event, none, none, false, some) }
}

fn copy_options_into(buffer: Buf, flags: c_int) {
    // SAFETY: a live buffer.
    unsafe { buf_copy_options(buffer, flags) };
}

fn check_cursor_column(win: Win) {
    // SAFETY: a live window.
    check_cursor_col(win);
}

fn check_cursor_line(win: Win) {
    // SAFETY: a live window.
    check_cursor_lnum(win);
}

// ---------------------------------------------------------------------------
// Creating an entry

/// Add a file to the buffer list, or answer the entry it already has.
///
/// `lnum` is the line to remember for it and `flags` the `BLN_*` set. The
/// answer is null when an autocommand deleted the buffer under us.
///
/// # Safety
///
/// `ffname_arg` must point at a NUL-terminated string, unaliased for the
/// call. `sfname_arg` must point at a NUL-terminated string, unaliased for
/// the call.
pub unsafe fn buflist_new(
    ffname_arg: *mut c_char,
    sfname_arg: *mut c_char,
    lnum: LineNr,
    flags: c_int,
) -> Option<Buf> {
    let mut ffname = ffname_arg;
    let mut sfname = sfname_arg;

    // Will allocate ffname.
    // SAFETY: two locals holding a name each.
    unsafe { fname_expand(&raw mut ffname, &raw mut sfname) };

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
    if flags & BLN_CURBUF as c_int != 0 && curbuf_reusable() {
        let cur = current_buf().expect("curbuf != NULL");
        let bufref = BufRef::of(cur);
        trigger_undo_ftplugin(cur, current_win());
        // It is as if this buffer were deleted. Watch out for autocommands
        // that change curbuf: if that happens, allocate a new buffer anyway.
        buf_freeall(cur, BFA_WIPE as c_int | BFA_DEL as c_int);
        if aborting() {
            // Autocommands may abort script processing.
            free(ffname);
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
            let owned = new_buffer();
            // SAFETY: the allocation just made, which `fresh` keeps alive
            // until the registry takes it over a few lines below.
            let buf = unsafe { Buf::new(owned.address()) };
            fresh = Some(owned);
            buf
        }
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
        // `fresh` is `Some` exactly when the buffer is not the reused
        // current one, which is the branch this is.
        buf = append_to_list(buf, fresh.take().expect("a fresh buffer was allocated"));
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
    buf_clear_file(buf);
    clrallmarks(buf, 0 as Timestamp);
    fmarks_check_names(buf);
    // Init 'buflisted'.
    buf.b_p_bl = if flags & BLN_LISTED as c_int != 0 {
        1
    } else {
        0
    };
    reset_update_subscribers(&mut buf);

    if flags & BLN_DUMMY as c_int == 0 && !announce_new_buffer(buf, flags) {
        return None;
    }

    buf.b_prompt_callback = Callback::None;
    buf.b_prompt_interrupt = Callback::None;
    buf.b_prompt_text = ptr::null_mut();
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
    // SAFETY: the allocation just made, which `owned` keeps alive.
    let mut buf = unsafe { Buf::new(owned.address()) };
    // Init the b: variables.
    // SAFETY: a fresh dictionary for the buffer's own `b:` scope.
    // The buffer's storage owns it: `init_var_dict` seeds it with
    // `DO_NOT_FREE_CNT` and `unref_var_dict` gives the block back.
    buf.b_vars = tv_dict_alloc().into_raw();
    let (vars, scope_var) = (buf.b_vars, &raw mut buf.b_bufvar);
    // SAFETY: the dictionary just allocated and the buffer's scope variable.
    unsafe { init_var_dict(vars, scope_var, VAR_SCOPE) };
    buf_init_changedtick(buf);
    owned
}

/// A `Buffer` that lives **outside** the handle registry and off the buffer
/// list: scratch storage whose only real content is a memline.
///
/// Two exist, and both are the editor's own business rather than the user's:
/// `ml_recover`'s, which holds the memline of the swap file being read, and
/// `open_spellbuf`'s, which backs a `.sug` word list with a swap file so it
/// need not stay in memory. Neither is given a handle, neither is registered
/// (so `find_buf`, `nvim_list_bufs` and every `FOR_ALL_BUFFERS` walk
/// are blind to them, which is the point), and neither goes through
/// `close_buffer`/`free_buffer` — each is freed by the code that made it.
///
/// The constructor is named so that "the registry owns every buffer" has a
/// stated exception rather than a silent one; see [`new_buffer`] for the
/// ordinary path.
///
/// Most fields stay zero and are *not* valid buffer state: string options
/// are null, there are no `b:` variables and no undo information. Only what
/// the caller fills in afterwards may be read.
pub(crate) fn alloc_unregistered_buffer() -> Owned<Buffer> {
    let mut storage = Box::<Buffer>::new_zeroed();
    let at = storage.as_mut_ptr();
    // The fields a zeroed `Buffer` is *not* a valid value for -- an empty
    // `Vec` holds a non-null dangling pointer, not a zero one, and a `HashMap`
    // holds a seeded hasher -- are the user-command list, the keymap, the
    // buffer's syntax block (`init_synblock`), what the memline owns, and
    // the two extmark tables.
    // SAFETY: all are inside the block just allocated, nothing has read or
    // dropped them, and `write` does not drop what was there.
    unsafe { (&raw mut (*at).b_ucmds).write(Vec::new()) };
    unsafe { (&raw mut (*at).b_kmap_ga).write(Vec::new()) };
    unsafe { init_synblock(&raw mut (*at).b_s) };
    unsafe { (&raw mut (*at).b_ml).write(MemLine::closed()) };
    unsafe { (&raw mut (*at).b_marktree).write(MarkTree::EMPTY) };
    unsafe { (&raw mut (*at).b_extmark_ns).write(id_map()) };
    // SAFETY: all-zero bytes are otherwise what upstream's
    // `xcalloc(1, sizeof(Buffer))` hands a fresh buffer.
    Owned::new(unsafe { storage.assume_init() })
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
        err(tr(c"W14: Warning: List of file names overflow"));
        if emsg_silent.get() == 0 && !in_assert_fails.get() {
            // Make sure it is noticed.
            msg_delay(3001 as uint64_t, true);
        }
        top_file_num.set(1);
    }
    buffer
}

fn init_hashtabs(mut buffer: Buf) {
    let (keywords, keywords_ic) = (
        &raw mut buffer.b_s.b_keywtab,
        &raw mut buffer.b_s.b_keywtab_ic,
    );
    // SAFETY: a hash table inside a live buffer.
    unsafe { hash_init(keywords) };
    // SAFETY: as above.
    unsafe { hash_init(keywords_ic) };
}

/// `kv_destroy` + `kv_init` of the two buffer-update subscriber arrays: a
/// reused buffer must not keep the old one's subscribers.
fn reset_update_subscribers(buffer: &mut Buf) {
    xfree_clear(&mut buffer.update_channels.items);
    buffer.update_channels.capacity = 0 as size_t;
    buffer.update_channels.size = 0 as size_t;
    xfree_clear(&mut buffer.update_callbacks.items);
    buffer.update_callbacks.capacity = 0 as size_t;
    buffer.update_callbacks.size = 0 as size_t;
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
    let empty = buf.b_ml.ml_mfp.is_null() || buf_is_empty(buf);
    buf.b_ffname.is_null()
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
        clear_opt(&mut buffer.b_p_fenc);
        clear_opt(&mut buffer.b_p_ff);
        clear_opt(&mut buffer.b_p_bh);
        clear_opt(&mut buffer.b_p_bt);
    }
    clear_opt(&mut buffer.b_p_def);
    clear_opt(&mut buffer.b_p_inc);
    clear_opt(&mut buffer.b_p_inex);
    clear_opt(&mut buffer.b_p_inde);
    clear_opt(&mut buffer.b_p_indk);
    clear_opt(&mut buffer.b_p_fp);
    clear_opt(&mut buffer.b_p_fex);
    clear_opt(&mut buffer.b_p_kp);
    clear_opt(&mut buffer.b_p_mps);
    clear_opt(&mut buffer.b_p_fo);
    clear_opt(&mut buffer.b_p_flp);
    clear_opt(&mut buffer.b_p_isk);
    clear_opt(&mut buffer.b_p_vsts);
    xfree_clear(&mut buffer.b_p_vsts_nopaste);
    xfree_clear(&mut buffer.b_p_vsts_array);
    clear_opt(&mut buffer.b_p_vts);
    xfree_clear(&mut buffer.b_p_vts_array);
    clear_opt(&mut buffer.b_p_keymap);
    buffer.b_kmap_ga = Vec::new();
    clear_opt(&mut buffer.b_p_com);
    clear_opt(&mut buffer.b_p_cms);
    clear_opt(&mut buffer.b_p_nf);
    clear_opt(&mut buffer.b_p_syn);
    clear_opt(&mut buffer.b_s.b_syn_isk);
    clear_opt(&mut buffer.b_s.b_p_spc);
    clear_opt(&mut buffer.b_s.b_p_spf);
    free_regprog(&mut buffer.b_s.b_cap_prog);
    clear_opt(&mut buffer.b_s.b_p_spl);
    clear_opt(&mut buffer.b_s.b_p_spo);
    clear_opt(&mut buffer.b_p_sua);
    clear_opt(&mut buffer.b_p_ft);
    clear_opt(&mut buffer.b_p_cink);
    clear_opt(&mut buffer.b_p_cino);
    clear_opt(&mut buffer.b_p_lop);
    clear_opt(&mut buffer.b_p_cinsd);
    clear_opt(&mut buffer.b_p_cinw);
    clear_opt(&mut buffer.b_p_cot);
    clear_opt(&mut buffer.b_p_cpt);
    clear_opt(&mut buffer.b_p_cfu);
    clear_callback(&mut buffer.b_cfu_cb);
    clear_opt(&mut buffer.b_p_ofu);
    clear_callback(&mut buffer.b_ofu_cb);
    clear_opt(&mut buffer.b_p_tsrfu);
    clear_callback(&mut buffer.b_tsrfu_cb);
    let cpt_count = buffer.b_p_cpt_count;
    clear_cpt(&mut buffer.b_p_cpt_cb, cpt_count);
    buffer.b_p_cpt_count = 0;
    clear_opt(&mut buffer.b_p_gefm);
    clear_opt(&mut buffer.b_p_gp);
    clear_opt(&mut buffer.b_p_mp);
    clear_opt(&mut buffer.b_p_efm);
    clear_opt(&mut buffer.b_p_ep);
    clear_opt(&mut buffer.b_p_path);
    clear_opt(&mut buffer.b_p_tags);
    clear_opt(&mut buffer.b_p_tc);
    clear_opt(&mut buffer.b_p_tfu);
    clear_callback(&mut buffer.b_tfu_cb);
    clear_opt(&mut buffer.b_p_ffu);
    clear_callback(&mut buffer.b_ffu_cb);
    clear_opt(&mut buffer.b_p_dict);
    clear_opt(&mut buffer.b_p_dia);
    clear_opt(&mut buffer.b_p_tsr);
    clear_opt(&mut buffer.b_p_qe);
    buffer.b_p_ac = -1;
    buffer.b_p_ar = -1;
    buffer.b_p_fs = -1;
    buffer.b_p_ul = NO_LOCAL_UNDOLEVEL as OptInt;
    clear_opt(&mut buffer.b_p_lw);
    clear_opt(&mut buffer.b_p_bkc);
    clear_opt(&mut buffer.b_p_menc);
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
            err(tr_raw(e_noalt.as_ptr()));
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
    let mut fm: *mut FileMark = ptr::null_mut();
    let mut restore_view = false;
    if lnum == 0 as LineNr {
        // Default line number: where the cursor was left last time.
        fm = buflist_findfmark(buf);
        // SAFETY: as above.
        (lnum, col) = unsafe { ((*fm).mark.lnum, (*fm).mark.col) };
        restore_view = true;
    }

    if options & GETF_SWITCH as c_int != 0 && !goto_existing_window(buf) {
        return Err(Failed);
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
        return Err(Failed);
    }

    if p_sol.get() == 0 && col != 0 {
        let mut win = current_win();
        win.w_cursor.col = col;
        check_cursor_column(win);
        win.w_cursor.coladd = 0 as ColNr;
        win.w_set_curswant = true;
    }
    if jop_flags.get() & kOptJopFlagView as c_int as u32 != 0 && restore_view {
        // SAFETY: the mark read above, which is still live.
        unsafe { mark_view_restore(fm) };
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
    // SAFETY: a live mark.
    let (lnum, col) = unsafe { ((*fm).mark.lnum, (*fm).mark.col) };

    let mut win = current_win();
    win.w_cursor.lnum = lnum;
    check_cursor_line(win);
    if p_sol.get() != 0 {
        win.w_cursor.col = 0 as ColNr;
    } else {
        win.w_cursor.col = col;
        check_cursor_column(win);
        win.w_cursor.coladd = 0 as ColNr;
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
///
/// # Safety
///
/// `fname` must point at a NUL-terminated string, unaliased for the call.
pub unsafe fn buflist_findname_exp(fname: *mut c_char) -> Option<Buf> {
    // SAFETY: a NUL-terminated name; the answer is an allocation or null.
    let ffname = unsafe { full_name_save(fname, true) };
    if ffname.is_null() {
        return None;
    }
    // SAFETY: the full name just built.
    let buf = unsafe { buflist_findname(ffname) };
    free(ffname);
    buf
}

/// The buffer whose full name is `ffname`, or whose file id matches it.
///
/// # Safety
///
/// `ffname` must point at a NUL-terminated string, unaliased for the call.
pub unsafe fn buflist_findname(ffname: *mut c_char) -> Option<Buf> {
    let mut file_id = NO_FILE_ID;
    // SAFETY: the caller's promise -- a NUL-terminated name -- and a local.
    let file_id_valid = unsafe { os_fileid(ffname, &raw mut file_id) };
    buflist_findname_file_id(ffname, &file_id, file_id_valid)
}

/// [`buflist_findname`] with the file id already looked up. Dummy buffers do
/// not count.
pub(crate) fn buflist_findname_file_id(
    ffname: *mut c_char,
    file_id: &FileID,
    file_id_valid: bool,
) -> Option<Buf> {
    let file_id = (&raw const *file_id).cast_mut();
    buffers_back().find(|buf| {
        // SAFETY: a live buffer, a NUL-terminated name and a live file id.
        !buf.b_flags.has(BufFlags::DUMMY)
            && !unsafe { otherfile_buf(*buf, ffname, file_id, file_id_valid) }
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
/// # Safety
///
/// `pattern` must point at a NUL-terminated string. `pattern_end` must point
/// at a NUL-terminated string.
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
        // SAFETY: the caller's NUL-terminated pattern.
        let pattern = unsafe { c_str(pattern) };
        semsg!("E93: More than one match for {pattern}");
    } else if matched < 0 {
        // SAFETY: the caller's NUL-terminated pattern.
        let pattern = unsafe { c_str(pattern) };
        semsg!("E94: No matching buffer for {pattern}");
    }
    matched
}

/// `%` (the current buffer) and `#` (the alternate file), which never reach
/// the regexp.
fn match_shorthand(head: c_char, diffmode: bool) -> c_int {
    let matched = if head == b'%' as c_char {
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
    let patlen = unsafe { cstr::bytes_at(pat) }.len();
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

            for b in buffers_back() {
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

/// `qsort`'s comparison over two `Buffer *`, most recently used first. Two
/// buffers entered in the same second tie, and the order of a tie is
/// whatever `qsort` lands on -- which is why the sort stays `qsort`.
///
/// # Safety
///
/// As `qsort`'s comparator: `s1` and `s2` must each point at an element of
/// the array being sorted, and the elements must be of the type this reads
/// them at.
pub(crate) unsafe extern "C" fn buf_time_compare(s1: *const c_void, s2: *const c_void) -> c_int {
    // SAFETY: `qsort` hands back two elements of the array it was given,
    // each holding a live buffer pointer.
    let (buf1, buf2) = unsafe {
        (
            Buf::new(*s1.cast::<*mut Buffer>()),
            Buf::new(*s2.cast::<*mut Buffer>()),
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
