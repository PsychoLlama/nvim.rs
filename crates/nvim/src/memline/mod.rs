//! The memline: a buffer's lines, stored in a B-tree of blocks that a
//! [`memfile`](super::memfile) keeps paged to a swap file.
//!
//! Pointer blocks branch by line count and data blocks hold the text; block
//! zero is the header that makes a swap file recognisable to another Nvim.
//! This module owns opening and closing one; the children own the rest —
//! [`tree`] the walk, [`edit`] the insert and delete, [`lines`] the API the
//! editor calls, [`offsets`] byte offsets, [`block0`] the header,
//! [`swapname`] where the swap file goes and [`recover`] reading one back.

#![deny(unsafe_op_in_unsafe_fn)]

use core::mem::offset_of;

use crate::api::private::helpers::cstr_as_string;
use crate::autocmd::{
    EVENT_BUFREADPOST, EVENT_BUFWINENTER, EVENT_SWAPEXISTS, apply_autocmds, has_autocmd,
};
use crate::buffer::{BufFlags, buf_inc_changedtick, buf_spname, open_buffer, setfname};
use crate::change::{changed_internal, unchanged};
use crate::cursor::{check_cursor, coladvance};
use crate::drawscreen::{UPD_NOT_VALID, redraw_curbuf_later};
use crate::eval::typval::{tv_dict_add_nr, tv_dict_add_str_len, tv_list_append_allocated_string};
use crate::eval::vars::{get_vim_var_str, set_vim_var_string};
use crate::event::libuv::{uv_strerror, uv_uptime};
use crate::ex_docmd::cmdmod_has;
use crate::fileio::{
    buf_store_file_info, modname, read_eintr, readfile, vim_deltempdir, vim_rename, vim_tempname,
};
use crate::getchar::flush_buffers;
use crate::global_cell::GlobalCell;
use crate::guard::{Allow, Suppress};
use crate::input::prompt_for_input;
use crate::main::{
    allbuf_lock, cmdline_row, curbuf, did_check_timestamps, getout, got_int, inhibit_delete_count,
    msg_ext_skip_flush, msg_row, msg_silent, need_check_timestamps, need_wait_return, no_lines_msg,
    p_dir, p_shm, p_uc, p_verbose, recoverymode, swap_exists_action,
};
use crate::mark::setpcmark;
use crate::mbyte::{mb_adjust_cursor, mb_utflen, utf_head_off, utf_ptr2char, utfc_ptr2len};
use crate::memfile::{
    MfDirty, mf_close, mf_close_file, mf_find, mf_fname, mf_free, mf_free_fnames, mf_get,
    mf_need_trans, mf_new, mf_new_page_size, mf_open, mf_open_file, mf_put, mf_set_dirty,
    mf_set_fnames, mf_sync, mf_trans_del,
};
use crate::memory::{xfree, xmalloc, xmemdupz, xrealloc, xstpcpy, xstrdup, xstrlcpy};
use crate::message::{
    do_dialog, emsg, iemsg, msg, msg_end, msg_ext_set_kind, msg_home_replace, msg_multiline,
    msg_outnum, msg_outtrans, msg_putchar, msg_puts, msg_puts_hl, msg_reset_scroll, msg_start,
    set_keep_msg, verb_msg,
};
use crate::option::{copy_option_part, get_fileformat, set_fileformat, set_option_value_give_err};
use crate::options::kOptFileencoding;
use crate::os::cshim::{gettext, memmove, strncasecmp, strncmp};
use crate::os::env::{expand_env, home_replace, home_replace_save, os_get_hostname, os_get_pid};
use crate::os::fs::{
    os_fileinfo, os_fileinfo_inode, os_fileinfo_link, os_fileinfo_size, os_isdir, os_mkdir_recurse,
    os_open, os_path_exists, os_remove, os_set_cloexec,
};
use crate::os::input::{line_breakcheck, os_char_avail};
use crate::os::proc::os_proc_running;
use crate::os::time::{os_ctime_r, os_time};
use crate::os::users::{os_get_uname, os_get_username};
use crate::path::{
    after_pathsep, concat_fnames, expand_wildcards, fix_fname, free_wild, path_fnamecmp,
    path_full_compare, path_is_absolute, path_tail, same_directory, shorten_dir, vim_full_name,
    vim_ispathsep,
};
use crate::pos::MAXLNUM;
use crate::semsg_c;
use crate::spell::spell_delete_wordlist;
use crate::statusline::get_trans_bufname;
use crate::strings::{kv_do_printf, vim_strchr, xstrnsave};
use crate::types::ui::kUIMessages;
use crate::types::{
    CmdModFlags, FAIL, FileInfo, NUL, OK, OptVal, OptValData, OptValType, String_0, StringBuilder,
    Timestamp, bhdr_T, blocknr_T, buf_T, colnr_T, dict_T, file_comparison, flush_buffers_T,
    infoptr_T, int16_t, int64_t, linenr_T, list_T, memfile_T, off_T, pos_T, size_t, ssize_t,
    time_t, uint8_t, uint16_t, uint64_t, uv_uid_t, varnumber_T,
};
use crate::ui::{ui_flush, ui_has};
use crate::undo::buf_is_changed;
use crate::version::min_vim_version_name;
use crate::winlayer::{Buf, buffers};
use ::libc::{__errno_location, close, lseek, readlink, strcasecmp, strcmp, strcpy, strlen};

// The carve of the transpiled module; see each child's docs.
mod block0;
pub use self::block0::*;
mod swapname;
pub use self::swapname::*;
mod recover;
pub use self::recover::*;
mod tree;
pub(crate) use self::tree::*;
mod edit;
pub(crate) use self::edit::*;
mod offsets;
pub use self::offsets::*;
mod lines;
pub use self::lines::*;
pub const kOptValTypeString: OptValType = 2;
pub const VIM_WARNING: ::core::ffi::c_uint = 2;
pub const READ_NEW: ::core::ffi::c_uint = 1;
pub const FLUSH_TYPEAHEAD: flush_buffers_T = 1;
pub const MFS_ZERO: ::core::ffi::c_uint = 8;
pub const MFS_FLUSH: ::core::ffi::c_uint = 4;
pub const MFS_STOP: ::core::ffi::c_uint = 2;
pub const MFS_ALL: ::core::ffi::c_uint = 1;
pub const MIN_SWAP_PAGE_SIZE: ::core::ffi::c_uint = 1048;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DataBlock {
    pub db_id: uint16_t,
    pub db_free: ::core::ffi::c_uint,
    pub db_txt_start: ::core::ffi::c_uint,
    pub db_txt_end: ::core::ffi::c_uint,
    pub db_line_count: ::core::ffi::c_long,
    pub db_index: [::core::ffi::c_uint; 0],
}
pub const DATA_ID: ::core::ffi::c_uint = 25697;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PointerEntry {
    pub pe_bnum: blocknr_T,
    pub pe_line_count: linenr_T,
    pub pe_old_lnum: linenr_T,
    pub pe_page_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PointerBlock {
    pub pb_id: uint16_t,
    pub pb_count: uint16_t,
    pub pb_count_max: uint16_t,
    pub pb_pointer: [PointerEntry; 0],
}
pub const PTR_ID: ::core::ffi::c_uint = 28788;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ZeroBlock {
    pub b0_id: [::core::ffi::c_char; 2],
    pub b0_version: [::core::ffi::c_char; 10],
    pub b0_page_size: [::core::ffi::c_char; 4],
    pub b0_mtime: [::core::ffi::c_char; 4],
    pub b0_ino: [::core::ffi::c_char; 4],
    pub b0_pid: [::core::ffi::c_char; 4],
    pub b0_uname: [::core::ffi::c_char; 40],
    pub b0_hname: [::core::ffi::c_char; 40],
    pub b0_fname: [::core::ffi::c_char; 900],
    pub b0_magic_long: ::core::ffi::c_long,
    pub b0_magic_int: ::core::ffi::c_int,
    pub b0_magic_short: int16_t,
    pub b0_magic_char: ::core::ffi::c_char,
}
pub const B0_HNAME_SIZE: ::core::ffi::c_uint = 40;
pub const B0_UNAME_SIZE: ::core::ffi::c_uint = 40;
pub const B0_FNAME_SIZE_ORG: ::core::ffi::c_uint = 900;
pub const B0_FNAME_SIZE_NOCRYPT: ::core::ffi::c_uint = 898;
pub const B0_FNAME_SIZE_CRYPT: ::core::ffi::c_uint = 890;
pub const B0_MAGIC_CHAR: ::core::ffi::c_uint = 85;
pub const B0_MAGIC_SHORT: ::core::ffi::c_uint = 269554195;
pub const B0_MAGIC_INT: ::core::ffi::c_uint = 539042339;
pub const B0_MAGIC_LONG: ::core::ffi::c_uint = 808530483;
pub const BLOCK0_ID1: ::core::ffi::c_uint = 48;
pub const BLOCK0_ID0: ::core::ffi::c_uint = 98;
pub const SEA_CHOICE_NONE: sea_choice_T = 0;
pub type sea_choice_T = ::core::ffi::c_uint;
pub const SEA_CHOICE_ABORT: sea_choice_T = 6;
pub const SEA_CHOICE_QUIT: sea_choice_T = 5;
pub const SEA_CHOICE_DELETE: sea_choice_T = 4;
pub const SEA_CHOICE_RECOVER: sea_choice_T = 3;
pub const SEA_CHOICE_EDIT: sea_choice_T = 2;
pub const SEA_CHOICE_READONLY: sea_choice_T = 1;
pub type upd_block0_T = ::core::ffi::c_uint;
pub const UB_SAME_DIR: upd_block0_T = 1;
pub const UB_FNAME: upd_block0_T = 0;
/// `action` for [`ml_find_line`]: only release the locked block.
pub const ML_FLUSH: ::core::ffi::c_int = 2;
/// `action` for [`ml_find_line`]: a line is about to be deleted here.
pub const ML_DELETE: ::core::ffi::c_int = 17;
/// `action` for [`ml_find_line`]: a line is about to be inserted here.
pub const ML_INSERT: ::core::ffi::c_int = 18;
/// `action` for [`ml_find_line`]: just look the line up.
pub const ML_FIND: ::core::ffi::c_int = 19;
/// Lines below which two neighbouring chunk-index entries are merged, and
/// the size a split aims for.
pub const MLCS_MINL: ::core::ffi::c_int = 400;
/// Lines above which a chunk-index entry is split in two.
pub const MLCS_MAXL: ::core::ffi::c_int = 800;
/// `flags` for `ml_append_int`: carry the `DB_MARKED` bit onto the new line.
pub const ML_APPEND_MARK: ::core::ffi::c_int = 2;
/// `flags` for `ml_append_int`: this is a fresh file being read in, so the
/// block may be numbered negatively and need not keep its position.
pub const ML_APPEND_NEW: ::core::ffi::c_int = 1;
/// `flags` for `ml_delete_int`: say "--No lines in buffer--" if the buffer
/// ends up empty.
pub const ML_DEL_MESSAGE: ::core::ffi::c_int = 1;
pub const kEqualFiles: file_comparison = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const O_RDWR: ::core::ffi::c_int = 0o2 as ::core::ffi::c_int;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KV_INITIAL_VALUE: StringBuilder = StringBuilder {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
pub const ML_CHNK_ADDLINE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ML_CHNK_DELLINE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ML_CHNK_UPDLINE: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
crate::flag_set! {
    /// What a buffer's memline is holding right now -- upstream's `ML_*`,
    /// the bits [`memline_T::ml_flags`] carries.
    ///
    /// c2rust emitted these as a bare `int` and re-emitted the `#define`s
    /// once per translation unit: seventeen copies of `ML_EMPTY` alone, in
    /// seventeen modules. There is one definition now.
    pub struct MlFlags;

    /// The buffer holds a single empty line and nothing has been written
    /// into it -- what a brand new buffer is, and what `:enew` returns one
    /// to. Cleared by the first line that is appended or replaced.
    const EMPTY = 0x1;
}
pub const BH_DIRTY: ::core::ffi::c_uint = 1 as ::core::ffi::c_uint;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SEA_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEA_QUIT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SEA_RECOVER: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SEA_READONLY: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const DB_MARKED: ::core::ffi::c_uint = (1 as ::core::ffi::c_int as ::core::ffi::c_uint)
    << ::core::mem::size_of::<::core::ffi::c_uint>()
        .wrapping_mul(8_usize)
        .wrapping_sub(1_usize);
pub const DB_INDEX_MASK: ::core::ffi::c_uint = !DB_MARKED;
pub const INDEX_SIZE: usize = ::core::mem::size_of::<::core::ffi::c_uint>();
pub const HEADER_SIZE: ::core::ffi::c_ulong = 24 as ::core::ffi::c_ulong;
pub const B0_DIRTY: ::core::ffi::c_int = 0x55 as ::core::ffi::c_int;
pub const B0_FF_MASK: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const B0_SAME_DIR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const B0_HAS_FENC: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const STACK_INCR: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
/// A translated static message, as the C string the message layer takes.
///
/// `gettext` asks only for a live NUL-terminated string, which is what a
/// `CStr` is; paying that once here is what keeps the forty-odd
/// `msg_puts(gettext(c"..."))` in this family out of an `unsafe` region.
fn tr(text: &::core::ffi::CStr) -> *mut ::core::ffi::c_char {
    // SAFETY: a `CStr` is NUL-terminated by construction.
    unsafe { gettext(text.as_ptr()) }
}

/// One translated static message on the report, in `hl_id`.
///
/// These four exist for the same reason [`tr`] does: the message layer asks
/// only for a live NUL-terminated string, and a `CStr` is one, so the
/// forty-odd reports `recover.rs` and `swapname.rs` print need not each be
/// an `unsafe` region -- and most of them went vertical, one line per
/// argument, because the argument list did not fit.
fn note(text: &::core::ffi::CStr, hl_id: ::core::ffi::c_int) {
    // SAFETY: `tr` answers a live NUL-terminated string.
    unsafe { msg_puts_hl(tr(text), hl_id, true) };
}

/// [`note`], appended to the message being built.
fn say(text: &::core::ffi::CStr) {
    // SAFETY: as [`note`].
    unsafe { msg_puts(tr(text)) };
}

/// [`note`], as an error.
fn complain(text: &::core::ffi::CStr) {
    // SAFETY: as [`note`].
    unsafe { emsg(tr(text)) };
}

/// [`note`], as a message of its own.
fn tell(text: &::core::ffi::CStr, hl_id: ::core::ffi::c_int) {
    // SAFETY: as [`note`].
    unsafe { msg(tr(text), hl_id) };
}

/// The lowest line number that may still carry a [`DB_MARKED`] bit, so
/// `ml_firstmarked` need not start its search at line one.
static lowest_marked: GlobalCell<linenr_T> = GlobalCell::new(0);

/// Published by `swapfile_info` for the ATTENTION dialog: whether the process
/// that owns the swap file it just described is still alive.
static proc_running: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);

/// Open a new memline for `buf`: an in-memory memfile with block zero, a root
/// pointer block and one data block holding a single empty line.
///
/// No swap file is created here; [`ml_open_file`] does that later.
///
/// # Safety
/// `buf` must point at a buffer with no memline open.
pub unsafe fn ml_open(buf: *mut buf_T) -> ::core::ffi::c_int {
    // SAFETY: the caller's buffer, reached through a handle that
    // borrows it for the one access that asked and no longer.
    let mut b = unsafe { Buf::new(buf) };
    // No stack, no cached block, no cached line, no chunk table yet.
    b.b_ml.stack_clear();
    b.b_ml.ml_locked = None;
    b.b_ml.clear_cache();
    b.b_ml.ml_chunks.free();

    if cmdmod_has(CmdModFlags::NOSWAPFILE) {
        b.b_p_swf = 0;
    }
    // A swap file may still be opened later, when 'updatecount' is set.
    unsafe {
        (*buf).b_may_swap = (*buf).terminal.is_null() && p_uc.get() != 0 && (*buf).b_p_swf != 0
    };

    let mfp = unsafe { mf_open(::core::ptr::null_mut(), 0) };
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut();
    if !mfp.is_null() {
        b.b_ml.ml_mfp = mfp;
        b.b_ml.ml_flags = MlFlags::EMPTY;
        b.b_ml.ml_line_count = 1;
        if unsafe { ml_open_blocks(buf, mfp, &mut hp) } {
            return OK;
        }
        if !hp.is_null() {
            unsafe { mf_put(mfp, hp, false, false) };
        }
        unsafe { mf_close(mfp, true) }; // also frees the swap file's name
    }
    b.b_ml.ml_mfp = ::core::ptr::null_mut();
    FAIL
}

/// Fill in the three blocks a fresh memline starts with. The block still held
/// is left in `*hp` so the caller's failure path can release it.
///
/// # Safety
/// `mfp` must be a memfile with no blocks in it yet.
unsafe fn ml_open_blocks(buf: *mut buf_T, mfp: *mut memfile_T, hp: &mut *mut bhdr_T) -> bool {
    // SAFETY: the caller's buffer, reached through a handle that
    // borrows it for the one access that asked and no longer.
    let mut b = unsafe { Buf::new(buf) };
    // Block zero: the header that says what the rest of the file means.
    *hp = unsafe { mf_new(mfp, false, 1) };
    if unsafe { (**hp).bh_bnum } != 0 {
        unsafe { iemsg(tr(c"E298: Didn't get block nr 0?")) };
        return false;
    }
    let b0p = unsafe { (**hp).bh_data } as *mut ZeroBlock;
    unsafe { (*b0p).b0_id[0] = BLOCK0_ID0 as ::core::ffi::c_char };
    unsafe { (*b0p).b0_id[1] = BLOCK0_ID1 as ::core::ffi::c_char };
    unsafe { (*b0p).b0_magic_long = B0_MAGIC_LONG as ::core::ffi::c_long };
    unsafe { (*b0p).b0_magic_int = B0_MAGIC_INT as ::core::ffi::c_int };
    unsafe { (*b0p).b0_magic_short = B0_MAGIC_SHORT as int16_t };
    unsafe { (*b0p).b0_magic_char = B0_MAGIC_CHAR as ::core::ffi::c_char };
    let version = b0p
        .wrapping_byte_add(offset_of!(ZeroBlock, b0_version))
        .cast::<::core::ffi::c_char>();
    let name = min_vim_version_name().as_ptr();
    // SAFETY: the field is ten bytes, which is what "VIM " and a five-byte
    // version name plus its terminator need.
    unsafe { xstrlcpy(xstpcpy(version, c"VIM ".as_ptr()), name, 6) };
    let page_size = unsafe { (*mfp).mf_page_size } as ::core::ffi::c_long;
    // SAFETY: block zero is the page `hp` holds. The borrow lasts only for
    // the store -- `&mut unsafe { .. }` would write into a discarded copy
    // of the field, which is how the page size never reached the file.
    unsafe { b0_store_number(page_size, &mut (*b0p).b0_page_size) };

    if !b.b_spell {
        let changed = b.b_changed != 0;
        unsafe { (*b0p).set_dirty(changed) };
        let fileformat = unsafe { get_fileformat(buf) } + 1;
        unsafe { (*b0p).set_flags(fileformat) };
        unsafe { set_b0_fname(b0p, buf) };
        unsafe {
            os_get_username(
                (&raw mut (*b0p).b0_uname).cast::<::core::ffi::c_char>(),
                B0_UNAME_SIZE as size_t,
            )
        };
        unsafe { (*b0p).b0_uname[B0_UNAME_SIZE as usize - 1] = NUL as ::core::ffi::c_char };
        unsafe {
            os_get_hostname(
                (&raw mut (*b0p).b0_hname).cast::<::core::ffi::c_char>(),
                B0_HNAME_SIZE as size_t,
            )
        };
        unsafe { (*b0p).b0_hname[B0_HNAME_SIZE as usize - 1] = NUL as ::core::ffi::c_char };
        let pid = os_get_pid() as ::core::ffi::c_long;
        unsafe { b0_store_number(pid, &mut (*b0p).b0_pid) };
    }

    // Always sync block zero, so that findswapname can read the file name
    // out of the swap file. Not for a help or spell buffer. This only
    // does anything once there is a swap file; otherwise it happens when
    // one is created.
    unsafe { mf_put(mfp, *hp, true, false) };
    if !b.b_help && !b.b_spell {
        // Best effort; there may not be a swap file yet.
        let _ = unsafe { mf_sync(mfp, 0) };
    }

    // Block one: the root pointer block, pointing at the one data block.
    *hp = unsafe { ml_new_ptr(mfp) };
    debug_assert!(!(*hp).is_null());
    if unsafe { (**hp).bh_bnum } != 1 {
        unsafe { iemsg(tr(c"E298: Didn't get block nr 1?")) };
        return false;
    }
    let pp = unsafe { (**hp).bh_data } as *mut PointerBlock;
    unsafe { (*pp).pb_count = 1 };
    let entry = pb_entries(pp);
    unsafe { (*entry).pe_bnum = 2 };
    unsafe { (*entry).pe_page_count = 1 };
    unsafe { (*entry).pe_old_lnum = 1 };
    unsafe { (*entry).pe_line_count = 1 }; // line count after the insertion below
    unsafe { mf_put(mfp, *hp, true, false) };

    // Block two: the first data block, holding one empty line.
    *hp = unsafe { ml_new_data(mfp, false, 1) };
    if unsafe { (**hp).bh_bnum } != 2 {
        unsafe { iemsg(tr(c"E298: Didn't get block nr 2?")) };
        return false;
    }
    let dp = unsafe { (**hp).bh_data } as *mut DataBlock;
    unsafe { (*dp).db_txt_start -= 1 }; // at the end of the block
    unsafe { *db_index(dp) = (*dp).db_txt_start };
    unsafe { (*dp).db_free -= 1 + INDEX_SIZE as ::core::ffi::c_uint };
    unsafe { (*dp).db_line_count = 1 };
    unsafe { *db_byte(dp, (*dp).db_txt_start as isize) = NUL as ::core::ffi::c_char };
    true
}

/// Open a swap file for every buffer that could use one.
///
/// # Safety
/// Must run on the main thread.
pub unsafe fn ml_open_files() {
    for buf in buffers() {
        if buf.b_p_ro == 0 || buf.b_changed != 0 {
            // SAFETY: a live buffer from the editor's own list, on the main
            // thread as the caller promised.
            unsafe { ml_open_file(buf.raw()) };
        }
    }
}

/// Open a swap file for `buf`'s memfile, if it has none yet.
///
/// If no usable file name can be found the memfile keeps no name and
/// remains memory-only, with no recovery possible.
///
/// # Safety
/// `buf` must point at a buffer.
pub unsafe fn ml_open_file(buf: *mut buf_T) {
    // SAFETY: the caller's buffer, reached through a handle that
    // borrows it for the one access that asked and no longer.
    let mut b = unsafe { Buf::new(buf) };
    let mfp = b.b_ml.ml_mfp;
    if mfp.is_null()
        || unsafe { (*mfp).mf_fd } >= 0
        || b.b_p_swf == 0
        || cmdmod_has(CmdModFlags::NOSWAPFILE)
        || !b.terminal.is_null()
    {
        return; // nothing to do
    }

    // A spell buffer gets a temp file name.
    if b.b_spell {
        let fname = unsafe { vim_tempname() };
        if !fname.is_null() {
            // A spell buffer keeps working without a swap file.
            let _ = unsafe { mf_open_file(mfp, fname) }; // consumes fname!
        }
        b.b_may_swap = false;
        return;
    }

    // Try every directory in 'directory'.
    let mut dirp = p_dir.get();
    let mut found_existing_dir = false;
    while unsafe { *dirp } != NUL as ::core::ffi::c_char {
        // Between choosing the name and creating the file another Nvim
        // may have created it; then the create fails and the next
        // directory is tried.
        let fname = unsafe {
            findswapname(
                buf,
                &raw mut dirp,
                ::core::ptr::null_mut(),
                &raw mut found_existing_dir,
            )
        };
        if dirp.is_null() {
            break; // out of memory
        }
        if fname.is_null() {
            continue;
        }
        if unsafe { mf_open_file(mfp, fname) }.is_err() {
            // consumes fname!
            continue;
        }
        unsafe { (*mfp).mf_dirty = MfDirty::YesNoSync }; // don't sync yet in ml_sync_all
        unsafe { ml_upd_block0(buf, UB_SAME_DIR) };

        // Flush block zero, so others can read it.
        if unsafe { mf_sync(mfp, MFS_ZERO as ::core::ffi::c_int) }.is_ok() {
            // Mark every block that belongs in the swap file dirty, for
            // when 'swapfile' was reset (deleting the file) and set again.
            unsafe { mf_set_dirty(mfp) };
            break;
        }
        // Writing block zero failed: close it and try another directory.
        unsafe { mf_close_file(buf, false) };
    }

    if unsafe { *p_dir.get() } != NUL as ::core::ffi::c_char && unsafe { mf_fname(mfp) }.is_null() {
        need_wait_return.set(true); // call wait_return() later
        let _no_prompt = Suppress::wait_return();
        unsafe {
            semsg_c!(
                tr(c"E303: Unable to open swap file for \"%s\", recovery impossible"),
                if !buf_spname(buf).is_null() {
                    buf_spname(buf)
                } else {
                    b.b_fname
                },
            )
        };
    }

    b.b_may_swap = false; // don't try to open a swap file again
}

/// Create the swap file now, if one is still wanted and this is a writable
/// file being opened or a read into an existing buffer.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn check_need_swap(newfile: bool) {
    // The swap dialog may prompt, and the user has to see it; E325 may
    // reset this again.
    let _loud = Allow::messages();
    if cur_buf().b_may_swap && (cur_buf().b_p_ro == 0 || !newfile) {
        unsafe { ml_open_file(curbuf.get()) };
    }
}

/// Close `buf`'s memline, deleting the swap file if `del_file`.
///
/// # Safety
/// `buf` must point at a buffer.
pub unsafe fn ml_close(buf: *mut buf_T, del_file: ::core::ffi::c_int) {
    // SAFETY: the caller's buffer, reached through a handle that
    // borrows it for the one access that asked and no longer.
    let mut b = unsafe { Buf::new(buf) };
    if b.b_ml.ml_mfp.is_null() {
        return; // not open
    }
    unsafe { mf_close((*buf).b_ml.ml_mfp, del_file != 0) }; // closes the .swp file
    // The cached line, if the memline owns it -- which it can only be
    // while a line is cached at all.
    if let Some(owned) = b.b_ml.take_owned() {
        unsafe { xfree(owned.cast()) };
    }
    b.b_ml.stack_free();
    b.b_ml.ml_chunks.free();
    b.b_ml.ml_mfp = ::core::ptr::null_mut();

    // Clear the "recovered" flag, so the ATTENTION prompt comes back the
    // next time this buffer is loaded.
    b.b_flags.clear(BufFlags::RECOVERED);
}

/// Close every memline and memfile. Only used when exiting.
///
/// # Safety
/// Must run on the main thread.
pub unsafe fn ml_close_all(del_file: bool) {
    for buf in buffers() {
        // SAFETY: a live buffer from the editor's own list, on the main
        // thread as the caller promised. `ml_close` drops the memline, not
        // the buffer, so the link the walk reads next stays good.
        unsafe { ml_close(buf.raw(), del_file as ::core::ffi::c_int) };
    }
    unsafe { spell_delete_wordlist() }; // delete the internal wordlist
    unsafe { vim_deltempdir() }; // delete the temp directory that was created
}

/// Close the memfile of every unmodified buffer. Only for use just before
/// exiting.
///
/// # Safety
/// Must run on the main thread.
pub unsafe fn ml_close_notmod() {
    for buf in buffers() {
        if !buf_is_changed(buf) {
            // SAFETY: a live buffer from the editor's own list, on the main
            // thread as the caller promised.
            unsafe { ml_close(buf.raw(), 1) };
        }
    }
}

pub const EOL_DOS: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
