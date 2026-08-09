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

use crate::semsg_c;
use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::autocmd::{
    EVENT_BUFREADPOST, EVENT_BUFWINENTER, EVENT_SWAPEXISTS, apply_autocmds, has_autocmd,
};
use crate::src::nvim::buffer::{buf_inc_changedtick, buf_spname, open_buffer, setfname};
use crate::src::nvim::change::{changed_internal, unchanged};
use crate::src::nvim::cursor::{check_cursor, coladvance};
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_curbuf_later};
use crate::src::nvim::eval::typval::{
    tv_dict_add_nr, tv_dict_add_str_len, tv_list_append_allocated_string,
};
use crate::src::nvim::eval::vars::{get_vim_var_str, set_vim_var_string};
use crate::src::nvim::event::libuv::{uv_strerror, uv_uptime};
use crate::src::nvim::fileio::{
    buf_store_file_info, modname, read_eintr, readfile, vim_deltempdir, vim_rename, vim_tempname,
};
use crate::src::nvim::getchar::flush_buffers;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::input::prompt_for_input;
use crate::src::nvim::main::{
    NameBuff, allbuf_lock, cmdline_row, cmdmod, curbuf, curwin, did_check_timestamps, firstbuf,
    getout, got_int, inhibit_delete_count, msg_ext_skip_flush, msg_row, msg_silent,
    need_check_timestamps, need_wait_return, no_lines_msg, no_wait_return, p_dir, p_shm, p_uc,
    p_verbose, recoverymode, swap_exists_action,
};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::mbyte::{
    mb_adjust_cursor, mb_utflen, utf_head_off, utf_ptr2char, utfc_ptr2len,
};
use crate::src::nvim::memfile::{
    MfDirty, mf_close, mf_close_file, mf_find, mf_fname, mf_free, mf_free_fnames, mf_get,
    mf_need_trans, mf_new, mf_new_page_size, mf_open, mf_open_file, mf_put, mf_set_dirty,
    mf_set_fnames, mf_sync, mf_trans_del,
};
use crate::src::nvim::memory::{xfree, xmalloc, xmemdupz, xrealloc, xstpcpy, xstrdup, xstrlcpy};
use crate::src::nvim::message::{
    do_dialog, emsg, iemsg, msg, msg_end, msg_ext_set_kind, msg_home_replace, msg_multiline,
    msg_outnum, msg_outtrans, msg_putchar, msg_puts, msg_puts_hl, msg_reset_scroll, msg_start,
    set_keep_msg, verb_msg,
};
use crate::src::nvim::option::{
    copy_option_part, get_fileformat, set_fileformat, set_option_value_give_err,
};
use crate::src::nvim::options::kOptFileencoding;
use crate::src::nvim::os::env::{
    expand_env, home_replace, home_replace_save, os_get_hostname, os_get_pid,
};
use crate::src::nvim::os::fs::{
    os_fileinfo, os_fileinfo_inode, os_fileinfo_link, os_fileinfo_size, os_isdir, os_mkdir_recurse,
    os_open, os_path_exists, os_remove, os_set_cloexec,
};
use crate::src::nvim::os::input::{line_breakcheck, os_char_avail};
use crate::src::nvim::os::libc::{
    __errno_location, close, gettext, lseek, memmove, readlink, strcasecmp, strcmp, strcpy, strlen,
    strncasecmp, strncmp,
};
use crate::src::nvim::os::proc::os_proc_running;
use crate::src::nvim::os::time::{os_ctime_r, os_time};
use crate::src::nvim::os::users::{os_get_uname, os_get_username};
use crate::src::nvim::path::{
    FreeWild, after_pathsep, concat_fnames, expand_wildcards, fix_fname, path_fnamecmp,
    path_full_compare, path_is_absolute, path_tail, same_directory, shorten_dir, vim_FullName,
    vim_ispathsep,
};
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::spell::spell_delete_wordlist;
use crate::src::nvim::statusline::get_trans_bufname;
use crate::src::nvim::strings::{kv_do_printf, vim_strchr, xstrnsave};
use crate::src::nvim::types::ui::kUIMessages;
use crate::src::nvim::types::{
    CMOD_NOSWAPFILE, FileInfo, OptVal, OptValData, OptValType, String_0, StringBuilder, Timestamp,
    bhdr_T, blocknr_T, buf_T, chunksize_T, colnr_T, dict_T, file_comparison, flush_buffers_T,
    infoptr_T, int16_t, int64_t, linenr_T, list_T, memfile_T, off_T, pos_T, size_t, ssize_t,
    time_t, uint8_t, uint16_t, uint64_t, uv_uid_t, varnumber_T,
};
use crate::src::nvim::ui::{ui_flush, ui_has};
use crate::src::nvim::undo::bufIsChanged;
use crate::src::nvim::version::min_vim_version_name;

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
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const VIM_WARNING: C2Rust_Unnamed_18 = 2;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const READ_NEW: C2Rust_Unnamed_19 = 1;
pub const FLUSH_TYPEAHEAD: flush_buffers_T = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const MFS_ZERO: C2Rust_Unnamed_20 = 8;
pub const MFS_FLUSH: C2Rust_Unnamed_20 = 4;
pub const MFS_STOP: C2Rust_Unnamed_20 = 2;
pub const MFS_ALL: C2Rust_Unnamed_20 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const MIN_SWAP_PAGE_SIZE: C2Rust_Unnamed_21 = 1048;
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
pub const DATA_ID: C2Rust_Unnamed_27 = 25697;
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
pub const PTR_ID: C2Rust_Unnamed_27 = 28788;
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
pub const B0_HNAME_SIZE: C2Rust_Unnamed_28 = 40;
pub const B0_UNAME_SIZE: C2Rust_Unnamed_28 = 40;
pub const B0_FNAME_SIZE_ORG: C2Rust_Unnamed_28 = 900;
pub const B0_FNAME_SIZE_NOCRYPT: C2Rust_Unnamed_28 = 898;
pub const B0_FNAME_SIZE_CRYPT: C2Rust_Unnamed_28 = 890;
pub const B0_MAGIC_CHAR: C2Rust_Unnamed_29 = 85;
pub const B0_MAGIC_SHORT: C2Rust_Unnamed_29 = 269554195;
pub const B0_MAGIC_INT: C2Rust_Unnamed_29 = 539042339;
pub const B0_MAGIC_LONG: C2Rust_Unnamed_29 = 808530483;
pub const BLOCK0_ID1: C2Rust_Unnamed_27 = 48;
pub const BLOCK0_ID0: C2Rust_Unnamed_27 = 98;
pub const SEA_CHOICE_NONE: sea_choice_T = 0;
pub type sea_choice_T = ::core::ffi::c_uint;
pub const SEA_CHOICE_ABORT: sea_choice_T = 6;
pub const SEA_CHOICE_QUIT: sea_choice_T = 5;
pub const SEA_CHOICE_DELETE: sea_choice_T = 4;
pub const SEA_CHOICE_RECOVER: sea_choice_T = 3;
pub const SEA_CHOICE_EDIT: sea_choice_T = 2;
pub const SEA_CHOICE_READONLY: sea_choice_T = 1;
pub const SHM_ATTENTION: C2Rust_Unnamed_25 = 65;
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
/// Lines below which two neighbouring `ml_chunksize` entries are merged, and
/// the size a split aims for.
pub const MLCS_MINL: ::core::ffi::c_int = 400;
/// Lines above which an `ml_chunksize` entry is split in two.
pub const MLCS_MAXL: ::core::ffi::c_int = 800;
/// `flags` for `ml_append_int`: carry the `DB_MARKED` bit onto the new line.
pub const ML_APPEND_MARK: ::core::ffi::c_int = 2;
/// `flags` for `ml_append_int`: this is a fresh file being read in, so the
/// block may be numbered negatively and need not keep its position.
pub const ML_APPEND_NEW: ::core::ffi::c_int = 1;
/// `flags` for `ml_delete_int`: say "--No lines in buffer--" if the buffer
/// ends up empty.
pub const ML_DEL_MESSAGE: ::core::ffi::c_int = 1;
pub const OPT_LOCAL: C2Rust_Unnamed_24 = 2;
pub const kEqualFiles: file_comparison = 1;
pub const EW_SILENT: C2Rust_Unnamed_26 = 32;
pub const EW_FILE: C2Rust_Unnamed_26 = 2;
pub const EW_KEEPALL: C2Rust_Unnamed_26 = 16;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const O_RDWR: ::core::ffi::c_int = 0o2 as ::core::ffi::c_int;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const KV_INITIAL_VALUE: StringBuilder = StringBuilder {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const BF_RECOVERED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const BF_DUMMY: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const ML_CHNK_ADDLINE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ML_CHNK_DELLINE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ML_CHNK_UPDLINE: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const ML_LINE_DIRTY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const ML_LOCKED_DIRTY: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const ML_LOCKED_POS: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const ML_ALLOCATED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const BH_DIRTY: ::core::ffi::c_uint = 1 as ::core::ffi::c_uint;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
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
    unsafe {
        // No stack, no cached block, no cached line, no chunk table yet.
        (*buf).b_ml.ml_stack_size = 0;
        (*buf).b_ml.ml_stack = ::core::ptr::null_mut();
        (*buf).b_ml.ml_stack_top = 0;
        (*buf).b_ml.ml_locked = ::core::ptr::null_mut();
        (*buf).b_ml.ml_line_lnum = 0;
        (*buf).b_ml.ml_line_offset = 0;
        (*buf).b_ml.ml_chunksize = ::core::ptr::null_mut();
        (*buf).b_ml.ml_usedchunks = 0;

        if (*cmdmod.ptr()).cmod_flags & CMOD_NOSWAPFILE as ::core::ffi::c_int != 0 {
            (*buf).b_p_swf = false_0;
        }
        // A swap file may still be opened later, when 'updatecount' is set.
        (*buf).b_may_swap = (*buf).terminal.is_null() && p_uc.get() != 0 && (*buf).b_p_swf != 0;

        let mfp = mf_open(::core::ptr::null_mut(), 0);
        let mut hp: *mut bhdr_T = ::core::ptr::null_mut();
        if !mfp.is_null() {
            (*buf).b_ml.ml_mfp = mfp;
            (*buf).b_ml.ml_flags = ML_EMPTY;
            (*buf).b_ml.ml_line_count = 1;
            if ml_open_blocks(buf, mfp, &mut hp) {
                return OK;
            }
            if !hp.is_null() {
                mf_put(mfp, hp, false, false);
            }
            mf_close(mfp, true); // also frees the swap file's name
        }
        (*buf).b_ml.ml_mfp = ::core::ptr::null_mut();
        FAIL
    }
}

/// Fill in the three blocks a fresh memline starts with. The block still held
/// is left in `*hp` so the caller's failure path can release it.
///
/// # Safety
/// `mfp` must be a memfile with no blocks in it yet.
unsafe fn ml_open_blocks(buf: *mut buf_T, mfp: *mut memfile_T, hp: &mut *mut bhdr_T) -> bool {
    unsafe {
        // Block zero: the header that says what the rest of the file means.
        *hp = mf_new(mfp, false, 1);
        if (**hp).bh_bnum != 0 {
            iemsg(gettext(c"E298: Didn't get block nr 0?".as_ptr()));
            return false;
        }
        let b0p = (**hp).bh_data as *mut ZeroBlock;
        (*b0p).b0_id[0] = BLOCK0_ID0 as ::core::ffi::c_char;
        (*b0p).b0_id[1] = BLOCK0_ID1 as ::core::ffi::c_char;
        (*b0p).b0_magic_long = B0_MAGIC_LONG as ::core::ffi::c_long;
        (*b0p).b0_magic_int = B0_MAGIC_INT as ::core::ffi::c_int;
        (*b0p).b0_magic_short = B0_MAGIC_SHORT as int16_t;
        (*b0p).b0_magic_char = B0_MAGIC_CHAR as ::core::ffi::c_char;
        xstrlcpy(
            xstpcpy(
                (&raw mut (*b0p).b0_version).cast::<::core::ffi::c_char>(),
                c"VIM ".as_ptr(),
            ),
            min_vim_version_name().as_ptr(),
            6,
        );
        b0_store_number(
            (*mfp).mf_page_size as ::core::ffi::c_long,
            &mut (*b0p).b0_page_size,
        );

        if !(*buf).b_spell {
            (*b0p).set_dirty((*buf).b_changed != 0);
            (*b0p).set_flags(get_fileformat(buf) + 1);
            set_b0_fname(b0p, buf);
            os_get_username(
                (&raw mut (*b0p).b0_uname).cast::<::core::ffi::c_char>(),
                B0_UNAME_SIZE as size_t,
            );
            (*b0p).b0_uname[B0_UNAME_SIZE as usize - 1] = NUL as ::core::ffi::c_char;
            os_get_hostname(
                (&raw mut (*b0p).b0_hname).cast::<::core::ffi::c_char>(),
                B0_HNAME_SIZE as size_t,
            );
            (*b0p).b0_hname[B0_HNAME_SIZE as usize - 1] = NUL as ::core::ffi::c_char;
            b0_store_number(os_get_pid() as ::core::ffi::c_long, &mut (*b0p).b0_pid);
        }

        // Always sync block zero, so that findswapname can read the file name
        // out of the swap file. Not for a help or spell buffer. This only
        // does anything once there is a swap file; otherwise it happens when
        // one is created.
        mf_put(mfp, *hp, true, false);
        if !(*buf).b_help && !(*buf).b_spell {
            mf_sync(mfp, 0);
        }

        // Block one: the root pointer block, pointing at the one data block.
        *hp = ml_new_ptr(mfp);
        debug_assert!(!(*hp).is_null());
        if (**hp).bh_bnum != 1 {
            iemsg(gettext(c"E298: Didn't get block nr 1?".as_ptr()));
            return false;
        }
        let pp = (**hp).bh_data as *mut PointerBlock;
        (*pp).pb_count = 1;
        let entry = pb_entries(pp);
        (*entry).pe_bnum = 2;
        (*entry).pe_page_count = 1;
        (*entry).pe_old_lnum = 1;
        (*entry).pe_line_count = 1; // line count after the insertion below
        mf_put(mfp, *hp, true, false);

        // Block two: the first data block, holding one empty line.
        *hp = ml_new_data(mfp, false, 1);
        if (**hp).bh_bnum != 2 {
            iemsg(gettext(c"E298: Didn't get block nr 2?".as_ptr()));
            return false;
        }
        let dp = (**hp).bh_data as *mut DataBlock;
        (*dp).db_txt_start -= 1; // at the end of the block
        *db_index(dp) = (*dp).db_txt_start;
        (*dp).db_free -= 1 + INDEX_SIZE as ::core::ffi::c_uint;
        (*dp).db_line_count = 1;
        *(dp as *mut ::core::ffi::c_char).offset((*dp).db_txt_start as isize) =
            NUL as ::core::ffi::c_char;
        true
    }
}

/// Open a swap file for every buffer that could use one.
///
/// # Safety
/// Must run on the main thread.
pub unsafe fn ml_open_files() {
    unsafe {
        let mut buf = firstbuf.get();
        while !buf.is_null() {
            if (*buf).b_p_ro == 0 || (*buf).b_changed != 0 {
                ml_open_file(buf);
            }
            buf = (*buf).b_next;
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
    unsafe {
        let mfp = (*buf).b_ml.ml_mfp;
        if mfp.is_null()
            || (*mfp).mf_fd >= 0
            || (*buf).b_p_swf == 0
            || (*cmdmod.ptr()).cmod_flags & CMOD_NOSWAPFILE as ::core::ffi::c_int != 0
            || !(*buf).terminal.is_null()
        {
            return; // nothing to do
        }

        // A spell buffer gets a temp file name.
        if (*buf).b_spell {
            let fname = vim_tempname();
            if !fname.is_null() {
                mf_open_file(mfp, fname); // consumes fname!
            }
            (*buf).b_may_swap = false;
            return;
        }

        // Try every directory in 'directory'.
        let mut dirp = p_dir.get();
        let mut found_existing_dir = false;
        while *dirp != NUL as ::core::ffi::c_char {
            // Between choosing the name and creating the file another Nvim
            // may have created it; then the create fails and the next
            // directory is tried.
            let fname = findswapname(
                buf,
                &raw mut dirp,
                ::core::ptr::null_mut(),
                &raw mut found_existing_dir,
            );
            if dirp.is_null() {
                break; // out of memory
            }
            if fname.is_null() {
                continue;
            }
            if mf_open_file(mfp, fname) != OK {
                // consumes fname!
                continue;
            }
            (*mfp).mf_dirty = MfDirty::YesNoSync; // don't sync yet in ml_sync_all
            ml_upd_block0(buf, UB_SAME_DIR);

            // Flush block zero, so others can read it.
            if mf_sync(mfp, MFS_ZERO as ::core::ffi::c_int) == OK {
                // Mark every block that belongs in the swap file dirty, for
                // when 'swapfile' was reset (deleting the file) and set again.
                mf_set_dirty(mfp);
                break;
            }
            // Writing block zero failed: close it and try another directory.
            mf_close_file(buf, false);
        }

        if *p_dir.get() != NUL as ::core::ffi::c_char && mf_fname(mfp).is_null() {
            need_wait_return.set(true); // call wait_return() later
            (*no_wait_return.ptr()) += 1;
            semsg_c!(
                gettext(c"E303: Unable to open swap file for \"%s\", recovery impossible".as_ptr()),
                if !buf_spname(buf).is_null() {
                    buf_spname(buf)
                } else {
                    (*buf).b_fname
                },
            );
            (*no_wait_return.ptr()) -= 1;
        }

        (*buf).b_may_swap = false; // don't try to open a swap file again
    }
}

/// Create the swap file now, if one is still wanted and this is a writable
/// file being opened or a read into an existing buffer.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn check_need_swap(newfile: bool) {
    unsafe {
        // The swap dialog may prompt, and the user has to see it; E325 may
        // reset this again.
        let old_msg_silent = msg_silent.get();
        msg_silent.set(0);
        if (*curbuf.get()).b_may_swap && ((*curbuf.get()).b_p_ro == 0 || !newfile) {
            ml_open_file(curbuf.get());
        }
        msg_silent.set(old_msg_silent);
    }
}

/// Close `buf`'s memline, deleting the swap file if `del_file`.
///
/// # Safety
/// `buf` must point at a buffer.
pub unsafe fn ml_close(buf: *mut buf_T, del_file: ::core::ffi::c_int) {
    unsafe {
        if (*buf).b_ml.ml_mfp.is_null() {
            return; // not open
        }
        mf_close((*buf).b_ml.ml_mfp, del_file != 0); // closes the .swp file
        if (*buf).b_ml.ml_line_lnum != 0
            && (*buf).b_ml.ml_flags & (ML_LINE_DIRTY | ML_ALLOCATED) != 0
        {
            xfree((*buf).b_ml.ml_line_ptr.cast());
        }
        xfree((*buf).b_ml.ml_stack.cast());
        xfree((*buf).b_ml.ml_chunksize.cast());
        (*buf).b_ml.ml_chunksize = ::core::ptr::null_mut();
        (*buf).b_ml.ml_mfp = ::core::ptr::null_mut();

        // Clear the "recovered" flag, so the ATTENTION prompt comes back the
        // next time this buffer is loaded.
        (*buf).b_flags &= !BF_RECOVERED;
    }
}

/// Close every memline and memfile. Only used when exiting.
///
/// # Safety
/// Must run on the main thread.
pub unsafe fn ml_close_all(del_file: bool) {
    unsafe {
        let mut buf = firstbuf.get();
        while !buf.is_null() {
            ml_close(buf, del_file as ::core::ffi::c_int);
            buf = (*buf).b_next;
        }
        spell_delete_wordlist(); // delete the internal wordlist
        vim_deltempdir(); // delete the temp directory that was created
    }
}

/// Close the memfile of every unmodified buffer. Only for use just before
/// exiting.
///
/// # Safety
/// Must run on the main thread.
pub unsafe fn ml_close_notmod() {
    unsafe {
        let mut buf = firstbuf.get();
        while !buf.is_null() {
            if !bufIsChanged(buf) {
                ml_close(buf, true_0);
            }
            buf = (*buf).b_next;
        }
    }
}

pub const EOL_DOS: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ENOENT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
